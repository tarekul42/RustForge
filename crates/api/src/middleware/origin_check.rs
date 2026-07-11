use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Allowed origins for the API.
#[derive(Clone)]
pub struct AllowedOrigins {
    origins: Arc<Vec<String>>,
}

impl AllowedOrigins {
    pub fn new(origins: Vec<String>) -> Self {
        Self {
            origins: Arc::new(origins),
        }
    }
}

/// Axum middleware that rejects requests with `Origin` headers not in the allowed list.
pub async fn origin_check_mw(
    State(allowed): State<AllowedOrigins>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    // If no origins are configured, allow all (development mode).
    if allowed.origins.is_empty() {
        return Ok(next.run(req).await);
    }

    let origin = req
        .headers()
        .get("Origin")
        .and_then(|v| v.to_str().ok());

    match origin {
        Some(o) if allowed.origins.contains(&o.to_string()) => Ok(next.run(req).await),
        Some(_) => Err((StatusCode::FORBIDDEN, "Origin not allowed")),
        None => {
            // Non-browser clients (e.g. mobile apps) may not send Origin.
            Ok(next.run(req).await)
        }
    }
}
