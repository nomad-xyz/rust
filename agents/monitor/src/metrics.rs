use std::sync::Arc;

use prometheus::{Encoder, Histogram, HistogramOpts, HistogramVec, IntCounter, IntCounterVec};
use tokio::task::JoinHandle;
use warp::Filter;

const NAMESPACE: &str = "nomad_monitor";

const LOCAL_TIME_BUCKETS: &[f64] = &[
    1_000.0,     // 1 sec
    5_000.0,     // 5 secs
    30_000.0,    // 30 secs
    60_000.0,    // 1 minu
    120_000.0,   // 2 min
    600_000.0,   // 10 min
    1_800_000.0, // 30 min
    3_600_000.0, // 1 hour
    7_200_000.0, // 2 hour
];
// time buckets for e2e metric
const E2E_TIME_BUCKETS: &[f64] = &[
    2_100_000.0,  // 35 minutes
    2_400_000.0,  // 40 minutes
    2_700_000.0,  // 45 minutes
    3_000_000.0,  // 50 minutes
    3_300_000.0,  // 55 minutes
    3_600_000.0,  // 1 hour
    7_200_000.0,  // 2 hours
    10_800_000.0, // 3 hours
];

const BLOCKS_BUCKETS: &[f64] = &[0.0, 1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 200.0, 500.0, 1000.0];

use crate::steps::{
    between::BetweenMetrics, dispatch_wait::DispatchWaitMetrics, e2e::E2EMetrics,
    relay_wait::RelayWaitMetrics, update_wait::UpdateWaitMetrics,
};

#[derive(Debug)]
pub(crate) struct Metrics {
    wallclock_times: prometheus::HistogramVec,
    event_blocks: prometheus::HistogramVec,
    event_counts: prometheus::IntCounterVec,

    dispatch_to_update_timers: prometheus::HistogramVec,
    dispatch_to_update_blocks: prometheus::HistogramVec,

    update_to_relay_timers: prometheus::HistogramVec,

    relay_to_process_timers: prometheus::HistogramVec,
    relay_to_process_blocks: prometheus::HistogramVec,

    e2e_timers: prometheus::HistogramVec,
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
        let e2e_timers = HistogramVec::new(
            HistogramOpts::new(
                "e2e_ms",
                "Ms between dispatch and associated process, as observed by this agent",
            )
            .namespace(NAMESPACE)
            .buckets(E2E_TIME_BUCKETS.to_vec())
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["chain"],
        )?;

        let update_to_relay_timers = HistogramVec::new(
            HistogramOpts::new(
                "update_to_relay_ms",
                "Ms between update and relay, as observed by this agent",
            )
            .namespace(NAMESPACE)
            .buckets(LOCAL_TIME_BUCKETS.to_vec())
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["chain", "emitter"],
        )?;

        let dispatch_to_update_timers = HistogramVec::new(
            HistogramOpts::new(
                "dispatch_to_update_ms",
                "Ms between dispatch and update, as observed by this agent",
            )
            .namespace(NAMESPACE)
            .buckets(LOCAL_TIME_BUCKETS.to_vec())
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["chain", "emitter"],
        )?;

        let dispatch_to_update_blocks = HistogramVec::new(
            HistogramOpts::new(
                "dispatch_to_update_blocks",
                "Blocks between dispatch and update, as observed by this agent",
            )
            .namespace(NAMESPACE)
            .buckets(BLOCKS_BUCKETS.to_vec())
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["chain", "emitter"],
        )?;

        let relay_to_process_timers = HistogramVec::new(
            HistogramOpts::new(
                "relay_to_process_ms",
                "Ms between relay and process, as observed by this agent",
            )
            .namespace(NAMESPACE)
            .buckets(LOCAL_TIME_BUCKETS.to_vec())
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["chain", "replica_of", "emitter"],
        )?;

        let relay_to_process_blocks = HistogramVec::new(
            HistogramOpts::new(
                "relay_to_process_blocks",
                "Blocks between dispatch and update, as observed by this agent",
            )
            .namespace(NAMESPACE)
            .buckets(BLOCKS_BUCKETS.to_vec())
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["chain", "replica_of", "emitter"],
        )?;

