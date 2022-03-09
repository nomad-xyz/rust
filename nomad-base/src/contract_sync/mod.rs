use crate::{IndexMode, IndexSettings, NomadDB};
use color_eyre::Result;
use futures_util::future::select_all;
use nomad_core::{CommonIndexer, HomeIndexer};
use tokio::{task::JoinHandle, time::sleep};
use tracing::{info, info_span};
use tracing::{instrument::Instrumented, Instrument};

use std::cmp::min;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod metrics;
mod schema;

pub use metrics::ContractSyncMetrics;
use schema::{CommonContractSyncDB, HomeContractSyncDB};

const UPDATES_LABEL: &str = "updates";
const MESSAGES_LABEL: &str = "messages";

/// Fast indexing with catching timelag vs. slow timelag indexing
pub enum UpdatesSyncMode {
    /// Index at tip with timelag to catch missed updates
    Fast,
    /// Index timelag blocks behind tip (lag handled by indexer)
    Slow,
}

/// Entity that drives the syncing of an agent's db with on-chain data.
/// Extracts chain-specific data (emitted updates, messages, etc) from an
/// `indexer` and fills the agent's db with this data. A CachingHome or
/// CachingReplica will use a contract sync to spawn syncing tasks to keep the
/// db up-to-date.
#[derive(Debug, Clone)]
pub struct ContractSync<I> {
    agent_name: String,
    contract_name: String,
    db: NomadDB,
    indexer: Arc<I>,
    index_settings: IndexSettings,
    finality: u8,
    metrics: ContractSyncMetrics,
}

impl<I> std::fmt::Display for ContractSync<I>
where
    I: CommonIndexer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<I> ContractSync<I> {
    /// Instantiate new ContractSync
    pub fn new(
        agent_name: String,
        contract_name: String,
        db: NomadDB,
        indexer: Arc<I>,
        index_settings: IndexSettings,
        finality: u8,
        metrics: ContractSyncMetrics,
    ) -> Self {
        Self {
            agent_name,
            contract_name,
            db,
            indexer,
            index_settings,
            finality,
            metrics,
        }
    }
}

impl<I> ContractSync<I>
where
    I: CommonIndexer + 'static,
{
    /// Spawn sync task to sync updates
    pub fn spawn_common(self) -> Instrumented<JoinHandle<Result<()>>> {
        let span = info_span!("ContractSync: Common", self = %self);
        let index_mode = self.index_settings.mode();

        tokio::spawn(async move {
            match index_mode {
                IndexMode::Updates => self.sync_updates(UpdatesSyncMode::Slow).await?,
                IndexMode::UpdatesAndMessages => self.sync_updates(UpdatesSyncMode::Slow).await?,
                IndexMode::FastUpdates => self.sync_updates(UpdatesSyncMode::Fast).await?,
            }
        })
        .instrument(span)
    }

    /// Spawn task that continuously looks for new on-chain updates and stores
    /// them in db. If run in UpdatesSyncMode::Fast mode, will index at the tip
    /// but use a manual timelag to catch any missed updates. In Slow mode,
    /// update  syncing will be run timelag blocks behind the tip.
    pub fn sync_updates(
        &self,
        updates_sync_mode: UpdatesSyncMode,
    ) -> Instrumented<JoinHandle<Result<()>>> {
        let span = info_span!("UpdateContractSync");

        let db = self.db.clone();
        let indexer = self.indexer.clone();
        let indexed_height = self.metrics.indexed_height.clone().with_label_values(&[
            UPDATES_LABEL,
            &self.contract_name,
            &self.agent_name,
        ]);
        let store_update_latency = self
            .metrics
            .store_event_latency
            .clone()
            .with_label_values(&[UPDATES_LABEL, &self.contract_name, &self.agent_name]);

        let stored_updates = self.metrics.stored_events.clone().with_label_values(&[
            UPDATES_LABEL,
            &self.contract_name,
            &self.agent_name,
        ]);

        let finality = self.finality as u32;
        let apply_timelag = move |from: u32, to: u32| (from - finality, to - finality);
        let config_from = self.index_settings.from();
        let chunk_size = self.index_settings.chunk_size();

        tokio::spawn(async move {
            let mut from = db
                .retrieve_update_latest_block_end()
                .map_or_else(|| config_from, |h| h);

            info!(from = from, "[Updates]: resuming indexer from {}", from);

            loop {
                indexed_height.set(from as i64);

                let tip = indexer.get_block_number().await?;
                if tip <= from {
                    // Sleep if we caught up to tip
                    sleep(Duration::from_secs(100)).await;
                    continue;
                }

                let to = min(from + chunk_size, tip);

                let (start, end) = match updates_sync_mode {
                    UpdatesSyncMode::Fast => {
                        let range = to - from;
                        let last_final_block = tip - finality;
                        let from = if to >= last_final_block {
                            last_final_block - range
                        } else {
                            from
                        };

                        (from, to)
                    }
                    UpdatesSyncMode::Slow => apply_timelag(from, to),
                };

                info!(
                    start = start,
                    end = end,
                    "[Updates]: indexing block heights {}...{}",
                    start,
                    end,
                );

                let sorted_updates = indexer.fetch_sorted_updates(start, end).await?;

                // If no updates found, update last seen block and next height
                // and continue
                if sorted_updates.is_empty() {
                    db.store_update_latest_block_end(to)?;
                    from = to;
                    continue;
                }

                // Store updates
                db.store_updates_and_meta(&sorted_updates)?;

                // Report latencies from emit to store if caught up
                if to == tip {
                    let current_timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("!timestamp")
                        .as_secs();
                    for update in sorted_updates.iter() {
                        let new_root = update.signed_update.update.new_root;

                        if let Some(event_timestamp) = update.metadata.timestamp {
                            let latency = current_timestamp - event_timestamp;
                            info!(
                                new_root = ?new_root,
                                latency = latency,
                                "Latency for update with new_root {}: {}.",
                                new_root,
                                latency,
                            );
                            store_update_latency.observe(latency as f64);
                        } else {
                            info!("No timestamp for update with new_root: {}.", new_root);
                        }
                    }
                }

                // Report amount of updates stored into db
                stored_updates.add(sorted_updates.len().try_into()?);

                // Move forward next height
                db.store_update_latest_block_end(to)?;
                from = to;
            }
        })
        .instrument(span)
    }
}

