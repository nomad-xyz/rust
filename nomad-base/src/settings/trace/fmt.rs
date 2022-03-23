use nomad_xyz_configuration::agent::{LogLevel, LogStyle};
use std::io::Stdout;
use tracing::{span, Subscriber};
use tracing_subscriber::{
    filter::LevelFilter,
    fmt::{
        self,
        format::{Compact, DefaultFields, Format, Full, Json, JsonFields, Pretty},
    },
    registry::LookupSpan,
    Layer,
};

/// Convert configuration LogLevel to tracing LevelFilter
pub fn log_level_to_level_filter(level: LogLevel) -> LevelFilter {
    match level {
        LogLevel::Off => LevelFilter::OFF,
        LogLevel::Error => LevelFilter::ERROR,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Trace => LevelFilter::TRACE,
        LogLevel::Info => LevelFilter::INFO,
    }
}

/// Unification of the fmt Subscriber formatting modes
///
/// You may be asking yourself, why does this exist. I ask myself the same thing
/// every day.
///
/// It exists because the type params on the Layer affect the type params type
/// params on the produced `Layered` Subscriber once the layer has been
/// applied. This increases the complexity of the code that instantiates the
/// `Registry` and adds the layers. Because each combination of layers produces
/// a different type, each combination must be handled explicitly. This is fine
/// if you expect a static configuration of layers, but since we really want
/// this to be configurable and the code to be legible, we do a little
/// unification here :)
#[derive(Debug)]
pub enum LogOutputLayer<S, N = DefaultFields, W = fn() -> Stdout> {
    /// Full log output (default mode)
    Full(fmt::Layer<S, N, Format<Full>, W>),
    /// Pretty log output
    Pretty(fmt::Layer<S, Pretty, Format<Pretty>, W>),
    /// Compact log output
    Compact(fmt::Layer<S, N, Format<Compact>, W>),
    /// Json log output
    Json(fmt::Layer<S, JsonFields, Format<Json>, W>),
}

impl<S> Default for LogOutputLayer<S> {
    fn default() -> Self {
        Self::Full(Default::default())
    }
}

impl<S> From<LogStyle> for LogOutputLayer<S> {
    fn from(style: LogStyle) -> Self {
        match style {
            LogStyle::Full => Self::Full(fmt::layer()),
            LogStyle::Pretty => Self::Pretty(fmt::layer().pretty()),
            LogStyle::Compact => Self::Compact(fmt::layer().compact()),
            LogStyle::Json => Self::Json(fmt::layer().json()),
        }
    }
}

