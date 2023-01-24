use prometheus::{Encoder, IntGaugeVec, Opts, Registry};
use std::sync::Arc;
use tokio::task::JoinHandle;

fn u16_from_env(s: impl AsRef<str>) -> Option<u16> {
    std::env::var(s.as_ref()).ok().and_then(|i| i.parse().ok())
}

#[derive(Debug)]
/// Metrics for a particular domain
pub struct Metrics {
    incoming_requests: Box<IntGaugeVec>,
    backoffs: Box<IntGaugeVec>,
    restarts: Box<IntGaugeVec>,
    listen_port: Option<u16>,
    /// Metrics registry for adding new metrics and gathering reports
    registry: Arc<Registry>,
}

impl Metrics {
    /// Track metrics for a particular agent name.
    pub fn new(listen_port: Option<u16>, registry: Arc<Registry>) -> prometheus::Result<Self> {
        let metrics = Self {
            incoming_requests: Box::new(IntGaugeVec::new(
                Opts::new(
                    "incoming_requests",
                    "Number of incoming requests to the server",
                )
                .namespace("nomad")
                .const_label("VERSION", env!("CARGO_PKG_VERSION")),
                &["request", "network", "agent"],
            )?),
            backoffs: Box::new(IntGaugeVec::new(
                Opts::new("backoffs_fired", "Number of backoffs fired by type")
                    .namespace("nomad")
                    .const_label("VERSION", env!("CARGO_PKG_VERSION")),
                &["type", "network", "agent"],
            )?),
            restarts: Box::new(IntGaugeVec::new(
                Opts::new("restarts", "Number of restarts of an agent")
                    .namespace("nomad")
                    .const_label("VERSION", env!("CARGO_PKG_VERSION")),
                &["network", "agent"],
            )?),
            registry,
            listen_port,
        };

        // TODO: only register these if they aren't already registered?

        metrics
            .registry
            .register(metrics.incoming_requests.clone())?;
        metrics.registry.register(metrics.backoffs.clone())?;
        metrics.registry.register(metrics.restarts.clone())?;

        Ok(metrics)
    }

    /// Call when a new request is handled.
    pub fn incoming_requests_inc(&self, request_name: &str, network: &str, agent: &str) {
        self.incoming_requests
            .with_label_values(&[request_name, network, agent])
            .inc()
    }

    /// Call when a backoff is fired.
    pub fn backoffs_inc(&self, backoff_type: &str, network: &str, agent: &str) {
        self.backoffs
            .with_label_values(&[backoff_type, network, agent])
            .inc()
    }

    /// Call when a restart signal successfully sent to a pod.
    pub fn restarts_inc(&self, network: &str, agent: &str) {
        self.restarts.with_label_values(&[network, agent]).inc()
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
    pub fn run_http_server(self: Arc<Self>) -> JoinHandle<()> {
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
