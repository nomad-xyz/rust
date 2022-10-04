use crate::{
    errors::Error, output::build_output_message, settings::Settings, Args, Message, Result,
};
use futures_util::future::join_all;
use nomad_base::{AttestationSigner, ChainSetup, ChainSetupType, ConnectionManagers, Homes};
use nomad_core::{
    Common, ConnectionManager, FailureNotification, FromSignerConf, Home,
    SignedFailureNotification, TxOutcome,
};
use nomad_xyz_configuration::AgentSecrets;

/// Main `KillSwitch` struct
#[derive(Debug)]
pub(crate) struct KillSwitch {
    /// A vector of all `ChannelKiller`
    channel_killers: Vec<ChannelKiller>,
}

/// The set of origin->destination networks
#[derive(Debug, Clone)]
pub(crate) struct Channel {
    /// Origin network
    pub(crate) home: String,
    /// Destination network
    pub(crate) replica: String,
}

/// The channel and contracts required or errors encountered
#[derive(Debug)]
struct ChannelKiller {
    /// The channel we want to kill
    channel: Channel,
    /// Home contract
    home_contract: Option<Homes>,
    /// Connection manager
    connection_manager: Option<ConnectionManagers>,
    /// Attestation signer
    attestation_signer: Option<AttestationSigner>,
    /// Contract init errors we've encountered
    errors: Vec<Error>,
}

impl ChannelKiller {
    /// Have we collected *any* errors
    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Take all available errors
    fn take_all_errors(&mut self) -> Vec<Error> {
        self.errors.drain(..).collect()
    }

    /// Create a `SignedFailureNotification`
    async fn create_signed_failure(&mut self) -> Result<SignedFailureNotification> {
        // Force unwrap here as we're not calling this on contract with errors
        let home_contract = self.home_contract.take().unwrap();
        let signer = self.attestation_signer.take().unwrap();
        let updater = home_contract
            .updater()
            .await
            .map_err(Error::UpdaterAddress)?;
        FailureNotification {
            home_domain: home_contract.local_domain(),
            updater: updater.into(),
        }
        .sign_with(&signer)
        .await
        .map_err(Error::AttestationSignerFailed)
    }

    /// Kill channel
    async fn kill(&mut self, signed_failure: &SignedFailureNotification) -> Result<TxOutcome> {
        // Force unwrap here as we're not calling this on contract with errors
        let connection_manager = self.connection_manager.take().unwrap();
        connection_manager
            .unenroll_replica(signed_failure)
            .await
            .map_err(Error::UnenrollmentFailed)
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

    /// Make `AttestationSigner` or return error
    async fn make_signer(channel: &Channel, settings: &Settings) -> Result<AttestationSigner> {
        let config = settings
            .attestation_signers
            .get(&channel.home)
            .ok_or_else(|| Error::MissingAttestationSignerConf(channel.home.clone()))?;
        AttestationSigner::try_from_signer_conf(config)
            .await
            .map_err(|report| Error::AttestationSignerInit(format!("{:#}", report)))
    }

    /// Build a new `KillSwitch`, configuring best effort and storing, not returning most errors
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
            // The one error we bail on, since there's nothing else left to do
            return Err(Error::NoNetworks);
        }

