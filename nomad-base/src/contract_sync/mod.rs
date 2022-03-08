use crate::NomadDB;
use nomad_core::{CommonIndexer, HomeIndexer};

use tokio::time::sleep;
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
    Fast {
        /// Chain finality (handled in contract sync since indexer indexes at
        /// tip not timelag)
        finality_blocks: u8,
    },
    /// Index timelag blocks behind tip (lag handled by indexer)
    Slow,
}

/// Entity that drives the syncing of an agent's db with on-chain data.
/// Extracts chain-specific data (emitted updates, messages, etc) from an
/// `indexer` and fills the agent's db with this data. A CachingHome or
/// CachingReplica will use a contract sync to spawn syncing tasks to keep the
/// db up-to-date.
#[derive(Debug)]
pub struct ContractSync<I> {
    agent_name: String,
    contract_name: String,
    db: NomadDB,
    indexer: Arc<I>,
    from_height: u32,
    chunk_size: u32,
    metrics: ContractSyncMetrics,
}

impl<I> ContractSync<I> {
    /// Instantiate new ContractSync
    pub fn new(
        agent_name: String,
        contract_name: String,
        db: NomadDB,
        indexer: Arc<I>,
        from_height: u32,
        chunk_size: u32,
        metrics: ContractSyncMetrics,
    ) -> Self {
        Self {
            agent_name,
            contract_name,
            db,
            indexer,
            from_height,
            chunk_size,
            metrics,
        }
    }
}

impl<I> ContractSync<I>
where
    I: CommonIndexer + 'static,
{
    /// Spawn task that continuously looks for new on-chain updates and stores
    /// them in db. If run in UpdatesSyncMode::Fast mode, will index at the tip
    /// but use a timelag to catch any missed updates. In Slow mode, update
    /// syncing will be run timelag blocks behind the tip.
    pub fn sync_updates(
        &self,
        updates_sync_mode: UpdatesSyncMode,
    ) -> Instrumented<tokio::task::JoinHandle<color_eyre::Result<()>>> {
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

        let config_from = self.from_height;
        let chunk_size = self.chunk_size;

        tokio::spawn(async move {
            let mut from = db
                .retrieve_update_latest_block_end()
                .map_or_else(|| config_from, |h| h + 1);

            info!(from = from, "[Updates]: resuming indexer from {}", from);

            loop {
                indexed_height.set(from as i64);

                let tip = indexer.get_block_number().await?;
                if tip <= from {
                    // Sleep if we caught up to tip
                    sleep(Duration::from_secs(100)).await;
                    continue;
                }

                let candidate = from + chunk_size;
                let to = min(tip, candidate);
                let size = to - from;

                let start = match updates_sync_mode {
                    UpdatesSyncMode::Fast { finality_blocks } => {
                        // Range includes size blocks behind last final block to
                        // catch missing
                        let last_final_block = tip - finality_blocks as u32;
                        if to >= last_final_block {
                            info!(
                                size = size,
                                last_final_block = last_final_block,
                                "[Fast Updates]: Reindexing {} blocks behind last final block {}.",
                                size,
                                last_final_block
                            );
                            last_final_block - size
                        } else {
                            from + 1
                        }
                    }
                    UpdatesSyncMode::Slow => from + 1,
                };

                info!(
                    from = from,
                    to = to,
                    "[Updates]: indexing block heights {}...{}",
                    from,
                    to
                );

                let sorted_updates = indexer.fetch_sorted_updates(start, to).await?;

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
    /// Spawn task that continuously looks for new on-chain messages and stores
    /// them in db. Indexing messages should ALWAYS be done with a timelag, as
    /// ordering of messages is not guaranteed like it is for updates. Running
    /// without a timelag could cause messages with the incorrectly ordered
    /// index to be stored.
    pub fn sync_messages(&self) -> Instrumented<tokio::task::JoinHandle<color_eyre::Result<()>>> {
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

        let config_from = self.from_height;
        let chunk_size = self.chunk_size;

        tokio::spawn(async move {
            let mut from = db
                .retrieve_message_latest_block_end()
                .map_or_else(|| config_from, |h| h + 1);

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

                info!(
                    from = from,
                    to = to,
                    "[Messages]: indexing block heights {}...{}",
                    from,
                    to
                );

                let sorted_messages = indexer.fetch_sorted_messages(from, to).await?;

                // If no messages found, update last seen block and next height
                // and continue
                if sorted_messages.is_empty() {
                    db.store_message_latest_block_end(to)?;
                    from = to + 1;
                    continue;
                }

                // Store messages
                db.store_messages(&sorted_messages)?;

                // Report amount of messages stored into db
                stored_messages.add(sorted_messages.len().try_into()?);

                // Move forward next height
                db.store_message_latest_block_end(to)?;
                from = to + 1;
            }
        })
        .instrument(span)
    }
}

// #[cfg(test)]
// mod test {
//     use mockall::*;
//     use nomad_test::mocks::MockIndexer;

//     use std::sync::Arc;

//     use ethers::core::types::H256;
//     use ethers::signers::LocalWallet;

//     use nomad_core::{
//         Encode, NomadMessage, RawCommittedMessage, SignedUpdateWithMeta, Update, UpdateMeta,
//     };
//     use nomad_test::test_utils;

//     use super::*;
//     use crate::CoreMetrics;

//     #[tokio::test]
//     async fn handles_missing_rpc_updates() {
//         test_utils::run_test_db(|db| async move {
//             let signer: LocalWallet =
//                 "1111111111111111111111111111111111111111111111111111111111111111"
//                     .parse()
//                     .unwrap();

//             let first_root = H256::from([0; 32]);
//             let second_root = H256::from([1; 32]);
//             let third_root = H256::from([2; 32]);
//             let fourth_root = H256::from([3; 32]);
//             let fifth_root = H256::from([4; 32]);

//             let first_update = Update {
//                 home_domain: 1,
//                 previous_root: first_root,
//                 new_root: second_root,
//             }
//             .sign_with(&signer)
//             .await
//             .expect("!sign");

//             let second_update = Update {
//                 home_domain: 1,
//                 previous_root: second_root,
//                 new_root: third_root,
//             }
//             .sign_with(&signer)
//             .await
//             .expect("!sign");

//             let third_update = Update {
//                 home_domain: 1,
//                 previous_root: third_root,
//                 new_root: fourth_root,
//             }
//             .sign_with(&signer)
//             .await
//             .expect("!sign");

//             let fourth_update = Update {
//                 home_domain: 1,
//                 previous_root: fourth_root,
//                 new_root: fifth_root,
//             }
//             .sign_with(&signer)
//             .await
//             .expect("!sign");

//             let mut mock_indexer = MockIndexer::new();
//             {
//                 let mut seq = Sequence::new();

//                 let first_update_with_meta = SignedUpdateWithMeta {
//                     signed_update: first_update.clone(),
//                     metadata: UpdateMeta {
//                         block_number: 5,
//                         timestamp: Default::default(),
//                     },
//                 };

//                 let second_update_with_meta = SignedUpdateWithMeta {
//                     signed_update: second_update.clone(),
//                     metadata: UpdateMeta {
//                         block_number: 15,
//                         timestamp: Default::default(),
//                     },
//                 };
//                 let second_update_with_meta_clone = second_update_with_meta.clone();

//                 let third_update_with_meta = SignedUpdateWithMeta {
//                     signed_update: third_update.clone(),
//                     metadata: UpdateMeta {
//                         block_number: 15,
//                         timestamp: Default::default(),
//                     },
//                 };

//                 let fourth_update_with_meta = SignedUpdateWithMeta {
//                     signed_update: fourth_update.clone(),
//                     metadata: UpdateMeta {
//                         block_number: 35,
//                         timestamp: Default::default(),
//                     },
//                 };
//                 let fourth_update_with_meta_clone_1 = fourth_update_with_meta.clone();
//                 let fourth_update_with_meta_clone_2 = fourth_update_with_meta.clone();

//                 // Return first update
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_updates()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![first_update_with_meta]));

//                 // Return second update, misses third update
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_updates()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![second_update_with_meta]));

