use std::sync::Arc;

use prometheus::{HistogramOpts, HistogramVec, IntCounterVec};
use tokio::task::JoinHandle;

#[derive(Debug)]
pub(crate) struct Metrics {
    wallclock_times: prometheus::HistogramVec,
    event_times: prometheus::HistogramVec,
    counts: prometheus::IntCounterVec,
}

impl Metrics {
    pub(crate) fn new() -> eyre::Result<Self> {
        let wallclock_times = HistogramVec::new(
            HistogramOpts::new(
                "inter_event_period_wallclock_ms",
                "Ms between events periods, as observed by this agent",
            )
            .namespace("nomad")
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["chain", "event", "emitter", "replica_of"],
        )?;

        let event_times = HistogramVec::new(
            HistogramOpts::new(
                "inter_event_chain_time_seconds",
                "Seconds between events, as marked by the chain timestamp",
            )
            .namespace("nomad")
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["chain", "event", "emitter", "replica_of"],
        )?;

        let counts = IntCounterVec::new(
            prometheus::core::Opts::new(
                "event_counts",
                "Counts of each event, labeled by name and chain",
            )
            .namespace("nomad")
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["chain", "event", "emitter", "replica_of"],
        )?;

        Ok(Self {
            wallclock_times,
            event_times,
            counts,
        })
    }

    pub(crate) fn run_http_server(self: Arc<Metrics>) -> JoinHandle<()> {
        todo!()
    }
}