        let futs = channels.into_iter().map(|channel| async {
            let home_contract = Self::make_home(&channel, &settings).await;
            let connection_manager = Self::make_connection_manager(&channel, &settings).await;
            let attestation_signer = Self::make_signer(&channel, &settings).await;
            let mut killer = ChannelKiller {
                channel,
                home_contract: None,
                connection_manager: None,
                attestation_signer: None,
                errors: vec![],
            };

            if let Err(err) = home_contract {
                killer.errors.push(err);
            } else {
                killer.home_contract = home_contract.ok();
            }

            if let Err(err) = connection_manager {
                killer.errors.push(err);
            } else {
                killer.connection_manager = connection_manager.ok();
            }

            if let Err(err) = attestation_signer {
                killer.errors.push(err);
            } else {
                killer.attestation_signer = attestation_signer.ok();
            }
            killer
        });
        let channel_killers = join_all(futs).await.into_iter().collect::<Vec<_>>();
        Ok(Self { channel_killers })
    }

    /// Collect all blocking errors, returning a `KillSwitch` with a set of channels
    /// that can actually fire off transactions, as well as any errors collected
    pub(crate) async fn get_blocking_errors(self) -> (Self, Option<Message>) {
        let (mut failed, maybe_ok): (Vec<_>, Vec<_>) = self
            .channel_killers
            .into_iter()
            .partition(|killer| killer.has_errors());

        // These are blocking errors for each channel
        let bad = failed
            .iter_mut()
            .map(|killer| (killer.channel.clone(), killer.take_all_errors()))
            .collect::<Vec<(_, _)>>();

        // Produce errors to stream before running txs
        let message = if bad.is_empty() {
            None
        } else {
            Some(build_output_message(bad, vec![]))
        };
        (
            KillSwitch {
                channel_killers: maybe_ok,
            },
            message,
        )
    }

    /// Run `KillSwitch` against remaining, non-blocked channels
    pub(crate) async fn run(mut self) -> Message {
        let futs = self
            .channel_killers
            .iter_mut()
            .map(|killer| async {
                let fut = async {
                    let signed_failure = killer.create_signed_failure().await?;
                    killer.kill(&signed_failure).await
                }
                .await;
                (killer.channel.clone(), fut)
            })
            .collect::<Vec<_>>();

        let results = join_all(futs).await;

        let (failed, ok): (Vec<_>, Vec<_>) =
            results.into_iter().partition(|(_, result)| result.is_err());

        // We encountered errors
        let bad = failed
            .into_iter()
            .map(|(channel, result)| (channel, vec![result.unwrap_err()]))
            .collect::<Vec<(_, _)>>();

        // These were successful
        let good = ok
            .into_iter()
            .map(|(channel, result)| (channel, result.unwrap()))
            .collect::<Vec<(_, _)>>();

        build_output_message(bad, good)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::App;
    use nomad_test::test_utils;
    use nomad_xyz_configuration::{ChainConf, Connection};
    use std::collections::HashMap;

    #[tokio::test]
    #[serial_test::serial]
    async fn it_makes_all_channels() {
        test_utils::run_test_with_env("../../fixtures/env.test-killswitch", || async move {
            let settings = Settings::new().await;
            assert!(settings.is_ok());

            let settings = settings.unwrap();
            let channels = KillSwitch::make_channels(&settings);

            // Networks are loaded as a HashMap so channels are built non-deterministically
            // We're just checking that we've created the correct number here
            assert_eq!(channels.len(), 4 * 3);
        })
        .await
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_makes_inbound_channels() {
        test_utils::run_test_with_env("../../fixtures/env.test-killswitch", || async move {
            let settings = Settings::new().await;
            assert!(settings.is_ok());

            let inbound: String = "goerli".into();
            let settings = settings.unwrap();
            let all_channels = KillSwitch::make_channels(&settings);
            let channels = KillSwitch::make_inbound_channels(&inbound, all_channels);

            // Inbound should equal all replicas and no homes
            assert_eq!(channels.len(), 3);
            for channel in &channels {
                assert_eq!(channel.replica, inbound);
            }
            for channel in &channels {
                assert_ne!(channel.home, inbound);
            }
        })
        .await
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_makes_good_chain_setup() {
        test_utils::run_test_with_env("../../fixtures/env.test-killswitch", || async move {
            let settings = Settings::new().await;
            assert!(settings.is_ok());

            let network: String = "goerli".into();
            let settings = settings.unwrap();
            let setup_type = ChainSetupType::Home {
                home_network: &network,
            };

            let chain_setup = KillSwitch::make_chain_setup(&network, setup_type, &settings);
            assert!(chain_setup.is_ok());
            let chain_setup = chain_setup.unwrap();

            assert_eq!(chain_setup.name, network);
            assert_matches!(chain_setup.chain, ChainConf::Ethereum(Connection::Http(_)));
        })
        .await
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_makes_bad_chain_setup() {
        test_utils::run_test_with_env("../../fixtures/env.test-killswitch", || async move {
            let settings = Settings::new().await;
            assert!(settings.is_ok());

            let network: String = "goerli".into();
            let mut settings = settings.unwrap();
            let setup_type = ChainSetupType::Home {
                home_network: &network,
            };
            settings.rpcs = HashMap::new(); // Bad rpc config

            let chain_setup = KillSwitch::make_chain_setup(&network, setup_type, &settings);
            assert_matches!(chain_setup.unwrap_err(), Error::MissingRPC(n) if n == network);
        })
        .await
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_makes_bad_home() {
        test_utils::run_test_with_env("../../fixtures/env.test-killswitch", || async move {
            let settings = Settings::new().await;
            assert!(settings.is_ok());

            let network: String = "goerli".into();
            let mut settings = settings.unwrap();
            let channel = Channel {
                home: network.clone(),
                replica: "rinkeby".into(),
            };
            settings.tx_submitters = HashMap::new(); // Bad tx_submitter config

            let home = KillSwitch::make_home(&channel, &settings).await;
            assert_matches!(home.unwrap_err(), Error::MissingTxSubmitterConf(n) if n == network);
        })
        .await
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_makes_bad_connection_manager() {
        test_utils::run_test_with_env("../../fixtures/env.test-killswitch", || async move {
            let settings = Settings::new().await;
            assert!(settings.is_ok());

            let network: String = "goerli".into();
            let mut settings = settings.unwrap();
            let channel = Channel {
                home: "rinkeby".into(),
                replica: network.clone(),
            };
            settings.tx_submitters = HashMap::new(); // Bad tx submitter config

            let xcm = KillSwitch::make_connection_manager(&channel, &settings).await;
            assert_matches!(xcm.unwrap_err(), Error::MissingTxSubmitterConf(n) if n == network);
        })
        .await
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_makes_bad_attestation_signer() {
        test_utils::run_test_with_env("../../fixtures/env.test-killswitch", || async move {
            let settings = Settings::new().await;
            assert!(settings.is_ok());

            let network: String = "goerli".into();
            let mut settings = settings.unwrap();
            let channel = Channel {
                home: network.clone(),
                replica: "rinkeby".into(),
            };
            settings.attestation_signers = HashMap::new(); // Bad attestation signer config

            let signer = KillSwitch::make_signer(&channel, &settings).await;
            assert_matches!(signer.unwrap_err(), Error::MissingAttestationSignerConf(n) if n == network);
        })
        .await
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_makes_a_killswitch_with_no_channels() {
        test_utils::run_test_with_env("../../fixtures/env.test-killswitch", || async move {
            let args = Args {
                app: App::TokenBridge,
                all: false,
                all_inbound: Some("avalanche".into()), // Unused network
                pretty: false,
            };
            let settings = Settings::new().await;
            assert!(settings.is_ok());
            let settings = settings.unwrap();

            let killswitch = KillSwitch::new(args, settings).await;
            assert_matches!(killswitch.unwrap_err(), Error::NoNetworks);
        })
        .await
    }

    /// `ChannelKiller` with errors
    fn make_bad_channel_killer() -> ChannelKiller {
        let channel = Channel {
            home: "goerli".into(),
            replica: "rinkeby".into(),
        };
        ChannelKiller {
            channel: channel.clone(),
            home_contract: None,
            connection_manager: None,
            attestation_signer: None,
            errors: vec![
                Error::MissingTxSubmitterConf(channel.home.clone()),
                Error::MissingTxSubmitterConf(channel.replica.clone()),
            ],
        }
    }

    #[test]
    fn it_has_errors() {
        let killer = make_bad_channel_killer();
        assert!(killer.has_errors());
    }

    #[test]
    fn it_takes_errors() {
        let mut killer = make_bad_channel_killer();
        let errors = killer.take_all_errors();
        assert_eq!(errors.len(), 2);
        assert_matches!(errors[0], Error::MissingTxSubmitterConf(_));
        assert_matches!(errors[1], Error::MissingTxSubmitterConf(_));
    }
}
