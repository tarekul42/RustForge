use axum::{
    extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router,
};
use std::sync::Arc;

use crate::middleware::request_id::get_current_request_id;
use crate::state::AppState;

/// Build the metrics router — path is relative to `/metrics`.
pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(metrics_handler))
}

/// Render Prometheus metrics, protected by `X-Metrics-Key` header.
async fn metrics_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let config = &state.config;

    // Require API key if one is configured
    if !config.observability.metrics_api_key.is_empty() {
        let provided = headers
            .get("X-Metrics-Key")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if provided != config.observability.metrics_api_key {
            let request_id = get_current_request_id();
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "success": false,
                    "error": {
                        "code": "FORBIDDEN",
                        "message": "invalid or missing X-Metrics-Key header"
                    },
                    "requestId": request_id
                })),
            ));
        }
    }

    let body = sw_shared::metrics::render();
    Ok((
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4",
        )],
        body,
    ))
}
