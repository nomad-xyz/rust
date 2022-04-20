//! Settings and configuration for Nomad agents
//!
//! This crate draws heavily on `nomad-xyz-configuration`. All public values are
//! drawn from this publicly hosted package. All secret values are drawn from
//! either a secrets.json file (see secrets.rs for more info) or a hosted
//! secrets manager backend.
//!
//! Agent Deployment Flow:
//!  1. Run /nomad-base/src/bin/secrets_template.rs, passing in RUN_ENV
//!     environment variable (RUN_ENV=<development | production> env cargo run
//!     --bin secrets-template). This will create a secrets.json template for
//!     the given RUN_ENV in the current directory.
//!  2. Override template secrets.json values (rpcs, tx signers, optional
//!     attestation signer) with environment variables.
//!  3. Run agents, passing in RUN_ENV and AGENT_HOME as environment variables.

use crate::{
    agent::AgentCore, CachingHome, CachingReplica, CommonIndexerVariants, CommonIndexers,
    ContractSync, ContractSyncMetrics, HomeIndexerVariants, HomeIndexers, Homes, NomadDB, Replicas,
};
use color_eyre::{eyre::bail, Report};
use nomad_core::{db::DB, Common, ContractLocator, Signers};
use nomad_ethereum::{make_home_indexer, make_replica_indexer};
use nomad_xyz_configuration::{agent::SignerConf, AgentSecrets};
use nomad_xyz_configuration::{contracts::CoreContracts, ChainConf, NomadConfig};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

/// Chain configuration
pub mod chains;
pub use chains::{ChainSetup, ChainSetupType};

/// Macros
pub mod macros;
pub use macros::*;

/// Tracing subscriber management
pub mod trace;

use nomad_xyz_configuration::agent::LogConfig;

/// Agent types
pub enum AgentType {
    /// Kathy
    Kathy,
    /// Updater
    Updater,
    /// Relayer
    Relayer,
    /// Processor
    Processor,
    /// Watcher
    Watcher,
}

/// Index data types and timelag settings
#[derive(serde::Deserialize, Debug, PartialEq, Clone)]
pub enum IndexDataTypes {
    /// Updates
    Updates,
    /// Updates and messages
    UpdatesAndMessages,
}

impl Default for IndexDataTypes {
    fn default() -> Self {
        Self::Updates
    }
}

/// Home indexing settings
#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IndexSettings {
    /// Data types to index
    #[serde(default)]
    pub data_types: IndexDataTypes,
    /// Whether or not to use timelag
    #[serde(default)]
    pub use_timelag: bool,
}

impl IndexSettings {
    /// Get agent-specific index settings unique to that agent
    pub fn from_agent_name(agent_name: &str) -> Self {
        match agent_name.to_lowercase().as_ref() {
            "kathy" => Self {
                data_types: IndexDataTypes::Updates,
                use_timelag: true,
            },
            "updater" => Self {
                data_types: IndexDataTypes::Updates,
                use_timelag: true,
            },
            "relayer" => Self {
                data_types: IndexDataTypes::Updates,
                use_timelag: false,
            },
            "processor" => Self {
                data_types: IndexDataTypes::UpdatesAndMessages,
                use_timelag: true,
            },
            "watcher" => Self {
                data_types: IndexDataTypes::Updates,
                use_timelag: false,
            },
            _ => std::panic!("Invalid agent-specific settings name!"),
        }
    }

    /// Get IndexDataTypes
    pub fn data_types(&self) -> IndexDataTypes {
        self.data_types.clone()
    }

    /// Get timelag on/off status
    pub fn timelag_on(&self) -> bool {
        self.use_timelag
    }
}

/// Settings. Usually this should be treated as a base config and used as
/// follows:
///
/// ```
/// use nomad_base::*;
/// use serde::Deserialize;
///
/// pub struct OtherSettings { /* anything */ };
///
/// #[derive(Debug, Deserialize)]
/// pub struct MySettings {
///     #[serde(flatten)]
///     base_settings: Settings,
///     #[serde(flatten)]
///     other_settings: (),
/// }
///
/// // Make sure to define MySettings::new()
/// impl MySettings {
///     fn new() -> Self {
///         unimplemented!()
///     }
/// }
/// ```
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// The path to use for the DB file
    pub db: String,
    /// Port to listen for prometheus scrape requests
    pub metrics: Option<u16>,
    /// Settings for the home indexer
    #[serde(default)]
    pub index: IndexSettings,
    /// The home configuration
    pub home: ChainSetup,
    /// The replica configurations
    pub replicas: HashMap<String, ChainSetup>,
    /// Optional connection manager configurations (set for watcher only)
    pub managers: Option<HashMap<String, ChainSetup>>,
    /// The tracing configuration
    pub logging: LogConfig,
    /// Transaction signers
    pub signers: HashMap<String, SignerConf>,
    /// Optional attestation signer
    pub attestation_signer: Option<SignerConf>,
}

