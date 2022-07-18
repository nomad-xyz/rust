use std::sync::Arc;

use prometheus::{Encoder, Histogram, HistogramOpts, HistogramVec, IntCounter, IntCounterVec};
use tokio::task::JoinHandle;
use warp::Filter;

use crate::between::BetweenMetrics;

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

    pub(crate) fn event_counter(
        &self,
        chain: &str,
        event: &str,
        emitter: &str,
        replica_of: Option<&str>,
    ) -> IntCounter {
        self.counts.with_label_values(&[
            chain.as_ref(),
            event.as_ref(),
            emitter.as_ref(),
            replica_of.unwrap_or("n/a"),
        ])
    }

    pub(crate) fn wallclock_latency(
        &self,
        chain: &str,
        event: &str,
        emitter: &str,
        replica_of: Option<&str>,
    ) -> Histogram {
        // &["chain", "event", "emitter", "replica_of"],

        self.wallclock_times.with_label_values(&[
            chain.as_ref(),
            event.as_ref(),
            emitter.as_ref(),
            replica_of.unwrap_or("n/a"),
        ])
    }

    pub(crate) fn block_latency(
        &self,
        chain: &str,
        event: &str,
        emitter: &str,
        replica_of: Option<&str>,
    ) -> Histogram {
        //            &["chain", "event", "emitter", "replica_of"],
        self.event_blocks.with_label_values(&[
            chain.as_ref(),
            event.as_ref(),
            emitter.as_ref(),
            replica_of.unwrap_or("n/a"),
        ])
    }

    pub(crate) fn between_metrics(
        &self,
        chain: &str,
        event: &str,
        emitter: &str,
        replica_of: Option<&str>,
    ) -> BetweenMetrics {
        BetweenMetrics {
            count: self.event_counter(chain, event, emitter, replica_of),
            wallclock_latency: self.wallclock_latency(chain, event, emitter, replica_of),
            block_latency: self.block_latency(chain, event, emitter, replica_of),
        }
    }
}
