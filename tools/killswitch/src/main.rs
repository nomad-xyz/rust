#[cfg(test)]
#[macro_use]
extern crate assert_matches;

mod errors;
mod killswitch;
mod secrets;
mod settings;

use crate::{errors::Error, killswitch::KillSwitch, secrets::Secrets, settings::Settings};
use clap::{ArgGroup, Parser, ValueEnum};
use std::{
    env,
    io::{stdout, Write},
    process::exit,
    sync::mpsc::channel,
};

/// AWS settings
const AWS_REGION: &str = "us-west-2";
const AWS_CREDENTIALS_PROFILE_DEVELOPMENT: &str = "nomad-xyz-dev";
const AWS_CREDENTIALS_PROFILE_PRODUCTION: &str = "nomad-xyz-prod";
const CONFIG_S3_BUCKET_DEVELOPMENT: &str = "nomad-killswitch-config-dev";
const CONFIG_S3_BUCKET_PRODUCTION: &str = "nomad-killswitch-config-prod";
const CONFIG_S3_KEY: &str = "config.yaml";

/// Local secrets. For testing only
const SECRETS_PATH_LOCAL_TESTING: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/killswitch_secrets.testing.yaml"
);

/// Result returning KillSwitch `Error`
pub(crate) type Result<T> = std::result::Result<T, Error>;

/// The environment we're targeting
#[derive(ValueEnum, Clone, Debug, Eq, PartialEq)]
enum Environment {
    /// The development environment
    Development,
    /// The production environment
    Production,
    /// Use local secrets. For testing only
    #[clap(hide = true)]
    LocalPath,
    /// Pull all secrets from environment. For testing only
    #[clap(hide = true)]
    AlreadySet,
}

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
    .args(&["all", "all_inbound"])
))]
struct Args {
    /// Which environment to target
    #[clap(long, value_enum)]
    environment: Environment,

    /// Which app to kill
    #[clap(long, value_enum)]
    app: App,

    /// Kill all available networks
    #[clap(long)]
    all: bool,

    /// Kill all replicas on network
    #[clap(long, value_name = "NETWORK")]
    all_inbound: Option<String>,

    /// Actually execute `killswitch`. This is an inverse of
    /// a `--dry_run` flag and makes more sense given the destructive
    /// nature of this utility
    #[clap(long, hide = true)]
    force: bool,
}

/// Exit codes as found in <sysexits.h>
enum ExitCode {
    Ok = 0,
    BadConfig = 78,
}

fn write_stdout(message: &str) {
    stdout().lock().write_all(message.as_bytes()).unwrap()
}

/// KillSwitch entry point
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    let command = env::args().collect::<Vec<_>>().join(" ");
    write_stdout("\n");
    write_stdout(&format!("Running `{}`\n", command));

    if Environment::AlreadySet != args.environment {
        write_stdout("Fetching secrets from S3 using local AWS credentials... ");
        let secrets = match &args.environment {
            Environment::Development => {
                Secrets::fetch(
                    AWS_CREDENTIALS_PROFILE_DEVELOPMENT,
                    AWS_REGION,
                    CONFIG_S3_BUCKET_DEVELOPMENT,
                    CONFIG_S3_KEY,
                )
                .await
            }
            Environment::Production => {
                Secrets::fetch(
                    AWS_CREDENTIALS_PROFILE_PRODUCTION,
                    AWS_REGION,
                    CONFIG_S3_BUCKET_PRODUCTION,
                    CONFIG_S3_KEY,
                )
                .await
            }
            Environment::LocalPath => Secrets::load(SECRETS_PATH_LOCAL_TESTING).await,
            Environment::AlreadySet => unreachable!(),
        };
        if let Err(error) = secrets {
            write_stdout(&format!("Failed: {}\n", error));
            exit(ExitCode::BadConfig as i32)
        }
        write_stdout("Ok\n");

        // Set `Secrets` as environment variables for `Settings` to pick up
        secrets.unwrap().set_environment();
    }

    write_stdout("Building settings from environment... ");
    let settings = Settings::new().await;
    if let Err(error) = settings {
        write_stdout(&format!("Failed: {}\n", error));
        exit(ExitCode::BadConfig as i32)
    }
    write_stdout("Ok\n");

    let settings = settings.unwrap();

    write_stdout("Checking `killswitch` for killable networks... ");
    let killswitch = KillSwitch::new(&args, settings).await;
    if let Err(error) = killswitch {
        write_stdout(&format!("Failed: {}\n", error));
        exit(ExitCode::BadConfig as i32)
    }
    write_stdout("Ok\n");

    let killswitch = killswitch.unwrap();

    write_stdout("\n");
    write_stdout("`killswitch` is ready to attempt to kill the selected channels:\n");
    for channel in killswitch.channels() {
        write_stdout(&format!(
            "[CHANNEL] {} -> {}\n",
            channel.home, channel.replica
        ));
    }

    if !&args.force {
        write_stdout("\n");
        write_stdout("[NOTICE] Nothing killed!\n");
        write_stdout("\n");
        write_stdout("To kill the selected networks, run the same command again with the `--force` flag added.\n");
        write_stdout("\n");
        exit(ExitCode::Ok as i32)
    } else {
        write_stdout("\n");
        write_stdout("Running `killswitch`...\n");
        let (tx, rx) = channel();
        let handles = killswitch.run(tx);
        for _ in 0..killswitch.channel_count() {
            let (channel, result) = rx.recv().unwrap();
            write_stdout("\n");
            write_stdout(&format!(
                "[CHANNEL] {} -> {}\n",
                channel.home, channel.replica
            ));
            match result {
                Ok(txid) => {
                    write_stdout(&format!(
                        "[SUCCESS] transaction id for unenrollment: {:?}\n",
                        txid
                    ));
                }
                Err(error) => {
                    write_stdout(&format!("[FAILURE] {}\n", error));
                }
            }
        }
        write_stdout("\n");
        for handle in handles {
            handle
                .await
                .expect("Should not happen. Errors should have been caught");
        }
        exit(ExitCode::Ok as i32)
    }
}
