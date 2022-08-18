mod killswitch;
mod settings;

use color_eyre::Result;
use killswitch::KillSwitch;
use settings::KillSwitchSettings as Settings;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let settings = Settings::new().await?;

    let killswitch = KillSwitch::from_settings(settings).await?;

    killswitch.run().await?;

    Ok(())
}
