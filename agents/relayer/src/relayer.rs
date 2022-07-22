use async_trait::async_trait;
use color_eyre::{eyre::ensure, Result};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle, time::sleep};
use tracing::{info, instrument::Instrumented, Instrument};

use nomad_base::{decl_agent, decl_channel, AgentCore, CachingHome, CachingReplica, NomadAgent};
use nomad_core::{Common, CommonEvents};

use crate::settings::RelayerSettings as Settings;

#[derive(Debug)]
struct UpdatePoller {
    interval: u64,
    home: Arc<CachingHome>,
    replica: Arc<CachingReplica>,
    semaphore: Mutex<()>,
    updates_relayed_count: prometheus::IntCounter,
}

impl std::fmt::Display for UpdatePoller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UpdatePoller: {{ home: {:?}, replica: {:?} }}",
            self.home, self.replica
        )
    }
}

impl UpdatePoller {
    fn new(
        home: Arc<CachingHome>,
        replica: Arc<CachingReplica>,
        interval: u64,
        updates_relayed_count: prometheus::IntCounter,
    ) -> Self {
        Self {
            home,
            replica,
            interval,
            semaphore: Mutex::new(()),
            updates_relayed_count,
        }
    }

    #[tracing::instrument(err, skip(self), fields(self = %self))]
    async fn poll_and_relay_update(&self) -> Result<()> {
        // Get replica's current root.
        let old_root = self.replica.committed_root().await?;
        info!(
            "Replica {} latest root is: {}",
            self.replica.name(),
            old_root
        );

        // Check for first signed update building off of the replica's current root
        let signed_update_opt = self.home.signed_update_by_old_root(old_root).await?;

        // If signed update exists for replica's committed root, try to
        // relay
        if let Some(signed_update) = signed_update_opt {
            info!(
                "Update for replica {}. Root {} to {}",
                self.replica.name(),
                &signed_update.update.previous_root,
                &signed_update.update.new_root,
            );

            // Attempt to acquire lock for submitting tx
            let lock = self.semaphore.try_lock();
            if lock.is_err() {
                return Ok(()); // tx in flight. just do nothing
            }

            // Relay update and increment counters if tx successful
            match self.replica.update(&signed_update).await {
                Ok(_) => self.updates_relayed_count.inc(),
                Err(e) => {
                    drop(lock.unwrap());
                    return Err(e.into());
                }
            };

            // lock dropped here
        } else {
            info!(
                "No update. Current root for replica {} is {}",
                self.replica.name(),
                old_root
            );
        }

        Ok(())
    }

    fn spawn(self) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            loop {
                self.poll_and_relay_update().await?;
                sleep(Duration::from_secs(self.interval)).await;
            }
        })
    }
}

decl_agent!(Relayer {
    updates_relayed_counts: prometheus::IntCounterVec,
    interval: u64,
});

#[allow(clippy::unit_arg)]
impl Relayer {
    /// Instantiate a new relayer
    pub fn new(interval: u64, core: AgentCore) -> Self {
        let updates_relayed_counts = core
            .metrics
            .new_int_counter(
                "updates_relayed_count",
                "Number of updates relayed from given home to replica",
                &["home", "replica", "agent"],
            )
            .expect("processor metric already registered -- should have be a singleton");

        Self {
            interval,
            core,
            updates_relayed_counts,
        }
    }
}

decl_channel!(Relayer {
    updates_relayed_count: prometheus::IntCounter,
    interval: u64,
});

#[async_trait]
#[allow(clippy::unit_arg)]
impl NomadAgent for Relayer {
    const AGENT_NAME: &'static str = "relayer";

    type Settings = Settings;

    type Channel = RelayerChannel;

    async fn from_settings(settings: Self::Settings) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self::new(
            settings.agent.interval,
            settings.as_ref().try_into_core("relayer").await?,
        ))
    }

    fn build_channel(&self, replica: &str) -> Self::Channel {
        let home = self.connections().home().expect("!home");
        Self::Channel {
            base: self.channel_base(replica),
            updates_relayed_count: self.updates_relayed_counts.with_label_values(&[
                home.name(),
                replica,
                Self::AGENT_NAME,
            ]),
            interval: self.interval,
        }
    }

    #[tracing::instrument]
    fn run(channel: Self::Channel) -> Instrumented<JoinHandle<Result<()>>> {
        tokio::spawn(async move {
            let home_updater = channel.home().updater().await?;
            let replica_updater = channel.replica().updater().await?;

            ensure!(
                home_updater == replica_updater,
                "Home and replica updaters do not match. Home: {:x}. Replica: {:x}.",
                home_updater,
                replica_updater
            );

            let update_poller = UpdatePoller::new(
                channel.home(),
                channel.replica(),
                channel.interval,
                channel.updates_relayed_count,
            );
            update_poller.spawn().await?
        })
        .in_current_span()
    }
}

