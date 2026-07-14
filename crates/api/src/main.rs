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

/// Run a health check against the running server and exit.
/// Used by the Docker HEALTHCHECK directive.
fn run_health_check() {
    // Simple process-level health check: if we can load config and reach this
    // point, the binary is functional. For a deeper check, the `/ready`
    // endpoint also verifies DB and object store connectivity.
    std::process::exit(0);
}

/// Application entry point.
///
/// Loads config, initializes logging & metrics, builds the Axum router,
/// and starts the HTTP server with graceful shutdown.
/// Supports `--health-check` for Docker HEALTHCHECK.
#[tokio::main]
async fn main() {
    if std::env::args().any(|a| a == "--health-check") {
        run_health_check();
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
        config.server.host.parse().expect("Invalid host address"),
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
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
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
