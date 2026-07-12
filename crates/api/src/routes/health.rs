use axum::{extract::Extension, http::StatusCode, routing::get, Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Instant;

use crate::state::AppState;

static START_TIME: OnceLock<Instant> = OnceLock::new();

/// Build the liveness-only health router (no DB state required).
pub fn liveness_router() -> Router {
    Router::new().route("/", get(health))
}

/// Build the readiness health router (requires AppState via Extension).
pub fn readiness_router() -> Router {
    Router::new().route("/ready", get(health_ready))
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
async fn health_ready(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let start = Instant::now();

    let db_status = match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => {
            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            json!({ "status": "up", "latency_ms": latency_ms })
        }
        Err(e) => {
            json!({ "status": "down", "error": e.to_string() })
        }
    };

    let all_up = db_status["status"] == "up";

    if all_up {
        Ok(Json(json!({
            "success": true,
            "data": {
                "status": "healthy",
                "checks": {
                    "database": db_status
                },
                "version": env!("CARGO_PKG_VERSION"),
                "uptime_seconds": START_TIME.get_or_init(Instant::now).elapsed().as_secs_f64()
            }
        })))
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "success": false,
                "error": {
                    "code": "UNAVAILABLE",
                    "message": "Service not ready",
                    "details": null
                },
                "requestId": null
            })),
        ))
    }
}