impl Settings {
    /// Private to preserve linearity of AgentCore::from_settings -- creating an agent consumes the settings.
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            metrics: self.metrics,
            index: self.index.clone(),
            home: self.home.clone(),
            replicas: self.replicas.clone(),
            managers: self.managers.clone(),
            logging: self.logging,
            signers: self.signers.clone(),
            attestation_signer: self.attestation_signer.clone(),
        }
    }
}

impl Settings {
    /// Try to get a signer instance by name
    pub async fn get_signer(&self, name: &str) -> Option<Signers> {
        Signers::try_from_signer_conf(self.signers.get(name)?)
            .await
            .ok()
    }

    /// Set agent-specific index data types
    pub fn set_index_data_types(&mut self, data_types: IndexDataTypes) {
        self.index.data_types = data_types;
    }

    /// Set agent-specific timelag on/off
    pub fn set_use_timelag(&mut self, use_timelag: bool) {
        self.index.use_timelag = use_timelag;
    }

    /// Get optional indexing timelag enum for home
    pub fn home_timelag(&self) -> Option<u8> {
        if self.index.timelag_on() {
            Some(self.home.finality)
        } else {
            None
        }
    }

    /// Get optional indexing timelag for a replica
    pub fn replica_timelag(&self, replica_name: &str) -> Option<u8> {
        if self.index.timelag_on() {
            let replica_finality = self.replicas.get(replica_name).expect("!replica").finality;
            Some(replica_finality)
        } else {
            None
        }
    }

    /// Try to get a Homes object
    pub async fn try_home(&self) -> Result<Homes, Report> {
        let signer = self.get_signer(&self.home.name).await;
        let opt_home_timelag = self.home_timelag();
        self.home.try_into_home(signer, opt_home_timelag).await
    }

    /// Try to get a home ContractSync
    pub async fn try_home_contract_sync(
        &self,
        agent_name: &str,
        db: DB,
        metrics: ContractSyncMetrics,
    ) -> Result<ContractSync<HomeIndexers>, Report> {
        let finality = self.home.finality;
        let index_settings = self.index.clone();
        let page_settings = self.home.page_settings.clone();

        let indexer = Arc::new(self.try_home_indexer().await?);
        let home_name = &self.home.name;

        let nomad_db = NomadDB::new(&home_name, db);

        Ok(ContractSync::new(
            agent_name.to_owned(),
            home_name.to_owned(),
            nomad_db,
            indexer,
            index_settings,
            page_settings,
            finality,
            metrics,
        ))
    }

    /// Try to get a CachingHome object
    pub async fn try_caching_home(
        &self,
        agent_name: &str,
        db: DB,
        metrics: ContractSyncMetrics,
    ) -> Result<CachingHome, Report> {
        let home = self.try_home().await?;
        let contract_sync = self
            .try_home_contract_sync(agent_name, db.clone(), metrics)
            .await?;
        let nomad_db = NomadDB::new(home.name(), db);

        Ok(CachingHome::new(home, contract_sync, nomad_db))
    }

    /// Try to get a Replicas object
    pub async fn try_replica(&self, replica_name: &str) -> Result<Replicas, Report> {
        let replica_setup = self.replicas.get(replica_name).expect("!replica");
        let signer = self.get_signer(replica_name).await;
        let opt_replica_timelag = self.replica_timelag(replica_name);

        replica_setup
            .try_into_replica(signer, opt_replica_timelag)
            .await
    }

    /// Try to get a replica ContractSync
    pub async fn try_replica_contract_sync(
        &self,
        replica_name: &str,
        agent_name: &str,
        db: DB,
        metrics: ContractSyncMetrics,
    ) -> Result<ContractSync<CommonIndexers>, Report> {
        let replica_setup = self.replicas.get(replica_name).expect("!replica");

        let finality = self.replicas.get(replica_name).expect("!replica").finality;
        let index_settings = self.index.clone();
        let page_settings = replica_setup.page_settings.clone();

        let indexer = Arc::new(self.try_replica_indexer(replica_setup).await?);
        let replica_name = &replica_setup.name;

        let nomad_db = NomadDB::new(&replica_name, db);

        Ok(ContractSync::new(
            agent_name.to_owned(),
            replica_name.to_owned(),
            nomad_db,
            indexer,
            index_settings,
            page_settings,
            finality,
            metrics,
        ))
    }

