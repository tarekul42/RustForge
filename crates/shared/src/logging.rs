use crate::config::ObservabilityConfig;
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*, Registry};

/// Initialize the tracing subscriber for structured logging.
///
/// Uses JSON formatting in production (APP_ENV=production) and
/// pretty formatting in development. Log level is configurable
/// via `ObservabilityConfig::log_level` or `RUST_LOG` env var.
pub fn init(config: &ObservabilityConfig) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let is_production = std::env::var("APP_ENV")
        .map(|v| v == "production")
        .unwrap_or(false);

    let subscriber = Registry::default().with(env_filter);

    if is_production {
        let json_layer = fmt::layer().json().flatten_event(true);
        subscriber.with(json_layer).init();
    } else {
        let pretty_layer = fmt::layer().pretty();
        subscriber.with(pretty_layer).init();
    }
}
