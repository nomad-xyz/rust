/// Configure a `tracing_subscriber::fmt` Layer outputting to stdout
pub mod fmt;

mod span_metrics;
pub use span_metrics::TimeSpanLifetime;