impl<I> ContractSync<I>
where
    I: HomeIndexer + 'static,
{
    /// Spawn sync task to sync home updates (and potentially messages)
    pub fn spawn_home(self) -> Instrumented<JoinHandle<Result<()>>> {
        let span = info_span!("ContractSync: Home", self = %self);
        let index_mode = self.index_settings.mode();

        tokio::spawn(async move {
            let tasks = match index_mode {
                IndexMode::Updates => vec![self.sync_updates(UpdatesSyncMode::Slow)],
                IndexMode::UpdatesAndMessages => vec![
                    self.sync_updates(UpdatesSyncMode::Slow),
                    self.sync_messages(),
                ],
                IndexMode::FastUpdates => {
                    vec![self.sync_updates(UpdatesSyncMode::Fast)]
                }
            };

            let (_, _, remaining) = select_all(tasks).await;
            for task in remaining.into_iter() {
                cancel_task!(task);
            }

            Ok(())
        })
        .instrument(span)
    }

    /// Spawn task that continuously looks for new on-chain messages and stores
    /// them in db. Indexing messages should ALWAYS be done with a timelag, as
    /// ordering of messages is not guaranteed like it is for updates. Running
    /// without a timelag could cause messages with the incorrectly ordered
    /// index to be stored.
    pub fn sync_messages(&self) -> Instrumented<JoinHandle<Result<()>>> {
        let span = info_span!("MessageContractSync");

        let db = self.db.clone();
        let indexer = self.indexer.clone();
        let indexed_height = self.metrics.indexed_height.clone().with_label_values(&[
            MESSAGES_LABEL,
            &self.contract_name,
            &self.agent_name,
        ]);

        let stored_messages = self.metrics.stored_events.clone().with_label_values(&[
            MESSAGES_LABEL,
            &self.contract_name,
            &self.agent_name,
        ]);

        let finality = self.finality as u32;
        let apply_timelag = move |from: u32, to: u32| (from - finality, to - finality);
        let config_from = self.index_settings.from();
        let chunk_size = self.index_settings.chunk_size();

        tokio::spawn(async move {
            let mut from = db
                .retrieve_message_latest_block_end()
                .map_or_else(|| config_from, |h| h);

            info!(from = from, "[Messages]: resuming indexer from {}", from);

            loop {
                indexed_height.set(from as i64);

                let tip = indexer.get_block_number().await?;
                if tip <= from {
                    // Sleep if caught up to tip
                    sleep(Duration::from_secs(100)).await;
                    continue;
                }

                let candidate = from + chunk_size;
                let to = min(tip, candidate);

                let (start, end) = apply_timelag(from, to);
                info!(
                    start = start,
                    end = end,
                    "[Messages]: indexing block heights {}...{}",
                    start,
                    end
                );

                let sorted_messages = indexer.fetch_sorted_messages(start, end).await?;

                // If no messages found, update last seen block and next height
                // and continue
                if sorted_messages.is_empty() {
                    db.store_message_latest_block_end(to)?;
                    from = to;
                    continue;
                }

                // Store messages
                db.store_messages(&sorted_messages)?;

                // Report amount of messages stored into db
                stored_messages.add(sorted_messages.len().try_into()?);

                // Move forward next height
                db.store_message_latest_block_end(to)?;
                from = to;
            }
        })
        .instrument(span)
    }
}