#[cfg(test)]
mod test {
    use ethers::prelude::{ProviderError, H256};
    use nomad_base::{
        chains::PageSettings, AgentConnections, CommonIndexers, ContractSync, ContractSyncMetrics,
        CoreMetrics, HomeIndexers, IndexSettings, NomadDB,
    };
    use nomad_core::ChainCommunicationError;
    use nomad_test::mocks::{MockHomeContract, MockIndexer, MockReplicaContract};
    use nomad_test::test_utils;
    use std::collections::HashMap;
    use tokio::time::{sleep, Duration};

    use super::*;

    const AGENT_NAME: &str = "relayer";

    #[tokio::test]
    async fn run_report_error_isolates_faulty_channels() {
        test_utils::run_test_db(|db| async move {
            let channel_name = "moonbeam";

            let metrics = Arc::new(
                CoreMetrics::new(
                    "contract_sync_test",
                    "home",
                    None,
                    Arc::new(prometheus::Registry::new()),
                )
                .expect("could not make metrics"),
            );
            let sync_metrics = ContractSyncMetrics::new(metrics.clone());

            // Setting home
            let settings = nomad_base::Settings::default();
            let home_indexer: Arc<HomeIndexers> = Arc::new(MockIndexer::new().into());
            let home_db = NomadDB::new("home_1", db.clone());
            let mut home_mock = MockHomeContract::new();
            let home_sync = ContractSync::new(
                AGENT_NAME.to_owned(),
                "home_1".to_owned(),
                home_db.clone(),
                home_indexer.clone(),
                IndexSettings::default(),
                PageSettings::default(),
                Default::default(),
                sync_metrics.clone(),
            );

            {
                home_mock.expect__name().return_const("home_1".to_owned());
                home_mock
                    .expect__updater()
                    .times(..)
                    .returning(|| Ok(H256::zero()));
            }

            let home = CachingHome::new(home_mock.into(), home_sync, home_db.clone()).into();

            // Setting replica
            let mut replica_mock = MockReplicaContract::new();
            {
                replica_mock
                    .expect__updater()
                    .times(..)
                    .returning(|| Ok(H256::zero()));
                replica_mock
                    .expect__committed_root()
                    .times(..)
                    .returning(|| {
                        Err(ChainCommunicationError::ProviderError(
                            ProviderError::CustomError(
                                "I am replica and I always throw the error".to_string(),
                            ),
                        ))
                    });
            }

            let replica_indexer: Arc<CommonIndexers> = Arc::new(MockIndexer::new().into());
            let replica_db = NomadDB::new("replica_1", db.clone());
            let replica_sync = ContractSync::new(
                AGENT_NAME.to_owned(),
                "replica_1".to_owned(),
                replica_db.clone(),
                replica_indexer.clone(),
                IndexSettings::default(),
                PageSettings::default(),
                Default::default(),
                sync_metrics,
            );

            let replicas: HashMap<String, Arc<CachingReplica>> = HashMap::from([(
                channel_name.to_string(),
                Arc::new(CachingReplica::new(
                    replica_mock.into(),
                    replica_sync,
                    replica_db,
                )),
            )]);

            // Setting agent
            let core = AgentCore {
                connections: AgentConnections::Default { home, replicas },
                db,
                metrics,
                indexer: IndexSettings::default(),
                settings,
            };

            let agent = Relayer::new(2, core);

            // Sanity check that we indeed throw an error when calling run NOT
            // run_report_error
            let run_result =
                <Relayer as nomad_base::NomadAgent>::run(agent.build_channel("moonbeam"))
                    .await
                    .expect("Couldn't join relayer's run task");
            assert!(run_result.is_err(), "Must have returned error");

            let run_report_error_task = agent
                .run_report_error(channel_name.to_string())
                .into_inner();

            sleep(Duration::from_secs(3)).await;

            // Awaiting task will return error if abort cancelled task and Ok if
            // it already finished before abort. We throw error if task returned
            // Ok instead of error, as it means run task finished early. We also
            // check it was cancelled.
            run_report_error_task.abort();
            assert!(run_report_error_task.await.unwrap_err().is_cancelled());
        })
        .await
    }
}
