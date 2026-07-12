use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;
use validator::Validate;

/// Top-level application configuration, loaded from TOML + env vars.
#[derive(Debug, Deserialize, Validate, Clone, Default)]
pub struct Config {
    /// Server bind configuration.
    #[serde(default)]
    pub server: ServerConfig,

    /// PostgreSQL connection configuration.
    #[serde(default)]
    #[validate(nested)]
    pub database: DatabaseConfig,

    /// Observability (logging, metrics, tracing) configuration.
    #[serde(default)]
    pub observability: ObservabilityConfig,

    /// Allowed CORS origins. Empty = allow all (development).
    #[serde(default)]
    pub allowed_origins: Option<Vec<String>>,

    /// Payment gateway (SSLCommerz) configuration.
    #[serde(default)]
    pub payment: PaymentConfig,

    /// Email (SMTP) configuration.
    #[serde(default)]
    pub email: EmailConfig,

    /// Background worker configuration.
    #[serde(default)]
    pub worker: WorkerConfig,

    /// S3-compatible object store configuration.
    #[serde(default)]
    pub s3: S3Config,
}

/// Server bind settings.
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct ServerConfig {
    /// Host address to bind to.
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to listen on.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Number of worker threads (0 = use Tokio default).
    #[serde(default)]
    pub workers: u32,
}

/// PostgreSQL connection settings.
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct DatabaseConfig {
    /// Postgres connection URL (required).
    #[validate(length(min = 1))]
    pub url: String,

    /// Maximum pool size.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Statement timeout in seconds.
    #[serde(default = "default_statement_timeout")]
    pub statement_timeout_secs: u64,
}

/// Observability stack settings.
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct ObservabilityConfig {
    /// Log level (trace, debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Port for the Prometheus metrics endpoint.
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,

    /// Optional OTLP gRPC endpoint for distributed tracing.
    #[serde(default)]
    pub otlp_endpoint: Option<String>,

    /// Trace sample ratio (0.0–1.0).
    #[serde(default = "default_trace_sample_ratio")]
    pub trace_sample_ratio: f64,
}

/// Payment gateway (SSLCommerz) configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct PaymentConfig {
    /// SSLCommerz store ID.
    #[serde(default)]
    pub store_id: String,
    /// SSLCommerz store password.
    #[serde(default)]
    pub store_passwd: String,
    /// Base URL (sandbox or production).
    #[serde(default = "default_payment_gateway_url")]
    pub base_url: String,
    /// Success callback URL.
    #[serde(default)]
    pub success_url: String,
    /// Failure callback URL.
    #[serde(default)]
    pub fail_url: String,
    /// Cancel callback URL.
    #[serde(default)]
    pub cancel_url: String,
    /// IPN notification URL.
    #[serde(default)]
    pub ipn_url: String,
}

impl Default for PaymentConfig {
    fn default() -> Self {
        Self {
            store_id: String::new(),
            store_passwd: String::new(),
            base_url: default_payment_gateway_url(),
            success_url: String::new(),
            fail_url: String::new(),
            cancel_url: String::new(),
            ipn_url: String::new(),
        }
    }
}

/// Email (SMTP) configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
    /// SMTP hostname.
    #[serde(default)]
    pub smtp_host: String,
    /// SMTP port.
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
    /// SMTP username.
    #[serde(default)]
    pub smtp_username: String,
    /// SMTP password.
    #[serde(default)]
    pub smtp_password: String,
    /// "From" email address.
    #[serde(default)]
    pub from_email: String,
    /// "From" display name.
    #[serde(default)]
    pub from_name: String,
    /// Directory containing email templates.
    #[serde(default = "default_template_dir")]
    pub template_dir: String,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: String::new(),
            smtp_port: default_smtp_port(),
            smtp_username: String::new(),
            smtp_password: String::new(),
            from_email: String::new(),
            from_name: "Skill Workshop".to_string(),
            template_dir: default_template_dir(),
        }
    }
}

/// Background worker configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct WorkerConfig {
    /// Unique worker instance ID (auto-generated if empty).
    #[serde(default)]
    pub worker_id: String,
    /// Poll interval in milliseconds.
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
    /// Maximum number of retry attempts per job.
    #[serde(default = "default_max_retry_attempts")]
    pub max_retry_attempts: u32,
    /// Base backoff seconds for retries.
    #[serde(default = "default_base_backoff_seconds")]
    pub base_backoff_seconds: i64,
    /// Job retention days (completed/failed jobs older than this are cleaned up).
    #[serde(default = "default_job_retention_days")]
    pub job_retention_days: i64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            worker_id: String::new(),
            poll_interval_ms: default_poll_interval_ms(),
            max_retry_attempts: default_max_retry_attempts(),
            base_backoff_seconds: default_base_backoff_seconds(),
            job_retention_days: default_job_retention_days(),
        }
    }
}

/// S3-compatible object store configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct S3Config {
    /// S3 bucket name for invoice PDFs.
    #[serde(default)]
    pub invoices_bucket: String,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            invoices_bucket: "invoices".to_string(),
        }
    }
}

fn default_smtp_port() -> u16 {
    587
}

fn default_template_dir() -> String {
    "templates".to_string()
}

fn default_poll_interval_ms() -> u64 {
    5000
}

fn default_max_retry_attempts() -> u32 {
    5
}

fn default_base_backoff_seconds() -> i64 {
    30
}

fn default_job_retention_days() -> i64 {
    30
}

fn default_payment_gateway_url() -> String {
    "https://sandbox.sslcommerz.com".to_string()
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    5000
}

fn default_max_connections() -> u32 {
    10
}

fn default_statement_timeout() -> u64 {
    30
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_metrics_port() -> u16 {
    5001
}

fn default_trace_sample_ratio() -> f64 {
    1.0
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            workers: 0,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: default_max_connections(),
            statement_timeout_secs: default_statement_timeout(),
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            metrics_port: default_metrics_port(),
            otlp_endpoint: None,
            trace_sample_ratio: default_trace_sample_ratio(),
        }
    }
}

impl Config {
    /// Load configuration from TOML files and environment variables.
    ///
    /// Sources (lowest to highest priority):
    /// 1. Compiled-in defaults
    /// 2. `config/default.toml`
    /// 3. Environment variables prefixed with `APP_`, nested with `__`
    ///
    /// Panics if required fields are missing or validation fails.
    pub fn load() -> Self {
        let config: Config = Figment::new()
            .merge(Toml::file("config/default.toml"))
            .merge(Env::prefixed("APP_").split("__"))
            .extract()
            .expect("Failed to load configuration. Check config/default.toml and APP_* env vars.");

        if let Err(errors) = config.validate() {
            panic!("Configuration validation failed: {errors}");
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_validation_fails_on_empty_database_url() {
        let config = Config {
            database: DatabaseConfig {
                url: String::new(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn config_validation_succeeds_with_required_fields() {
        let config = Config {
            database: DatabaseConfig {
                url: "postgres://user:pass@localhost:5432/db".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn config_has_sensible_defaults() {
        let config = Config::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 5000);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.observability.log_level, "info");
    }
}
