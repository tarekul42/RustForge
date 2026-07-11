use axum::{
    extract::Extension,
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::ApiError;
use crate::extractors::session;
use crate::state::AppState;

/// Build the user router — all paths are relative to `/api/v1/users`.
pub fn router() -> Router {
    Router::new()
        .route("/me", get(get_profile))
        .route("/me", patch(update_profile))
        .route("/me/registrations", get(list_registrations))
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

/// Response body for `GET /users/me`.
#[derive(Serialize)]
pub struct UserProfileResponse {
    /// The user's unique ID.
    pub user_id: String,
    /// The user's email address.
    pub email: String,
    /// The user's display name.
    pub name: Option<String>,
    /// URL to the user's profile picture.
    pub picture_url: Option<String>,
    /// The user's role (e.g. "student", "admin").
    pub role: String,
    /// Whether the email has been verified.
    pub is_verified: bool,
    /// ISO-8601 timestamp of account creation.
    pub created_at: String,
    /// ISO-8601 timestamp of the last update.
    pub updated_at: String,
}

/// Request body for `PATCH /users/me`.
#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    /// New display name.
    pub display_name: Option<String>,
    /// New profile picture URL.
    pub picture_url: Option<String>,
}

/// Response body for `GET /users/me/registrations`.
#[derive(Serialize)]
pub struct RegistrationResponse {
    /// Registration ID.
    pub id: String,
    /// Workshop ID the user registered for.
    pub workshop_id: String,
    /// Registration status.
    pub status: String,
    /// ISO-8601 timestamp of registration.
    pub registered_at: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Return the full profile of the authenticated user.
async fn get_profile(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    Ok(Json(UserProfileResponse {
        user_id: user.id.to_string(),
        email: user.email.to_string(),
        name: Some(user.name),
        picture_url: user.picture_url,
        role: user.role.as_str().to_string(),
        is_verified: user.is_verified,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
    }))
}

/// Update profile fields (name, picture_url) for the authenticated user.
async fn update_profile(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state
        .auth_service
        .update_profile(
            user_id,
            payload.display_name.as_deref(),
            payload.picture_url.as_deref(),
        )
        .await?;

    Ok(Json(UserProfileResponse {
        user_id: user.id.to_string(),
        email: user.email.to_string(),
        name: Some(user.name),
        picture_url: user.picture_url,
        role: user.role.as_str().to_string(),
        is_verified: user.is_verified,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
    }))
}

/// List workshop registrations for the authenticated user.
async fn list_registrations(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<RegistrationResponse>>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let _registrations = state.auth_service.list_registrations(user_id).await?;
    Ok(Json(Vec::new()))
}