        let wallclock_times = HistogramVec::new(
            HistogramOpts::new(
                "inter_event_period_wallclock_ms",
                "Ms between events periods, as observed by this agent",
            )
            .namespace(NAMESPACE)
            .buckets(LOCAL_TIME_BUCKETS.to_vec())
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["chain", "event", "emitter", "replica_of"],
        )?;

        let event_blocks = HistogramVec::new(
            HistogramOpts::new(
                "inter_event_blocks",
                "Blocks between events, as marked by the chain (i.e. 0 means same block, 1 means next block, etc)",
            )
            .namespace(NAMESPACE)
            .buckets(BLOCKS_BUCKETS.to_vec())
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["chain", "event", "emitter", "replica_of"],
        )?;

        let counts = IntCounterVec::new(
            prometheus::core::Opts::new(
                "event_counts",
                "Counts of each event, labeled by name and chain",
            )
            .namespace(NAMESPACE)
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
        registry
            .register(Box::new(dispatch_to_update_timers.clone()))
            .expect("unable to register metric");
        registry
            .register(Box::new(dispatch_to_update_blocks.clone()))
            .expect("unable to register metric");
        registry
            .register(Box::new(update_to_relay_timers.clone()))
            .expect("unable to register metric");
        registry
            .register(Box::new(relay_to_process_timers.clone()))
            .expect("unable to register metric");
        registry
            .register(Box::new(relay_to_process_blocks.clone()))
            .expect("unable to register metric");

        Ok(Self {
            wallclock_times,
            event_blocks,
            event_counts: counts,
            dispatch_to_update_blocks,
            dispatch_to_update_timers,
            update_to_relay_timers,
            relay_to_process_timers,
            relay_to_process_blocks,
            e2e_timers,
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
        network: &str,
        event: &str,
        emitter: &str,
        replica_of: Option<&str>,
    ) -> IntCounter {
        self.event_counts
            .with_label_values(&[network, event, emitter, replica_of.unwrap_or("n/a")])
    }

    pub(crate) fn wallclock_latency(
        &self,
        network: &str,
        event: &str,
        emitter: &str,
        replica_of: Option<&str>,
    ) -> Histogram {
        self.wallclock_times.with_label_values(&[
            network,
            event,
            emitter,
            replica_of.unwrap_or("n/a"),
        ])
    }

    pub(crate) fn block_latency(
        &self,
        network: &str,
        event: &str,
        emitter: &str,
        replica_of: Option<&str>,
    ) -> Histogram {
        self.event_blocks
            .with_label_values(&[network, event, emitter, replica_of.unwrap_or("n/a")])
    }

    pub(crate) fn between_metrics(
        &self,
        network: &str,
        event: &str,
        emitter: &str,
        replica_of: Option<&str>,
    ) -> BetweenMetrics {
        BetweenMetrics {
            count: self.event_counter(network, event, emitter, replica_of),
            wallclock_latency: self.wallclock_latency(network, event, emitter, replica_of),
            block_latency: self.block_latency(network, event, emitter, replica_of),
        }
    }

    pub(crate) fn dispatch_wait_metrics(
        &self,
        network: &str,
        emitter: &str,
    ) -> DispatchWaitMetrics {
        DispatchWaitMetrics {
            timer: self
                .dispatch_to_update_timers
                .with_label_values(&[network, emitter]),
            blocks: self
                .dispatch_to_update_blocks
                .with_label_values(&[network, emitter]),
        }
    }

    pub(crate) fn update_wait_metrics(&self, network: &str, emitter: &str) -> UpdateWaitMetrics {
        UpdateWaitMetrics {
            times: self
                .update_to_relay_timers
                .with_label_values(&[network, emitter]),
        }
    }

    pub(crate) fn relay_wait_metrics(
        &self,
        network: &str,
        replica_of: &str,
        emitter: &str,
    ) -> RelayWaitMetrics {
        RelayWaitMetrics {
            timers: self
                .relay_to_process_timers
                .with_label_values(&[network, replica_of, emitter]),
            blocks: self
                .relay_to_process_blocks
                .with_label_values(&[network, replica_of, emitter]),
        }
    }

    pub(crate) fn e2e_metrics<'a>(&self, networks: impl Iterator<Item = &'a str>) -> E2EMetrics {
        let timers = networks
            .map(|network| {
                (
                    network.to_owned(),
                    self.e2e_timers.with_label_values(&[network]),
                )
            })
            .collect();

        E2EMetrics { timers }
    }
}