    /// Try to get a CachingReplica object
    pub async fn try_caching_replica(
        &self,
        replica_name: &str,
        agent_name: &str,
        db: DB,
        metrics: ContractSyncMetrics,
    ) -> Result<CachingReplica, Report> {
        let replica = self.try_replica(replica_name).await?;
        let contract_sync = self
            .try_replica_contract_sync(replica_name, agent_name, db.clone(), metrics)
            .await?;
        let nomad_db = NomadDB::new(replica.name(), db);

        Ok(CachingReplica::new(replica, contract_sync, nomad_db))
    }

    /// Try to get all replicas from this settings object
    pub async fn try_caching_replicas(
        &self,
        agent_name: &str,
        db: DB,
        metrics: ContractSyncMetrics,
    ) -> Result<HashMap<String, Arc<CachingReplica>>, Report> {
        let mut result = HashMap::default();
        for (k, v) in self.replicas.iter().filter(|(_, v)| v.disabled.is_none()) {
            if k != &v.name {
                bail!(
                    "Replica key does not match replica name:\n key: {}  name: {}",
                    k,
                    v.name
                );
            }

            let caching_replica = self
                .try_caching_replica(k, agent_name, db.clone(), metrics.clone())
                .await?;
            result.insert(v.name.clone(), Arc::new(caching_replica));
        }
        Ok(result)
    }

    /// Try to get an indexer object for a home. Note that indexers are NOT
    /// instantiated with a built in timelag. The timelag is handled by the
    /// ContractSync.
    pub async fn try_home_indexer(&self) -> Result<HomeIndexers, Report> {
        let timelag = self.home_timelag();

        match &self.home.chain {
            ChainConf::Ethereum(conn) => Ok(HomeIndexerVariants::Ethereum(
                make_home_indexer(
                    conn.clone(),
                    &ContractLocator {
                        name: self.home.name.clone(),
                        domain: self.home.domain,
                        address: self.home.address,
                    },
                    timelag,
                    self.home.page_settings.from,
                    self.home.page_settings.page_size,
                )
                .await?,
            )
            .into()),
        }
    }

    /// Try to get an indexer object for a replica. Note that indexers are NOT
    /// instantiated with a built in timelag. The timelag is handled by the
    /// ContractSync.
    pub async fn try_replica_indexer(&self, setup: &ChainSetup) -> Result<CommonIndexers, Report> {
        let timelag = self.replica_timelag(&setup.name);

        match &setup.chain {
            ChainConf::Ethereum(conn) => Ok(CommonIndexerVariants::Ethereum(
                make_replica_indexer(
                    conn.clone(),
                    &ContractLocator {
                        name: setup.name.clone(),
                        domain: setup.domain,
                        address: setup.address,
                    },
                    timelag,
                    setup.page_settings.from,
                    setup.page_settings.page_size,
                )
                .await?,
            )
            .into()),
        }
    }

    /// Try to generate an agent core for a named agent
    pub async fn try_into_core(&self, name: &str) -> Result<AgentCore, Report> {
        let metrics = Arc::new(crate::metrics::CoreMetrics::new(
            name,
            &self.home.name,
            self.metrics,
            Arc::new(prometheus::Registry::new()),
        )?);
        let sync_metrics = ContractSyncMetrics::new(metrics.clone());

        let db = DB::from_path(&self.db)?;
        let home = Arc::new(
            self.try_caching_home(name, db.clone(), sync_metrics.clone())
                .await?,
        );
        let replicas = self
            .try_caching_replicas(name, db.clone(), sync_metrics.clone())
            .await?;

        Ok(AgentCore {
            home,
            replicas,
            db,
            settings: self.clone(),
            metrics,
            indexer: self.index.clone(),
        })
    }

