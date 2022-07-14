//! Kathy is chatty. She sends random messages to random recipients

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod kathy;
mod settings;

use crate::{kathy::Kathy, settings::KathySettings as Settings};
use color_eyre::Result;
use nomad_base::NomadAgent;

use tracing_subscriber::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let agent = {
        // sets the subscriber for this scope only
        let _ = tracing_subscriber::FmtSubscriber::builder()
            .json()
            .with_level(true)
            .set_default();
        let settings = Settings::new()?;
        Kathy::from_settings(settings).await?
    };

    agent.start_tracing(agent.metrics().span_duration())?;
    let _ = agent.metrics().run_http_server();

    agent.run_all().await?
}
