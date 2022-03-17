use color_eyre::Result;
use tracing_subscriber::{filter::LevelFilter, prelude::*};

use crate::settings::trace::fmt::Style;

/// Configure a `tracing_subscriber::fmt` Layer outputting to stdout
pub mod fmt;

use self::fmt::LogOutputLayer;

mod span_metrics;

pub use span_metrics::TimeSpanLifetime;

/// Logging level
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Level {
    /// Off
    Off,
    /// Error
    Error,
    /// Warn
    Warn,
    /// Debug
    Debug,
    /// Trace
    Trace,
    /// Info
    #[serde(other)]
    Info,
}

impl Level {
    fn to_filter(self) -> LevelFilter {
        self.into()
    }
}

impl Default for Level {
    fn default() -> Self {
        Level::Info
    }
}

impl From<Level> for LevelFilter {
    fn from(level: Level) -> LevelFilter {
        match level {
            Level::Off => LevelFilter::OFF,
            Level::Error => LevelFilter::ERROR,
            Level::Warn => LevelFilter::WARN,
            Level::Debug => LevelFilter::DEBUG,
            Level::Trace => LevelFilter::TRACE,
            Level::Info => LevelFilter::INFO,
        }
    }
}

/// Configuration for the tracing subscribers used by Nomad agents
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TracingConfig {
    #[serde(default)]
    fmt: Style,
    #[serde(default)]
    level: Level,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            fmt: Style::Pretty,
            level: Level::Trace,
        }
    }
}

impl TracingConfig {
    /// Attempt to instantiate and register a tracing subscriber setup from settings.
    pub fn start_tracing(&self, latencies: prometheus::HistogramVec) -> Result<()> {
        let fmt_layer: LogOutputLayer<_> = self.fmt.into();
        let err_layer = tracing_error::ErrorLayer::default();

        let subscriber = tracing_subscriber::Registry::default()
            .with(TimeSpanLifetime::new(latencies))
            .with(self.level.to_filter())
            .with(fmt_layer)
            .with(err_layer);

        subscriber.try_init()?;
        Ok(())
    }
}
