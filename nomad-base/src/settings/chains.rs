use color_eyre::Result;
use nomad_core::{ContractLocator, Signers};
use nomad_ethereum::{make_conn_manager, make_home, make_replica};
use nomad_types::NomadIdentifier;
use nomad_xyz_configuration::{
    contracts::CoreContracts, ethereum, AgentSecrets, ChainConf, ConnectionManagerGasLimits,
    HomeGasLimits, NomadConfig, ReplicaGasLimits,
};
use serde::Deserialize;

use crate::{
    home::Homes, replica::Replicas, xapp::ConnectionManagers, HomeVariants, ReplicaVariants,
};

/// Chain specific page settings for indexing
#[derive(Clone, Debug, Deserialize, Default)]
pub struct PageSettings {
    /// What block to start indexing at
    pub from: u32,
    /// Index page size
    pub page_size: u32,
}

/// What type of chain setup you are retrieving
#[derive(Debug, Clone)]
pub enum ChainSetupType<'a> {
    /// Home
    Home {
        /// Home network
        home_network: &'a str,
    },
    /// Replica
    Replica {
        /// Home network
        home_network: &'a str,
        /// Remote network
        remote_network: &'a str,
    },
    /// Connection manager
    ConnectionManager {
        /// Remote network
        remote_network: &'a str,
    },
}

/// A chain setup is a domain ID, an address on that chain (where the home or
/// replica is deployed) and details for connecting to the chain API.
#[derive(Clone, Debug, Deserialize, Default)]
pub struct ChainSetup {
    /// Chain name
    pub name: String,
    /// Chain domain identifier
    pub domain: u32,
    /// Address of contract on the chain
    pub address: NomadIdentifier,
    /// Paging settings
    pub page_settings: PageSettings,
    /// Network specific finality in blocks
    pub finality: u8,
    /// Network specific block time in seconds
    pub block_time: u64,
    /// The chain connection details
    #[serde(flatten)]
    pub chain: ChainConf,
    /// Set this key to disable the replica. Does nothing for homes.
    #[serde(default)]
    pub disabled: Option<String>,
}

impl ChainSetup {
    /// Instatiate ChainSetup from NomadConfig
    pub fn from_config_and_secrets(
        setup_type: ChainSetupType,
        config: &NomadConfig,
        secrets: &AgentSecrets,
    ) -> Self {
        let resident_network: String = match &setup_type {
            ChainSetupType::Home { home_network } => home_network,
            ChainSetupType::Replica { remote_network, .. } => remote_network,
            ChainSetupType::ConnectionManager { remote_network } => remote_network,
        }
        .to_string();

        let domain = config
            .protocol()
            .get_network(resident_network.clone().into())
            .expect("!domain");
        let domain_number = domain.domain;
        let finality = domain.specs.finalization_blocks;
        let block_time = domain.specs.block_time;
        let core = config.core().get(&resident_network).expect("!core");
        let (address, page_settings) = match core {
            CoreContracts::Evm(core) => {
                let address = match &setup_type {
                    ChainSetupType::Home { .. } => core.home.proxy,
                    ChainSetupType::Replica { home_network, .. } => {
                        core.replicas
                            .get(&home_network.to_string())
                            .expect("!replica")
                            .proxy
                    }
                    ChainSetupType::ConnectionManager { .. } => core.x_app_connection_manager,
                };

                let page_settings = PageSettings {
                    from: core.deploy_height,
                    page_size: domain.specs.index_page_size,
                };

                (address, page_settings)
            }
        };

        let chain = secrets
            .rpcs
            .get(&resident_network)
            .expect("!rpc")
            .to_owned();

        Self {
            name: resident_network,
            domain: domain_number,
            address,
            page_settings,
            finality,
            block_time,
            chain,
            disabled: None,
        }
    }

    /// Try to convert the chain setting into a Home contract
    pub async fn try_into_home(
        &self,
        submitter_conf: TransactionSubmitterConf,
        timelag: Option<u8>,
        gas: Option<HomeGasLimits>,
    ) -> Result<Homes> {
        match &self.chain {
            ChainConf::Ethereum(conf) => {
                let submitter_conf: ethereum::TransactionSubmitterConf = submitter_conf.into();
                // let submitter =

                Ok(HomeVariants::Ethereum(
                    make_home(
                        conf.clone(),
                        &ContractLocator {
                            name: self.name.clone(),
                            domain: self.domain,
                            address: self.address,
                        },
                        signer,
                        timelag,
                        gas,
                    )
                    .await?,
                )
                .into())
            }
        }
    }

    /// Try to convert the chain setting into a replica contract
    pub async fn try_into_replica(
        &self,
        signer: Option<Signers>,
        gas: Option<ReplicaGasLimits>,
    ) -> Result<Replicas> {
        match &self.chain {
            ChainConf::Ethereum(conf) => Ok(ReplicaVariants::Ethereum(
                make_replica(
                    conf.clone(),
                    &ContractLocator {
                        name: self.name.clone(),
                        domain: self.domain,
                        address: self.address,
                    },
                    signer,
                    None, // never need timelag for replica
                    gas,
                )
                .await?,
            )
            .into()),
        }
    }

    /// Try to convert chain setting into XAppConnectionManager contract
    pub async fn try_into_connection_manager(
        &self,
        signer: Option<Signers>,
        gas: Option<ConnectionManagerGasLimits>,
    ) -> Result<ConnectionManagers> {
        match &self.chain {
            ChainConf::Ethereum(conf) => Ok(ConnectionManagers::Ethereum(
                make_conn_manager(
                    conf.clone(),
                    &ContractLocator {
                        name: self.name.clone(),
                        domain: self.domain,
                        address: self.address,
                    },
                    signer,
                    None, // Never need timelag for xapp connection manager
                    gas,
                )
                .await?,
            )),
        }
    }
}
