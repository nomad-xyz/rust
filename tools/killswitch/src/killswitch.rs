use crate::{settings::KillSwitchSettings, Args};
use color_eyre::Result;

#[derive(Debug)]
pub(crate) struct KillSwitch {}

impl KillSwitch {
    pub(crate) async fn from_settings(_settings: KillSwitchSettings) -> Result<Self> {
        //

        return Ok(Self {});
    }

    pub(crate) async fn run(&self, args: Args) -> Result<()> {
        //

        return Ok(());
    }
}
