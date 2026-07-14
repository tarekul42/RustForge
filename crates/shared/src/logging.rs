use crate::config::ObservabilityConfig;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Registry,
};

/// Initialize the tracing subscriber for structured logging.
///
/// Uses JSON formatting in production (`APP_ENV=production`) and
/// pretty formatting in development. Log level is configurable
/// via `ObservabilityConfig::log_level` or `RUST_LOG` env var.
///
/// If `config.otlp_endpoint` is set, also exports spans via OTLP
/// (e.g. to Jaeger, Tempo, or Grafana Cloud).
///
/// # Panics
///
/// Panics if the OTLP exporter cannot be built (e.g., invalid endpoint).
pub fn init(config: &ObservabilityConfig) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let is_production = std::env::var("APP_ENV").is_ok_and(|v| v == "production");

    if let Some(endpoint) = &config.otlp_endpoint {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()
            .expect("Failed to build OTLP span exporter");

        let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_sampler(opentelemetry_sdk::trace::Sampler::TraceIdRatioBased(
                config.trace_sample_ratio,
            ))
            .build();

        if is_production {
            Registry::default()
                .with(env_filter)
                .with(fmt::layer().json().flatten_event(true))
                .with(
                    tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("sw-shared")),
                )
                .init();
        } else {
            Registry::default()
                .with(env_filter)
                .with(fmt::layer().pretty())
                .with(
                    tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("sw-shared")),
                )
                .init();
        }
    } else if is_production {
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
