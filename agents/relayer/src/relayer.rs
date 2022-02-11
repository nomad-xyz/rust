use async_trait::async_trait;
use color_eyre::Result;
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
            if self.replica.update(&signed_update).await.is_ok() {
                self.updates_relayed_count.inc();
            }

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
            settings.interval.parse().expect("invalid uint"),
            settings.as_ref().try_into_core("relayer").await?,
        ))
    }

    fn build_channel(&self, replica: &str) -> Self::Channel {
        Self::Channel {
            base: self.channel_base(replica),
            updates_relayed_count: self.updates_relayed_counts.with_label_values(&[
                self.home().name(),
                replica,
                Self::AGENT_NAME,
            ]),
            interval: self.interval,
        }
    }

    #[tracing::instrument]
    fn run(channel: Self::Channel) -> Instrumented<JoinHandle<Result<()>>> {
        tokio::spawn(async move {
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

    use ethers::prelude::ProviderError;
    use ethers::types::H256;
    use nomad_base::trace::TracingConfig;
    use nomad_base::{ChainConf, SignerConf};
    use nomad_base::{
        ChainSetup, CommonIndexers, CoreMetrics, HomeIndexers, HomeVariants, IndexSettings,
        NomadDB, ReplicaVariants, Replicas,
    };
    use nomad_core::utils::HexString;
    use nomad_core::ChainCommunicationError;
    use nomad_test::mocks::{MockHomeContract, MockIndexer, MockReplicaContract};
    use nomad_test::test_utils;
    use std::collections::HashMap;
    use std::str::FromStr;

    use super::*;

    #[tokio::test]
    async fn it_isolates_faulty_channels() {
        test_utils::run_test_db(|db| async move {
            println!("kek");

            let mut h: HashMap<String, ChainSetup> = HashMap::new();
            h.insert(
                "moonbeam".to_string(),
                ChainSetup {
                    name: "moonbeam".to_string(),
                    domain: "2".to_string(),
                    address: "kek".to_string(),
                    timelag: 3,
                    chain: ChainConf::default(),
                    disabled: None,
                },
            );

            let mut h1: HashMap<String, SignerConf> = HashMap::new();
            h1.insert(
                "moonbeam".to_string(),
                SignerConf::HexKey {
                    key: HexString::from_str(
                        "1234567812345678123456781234567812345678123456781234567812345678",
                    )
                    .unwrap(),
                },
            );
            let settings = nomad_base::Settings {
                db: "...".to_string(),
                metrics: None,
                index: IndexSettings::default(),
                use_timelag: false,
                home: ChainSetup {
                    name: "ethereum".to_string(),
                    domain: "1".to_string(),
                    address: "kek".to_string(),
                    timelag: 3,
                    chain: ChainConf::default(),
                    disabled: None,
                },
                replicas: h,
                tracing: TracingConfig::default(),
                signers: h1,
            };

            println!("kek");

            // let db = NomadDB::new("replica_1", db);
            let metrics = Arc::new(
                CoreMetrics::new(
                    "contract_sync_test",
                    None,
                    Arc::new(prometheus::Registry::new()),
                )
                .expect("could not make metrics"),
            );
            // maybe mock settings?

            let home_indexer = Arc::new(MockIndexer::new().into());
            let replica_indexer = Arc::new(MockIndexer::new().into());
            let home_db = NomadDB::new("home_1", db.clone());

            let mut home = MockHomeContract::new();

            {
                home.expect__name().return_const("home_1".to_owned());
            }

            let home = CachingHome::new(home.into(), home_db.clone(), home_indexer).into();
            let mut replica = MockReplicaContract::new();
            {
                // replica
                //     .expect__committed_root()
                //     .return_once(|| Ok(H256::zero()));
                replica
                    .expect__committed_root()
                    .times(1..1000)
                    .returning(|| {
                        Err(ChainCommunicationError::ProviderError(
                            ProviderError::CustomError("KEK".to_string()),
                        ))
                    });
            }

            // let replicas = settings.try_caching_replicas(db).await.unwrap();
            let mut replicas: HashMap<String, Arc<CachingReplica>> = HashMap::new();

            replicas.insert(
                "moonbeam".to_string(),
                Arc::new(CachingReplica::new(
                    replica.into(),
                    home_db,
                    replica_indexer,
                )),
            );

            let core = AgentCore {
                home,     //Arc<CachingHome>,
                replicas, //HashMap<String, Arc<CachingReplica>>,
                db,
                metrics,
                /// The height at which to start indexing the Home
                indexer: IndexSettings::default(),
                /// Settings this agent was created with
                settings,
            };

            let agent = Relayer::new(5, core);

            match agent.run_many(&["moonbeam"]).await {
                Ok(ok) => match ok {
                    Err(e) => {
                        println!("ACTUAL EEEEE: {}", e);
                    }
                    _ => {
                        println!("ok...");
                    }
                },
                Err(e) => {
                    println!("JOIN EEEEEE: {}", e);
                }
            }
        })
        .await
    }
}
