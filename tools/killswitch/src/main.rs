mod channel;
mod killswitch;
mod settings;

use clap::{ArgGroup, Parser, ValueEnum};
use color_eyre::Result;
use killswitch::KillSwitch;
use settings::KillSwitchSettings as Settings;

#[derive(ValueEnum, Clone, Debug)]
enum App {
    TokenBridge,
}

#[derive(Parser, Debug)]
#[clap(group(
    ArgGroup::new("which_networks")
    .required(true)
    .multiple(false)
    .args(&["all", "all-inbound"])
))]
struct Args {
    #[clap(long, arg_enum)]
    app: App,

    #[clap(long)]
    all: bool,

    #[clap(long, value_name = "HOME")]
    all_inbound: Option<String>,
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
