#![deny(missing_docs)]
#![allow(clippy::ignored_unit_patterns)]
#![doc = "API binary: HTTP server entry point for the Skill Workshop Platform."]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use sw_api::app::build_app;
use sw_api::state::AppState;
use sw_shared::config::Config;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{error, info};

/// Run a health check and exit.
/// Used by the Docker HEALTHCHECK directive.
async fn run_health_check() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => std::process::exit(1),
    };

    let pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
    {
        Ok(p) => p,
        Err(_) => std::process::exit(1),
    };

    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => std::process::exit(0),
        Err(_) => std::process::exit(1),
    }
}

/// Application entry point.
///
/// Loads config, initializes logging & metrics, builds the Axum router,
/// and starts the HTTP server with graceful shutdown.
/// Supports `--health-check` for Docker HEALTHCHECK.
#[tokio::main]
async fn main() {
    if std::env::args().any(|a| a == "--health-check") {
        run_health_check().await;
    }

    let config = Config::load();

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to connect to database: {e}");
        });

    let state = Arc::new(AppState::new(config.clone(), pool).await);
    let app = build_app(state);

    let addr = SocketAddr::new(
        config.server.host.parse().unwrap_or_else(|e| panic!("Invalid host address: {e}")),
        config.server.port,
    );
    let listener = TcpListener::bind(addr).await.unwrap_or_else(|e| {
        panic!("Failed to bind to {addr}: {e}");
    });

    info!("Server listening on {addr}");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap_or_else(|e| {
        error!("Server error: {e}");
    });

    info!("Server shut down gracefully");
}

/// Waits for SIGTERM or SIGINT and initiates graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .unwrap_or_else(|e| panic!("failed to install Ctrl+C handler: {e}"));
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .unwrap_or_else(|e| panic!("failed to install SIGTERM handler: {e}"))
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown...");
    tokio::time::sleep(Duration::from_millis(100)).await;
}
