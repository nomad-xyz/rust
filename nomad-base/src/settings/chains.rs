use color_eyre::Report;
use nomad_core::{ContractLocator, NomadIdentifier, Signers};
use nomad_ethereum::{make_conn_manager, make_home, make_replica, Connection};
use nomad_xyz_configuration::{common::NameOrDomain, contracts::CoreContracts, NomadConfig};
use serde::Deserialize;

use crate::{
    home::Homes, replica::Replicas, xapp::ConnectionManagers, HomeVariants, ReplicaVariants,
};

/// A connection to _some_ blockchain.
///
/// Specify the chain name (enum variant) in toml under the `chain` key
/// Specify the connection details as a toml object under the `connection` key.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "rpcStyle", content = "connection", rename_all = "camelCase")]
pub enum ChainConf {
    /// Ethereum configuration
    Ethereum(Connection),
}

impl Default for ChainConf {
    fn default() -> Self {
        Self::Ethereum(Default::default())
    }
}

/// Chain specific page settings for indexing
#[derive(Clone, Debug, Deserialize, Default)]
pub struct PageSettings {
    /// What block to start indexing at
    pub from: u32,
    /// Index page size
    pub page_size: u32,
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
    /// The chain connection details
    #[serde(flatten)]
    pub chain: ChainConf,
    /// Set this key to disable the replica. Does nothing for homes.
    #[serde(default)]
    pub disabled: Option<String>,
}

impl ChainSetup {
    /// Instantiate home ChainSetup from NomadConfig
    pub fn home_from_nomad_config(network: &str, config: &NomadConfig) -> Self {
        let domain = config
            .protocol()
            .get_network(network.to_owned().into())
            .expect("!domain");
        let domain_number = domain.domain.try_into().unwrap(); // TODO: fix uint
        let finality = domain.specs.finalization_blocks.try_into().unwrap(); // TODO: fix uint

        let home_core = config.core().get(network).expect("!core");
        let (address, page_settings, chain) = match home_core {
            CoreContracts::Evm(core) => {
                let address = (*core.home.proxy).into(); // TODO: fix repeated type
                let page_settings = PageSettings {
                    from: core.deploy_height.try_into().unwrap(), // TODO: fix uint
                    page_size: domain.specs.index_page_size.try_into().unwrap(), // TODO: fix uint
                };

                let chain = ChainConf::Ethereum(Connection::Http {
                    url: "TODO: get secret rpc url".into(),
                }); // TODO: draw on secrets

                (address, page_settings, chain)
            }
        };

        Self {
            name: network.to_owned(),
            domain: domain_number,
            address,
            page_settings,
            finality,
            chain,
            disabled: None,
        }
    }

    /// Instatiate replica ChainSetup from NomadConfig
    pub fn replica_from_nomad_config(
        home_network: &str,
        remote_network: &str,
        config: &NomadConfig,
    ) -> Self {
        let remote_domain = config
            .protocol()
            .get_network(NameOrDomain::Name(remote_network.to_owned()))
            .expect("!replica domain");
        let domain_number = remote_domain.domain.try_into().unwrap(); // TODO: fix uint
        let finality = remote_domain.specs.finalization_blocks.try_into().unwrap(); // TODO: fix uint

        let remote_core = config.core().get(remote_network).expect("!replica core");
        let (address, page_settings, chain) = match remote_core {
            CoreContracts::Evm(core) => {
                let address = (*config
                    .locate_replica_of(
                        home_network.to_owned().into(),
                        remote_network.to_owned().into(),
                    )
                    .expect("!replica"))
                .into(); // TODO: fix repeated type
                let page_settings = PageSettings {
                    from: core.deploy_height.try_into().unwrap(), // TODO: fix uint
                    page_size: remote_domain.specs.index_page_size.try_into().unwrap(), // TODO: fix uint
                };

                let chain = ChainConf::Ethereum(Connection::Http {
                    url: "TODO: get secret rpc url".into(),
                }); // TODO: draw on secrets

                (address, page_settings, chain)
            }
        };

        Self {
            name: remote_network.to_owned(),
            domain: domain_number,
            address,
            page_settings,
            finality,
            chain,
            disabled: None,
        }
    }

    /// Try to convert the chain setting into a Home contract
    pub async fn try_into_home(
        &self,
        signer: Option<Signers>,
        timelag: Option<u8>,
    ) -> Result<Homes, Report> {
        match &self.chain {
            ChainConf::Ethereum(conf) => Ok(HomeVariants::Ethereum(
                make_home(
                    conf.clone(),
                    &ContractLocator {
                        name: self.name.clone(),
                        domain: self.domain,
                        address: self.address,
                    },
                    signer,
                    timelag,
                )
                .await?,
            )
            .into()),
        }
    }

    /// Try to convert the chain setting into a replica contract
    pub async fn try_into_replica(
        &self,
        signer: Option<Signers>,
        timelag: Option<u8>,
    ) -> Result<Replicas, Report> {
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
                    timelag,
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
        timelag: Option<u8>,
    ) -> Result<ConnectionManagers, Report> {
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
                    timelag,
                )
                .await?,
            )),
        }
    }
}
