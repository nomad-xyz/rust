use std::{collections::HashMap, sync::Arc};

use prometheus::{
    Encoder, Histogram, HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGaugeVec,
};
use tokio::task::JoinHandle;
use warp::Filter;

const NAMESPACE: &str = "nomad_monitor";

const TIME_BUCKETS: &[f64] = &[
    1.0,     // 1 sec
    5.0,     // 5 secs
    30.0,    // 30 secs
    60.0,    // 1 min
    120.0,   // 2 min
    600.0,   // 10 min
    1_800.0, // 30 min
    3_600.0, // 1 hour
    7_200.0, // 2 hour
];
// time buckets for e2e metric
const E2E_TIME_BUCKETS: &[f64] = &[
    2_100.0,  // 35 min
    2_400.0,  // 40 min
    2_700.0,  // 45 min
    3_000.0,  // 50 min
    3_300.0,  // 55 min
    3_600.0,  // 1 hour
    7_200.0,  // 2 hours
    10_800.0, // 3 hours
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
    unprocessed_dispatches: prometheus::IntGaugeVec,
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
        let unprocessed_dispatches = IntGaugeVec::new(
            prometheus::core::Opts::new(
                "unprocessed_messages",
                "Dispatch events for which no corresponding process has been observed",
            )
            .namespace(NAMESPACE)
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["home_network"],
        )?;

        let e2e_timers = HistogramVec::new(
            HistogramOpts::new(
                "e2e_sec",
                "Seconds between dispatch and associated process, as observed by this agent",
            )
            .namespace(NAMESPACE)
            .buckets(E2E_TIME_BUCKETS.to_vec())
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["chain"],
        )?;

        let update_to_relay_timers = HistogramVec::new(
            HistogramOpts::new(
                "update_to_relay_secs",
                "Seconds between update and relay, as observed by this agent",
            )
            .namespace(NAMESPACE)
            .buckets(TIME_BUCKETS.to_vec())
            .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            &["chain", "emitter", "replica_chain"],
        )?;

        let dispatch_to_update_timers = HistogramVec::new(
            HistogramOpts::new(
                "dispatch_to_update_secs",
                "Seconds between dispatch and update, as observed by this agent",
            )
            .namespace(NAMESPACE)
            .buckets(TIME_BUCKETS.to_vec())
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
                "relay_to_process_secs",
                "Seconds between relay and process, as observed by this agent",
            )
            .namespace(NAMESPACE)
            .buckets(TIME_BUCKETS.to_vec())
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
                "inter_event_period_wallclock_secs",
                "Seconds between events periods, as observed by this agent",
            )
            .namespace(NAMESPACE)
            .buckets(TIME_BUCKETS.to_vec())
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
        registry
            .register(Box::new(e2e_timers.clone()))
            .expect("unable to register metric");
        registry
            .register(Box::new(unprocessed_dispatches.clone()))
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
            unprocessed_dispatches,
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
                        warp::http::Response::builder()
                            .header("Location", "/metrics")
                            .status(301)
                            .body("".to_string())
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

    pub(crate) fn update_wait_metrics(
        &self,
        home_network: &str,
        replica_networks: &[&str],
        emitter: &str,
    ) -> UpdateWaitMetrics {
        let times = replica_networks
            .iter()
            .map(|replica_network| {
                let timer = self.update_to_relay_timers.with_label_values(&[
                    home_network,
                    emitter,
                    replica_network,
                ]);
                (replica_network.to_string(), timer)
            })
            .collect();

        UpdateWaitMetrics { times }
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
        let mut gauges = HashMap::new();
        let mut timers = HashMap::new();

        for network in networks {
            timers.insert(
                network.to_owned(),
                self.e2e_timers.with_label_values(&[network]),
            );
            gauges.insert(
                network.to_owned(),
                self.unprocessed_dispatches.with_label_values(&[network]),
            );
        }

        E2EMetrics { timers, gauges }
    }
}
