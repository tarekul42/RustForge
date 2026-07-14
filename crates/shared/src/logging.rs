use crate::config::ObservabilityConfig;
use tracing_subscriber::{
    Registry, filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

/// Initialize the tracing subscriber for structured logging.
///
/// Uses JSON formatting in production (`APP_ENV=production`) and
/// pretty formatting in development. Log level is configurable
/// via `ObservabilityConfig::log_level` or `RUST_LOG` env var.
pub fn init(config: &ObservabilityConfig) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let is_production = std::env::var("APP_ENV").is_ok_and(|v| v == "production");

    if is_production {
        Registry::default()
            .with(env_filter)
            .with(fmt::layer().json().flatten_event(true))
            .init();
    } else {
        Registry::default()
            .with(env_filter)
            .with(fmt::layer().pretty())
            .init();
    }
}
