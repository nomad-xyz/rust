//! The processor observes replicas for updates and proves + processes them
//!
//! At a regular interval, the processor polls Replicas for updates.
//! If there are updates, the processor submits a proof of their
//! validity and processes on the Replica's chain

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod processor;
mod prover_sync;
mod push;
mod settings;

use color_eyre::Result;
use tracing::info_span;

use crate::{processor::Processor, settings::ProcessorSettings as Settings};
use nomad_base::NomadAgent;

use tracing_subscriber::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let agent = {
        // sets the subscriber for this scope only
        let _sub = tracing_subscriber::FmtSubscriber::builder()
            .json()
            .with_level(true)
            .set_default();
        {
            let span = info_span!("ProcessorBootup");
            let _span = span.enter();

            let settings = Settings::new()?;
            Processor::from_settings(settings).await?
        }
    };

    // TODO: top-level root span customizations?
    let metrics_guard = agent.start_tracing(agent.metrics().span_duration());

    let _ = agent.metrics().run_http_server();

    agent.run_all().await??;
    Ok(())
}
