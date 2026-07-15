use crate::extractors::session;
use crate::state::AppState;
use axum::http::{HeaderMap, StatusCode};
use std::sync::Arc;

/// Result of a successful authentication containing session and user info.
pub struct AuthUser {
    /// The authenticated user's ID.
    pub user_id: sw_domain::value_objects::ids::UserId,
    /// The authenticated user's role.
    pub role: sw_domain::aggregates::user::UserRole,
    /// The session ID.
    pub session_id: sw_domain::value_objects::ids::SessionId,
}

/// Resolve an authenticated user from request headers (session cookie).
///
/// Returns the `AuthUser` on success, or a `(StatusCode, &str)` error tuple.
pub async fn resolve_auth_user(
    headers: &HeaderMap,
    state: &Arc<AppState>,
) -> Result<AuthUser, (StatusCode, &'static str)> {
    let (session_id, user_id) = session::resolve_session(headers, state).await?;
    let user = state
        .auth_service
        .get_user(user_id)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "User not found"))?;

    Ok(AuthUser {
        user_id,
        role: user.role(),
        session_id,
    })
}
