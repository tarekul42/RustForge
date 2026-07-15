use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use sw_shared::config::DatabaseConfig;

/// Create a `PgPool` from the application database configuration.
///
/// Applies `max_connections` and `statement_timeout` from config.
pub async fn create_pool(config: &DatabaseConfig) -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Failed to connect to PostgreSQL. Check DATABASE_URL or config/database.url: {e}"
            )
        });

    // Set per-connection statement timeout.
    let timeout_ms = (config.statement_timeout_secs * 1000) as i64;
    sqlx::query("SET statement_timeout = $1")
        .bind(timeout_ms)
        .execute(&pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to set statement_timeout: {e}"));

    pool
}
