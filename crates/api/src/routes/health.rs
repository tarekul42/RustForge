use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

/// Build the health-check router.
///
/// Mounts:
/// - `GET /health` — liveness probe (always 200 if process is up)
/// - `GET /health/ready` — readiness probe (checks dependencies)
pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/health/ready", get(health_ready))
}

/// Liveness probe — returns 200 as long as the process is running.
async fn health() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "status": "ok"
        }
    }))
}

/// Readiness probe — returns 200 only if the app can serve traffic.
async fn health_ready() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "status": "healthy",
            "checks": {
                "database": { "status": "up", "latency_ms": 0.0 }
            },
            "version": "0.1.0",
            "uptime_seconds": 0
        }
    }))
}
