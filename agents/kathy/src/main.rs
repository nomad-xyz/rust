//! Kathy is chatty. She sends random messages to random recipients

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod kathy;
mod settings;

use crate::{kathy::Kathy, settings::KathySettings as Settings};
use color_eyre::Result;
use nomad_base::NomadAgent;

use tracing::info_span;
use tracing_subscriber::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let _bootup_guard = tracing_subscriber::FmtSubscriber::builder()
        .json()
        .with_level(true)
        .set_default();

    let span = info_span!("KathyBootup");
    let _span = span.enter();

    let settings = Settings::new()?;
    let agent = Kathy::from_settings(settings).await?;

    drop(_span);
    drop(span);

    let _tracing_guard = agent.start_tracing(agent.metrics().span_duration());

    let _ = agent.metrics().run_http_server();

    agent.run_all().await?
}