    /// Instantiate Settings block from NomadConfig
    pub fn from_config_and_secrets(
        agent_name: &str,
        home_network: &str,
        config: &NomadConfig,
        secrets: &AgentSecrets,
    ) -> Self {
        let agent = config.agent().get(home_network).expect("!agent config");

        let db = agent.db.to_str().expect("!db").to_owned();
        let metrics = agent.metrics;
        let index = IndexSettings::from_agent_name(agent_name);

        let home = ChainSetup::from_config_and_secrets(
            ChainSetupType::Home { home_network },
            config,
            secrets,
        );

        let connections = &config
            .protocol()
            .networks
            .get(home_network)
            .expect("!replica networks")
            .connections;
        let replicas = connections
            .iter()
            .map(|remote_network| {
                (
                    remote_network.to_owned(),
                    ChainSetup::from_config_and_secrets(
                        ChainSetupType::Replica {
                            home_network,
                            remote_network,
                        },
                        config,
                        secrets,
                    ),
                )
            })
            .collect();

        // Create connection managers if watcher
        let managers: Option<HashMap<String, ChainSetup>> =
            if agent_name.to_lowercase() == "watcher" {
                Some(
                    connections
                        .iter()
                        .map(|remote_network| {
                            (
                                remote_network.to_owned(),
                                ChainSetup::from_config_and_secrets(
                                    ChainSetupType::ConnectionManager { remote_network },
                                    config,
                                    secrets,
                                ),
                            )
                        })
                        .collect(),
                )
            } else {
                None
            };

        Self {
            db,
            metrics,
            home,
            replicas,
            managers,
            index,
            logging: agent.logging,
            signers: secrets.transaction_signers.clone(),
            attestation_signer: secrets.attestation_signer.clone(),
        }
    }

    /// Validate base config against NomadConfig and AgentSecrets blocks
    pub fn validate_against_config_and_secrets(
        &self,
        agent_name: &str,
        home_network: &str,
        config: &NomadConfig,
        secrets: &AgentSecrets,
    ) -> color_eyre::Result<()> {
        let agent = config.agent().get(home_network).unwrap();
        assert_eq!(self.db, agent.db.to_str().unwrap());
        assert_eq!(self.metrics, agent.metrics);
        assert_eq!(self.logging, agent.logging);

        let index_settings = IndexSettings::from_agent_name(agent_name);
        assert_eq!(self.index, index_settings);

        let config_home_domain = config
            .protocol()
            .get_network(home_network.to_owned().into())
            .unwrap();
        assert_eq!(self.home.name, config_home_domain.name);
        assert_eq!(self.home.domain, config_home_domain.domain);
        assert_eq!(
            self.home.page_settings.page_size,
            config_home_domain.specs.index_page_size
        );
        assert_eq!(
            self.home.finality,
            config_home_domain.specs.finalization_blocks
        );

        let config_home_core = config.core().get(home_network).unwrap();
        match config_home_core {
            CoreContracts::Evm(core) => {
                assert_eq!(self.home.address, core.home.proxy);
                assert_eq!(self.home.page_settings.from, core.deploy_height);
            }
        }

        let home_chain_conf = secrets.rpcs.get(home_network).unwrap();
        assert_eq!(&self.home.chain, home_chain_conf);

        let home_connections = &config
            .protocol()
            .networks
            .get(home_network)
            .expect("!networks")
            .connections;
        for remote_network in home_connections {
            let replica_setup = self.replicas.get(remote_network).unwrap();
            let config_replica_domain = config
                .protocol()
                .get_network(remote_network.to_owned().into())
                .unwrap();

            assert_eq!(replica_setup.name, config_replica_domain.name);
            assert_eq!(replica_setup.domain, config_replica_domain.domain);
            assert_eq!(
                replica_setup.page_settings.page_size,
                config_replica_domain.specs.index_page_size
            );
            assert_eq!(
                replica_setup.finality,
                config_replica_domain.specs.finalization_blocks
            );

            let config_replica_core = config.core().get(remote_network).unwrap();
            match config_replica_core {
                CoreContracts::Evm(core) => {
                    assert_eq!(
                        replica_setup.address,
                        core.replicas.get(home_network).unwrap().proxy
                    );
                    assert_eq!(replica_setup.page_settings.from, core.deploy_height);
                }
            }

            let replica_chain_conf = secrets.rpcs.get(remote_network).unwrap();
            assert_eq!(&replica_setup.chain, replica_chain_conf);
        }

        for (network, signer) in self.signers.iter() {
            let secret_signer = secrets.transaction_signers.get(network).unwrap();
            assert_eq!(signer, secret_signer);
        }

        Ok(())
    }
}
