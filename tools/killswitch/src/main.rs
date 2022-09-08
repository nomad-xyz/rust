mod errors;
mod killswitch;
mod output;
mod settings;

use crate::killswitch::KillSwitch;
use crate::output::Message;
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
enum ExitCode {
    Ok = 0,
    BadConfig = 78,
}

/// Print `Output` to stdout as json
fn report(message: Message, pretty: bool) {
    let command = env::args().collect::<Vec<_>>().join(" ");
    let output = Output { command, message };
    let json = if pretty {
        serde_json::to_string_pretty(&output)
    } else {
        serde_json::to_string(&output)
    }
    .expect("Serialization error. Should never happen");
    stdout()
        .lock()
        .write_all(&format!("{}\n", json).into_bytes())
        .expect("Write to stdout error. Should never happen");
}

/// KillSwitch entry point
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    let pretty = args.pretty;

    let settings = Settings::new().await;
    if let Err(error) = settings {
        report(error.into(), pretty);
        exit(ExitCode::BadConfig as i32)
    }

    let killswitch = KillSwitch::new(args, settings.unwrap()).await;
    if let Err(error) = killswitch {
        report(error.into(), pretty);
        exit(ExitCode::BadConfig as i32)
    }

    let _ = killswitch.unwrap().run().await;
    // Report

    exit(ExitCode::Ok as i32)
}
