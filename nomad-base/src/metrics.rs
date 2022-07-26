//! Useful metrics that all agents should track.

use color_eyre::Result;
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry,
};
use std::sync::Arc;
use tokio::task::JoinHandle;

const NAMESPACE: &str = "nomad";

fn u16_from_env(s: impl AsRef<str>) -> Option<u16> {
    std::env::var(s.as_ref()).ok().and_then(|i| i.parse().ok())
}

#[derive(Debug)]
/// Metrics for a particular domain
pub struct CoreMetrics {
    agent_name: String,
    home_name: String,
    transactions: Box<IntGaugeVec>,
    wallet_balance: Box<IntGaugeVec>,
    channel_faults: Box<IntGaugeVec>,
    rpc_latencies: Box<HistogramVec>,
    span_durations: Box<HistogramVec>,
    home_failure_checks: Box<IntGaugeVec>,
    home_failure_observations: Box<IntGaugeVec>,
    listen_port: Option<u16>,
    /// Metrics registry for adding new metrics and gathering reports
    registry: Arc<Registry>,
}

impl CoreMetrics {
    /// Track metrics for a particular agent name.
    pub fn new<S: Into<String>>(
        for_agent: S,
        home_name: S,
        listen_port: Option<u16>,
        registry: Arc<Registry>,
    ) -> prometheus::Result<CoreMetrics> {
        let metrics = CoreMetrics {
            agent_name: for_agent.into(),
            home_name: home_name.into(),
            transactions: Box::new(IntGaugeVec::new(
                Opts::new(
                    "transactions_total",
                    "Number of transactions sent by this agent since boot",
                )
                .namespace(NAMESPACE)
                .const_label("VERSION", env!("CARGO_PKG_VERSION")),
                &["chain", "wallet", "agent"],
            )?),
            wallet_balance: Box::new(IntGaugeVec::new(
                Opts::new(
                    "wallet_balance_total",
                    "Balance of the smart contract wallet",
                )
                .namespace(NAMESPACE)
                .const_label("VERSION", env!("CARGO_PKG_VERSION")),
                &["chain", "wallet", "agent"],
            )?),
            channel_faults: Box::new(IntGaugeVec::new(
                Opts::new(
                    "channel_faults",
                    "Number of per home <> replica channel faults (errors)",
                )
                .namespace(NAMESPACE)
                .const_label("VERSION", env!("CARGO_PKG_VERSION")),
                &["home", "replica", "agent"],
            )?),
            rpc_latencies: Box::new(HistogramVec::new(
                HistogramOpts::new(
                    "rpc_duration_ms",
                    "Duration from dispatch to receipt-of-response for RPC calls",
                )
                .namespace(NAMESPACE)
                .const_label("VERSION", env!("CARGO_PKG_VERSION")),
                &["chain", "method", "agent"],
            )?),
            span_durations: Box::new(HistogramVec::new(
                HistogramOpts::new(
                    "span_duration_sec",
                    "Duration from span creation to span destruction",
                )
                .namespace(NAMESPACE)
                .const_label("VERSION", env!("CARGO_PKG_VERSION")),
                &["span_name", "target"],
            )?),
            home_failure_checks: Box::new(IntGaugeVec::new(
                Opts::new(
                    "home_failure_checks",
                    "Number of times agent has checked home for failed state",
                )
                .namespace(NAMESPACE)
                .const_label("VERSION", env!("CARGO_PKG_VERSION")),
                &["home", "agent"]
            )?),
            home_failure_observations: Box::new(IntGaugeVec::new(
                Opts::new(
                    "home_failure_observations",
                    "Number of times agent has seen the home failed (anything > 0 is major red flag!)",
                )
                .namespace(NAMESPACE)
                .const_label("VERSION", env!("CARGO_PKG_VERSION")),
                &["home", "agent"]
            )?),
            registry,
            listen_port,
        };

        // TODO: only register these if they aren't already registered?

        metrics.registry.register(metrics.transactions.clone())?;
        metrics.registry.register(metrics.wallet_balance.clone())?;
        metrics.registry.register(metrics.rpc_latencies.clone())?;
        metrics.registry.register(metrics.span_durations.clone())?;
        metrics.registry.register(metrics.channel_faults.clone())?;
        metrics
            .registry
            .register(metrics.home_failure_checks.clone())?;
        metrics
            .registry
            .register(metrics.home_failure_observations.clone())?;

        Ok(metrics)
    }

