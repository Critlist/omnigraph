use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize the logging system with sensible defaults
pub fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,og=debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Initialize logging with a custom filter
pub fn init_logging_with_filter(filter: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Convenience macros for structured logging
#[macro_export]
macro_rules! log_parse_start {
    ($path:expr) => {
        tracing::info!(path = %$path, "Starting parse");
    };
}

#[macro_export]
macro_rules! log_parse_complete {
    ($path:expr, $nodes:expr, $time_ms:expr) => {
        tracing::info!(
            path = %$path,
            nodes = $nodes,
            time_ms = $time_ms,
            "Parse complete"
        );
    };
}

#[macro_export]
macro_rules! log_metric_computation {
    ($metric:expr, $time_ms:expr) => {
        tracing::debug!(
            metric = %$metric,
            time_ms = $time_ms,
            "Computed metric"
        );
    };
}