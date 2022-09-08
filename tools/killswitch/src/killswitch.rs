use crate::{errors::Error, settings::Settings, Args, Result};
use futures_util::future::join_all;
use nomad_base::{ChainSetup, ChainSetupType, ConnectionManagers, Homes};
use nomad_core::{
    Common, ConnectionManager, FailureNotification, Home, SignedFailureNotification, Signers,
    TxOutcome,
};
use nomad_xyz_configuration::AgentSecrets;

/// Main `KillSwitch` struct
pub(crate) struct KillSwitch {
    /// A vector of all `ChannelKiller`
    channel_killers: Vec<ChannelKiller>,
}

/// The set of origin->destination networks
#[derive(Debug, Clone)]
struct Channel {
    /// Origin network
    home: String,
    /// Destination network
    replica: String,
}

/// The channel and contracts required, or errors encountered
struct ChannelKiller {
    /// The channel we want to kill
    channel: Channel,
    /// Home contract or encountered error
    home_contract: Option<Result<Homes>>,
    /// Connection manager or encountered error
    connection_manager: Option<Result<ConnectionManagers>>,
    /// Attestation signer or encountered error
    attestation_signer: Option<Result<Signers>>,
}

impl ChannelKiller {
    /// Create a `SignedFailureNotification`
    async fn create_signed_failure(&mut self) -> Result<SignedFailureNotification> {
        let home_contract = self.home_contract.take().unwrap()?;
        let signer = self.attestation_signer.take().unwrap()?;
        let updater = home_contract
            .updater()
            .await
            .map_err(Error::UpdaterAddress)?;
        Ok(FailureNotification {
            home_domain: home_contract.local_domain(),
            updater: updater.into(),
        }
        .sign_with(&signer)
        .await
        .map_err(Error::AttestationSignerFailed)?)
    }

    /// Kill channel
    async fn kill(&mut self, signed_failure: &SignedFailureNotification) -> Result<TxOutcome> {
        let connection_manager = self.connection_manager.take().unwrap()?;
        Ok(connection_manager
            .unenroll_replica(signed_failure)
            .await
            .map_err(Error::UnenrollmentFailed)?)
    }
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
            return Err(Error::MissingRPC(network.clone()));
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
        let submitter_config = settings
            .tx_submitters
            .get(&channel.home)
            .ok_or_else(|| Error::MissingTxSubmitterConf(channel.home.clone()))?;
        chain_setup
            .try_into_home(Some(submitter_config.clone()), None, None)
            .await
            .map_err(|report| Error::HomeInit(format!("{:#}", report)))
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
            .ok_or_else(|| Error::MissingTxSubmitterConf(channel.replica.clone()))?;
        chain_setup
            .try_into_connection_manager(Some(submitter_config.clone()), None)
            .await
            .map_err(|report| Error::ConnectionManagerInit(format!("{:#}", report)))
    }

    async fn make_signer(channel: &Channel, settings: &Settings) -> Result<Signers> {
        let config = settings
            .attestation_signers
            .get(&channel.home)
            .ok_or_else(|| Error::MissingAttestationSignerConf(channel.home.clone()))?;
        Signers::try_from_signer_conf(config)
            .await
            .map_err(|report| Error::AttestationSignerInit(format!("{:#}", report)))
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
            return Err(Error::NoNetworks);
        }

        let futs = channels.into_iter().map(|channel| async {
            let home_contract = Self::make_home(&channel, &settings).await;
            let connection_manager = Self::make_connection_manager(&channel, &settings).await;
            let attestation_signer = Self::make_signer(&channel, &settings).await;
            ChannelKiller {
                channel,
                home_contract: Some(home_contract),
                connection_manager: Some(connection_manager),
                attestation_signer: Some(attestation_signer),
            }
        });
        let channel_killers = join_all(futs).await.into_iter().collect::<Vec<_>>();

        Ok(Self { channel_killers })
    }

    /// Run `KillSwitch` against configuration
    pub(crate) async fn run(&mut self) {
        let futs = self
            .channel_killers
            .iter_mut()
            .map(|killer| async {
                let futs = async {
                    let signed_failure = killer.create_signed_failure().await?;
                    killer.kill(&signed_failure).await
                }
                .await;
                (killer.channel.clone(), futs)
            })
            .collect::<Vec<_>>();
        let results = join_all(futs).await.into_iter().collect::<Vec<_>>();

        println!("{:?}", results);
        // Build results object
    }
}
