#![allow(dead_code)] // TODO: Remove me

use crate::{errors::Error, settings::Settings, Args, Result};
use futures_util::future::join_all;
use nomad_base::{ChainSetup, ChainSetupType, ConnectionManagers};
use nomad_core::SignedFailureNotification;
use nomad_xyz_configuration::AgentSecrets;
use std::collections::{HashMap, HashSet};

/// Main `KillSwitch` struct
#[derive(Debug)]
pub(crate) struct KillSwitch {
    /// Set of replicas by network we're disconnecting
    /// or errors encountered attempting to configure replicas
    replicas: HashMap<String, Result<HashSet<String>>>,
    /// Connection managers by network / replica pair or
    /// error encountered from missing config or during initialization
    connection_managers: HashMap<NetworkReplicaPair, Result<ConnectionManagers>>,
}

/// A hashable pair of network and replica
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct NetworkReplicaPair {
    /// The network
    network: String,
    /// The replica
    replica: String,
}

impl KillSwitch {
    /// Get replicas for network, returning errors for a missing network or an empty replica set
    fn get_replicas(network: &String, settings: &Settings) -> Result<HashSet<String>> {
        let core_contracts = settings.config.core().get(network).ok_or_else(|| {
            Error::MissingNetwork(format!("Network {} was not found in core", network))
        })?;
        let replicas = core_contracts
            .replicas()
            .map(Clone::clone)
            .collect::<HashSet<String>>();
        if replicas.is_empty() {
            return Err(Error::MissingReplicas(format!(
                "No replicas found for {} in core",
                network
            )));
        }
        Ok(replicas)
    }

    /// Configure connection manager, returning errors for missing config or bad init
    async fn get_connection_manager(
        network: &String,
        settings: &Settings,
    ) -> Result<ConnectionManagers> {
        let chain_setup_type = ChainSetupType::ConnectionManager {
            remote_network: network, // resident network
        };
        // `from_config_and_secrets` will panic here, return an error instead
        if settings.rpcs.get(network).is_none() {
            return Err(Error::MissingRPC(format!(
                "No rpc config found for {}",
                network
            )));
        }
        // We just need the rpc here
        let secrets = AgentSecrets {
            rpcs: settings.rpcs.clone(),
            tx_submitters: Default::default(),
            attestation_signer: None,
        };
        let chain_setup =
            ChainSetup::from_config_and_secrets(chain_setup_type, &settings.config, &secrets);
        let submitter_config = settings.tx_submitters.get(network).ok_or_else(|| {
            Error::MissingTxSubmitter(format!("No tx submitter config found for {}", network))
        })?;
        chain_setup
            .try_into_connection_manager(Some(submitter_config.clone()), None)
            .await
            .map_err(|report| {
                Error::ConnectionManagerInit(format!(
                    "Connection manager init failed: {:?}",
                    report
                ))
            })
    }

    /// Create a `SignedFailureNotification`
    async fn create_signed_failure(&self) -> Result<SignedFailureNotification> {
        unimplemented!()
    }

    /// Build a new `KillSwitch`, configuring best effort and storing, not returning errors
    pub(crate) async fn new(args: Args, settings: Settings) -> Self {
        let networks = if args.all {
            Vec::from_iter(settings.config.networks.clone())
        } else {
            vec![args
                .all_inbound
                .expect("Should not happen. Clap requires this to be present")]
        };

        let replicas = networks
            .iter()
            .map(|network| (network.clone(), Self::get_replicas(network, &settings)))
            .collect::<HashMap<String, Result<HashSet<String>>>>();

        let available_replicas = replicas
            .iter()
            .filter_map(|(network, replicas)| {
                replicas
                    .as_ref()
                    .map(|replicas| Some((network.clone(), replicas.clone())))
                    .ok()
                    .flatten()
            })
            .flat_map(|(network, replicas)| {
                replicas.into_iter().map(move |replica| NetworkReplicaPair {
                    network: network.clone(),
                    replica,
                })
            })
            .collect::<Vec<NetworkReplicaPair>>();

        let connection_managers = available_replicas.iter().map(|pair| async {
            (
                pair.clone(),
                Self::get_connection_manager(&pair.network, &settings).await,
            )
        });

        let connection_managers = join_all(connection_managers).await.into_iter().collect();

        Self {
            replicas,
            connection_managers,
        }
    }

    /// Run `KillSwitch` against configuration
    pub(crate) async fn run(&self) {
        unimplemented!()
    }
}
