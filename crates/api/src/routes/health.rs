use axum::{extract::State, http::HeaderMap, http::StatusCode, routing::get, Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Instant;

use crate::extractors::auth_user;
use crate::middleware::request_id::get_current_request_id;
use crate::state::AppState;

static START_TIME: OnceLock<Instant> = OnceLock::new();

/// Build the liveness-only health router (no DB state required).
pub fn liveness_router() -> Router {
    Router::new().route("/", get(health))
}

/// Build the readiness health router (requires AppState).
pub fn readiness_router() -> Router<Arc<AppState>> {
    Router::new().route("/ready", get(health_ready))
}

/// Build the admin dashboard router (requires admin auth).
pub fn dashboard_router() -> Router<Arc<AppState>> {
    Router::new().route("/dashboard", get(health_dashboard))
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
    State(state): State<Arc<AppState>>,
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

    let os_start = Instant::now();
    let os_status = match state.object_store.health_check().await {
        Ok(_) => {
            let latency_ms = os_start.elapsed().as_secs_f64() * 1000.0;
            json!({ "status": "up", "latency_ms": latency_ms })
        }
        Err(e) => {
            json!({ "status": "down", "error": e.to_string() })
        }
    };

    let all_up = db_status["status"] == "up" && os_status["status"] == "up";

    if all_up {
        Ok(Json(json!({
            "success": true,
            "data": {
                "status": "healthy",
                "checks": {
                    "database": db_status,
                    "object_store": os_status
                },
                "version": env!("CARGO_PKG_VERSION"),
                "uptime_seconds": START_TIME.get_or_init(Instant::now).elapsed().as_secs_f64()
            }
        })))
    } else {
        let request_id = get_current_request_id();
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "success": false,
                "error": {
                    "code": "UNAVAILABLE",
                    "message": "Service not ready",
                    "details": null
                },
                "requestId": request_id
            })),
        ))
    }
}

/// Admin dashboard — detailed health info (DB pool, queue depth, etc.).
async fn health_dashboard(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = auth_user::resolve_auth_user(&headers, &state)
        .await
        .map_err(|(code, msg)| {
            let request_id = get_current_request_id();
            (
                code,
                Json(json!({
                    "success": false,
                    "error": { "code": "UNAUTHORIZED", "message": msg },
                    "requestId": request_id
                })),
            )
        })?;

    use sw_domain::aggregates::user::UserRole;
    if !matches!(auth.role, UserRole::Admin | UserRole::SuperAdmin) {
        let request_id = get_current_request_id();
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "success": false,
                "error": { "code": "FORBIDDEN", "message": "Admin access required" },
                "requestId": request_id
            })),
        ));
    }

    let start = Instant::now();

    // DB connectivity check
    let db_status = match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => json!({ "status": "up" }),
        Err(_) => json!({ "status": "down" }),
    };

    // Pool stats
    let pool_size = state.pool.size() as i64;
    let pool_idle = state.pool.num_idle() as i64;
    let pool_active = pool_size - pool_idle;

    // Job queue depth
    let job_depth = sqlx::query_as::<_, (String, i64)>(
        "SELECT status, COUNT(*)::int8 as count FROM jobs GROUP BY status",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let queue_depth: serde_json::Map<String, serde_json::Value> = job_depth
        .into_iter()
        .map(|(status, count)| (status, json!(count)))
        .collect();

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(Json(json!({
        "success": true,
        "data": {
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": START_TIME.get_or_init(Instant::now).elapsed().as_secs_f64(),
            "latency_ms": elapsed_ms,
            "database": {
                "connectivity": db_status,
                "pool": {
                    "size": pool_size,
                    "active": pool_active,
                    "idle": pool_idle
                }
            },
            "queue": {
                "depth": queue_depth
            }
        }
    })))
}