//                 // Next block range is empty updates
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_updates()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![]));

//                 // second --> fourth update seen as invalid
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_updates()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![fourth_update_with_meta]));

//                 // Indexer goes back and tries empty block range
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_updates()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![]));

//                 // Indexer tries to move on to realized missing block range but
//                 // can't
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_updates()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![fourth_update_with_meta_clone_1]));

//                 // Indexer goes back further and gets missing third update
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_updates()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| {
//                         Ok(vec![second_update_with_meta_clone, third_update_with_meta])
//                     });

//                 // Reindexes the empty block range
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_updates()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![]));

//                 // Return fourth update
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_updates()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![fourth_update_with_meta_clone_2]));

//                 // Return empty vec for remaining calls
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_updates()
//                     .return_once(move |_, _| Ok(vec![]));
//             }

//             let nomad_db = NomadDB::new("home_1", db);
//             let chunk_size = 10;
//             let from_height = 0;

//             let indexer = Arc::new(mock_indexer);
//             let metrics = Arc::new(
//                 CoreMetrics::new(
//                     "contract_sync_test",
//                     None,
//                     Arc::new(prometheus::Registry::new()),
//                 )
//                 .expect("could not make metrics"),
//             );

//             let sync_metrics = ContractSyncMetrics::new(metrics);

//             let contract_sync = ContractSync::new(
//                 "agent".to_owned(),
//                 "home_1".to_owned(),
//                 nomad_db.clone(),
//                 indexer.clone(),
//                 from_height,
//                 chunk_size,
//                 sync_metrics,
//             );

