use crate::{settings::KillSwitchSettings, Args};
use color_eyre::Result;
use ethers::core::types::Signature;
use nomad_core::{FailureNotification, SignedFailureNotification};

#[derive(Debug)]
pub(crate) struct KillSwitch {}

impl KillSwitch {
    pub(crate) async fn new(_args: Args, _settings: KillSwitchSettings) -> Result<Self> {
        //

        return Ok(Self {});
    }

    pub(crate) async fn run(&self) -> Result<()> {
        //

        return Ok(());
    }

    async fn create_signed_failure(&self) -> SignedFailureNotification {
        SignedFailureNotification {
            notification: FailureNotification {
                home_domain: 0,
                updater: Default::default(),
            },
            signature: Signature {
                r: Default::default(),
                s: Default::default(),
                v: 0,
            },
        }
    }
}
