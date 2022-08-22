mod channel;
mod killswitch;
mod settings;

use clap::{Parser, ValueEnum};
use color_eyre::Result;
use killswitch::KillSwitch;
use settings::KillSwitchSettings as Settings;

#[derive(ValueEnum, Clone, Debug)]
enum App {
    TokenBridge,
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, arg_enum)]
    app: App,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let settings = Settings::new().await?;

    let killswitch = KillSwitch::new(args, settings).await?;

    killswitch.run().await?;

    Ok(())
}
