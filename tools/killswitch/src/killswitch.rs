use crate::{channel::Channel, settings::KillSwitchSettings, Args};
use color_eyre::Result;
use ethers::core::types::Signature;
use nomad_base::{ChainSetup, ChainSetupType};
use nomad_core::{FailureNotification, SignedFailureNotification};
use nomad_xyz_configuration::{AgentSecrets, ChainConf, NomadConfig, TxSubmitterConf};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub(crate) struct KillSwitch {
    channels: Vec<Channel>,
}

impl KillSwitch {
    pub(crate) async fn new(args: Args, settings: KillSwitchSettings) -> Result<Self> {
        let homes = if args.all {
            settings
                .config
                .bridge()
                .keys()
                .map(Clone::clone)
                .collect::<Vec<String>>()
        } else {
            vec![args
                .all_inbound
                .expect("Should not happen. Clap requires this to be present")]
        };

        let chain_setups = homes
            .iter()
            .map(|home| {
                let replicas = settings
                    .config
                    .protocol()
                    .networks
                    .get(home)
                    .map(|n| n.connections.clone())
                    .unwrap_or(HashSet::new());

                let chain_setups = replicas
                    .iter()
                    .map(|replica| {
                        let rpc = settings.rpcs.get(replica).map(|rpc| rpc.clone()).flatten();

                        let chain_setup = rpc.map(|chain_conf| {
                            let secrets = AgentSecrets {
                                rpcs: HashMap::from([(replica.clone(), chain_conf)]),
                                tx_submitters: Default::default(),
                                attestation_signer: None,
                            };
                            ChainSetup::from_config_and_secrets(
                                ChainSetupType::ConnectionManager {
                                    remote_network: replica,
                                },
                                &settings.config,
                                &secrets,
                            )
                        });
                        ((home.clone(), replica.clone()), chain_setup)
                    })
                    .collect::<HashMap<_, _>>();
                (home.clone(), chain_setups)
            })
            .collect::<HashMap<_, _>>();

        

        return Ok(Self {
            channels: Vec::new(),
        });
    }

    pub(crate) async fn run(&self) -> Result<()> {
        //

        return Ok(());
    }

    async fn create_signed_failure(&self) -> SignedFailureNotification {
        unimplemented!()
    }
}
