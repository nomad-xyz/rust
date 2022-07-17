use std::sync::Arc;

use prometheus::{Encoder, HistogramOpts, HistogramVec, IntCounterVec};
use tokio::task::JoinHandle;
use warp::Filter;

#[derive(Debug)]
pub(crate) struct Metrics {
    wallclock_times: prometheus::HistogramVec,
    event_blocks: prometheus::HistogramVec,
    counts: prometheus::IntCounterVec,
}

fn u16_from_env(s: impl AsRef<str>) -> Option<u16> {
    std::env::var(s.as_ref()).ok().and_then(|i| i.parse().ok())
}

fn gather() -> prometheus::Result<Vec<u8>> {
    let collected_metrics = prometheus::default_registry().gather();
    let mut out_buf = Vec::with_capacity(1024 * 64);
    let encoder = prometheus::TextEncoder::new();
    encoder.encode(&collected_metrics, &mut out_buf)?;
    Ok(out_buf)
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

        let event_blocks = HistogramVec::new(
            HistogramOpts::new(
                "inter_event_blocks",
                "Blocks between events, as marked by the chain timestamp",
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

        let registry = prometheus::default_registry();
        registry
            .register(Box::new(wallclock_times.clone()))
            .expect("unable to register metric");
        registry
            .register(Box::new(event_blocks.clone()))
            .expect("unable to register metric");
        registry
            .register(Box::new(counts.clone()))
            .expect("unable to register metric");

        Ok(Self {
            wallclock_times,
            event_blocks,
            counts,
        })
    }

    pub(crate) fn run_http_server(self: Arc<Metrics>) -> JoinHandle<()> {
        let port = u16_from_env("METRICS_PORT").unwrap_or(9090);

        tracing::info!(
            port,
            "starting prometheus server on 0.0.0.0:{port}",
            port = port
        );

        tokio::spawn(async move {
            warp::serve(
                warp::path!("metrics")
                    .map(move || {
                        warp::reply::with_header(
                            gather().expect("failed to encode metrics"),
                            "Content-Type",
                            // OpenMetrics specs demands "application/openmetrics-text; version=1.0.0; charset=utf-8"
                            // but the prometheus scraper itself doesn't seem to care?
                            // try text/plain to make web browsers happy.
                            "text/plain; charset=utf-8",
                        )
                    })
                    .or(warp::any().map(|| {
                        warp::reply::with_status(
                            "go look at /metrics",
                            warp::http::StatusCode::NOT_FOUND,
                        )
                    })),
            )
            .run(([0, 0, 0, 0], port))
            .await;
        })
    }
}
