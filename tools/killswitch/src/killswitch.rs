#![allow(dead_code)] // TODO: Remove me

use crate::{errors::Error, settings::Settings, Args, Result};
use futures_util::future::join_all;
use nomad_base::{ChainSetup, ChainSetupType, ConnectionManagers, Homes};
use nomad_core::{ConnectionManager, SignedFailureNotification};
use nomad_xyz_configuration::AgentSecrets;
use std::collections::{HashMap, HashSet};

/// Main `KillSwitch` struct
#[derive(Debug)]
pub(crate) struct KillSwitch {
    /// A vector of all channels we intend to kill
    channels: Vec<Channel>,
}

/// The set of origin->destination networks and contracts
#[derive(Debug)]
struct Channel {
    /// Origin network
    home: String,
    /// Destination network
    replica: String,
    /// Home contract or encountered error
    home_contract: Option<Result<Homes>>,
    /// Connection manager or encountered error
    connection_manager: Option<Result<ConnectionManagers>>,
}

impl KillSwitch {
    /// Get all available home->network channels in config
    fn make_channels(settings: &Settings) -> Vec<Channel> {
        settings
            .config
            .protocol()
            .networks
            .iter()
            .flat_map(|(home, domain)| {
                domain.connections.iter().map(|replica| Channel {
                    home: home.clone(),
                    replica: replica.clone(),
                    home_contract: None,
                    connection_manager: None,
                })
            })
            .collect()
    }

    /// Filter channels where destination network hosts a replica
    fn make_inbound_channels(to: &String, all: Vec<Channel>) -> Vec<Channel> {
        all.into_iter().filter(|c| &c.replica == to).collect()
    }

    /// Get `ChainSetup` for network given `ChainSetupType`
    fn make_chain_setup(
        network: &String,
        setup_type: ChainSetupType,
        settings: &Settings,
    ) -> Result<ChainSetup> {
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
        Ok(ChainSetup::from_config_and_secrets(
            setup_type,
            &settings.config,
            &secrets,
        ))
    }

    /// Make `Homes` or return error
    async fn make_home(channel: &Channel, settings: &Settings) -> Result<Homes> {
        let setup_type = ChainSetupType::Home {
            home_network: &channel.home,
        };
        let chain_setup = Self::make_chain_setup(&channel.home, setup_type, settings)?;
        let submitter_config = settings.tx_submitters.get(&channel.home).ok_or_else(|| {
            Error::MissingTxSubmitter(format!(
                "No tx submitter config found for {}",
                &channel.home
            ))
        })?;
        chain_setup
            .try_into_home(Some(submitter_config.clone()), None, None)
            .await
            .map_err(|report| {
                Error::ConnectionManagerInit(format!(
                    "Connection manager init failed: {:?}",
                    report
                ))
            })
    }

    /// Make `ConnectionManagers` or return error
    async fn make_connection_manager(
        channel: &Channel,
        settings: &Settings,
    ) -> Result<ConnectionManagers> {
        let setup_type = ChainSetupType::ConnectionManager {
            remote_network: &channel.replica,
        };
        let chain_setup = Self::make_chain_setup(&channel.replica, setup_type, settings)?;
        let submitter_config = settings
            .tx_submitters
            .get(&channel.replica)
            .ok_or_else(|| {
                Error::MissingTxSubmitter(format!(
                    "No tx submitter config found for {}",
                    &channel.replica
                ))
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

    /// Build a new `KillSwitch`, configuring best effort and storing, not returning errors
    pub(crate) async fn new(args: Args, settings: Settings) -> Result<Self> {
        let channels = if args.all {
            Self::make_channels(&settings)
        } else {
            let destination_network = args
                .all_inbound
                .expect("Should not happen. Clap requires this to be present");
            let all = Self::make_channels(&settings);
            Self::make_inbound_channels(&destination_network, all)
        };
        if channels.is_empty() {
            return Err(Error::NoNetworks(format!(
                "No available networks in config to disconnect"
            )));
        }

        let futs = channels.into_iter().map(|channel| async {
            let home_contract = Self::make_home(&channel, &settings).await;
            let connection_manager = Self::make_connection_manager(&channel, &settings).await;
            Channel {
                home: channel.home,
                replica: channel.replica,
                home_contract: Some(home_contract),
                connection_manager: Some(connection_manager),
            }
        });
        let channels = join_all(futs).await.into_iter().collect::<Vec<_>>();

        Ok(Self { channels })
    }

    /// Run `KillSwitch` against configuration
    pub(crate) async fn run(&self) {
        unimplemented!()
    }

    /// Create a `SignedFailureNotification`
    async fn create_signed_failure(&self) -> Result<SignedFailureNotification> {
        unimplemented!()
    }
}
