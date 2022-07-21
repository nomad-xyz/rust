//! The relayer forwards signed updates from the home to chain to replicas
//!
//! At a regular interval, the relayer polls Home for signed updates and
//! submits them as updates with a pending timelock on the replica.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod relayer;
mod settings;

use crate::{relayer::Relayer, settings::RelayerSettings as Settings};
use color_eyre::Result;
use nomad_base::NomadAgent;

use tracing::info_span;
use tracing_subscriber::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // sets the subscriber for this scope only
    let _bootup_guard = tracing_subscriber::FmtSubscriber::builder()
        .json()
        .with_level(true)
        .set_default();

    let span = info_span!("RelayerBootup");
    let _span = span.enter();

    let settings = Settings::new().await?;
    let agent = Relayer::from_settings(settings).await?;

    drop(_span);
    drop(span);

    let _tracing_guard = agent.start_tracing(agent.metrics().span_duration());

    let _ = agent.metrics().run_http_server();

    agent.run_all().await??;
    Ok(())
}
