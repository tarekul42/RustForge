use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::OnceLock;

static PROMETHEUS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Install the Prometheus metrics exporter.
///
/// Must be called once at startup, after `logging::init()`.
///
/// # Panics
///
/// Panics if the recorder is already installed or if installation fails.
pub fn init() {
    let handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .unwrap_or_else(|e| panic!("Failed to install Prometheus metrics exporter: {e}"));
    assert!(
        PROMETHEUS_HANDLE.set(handle).is_ok(),
        "metrics::init() called more than once"
    );
}

/// Render the Prometheus text exposition format for all registered metrics.
///
/// # Panics
///
/// Panics if `init()` has not been called before this function.
pub fn render() -> String {
    PROMETHEUS_HANDLE
        .get()
        .unwrap_or_else(|| panic!("metrics::init() must be called before render()"))
        .render()
}

/// Register metric descriptions for Prometheus exposition.
///
/// Should be called after `init()` so that the Prometheus recorder
/// is available to receive the descriptions.
pub fn register_descriptions() {
    // HTTP
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

    // Database
    metrics::describe_histogram!(
        "db_query_duration_seconds",
        metrics::Unit::Seconds,
        "Database query latency in seconds"
    );
    metrics::describe_gauge!("db_connections_active", "Active Postgres connections");
    metrics::describe_gauge!("db_connections_idle", "Idle Postgres connections");

    // Background jobs
    metrics::describe_gauge!("job_queue_depth", "Job queue size by type and status");
    metrics::describe_histogram!(
        "job_duration_seconds",
        metrics::Unit::Seconds,
        "Job execution time in seconds"
    );
    metrics::describe_counter!(
        "job_attempts_total",
        "Total number of job execution attempts"
    );

    // External service calls
    metrics::describe_histogram!(
        "external_call_duration_seconds",
        metrics::Unit::Seconds,
        "External service call latency in seconds"
    );
    metrics::describe_counter!(
        "external_call_total",
        "Total number of external service calls"
    );

    // Business metrics
    metrics::describe_counter!("otp_send_total", "OTP send attempts by outcome");
    metrics::describe_counter!("payment_total", "Payment state transitions by status");
    metrics::describe_counter!("enrollment_total", "Enrollment state transitions by status");
    metrics::describe_gauge!("session_active_count", "Active session count");
}