impl<S> Layer<S> for LogOutputLayer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn register_callsite(
        &self,
        metadata: &'static tracing::Metadata<'static>,
    ) -> tracing::subscriber::Interest {
        match self {
            LogOutputLayer::Full(inner) => inner.register_callsite(metadata),
            LogOutputLayer::Pretty(inner) => inner.register_callsite(metadata),
            LogOutputLayer::Compact(inner) => inner.register_callsite(metadata),
            LogOutputLayer::Json(inner) => inner.register_callsite(metadata),
        }
    }

    fn enabled(
        &self,
        metadata: &tracing::Metadata<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        match self {
            LogOutputLayer::Full(inner) => inner.enabled(metadata, ctx),
            LogOutputLayer::Pretty(inner) => inner.enabled(metadata, ctx),
            LogOutputLayer::Compact(inner) => inner.enabled(metadata, ctx),
            LogOutputLayer::Json(inner) => inner.enabled(metadata, ctx),
        }
    }

    fn new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        match self {
            LogOutputLayer::Full(inner) => inner.new_span(attrs, id, ctx),
            LogOutputLayer::Pretty(inner) => inner.new_span(attrs, id, ctx),
            LogOutputLayer::Compact(inner) => inner.new_span(attrs, id, ctx),
            LogOutputLayer::Json(inner) => inner.new_span(attrs, id, ctx),
        }
    }

    fn max_level_hint(&self) -> Option<tracing::metadata::LevelFilter> {
        match self {
            LogOutputLayer::Full(inner) => inner.max_level_hint(),
            LogOutputLayer::Pretty(inner) => inner.max_level_hint(),
            LogOutputLayer::Compact(inner) => inner.max_level_hint(),
            LogOutputLayer::Json(inner) => inner.max_level_hint(),
        }
    }

    fn on_record(
        &self,
        span: &span::Id,
        values: &span::Record<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        match self {
            LogOutputLayer::Full(inner) => inner.on_record(span, values, ctx),
            LogOutputLayer::Pretty(inner) => inner.on_record(span, values, ctx),
            LogOutputLayer::Compact(inner) => inner.on_record(span, values, ctx),
            LogOutputLayer::Json(inner) => inner.on_record(span, values, ctx),
        }
    }

    fn on_follows_from(
        &self,
        span: &span::Id,
        follows: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        match self {
            LogOutputLayer::Full(inner) => inner.on_follows_from(span, follows, ctx),
            LogOutputLayer::Pretty(inner) => inner.on_follows_from(span, follows, ctx),
            LogOutputLayer::Compact(inner) => inner.on_follows_from(span, follows, ctx),
            LogOutputLayer::Json(inner) => inner.on_follows_from(span, follows, ctx),
        }
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        match self {
            LogOutputLayer::Full(inner) => inner.on_event(event, ctx),
            LogOutputLayer::Pretty(inner) => inner.on_event(event, ctx),
            LogOutputLayer::Compact(inner) => inner.on_event(event, ctx),
            LogOutputLayer::Json(inner) => inner.on_event(event, ctx),
        }
    }

    fn on_enter(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        match self {
            LogOutputLayer::Full(inner) => inner.on_enter(id, ctx),
            LogOutputLayer::Pretty(inner) => inner.on_enter(id, ctx),
            LogOutputLayer::Compact(inner) => inner.on_enter(id, ctx),
            LogOutputLayer::Json(inner) => inner.on_enter(id, ctx),
        }
    }

    fn on_exit(&self, id: &span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        match self {
            LogOutputLayer::Full(inner) => inner.on_exit(id, ctx),
            LogOutputLayer::Pretty(inner) => inner.on_exit(id, ctx),
            LogOutputLayer::Compact(inner) => inner.on_exit(id, ctx),
            LogOutputLayer::Json(inner) => inner.on_exit(id, ctx),
        }
    }

    fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        match self {
            LogOutputLayer::Full(inner) => inner.on_close(id, ctx),
            LogOutputLayer::Pretty(inner) => inner.on_close(id, ctx),
            LogOutputLayer::Compact(inner) => inner.on_close(id, ctx),
            LogOutputLayer::Json(inner) => inner.on_close(id, ctx),
        }
    }

    fn on_id_change(
        &self,
        old: &span::Id,
        new: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        match self {
            LogOutputLayer::Full(inner) => inner.on_id_change(old, new, ctx),
            LogOutputLayer::Pretty(inner) => inner.on_id_change(old, new, ctx),
            LogOutputLayer::Compact(inner) => inner.on_id_change(old, new, ctx),
            LogOutputLayer::Json(inner) => inner.on_id_change(old, new, ctx),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[derive(serde::Deserialize)]
    struct TestStyle {
        style: LogStyle,
    }

    #[test]
    fn it_deserializes_formatting_strings() {
        let case = r#"{"style": "pretty"}"#;
        assert_eq!(
            serde_json::from_str::<TestStyle>(case).unwrap().style,
            LogStyle::Pretty
        );

        let case = r#"{"style": "compact"}"#;
        assert_eq!(
            serde_json::from_str::<TestStyle>(case).unwrap().style,
            LogStyle::Compact
        );

        let case = r#"{"style": "full"}"#;
        assert_eq!(
            serde_json::from_str::<TestStyle>(case).unwrap().style,
            LogStyle::Full
        );

        let case = r#"{"style": "json"}"#;
        assert_eq!(
            serde_json::from_str::<TestStyle>(case).unwrap().style,
            LogStyle::Json
        );

        let case = r#"{"style": "toast"}"#;
        assert_eq!(
            serde_json::from_str::<TestStyle>(case).unwrap().style,
            LogStyle::Full
        );
    }
}
