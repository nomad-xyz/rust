mod killswitch;
mod settings;

use clap::Parser;
use color_eyre::Result;
use killswitch::KillSwitch;
use settings::KillSwitchSettings as Settings;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    app: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let settings = Settings::new().await?;

    let killswitch = KillSwitch::from_settings(settings).await?;

    killswitch.run(args).await?;

    Ok(())
}
