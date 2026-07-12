use axum::{
    extract::{Extension, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::state::AppState;

/// Axum middleware that rejects requests with `Origin` headers not in the allowed list.
///
/// Reads allowed origins from `AppState::config::allowed_origins`.
pub async fn origin_check_mw(
    Extension(state): Extension<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    let allowed = &state.config.allowed_origins;

    // If no origins are configured, allow all (development mode).
    let origins = match allowed {
        Some(o) if !o.is_empty() => o,
        _ => return Ok(next.run(req).await),
    };

    let origin = req
        .headers()
        .get("Origin")
        .and_then(|v| v.to_str().ok());

    match origin {
        Some(o) if origins.contains(&o.to_string()) => Ok(next.run(req).await),
        Some(_) => Err((StatusCode::FORBIDDEN, "Origin not allowed")),
        None => {
            // Non-browser clients (e.g. mobile apps) may not send Origin.
            Ok(next.run(req).await)
        }
    }
}
