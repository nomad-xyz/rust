use crate::chains::PageSettings;
use crate::{IndexDataTypes, IndexSettings, NomadDB};
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

/// Entity that drives the syncing of an agent's db with on-chain data.
/// Extracts chain-specific data (emitted updates, messages, etc) from an
/// `indexer` and fills the agent's db with this data. A CachingHome or
/// CachingReplica will use a contract sync to spawn syncing tasks to keep the
/// db up-to-date.
#[derive(Debug, Clone)]
pub struct ContractSync<I> {
    agent_name: String,
    home: String,
    replica: String,
    db: NomadDB,
    indexer: Arc<I>,
    index_settings: IndexSettings,
    page_settings: PageSettings,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        agent_name: String,
        home: String,
        replica: String,
        db: NomadDB,
        indexer: Arc<I>,
        index_settings: IndexSettings,
        page_settings: PageSettings,
        finality: u8,
        metrics: ContractSyncMetrics,
    ) -> Self {
        Self {
            agent_name,
            home,
            replica,
            db,
            indexer,
            index_settings,
            page_settings,
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
        tokio::spawn(async move { self.sync_updates().await? }).instrument(span)
    }

    /// Spawn task that continuously looks for new on-chain updates and stores
    /// them in db. If run in timelag is off, will index at the tip
    /// but use a manual timelag to catch any missed updates. If timelag on,
    /// update  syncing will be run timelag blocks behind the tip.
    pub fn sync_updates(&self) -> Instrumented<JoinHandle<Result<()>>> {
        let span = info_span!("UpdateContractSync");

        let db = self.db.clone();
        let indexer = self.indexer.clone();
        let indexed_height = self.metrics.indexed_height.with_label_values(&[
            UPDATES_LABEL,
            &self.home,
            &self.replica,
            &self.agent_name,
        ]);
        let store_update_latency = self
            .metrics
            .store_event_latency
            .clone()
            .with_label_values(&[UPDATES_LABEL, &self.home, &self.replica, &self.agent_name]);

        let stored_updates = self.metrics.stored_events.with_label_values(&[
            UPDATES_LABEL,
            &self.home,
            &self.replica,
            &self.agent_name,
        ]);

        let timelag_on = self.index_settings.timelag_on();
        let finality = self.finality as u32;
        let config_from = self.page_settings.from;
        let chunk_size = self.page_settings.page_size;

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

                let (start, end) = if timelag_on {
                    // if timelag on, don't modify range
                    (from, to)
                } else {
                    let range = to - from;
                    let last_final_block = tip - finality;

                    // If range includes non-final blocks, include range
                    // blocks behind last final block
                    let from = if to >= last_final_block {
                        last_final_block - range
                    } else {
                        from
                    };

                    (from, to)
                };

                info!(
                    start = start,
                    end = end,
                    "[Updates]: BLAH indexing block heights {}...{}",
                    start,
                    end,
                );

                let sorted_updates = indexer.fetch_sorted_updates(start, end).await?;

                info!("Indexed block heights");

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
        let data_types = self.index_settings.data_types();

        tokio::spawn(async move {
            let tasks = match data_types {
                IndexDataTypes::Updates => vec![self.sync_updates()],
                IndexDataTypes::UpdatesAndMessages => {
                    vec![self.sync_updates(), self.sync_messages()]
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
        let indexed_height = self.metrics.indexed_height.with_label_values(&[
            MESSAGES_LABEL,
            &self.home,
            &self.replica,
            &self.agent_name,
        ]);

        let stored_messages = self.metrics.stored_events.with_label_values(&[
            MESSAGES_LABEL,
            &self.home,
            &self.replica,
            &self.agent_name,
        ]);

        let timelag_on = self.index_settings.timelag_on();
        let config_from = self.page_settings.from;
        let chunk_size = self.page_settings.page_size;

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

                // timelag always applied
                let (start, end) = if timelag_on {
                    (from, to)
                } else {
                    panic!("Syncing messages with timelag off should never happen!");
                };

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

#[cfg(test)]
mod test {
    use mockall::*;
    use nomad_test::mocks::MockIndexer;

    use std::sync::Arc;

    use ethers::core::types::H256;
    use ethers::signers::LocalWallet;

    use crate::chains::PageSettings;
    use nomad_core::{SignedUpdateWithMeta, Update, UpdateMeta};
    use nomad_test::test_utils;

    use super::*;
    use crate::CoreMetrics;

    const FINALITY: u8 = 5;

    /* RPC Behavior:
     *  Starting Tip: block 20
     *  Starting Last Final Block: block 15
     *
     *  Finality: 5 blocks
     *  Chunk Size: 10 blocks
     *
     * Responses
     *  - original 10-20, total indexed 5-20, final indexed 5-15: 1st update @
     *    block 18
     *  - original 20-30, total indexed 15-30, final indexed 15-25: 2nd update
     *    @ block 26
     *  - original 30-40, total indexed 25-40, final indexed 25-35: empty
     *    (3rd update reorged out of range 35-40)
     *  - original 40-50, total indexed 35-50, final indexed 35-45: 3rd update @
     *    FINAL block 37, 4th update @ block 48
     */
    #[tokio::test]
    async fn handles_reorgs_when_syncing_at_tip() {
        test_utils::run_test_db(|db| async move {
            let signer: LocalWallet =
                "1111111111111111111111111111111111111111111111111111111111111111"
                    .parse()
                    .unwrap();

            let first_root = H256::from([0; 32]);
            let second_root = H256::from([1; 32]);
            let third_root = H256::from([2; 32]);
            let fourth_root = H256::from([3; 32]);
            let fifth_root = H256::from([4; 32]);

            let first_update = Update {
                home_domain: 1,
                previous_root: first_root,
                new_root: second_root,
            }
            .sign_with(&signer)
            .await
            .expect("!sign");

            let second_update = Update {
                home_domain: 1,
                previous_root: second_root,
                new_root: third_root,
            }
            .sign_with(&signer)
            .await
            .expect("!sign");

            let third_update = Update {
                home_domain: 1,
                previous_root: third_root,
                new_root: fourth_root,
            }
            .sign_with(&signer)
            .await
            .expect("!sign");

            let fourth_update = Update {
                home_domain: 1,
                previous_root: fourth_root,
                new_root: fifth_root,
            }
            .sign_with(&signer)
            .await
            .expect("!sign");

            let mut mock_indexer = MockIndexer::new();
            {
                let mut seq = Sequence::new();

                let first_update_with_meta = SignedUpdateWithMeta {
                    signed_update: first_update.clone(),
                    metadata: UpdateMeta {
                        block_number: 18,
                        timestamp: Default::default(),
                    },
                };

                let second_update_with_meta = SignedUpdateWithMeta {
                    signed_update: second_update.clone(),
                    metadata: UpdateMeta {
                        block_number: 26,
                        timestamp: Default::default(),
                    },
                };

                let third_update_with_meta = SignedUpdateWithMeta {
                    signed_update: third_update.clone(),
                    metadata: UpdateMeta {
                        block_number: 37,
                        timestamp: Default::default(),
                    },
                };

                let fourth_update_with_meta = SignedUpdateWithMeta {
                    signed_update: fourth_update.clone(),
                    metadata: UpdateMeta {
                        block_number: 48,
                        timestamp: Default::default(),
                    },
                };

                // Return first update in range 5-20
                mock_indexer
                    .expect__get_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(20));
                mock_indexer
                    .expect__fetch_sorted_updates()
                    .withf(move |from: &u32, to: &u32| *from == 5 && *to == 20)
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| Ok(vec![first_update_with_meta]));

                // Return second update in range 15-30
                mock_indexer
                    .expect__get_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(30));
                mock_indexer
                    .expect__fetch_sorted_updates()
                    .withf(move |from: &u32, to: &u32| *from == 15 && *to == 30)
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| Ok(vec![second_update_with_meta]));

                // Return empty for range 25-40 (misses 3rd update between
                // non-final range 35-40)
                mock_indexer
                    .expect__get_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(40));
                mock_indexer
                    .expect__fetch_sorted_updates()
                    .withf(move |from: &u32, to: &u32| *from == 25 && *to == 40)
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| Ok(vec![]));

                // Return both missing 3rd and new 4th updates in range 35-50
                mock_indexer
                    .expect__get_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(50));
                mock_indexer
                    .expect__fetch_sorted_updates()
                    .withf(move |from: &u32, to: &u32| *from == 35 && *to == 50)
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(move |_, _| {
                        Ok(vec![third_update_with_meta, fourth_update_with_meta])
                    });

                // Return empty vec for remaining calls
                mock_indexer
                    .expect__get_block_number()
                    .times(1)
                    .in_sequence(&mut seq)
                    .return_once(|| Ok(60));
                mock_indexer
                    .expect__fetch_sorted_updates()
                    .return_once(move |_, _| Ok(vec![]));
            }

            let nomad_db = NomadDB::new("home_1", db);
            let index_settings = IndexSettings {
                data_types: IndexDataTypes::Updates,
                use_timelag: false,
            };
            let page_settings = PageSettings {
                from: 10,
                page_size: 10,
            };

            let indexer = Arc::new(mock_indexer);
            let metrics = Arc::new(
                CoreMetrics::new(
                    "contract_sync_test",
                    "home",
                    None,
                    Arc::new(prometheus::Registry::new()),
                )
                .expect("could not make metrics"),
            );

            let sync_metrics = ContractSyncMetrics::new(metrics);

            let contract_sync = ContractSync::new(
                "agent".to_owned(),
                "home_1".to_owned(),
                "replica_1".to_owned(),
                nomad_db.clone(),
                indexer.clone(),
                index_settings,
                page_settings,
                FINALITY,
                sync_metrics,
            );

            let sync_task = contract_sync.sync_updates();
            sleep(Duration::from_secs(3)).await;
            cancel_task!(sync_task);

            assert_eq!(
                nomad_db
                    .update_by_previous_root(first_root)
                    .expect("!db")
                    .expect("!update"),
                first_update.clone()
            );
            assert_eq!(
                nomad_db
                    .update_by_previous_root(second_root)
                    .expect("!db")
                    .expect("!update"),
                second_update.clone()
            );
            assert_eq!(
                nomad_db
                    .update_by_previous_root(third_root)
                    .expect("!db")
                    .expect("!update"),
                third_update.clone()
            );
            assert_eq!(
                nomad_db
                    .update_by_previous_root(fourth_root)
                    .expect("!db")
                    .expect("!update"),
                fourth_update.clone()
            );
        })
        .await
    }
}
