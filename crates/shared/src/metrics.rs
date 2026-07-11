/// Install the Prometheus metrics exporter.
///
/// Must be called once at startup, after `logging::init()`.
pub fn init() {
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .install()
        .expect("Failed to install Prometheus metrics exporter");
}

/// Register metric descriptions for Prometheus exposition.
///
/// Should be called after `init()` so that the Prometheus recorder
/// is available to receive the descriptions.
pub fn register_descriptions() {
    metrics::describe_histogram!(
        "http_request_duration_seconds",
        metrics::Unit::Seconds,
        "Request latency in seconds"
    );
    metrics::describe_counter!("http_requests_total", "Total number of HTTP requests");
    metrics::describe_gauge!(
        "http_in_flight_requests",
        "Current number of in-flight requests"
    );
}
