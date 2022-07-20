//! The updater signs updates and submits them to the home chain.
//!
//! This updater polls the Home for queued updates at a regular interval.
//! It signs them and submits them back to the home chain.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod produce;
mod settings;
mod submit;
mod updater;

use crate::{settings::UpdaterSettings as Settings, updater::Updater};
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

    let span = info_span!("UpdaterBootup");
    let _span = span.enter();

    let settings = Settings::new()?;
    let agent = Updater::from_settings(settings).await?;

    drop(_span);
    drop(span);

    let _tracing_guard = agent.start_tracing(agent.metrics().span_duration());

    let _ = agent.metrics().run_http_server();

    agent.run_all().await??;
    Ok(())
}