    /// Register an int gauge vec
    pub fn new_int_gauge_vec(
        &self,
        metric_name: &str,
        help: &str,
        labels: &[&str],
    ) -> Result<prometheus::IntGaugeVec> {
        let gauge_vec = IntGaugeVec::new(
            Opts::new(metric_name, help)
                .namespace(NAMESPACE)
                .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            labels,
        )?;
        self.registry.register(Box::new(gauge_vec.clone()))?;

        Ok(gauge_vec)
    }

    /// Register an int counter.
    pub fn new_int_counter(
        &self,
        metric_name: &str,
        help: &str,
        labels: &[&str],
    ) -> Result<prometheus::IntCounterVec> {
        let counter = IntCounterVec::new(
            Opts::new(metric_name, help)
                .namespace(NAMESPACE)
                .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            labels,
        )?;

        self.registry.register(Box::new(counter.clone()))?;

        Ok(counter)
    }

    /// Register a histogram.
    pub fn new_histogram(
        &self,
        metric_name: &str,
        help: &str,
        labels: &[&str],
        buckets: &[f64],
    ) -> Result<prometheus::HistogramVec> {
        let histogram = HistogramVec::new(
            HistogramOpts::new(metric_name, help)
                .namespace(NAMESPACE)
                .buckets(buckets.to_owned())
                .const_label("VERSION", env!("CARGO_PKG_VERSION")),
            labels,
        )?;

        self.registry.register(Box::new(histogram.clone()))?;

        Ok(histogram)
    }

    /// Call with the new balance when gas is spent.
    pub fn wallet_balance_changed(
        &self,
        chain: &str,
        address: ethers::types::Address,
        current_balance: ethers::types::U256,
    ) {
        self.wallet_balance
            .with_label_values(&[chain, &format!("{:x}", address), &self.agent_name])
            .set(current_balance.as_u64() as i64) // XXX: truncated data
    }

    /// Return single gauge for one home <> replica channel
    pub fn channel_faults_gauge(&self, replica: &str) -> IntGauge {
        self.channel_faults
            .with_label_values(&[&self.home_name, replica, &self.agent_name])
    }

    /// Return home failure checks gauge
    pub fn home_failure_checks(&self) -> IntGauge {
        self.home_failure_checks
            .with_label_values(&[&self.home_name, &self.agent_name])
    }

    /// Return home failure observations gauge
    pub fn home_failure_observations(&self) -> IntGauge {
        self.home_failure_observations
            .with_label_values(&[&self.home_name, &self.agent_name])
    }

    /// Call with RPC duration after it is complete
    pub fn rpc_complete(&self, chain: &str, method: &str, duration_ms: f64) {
        self.rpc_latencies
            .with_label_values(&[chain, method, &self.agent_name])
            .observe(duration_ms)
    }

    /// Histogram for measuring span durations.
    ///
    /// Labels needed: `span_name`, `target`.
    pub fn span_duration(&self) -> HistogramVec {
        *self.span_durations.clone()
    }

    /// Gather available metrics into an encoded (plaintext, OpenMetrics format) report.
    pub fn gather(&self) -> prometheus::Result<Vec<u8>> {
        let collected_metrics = self.registry.gather();
        let mut out_buf = Vec::with_capacity(1024 * 64);
        let encoder = prometheus::TextEncoder::new();
        encoder.encode(&collected_metrics, &mut out_buf)?;
        Ok(out_buf)
    }

    /// Run an HTTP server serving OpenMetrics format reports on `/metrics`
    ///
    /// This is compatible with Prometheus, which ought to be configured to scrape me!
    pub fn run_http_server(self: Arc<CoreMetrics>) -> JoinHandle<()> {
        use warp::Filter;

        // Default to port 9090
        let port = u16_from_env("METRICS_PORT")
            .or(self.listen_port)
            .unwrap_or(9090);
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
                            self.gather().expect("failed to encode metrics"),
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
}
