mod errors;
mod killswitch;
mod output;
mod settings;

use crate::killswitch::KillSwitch;
use crate::{errors::Error, output::Output, settings::Settings};
use clap::{ArgGroup, Parser, ValueEnum};
use std::{
    env,
    io::{stdout, Write},
    process::exit,
};

/// Result returning KillSwitch `Error`
pub(crate) type Result<T> = std::result::Result<T, Error>;

/// What we're killing, currently only `TokenBridge`
#[derive(ValueEnum, Clone, Debug)]
enum App {
    /// The token bridge
    TokenBridge,
}

/// Command line args
#[derive(Parser, Debug)]
#[clap(group(
    ArgGroup::new("which_networks")
    .required(true)
    .multiple(false)
    .args(&["all", "all-inbound"])
))]
struct Args {
    /// Which app to kill
    #[clap(long, arg_enum)]
    app: App,

    /// Kill all available networks
    #[clap(long)]
    all: bool,

    /// Kill all replicas on network
    #[clap(long, value_name = "NETWORK")]
    all_inbound: Option<String>,

    // The most common form of streaming JSON is line delimited
    // hide this behind a (hidden) flag so it's not abused
    #[clap(long, hide = true)]
    pretty: bool,
}

/// Exit codes as found in <sysexits.h>
enum ExitCodes {
    Ok = 0,
    BadConfig = 78,
}

/// KillSwitch entry point
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cmd = env::args().collect::<Vec<_>>().join(" ");
    let args = Args::parse();
    let jsonify = if args.pretty {
        serde_json::to_string_pretty
    } else {
        serde_json::to_string
    };

    let settings = Settings::new().await;

    if let Err(error) = settings {
        let output = Output {
            command: cmd,
            message: error.into(),
        };
        let json = jsonify(&output).expect("Serialization error. Should never happen");
        stdout()
            .lock()
            .write_all(&format!("{}\n", json).into_bytes())
            .expect("Write to stdout error. Should never happen");
        exit(ExitCodes::BadConfig as i32)
    }

    let _killswitch = KillSwitch::new(args, settings.unwrap()).await;

    // setup killswitch
    // run killswitch

    exit(ExitCodes::Ok as i32)
}