//             let sync_task = contract_sync.sync_updates();
//             sleep(Duration::from_secs(3)).await;
//             cancel_task!(sync_task);

//             assert_eq!(
//                 nomad_db
//                     .update_by_previous_root(first_root)
//                     .expect("!db")
//                     .expect("!update"),
//                 first_update.clone()
//             );
//             assert_eq!(
//                 nomad_db
//                     .update_by_previous_root(second_root)
//                     .expect("!db")
//                     .expect("!update"),
//                 second_update.clone()
//             );
//             assert_eq!(
//                 nomad_db
//                     .update_by_previous_root(third_root)
//                     .expect("!db")
//                     .expect("!update"),
//                 third_update.clone()
//             );
//             assert_eq!(
//                 nomad_db
//                     .update_by_previous_root(fourth_root)
//                     .expect("!db")
//                     .expect("!update"),
//                 fourth_update.clone()
//             );
//         })
//         .await
//     }

//     #[tokio::test]
//     async fn handles_missing_rpc_messages() {
//         test_utils::run_test_db(|db| async move {
//             let first_root = H256::from([0; 32]);
//             let second_root = H256::from([1; 32]);
//             let third_root = H256::from([2; 32]);

//             let mut message_vec = vec![];
//             NomadMessage {
//                 origin: 1000,
//                 destination: 2000,
//                 sender: H256::from([10; 32]),
//                 nonce: 1,
//                 recipient: H256::from([11; 32]),
//                 body: [10u8; 5].to_vec(),
//             }
//             .write_to(&mut message_vec)
//             .expect("!write_to");

//             let first_message = RawCommittedMessage {
//                 leaf_index: 0,
//                 committed_root: first_root,
//                 message: message_vec.clone(),
//             };

//             let second_message = RawCommittedMessage {
//                 leaf_index: 1,
//                 committed_root: second_root,
//                 message: message_vec.clone(),
//             };
//             let second_message_clone = second_message.clone();

//             let third_message = RawCommittedMessage {
//                 leaf_index: 2,
//                 committed_root: second_root,
//                 message: message_vec.clone(),
//             };

//             let fourth_message = RawCommittedMessage {
//                 leaf_index: 3,
//                 committed_root: third_root,
//                 message: message_vec.clone(),
//             };
//             let fourth_message_clone_1 = fourth_message.clone();
//             let fourth_message_clone_2 = fourth_message.clone();

//             let mut mock_indexer = MockIndexer::new();
//             {
//                 let mut seq = Sequence::new();

//                 // Return first message
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_messages()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![first_message.clone()]));

//                 // Return second message, misses third message
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_messages()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![second_message]));

//                 // Next block range is empty updates
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_messages()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![]));

//                 // second --> fourth message seen as invalid
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_messages()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![fourth_message]));

//                 // Indexer goes back and tries empty block range
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_messages()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![]));

//                 // Indexer tries to move on to realized missing block range but
//                 // can't
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_messages()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![fourth_message_clone_1]));

//                 // Indexer goes back further and gets missing third message
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_messages()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![second_message_clone, third_message]));

//                 // Reindexes empty block range
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_messages()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![]));

//                 // Return fourth message
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_messages()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(move |_, _| Ok(vec![fourth_message_clone_2]));

//                 // Return empty vec for remaining calls
//                 mock_indexer
//                     .expect__get_block_number()
//                     .times(1)
//                     .in_sequence(&mut seq)
//                     .return_once(|| Ok(100));
//                 mock_indexer
//                     .expect__fetch_sorted_messages()
//                     .return_once(move |_, _| Ok(vec![]));
//             }

//             let nomad_db = NomadDB::new("home_1", db);
//             let chunk_size = 10;
//             let from_height = 0;

//             let indexer = Arc::new(mock_indexer);
//             let metrics = Arc::new(
//                 CoreMetrics::new(
//                     "contract_sync_test",
//                     None,
//                     Arc::new(prometheus::Registry::new()),
//                 )
//                 .expect("could not make metrics"),
//             );

//             let sync_metrics = ContractSyncMetrics::new(metrics);

//             let contract_sync = ContractSync::new(
//                 "agent".to_owned(),
//                 "home_1".to_owned(),
//                 nomad_db.clone(),
//                 indexer.clone(),
//                 from_height,
//                 chunk_size,
//                 sync_metrics,
//             );

//             let sync_task = contract_sync.sync_messages();
//             sleep(Duration::from_secs(3)).await;
//             cancel_task!(sync_task);

//             assert!(nomad_db.message_by_leaf_index(0).expect("!db").is_some());
//             assert!(nomad_db.message_by_leaf_index(1).expect("!db").is_some());
//             assert!(nomad_db.message_by_leaf_index(2).expect("!db").is_some());
//             assert!(nomad_db.message_by_leaf_index(3).expect("!db").is_some());
//         })
//         .await
//     }
// }
