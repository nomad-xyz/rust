#[cfg(test)]
#[macro_use]
extern crate assert_matches;

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

    // Try to build `NomadConfig`, exiting immediately if we can't
    let settings = Settings::new().await;
    if let Err(error) = settings {
        // We've hit a blocking error, bail
        report(error.into(), pretty);
        exit(ExitCode::BadConfig as i32)
    }

    // Try to build `KillSwitch`. If we hit `NoNetworks` error, nothing to do, bail
    let killswitch = KillSwitch::new(args, settings.unwrap()).await;
    if let Err(error) = killswitch {
        // We've hit a blocking error, bail
        report(error.into(), pretty);
        exit(ExitCode::BadConfig as i32)
    }

    // Get errors that block individual channels, report before proceeding. Do not bail
    let (killswitch, errors) = killswitch.unwrap().get_blocking_errors().await;
    if let Some(errors) = errors {
        // Stream these blocking errors before running transactions
        // so users can be updated as fast as possible
        report(errors, pretty);
    }

    // Run all channels that *could* succeed
    let results = killswitch.run().await;

    // Give users final results, exit ok
    report(results, pretty);
    exit(ExitCode::Ok as i32)
}
