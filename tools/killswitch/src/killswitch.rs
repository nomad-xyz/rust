use crate::{errors::Error, settings::Settings, Args, Result};
use ethers::prelude::H256;
use nomad_base::{AttestationSigner, ChainSetup, ChainSetupType, ConnectionManagers, Homes};
use nomad_core::{Common, ConnectionManager, FailureNotification, FromSignerConf, Home};
use nomad_xyz_configuration::AgentSecrets;
use std::sync::mpsc::Sender;
use tokio::task::JoinHandle;

/// Main `KillSwitch` struct
#[derive(Debug)]
pub(crate) struct KillSwitch {
    /// Our `Settings`
    settings: Settings,
    /// A vector of all `Channel`s
    channels: Vec<Channel>,
}

/// The set of origin -> destination networks
#[derive(Debug, Clone)]
pub(crate) struct Channel {
    /// Origin network
    pub(crate) home: String,
    /// Destination network
    pub(crate) replica: String,
}

impl KillSwitch {
    /// Get a count of the `Channel`s we're configured for
    pub(crate) fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Get a copy of the `Channel`s we're configured for
    pub(crate) fn channels(&self) -> Vec<Channel> {
        self.channels.clone()
    }

    /// Get all available home->replica channels in config
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

    /// Build a new `KillSwitch`
    pub(crate) async fn new(args: &Args, settings: Settings) -> Result<Self> {
        let channels = if args.all {
            Self::make_channels(&settings)
        } else {
            let destination_network = args
                .all_inbound
                .clone()
                .expect("Should not happen. Clap requires this to be present");
            let all = Self::make_channels(&settings);
            Self::make_inbound_channels(&destination_network, all)
        };
        if channels.is_empty() {
            // The one error we bail on, since there's nothing else left to do
            return Err(Error::NoNetworks);
        }
        Ok(Self { settings, channels })
    }

    /// Run `KillSwitch` against channels in parallel, sending results back via `mpsc::channel`
    pub(crate) fn run(&self, output: Sender<(Channel, Result<H256>)>) -> Vec<JoinHandle<()>> {
        let mut handles = Vec::new();
        // Run our channels in parallel
        for channel in &self.channels {
            let output = output.clone();
            let channel = channel.clone();
            let settings = self.settings.clone();
            let handle = tokio::spawn(async move {
                // Build our contracts and signers, if we fail here, bail
                let setup = tokio::try_join!(
                    Self::make_home(&channel, &settings),
                    Self::make_connection_manager(&channel, &settings),
                    Self::make_signer(&channel, &settings),
                );
                // Maybe bail
                if let Err(error) = setup {
                    output
                        .send((channel.clone(), Err(error)))
                        .expect("Should not happen. Channel should be ok during operation");
                    return;
                }
                // Create our signed failure notification and attempt to unenroll replica
                let (home_contract, connection_manager, attestation_signer) = setup.unwrap();
                let result = async move {
                    let updater = home_contract
                        .updater()
                        .await
                        .map_err(Error::UpdaterAddress)?;
                    let failure = FailureNotification {
                        home_domain: home_contract.local_domain(),
                        updater: updater.into(),
                    };
                    let signed_failure = failure
                        .sign_with(&attestation_signer)
                        .await
                        .map_err(Error::AttestationSignerFailed)?;
                    connection_manager
                        .unenroll_replica(&signed_failure)
                        .await
                        .map_err(Error::UnenrollmentFailed)
                }
                .await
                .map(|tx_outcome| tx_outcome.txid);
                output
                    .send((channel.clone(), result))
                    .expect("Should not happen. Channel should be ok during operation");
            });
            handles.push(handle);
        }
        handles
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{App, Environment};
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
                environment: Some(Environment::Testing),
                environment_override: None,
                all: false,
                all_inbound: Some("avalanche".into()), // Unused network
                force: true,
            };
            let settings = Settings::new().await;
            assert!(settings.is_ok());
            let settings = settings.unwrap();

            let killswitch = KillSwitch::new(&args, settings).await;
            assert_matches!(killswitch.unwrap_err(), Error::NoNetworks);
        })
        .await
    }
}
