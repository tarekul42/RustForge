use crate::state::AppState;
use axum::http::{HeaderMap, StatusCode};
use std::sync::Arc;

/// Helper to extract a session token from request headers.
///
/// Returns the raw token string if a `session` cookie is present.
pub fn extract_token(headers: &HeaderMap) -> Result<&str, (StatusCode, &'static str)> {
    let cookies = headers
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    cookies
        .split(';')
        .filter_map(|c| c.trim().strip_prefix("session="))
        .next()
        .ok_or((StatusCode::UNAUTHORIZED, "Missing session cookie"))
}

/// Resolve a session (session_id + user_id) from request headers.
///
/// Handlers call this manually instead of using a FromRequestParts extractor.
pub async fn resolve_session(
    headers: &HeaderMap,
    state: &Arc<AppState>,
) -> Result<
    (
        sw_domain::value_objects::ids::SessionId,
        sw_domain::value_objects::ids::UserId,
    ),
    (StatusCode, &'static str),
> {
    let token = extract_token(headers)?;

    let lookup = state
        .auth_service
        .lookup_session(token)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid session"))?;

    match lookup {
        Some((session_id, user_id)) => Ok((session_id, user_id)),
        None => Err((StatusCode::UNAUTHORIZED, "Session expired or invalid")),
    }
}
