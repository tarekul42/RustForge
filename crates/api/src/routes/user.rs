use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, patch},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::extractors::session;
use crate::state::AppState;

/// Build the user router — all paths are relative to `/api/v1/users`.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/me", get(get_profile))
        .route("/me", patch(update_profile))
        .route("/me/registrations", get(list_registrations))
        .route("/", get(list_users))
        .route("/{id}", get(get_user))
        .route("/{id}", patch(admin_update_user))
        .route("/{id}", delete(admin_delete_user))
}

/// Response body for `GET /users/me` and `GET /users/:id`.
#[derive(Serialize, ToSchema)]
#[schema(example = json!({"user_id": "123e4567-e89b-12d3-a456-426614174000", "email": "user@example.com", "name": "Alice", "picture_url": null, "role": "student", "is_verified": true, "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-06-01T00:00:00Z"}))]
pub(crate) struct UserProfileResponse {
    /// The user's unique ID.
    #[schema(value_type = String)]
    pub user_id: String,
    /// The user's email address.
    pub email: String,
    /// The user's display name.
    pub name: Option<String>,
    /// URL to the user's profile picture.
    pub picture_url: Option<String>,
    /// The user's role (e.g. "student", "admin").
    #[schema(example = "student")]
    pub role: String,
    /// Whether the email has been verified.
    pub is_verified: bool,
    /// ISO-8601 timestamp of account creation.
    pub created_at: String,
    /// ISO-8601 timestamp of the last update.
    pub updated_at: String,
}

/// Request body for `PATCH /users/me`.
#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"display_name": "Alice", "picture_url": "https://example.com/avatar.png"}))]
pub(crate) struct UpdateProfileRequest {
    /// New display name.
    pub display_name: Option<String>,
    /// New profile picture URL.
    pub picture_url: Option<String>,
}

/// Response body for `GET /users/me/registrations`.
#[derive(Serialize, ToSchema)]
#[schema(example = json!({"id": "123e4567-e89b-12d3-a456-426614174000", "workshop_id": "223e4567-e89b-12d3-a456-426614174001", "status": "confirmed", "registered_at": "2026-07-01T10:00:00Z"}))]
pub(crate) struct RegistrationResponse {
    /// Registration ID.
    #[schema(value_type = String)]
    pub id: String,
    /// Workshop ID the user registered for.
    #[schema(value_type = String)]
    pub workshop_id: String,
    /// Registration status.
    pub status: String,
    /// ISO-8601 timestamp of registration.
    pub registered_at: String,
}

// Admin request/response types

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"name": "Alice", "role": "admin", "status": "active", "phone": "+1234567890", "age": 30, "address": "123 Main St", "expertise": "Rust", "bio": "Software engineer"}))]
pub(crate) struct AdminUpdateUserRequest {
    name: Option<String>,
    role: Option<String>,
    status: Option<String>,
    phone: Option<String>,
    age: Option<i16>,
    address: Option<String>,
    expertise: Option<String>,
    bio: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    tag = "users",
    responses(
        (status = 200, description = "User profile retrieved", body = UserProfileResponse),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub(crate) async fn get_profile(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    Ok(Json(UserProfileResponse {
        user_id: user.id().to_string(),
        email: user.email().to_string(),
        name: Some(user.name().to_string()),
        picture_url: user.picture_url().map(|s| s.to_string()),
        role: user.role().as_str().to_string(),
        is_verified: user.is_verified(),
        created_at: user.created_at().to_rfc3339(),
        updated_at: user.updated_at().to_rfc3339(),
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/me",
    tag = "users",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = UserProfileResponse),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub(crate) async fn update_profile(
    State(state): State<Arc<AppState>>,
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
        user_id: user.id().to_string(),
        email: user.email().to_string(),
        name: Some(user.name().to_string()),
        picture_url: user.picture_url().map(|s| s.to_string()),
        role: user.role().as_str().to_string(),
        is_verified: user.is_verified(),
        created_at: user.created_at().to_rfc3339(),
        updated_at: user.updated_at().to_rfc3339(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/me/registrations",
    tag = "users",
    responses(
        (status = 200, description = "List of user registrations", body = Vec<RegistrationResponse>),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub(crate) async fn list_registrations(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<RegistrationResponse>>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;

    let enrollments = state.enrollment_service.list_by_user(user_id).await?;

    Ok(Json(
        enrollments
            .into_iter()
            .map(|e| RegistrationResponse {
                id: e.id().to_string(),
                workshop_id: e.workshop_id().to_string(),
                status: e.status().as_str().to_string(),
                registered_at: e.created_at().to_rfc3339(),
            })
            .collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "users",
    responses(
        (status = 200, description = "List of all users (admin)", body = Vec<UserProfileResponse>),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub(crate) async fn list_users(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<UserProfileResponse>>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let caller = state.auth_service.get_user(user_id).await?;
    if !caller.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }
    let users = state.user_admin_service.list().await?;
    let responses = users
        .into_iter()
        .map(|u| UserProfileResponse {
            user_id: u.id().to_string(),
            email: u.email().to_string(),
            name: Some(u.name().to_string()),
            picture_url: u.picture_url().map(|s| s.to_string()),
            role: u.role().as_str().to_string(),
            is_verified: u.is_verified(),
            created_at: u.created_at().to_rfc3339(),
            updated_at: u.updated_at().to_rfc3339(),
        })
        .collect();
    Ok(Json(responses))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(
        ("id" = String, Path, description = "User UUID"),
    ),
    responses(
        (status = 200, description = "User profile retrieved (admin)", body = UserProfileResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
    ),
)]
pub(crate) async fn get_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let caller = state.auth_service.get_user(user_id).await?;
    if !caller.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    let target_id = sw_domain::value_objects::ids::UserId::from_uuid(id);
    let target = state.auth_service.get_user(target_id).await?;
    Ok(Json(UserProfileResponse {
        user_id: target.id().to_string(),
        email: target.email().to_string(),
        name: Some(target.name().to_string()),
        picture_url: target.picture_url().map(|s| s.to_string()),
        role: target.role().as_str().to_string(),
        is_verified: target.is_verified(),
        created_at: target.created_at().to_rfc3339(),
        updated_at: target.updated_at().to_rfc3339(),
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/{id}",
    tag = "users",
    request_body = AdminUpdateUserRequest,
    params(
        ("id" = String, Path, description = "User UUID"),
    ),
    responses(
        (status = 200, description = "User updated (admin)", body = UserProfileResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
    ),
)]
pub(crate) async fn admin_update_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<AdminUpdateUserRequest>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let caller = state.auth_service.get_user(user_id).await?;
    if !caller.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }
    let target_id = sw_domain::value_objects::ids::UserId::from_uuid(id);
    let user = state
        .user_admin_service
        .update(sw_application::services::user::UpdateUserInput {
            user_id: target_id,
            name: payload.name,
            role: payload.role,
            status: payload.status,
            phone: payload.phone,
            age: payload.age,
            address: payload.address,
            expertise: payload.expertise,
            bio: payload.bio,
        })
        .await?;
    Ok(Json(UserProfileResponse {
        user_id: user.id().to_string(),
        email: user.email().to_string(),
        name: Some(user.name().to_string()),
        picture_url: user.picture_url().map(|s| s.to_string()),
        role: user.role().as_str().to_string(),
        is_verified: user.is_verified(),
        created_at: user.created_at().to_rfc3339(),
        updated_at: user.updated_at().to_rfc3339(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(
        ("id" = String, Path, description = "User UUID"),
    ),
    responses(
        (status = 200, description = "User deleted (admin)"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
    ),
)]
pub(crate) async fn admin_delete_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let caller = state.auth_service.get_user(user_id).await?;
    if !caller.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }
    let target_id = sw_domain::value_objects::ids::UserId::from_uuid(id);
    state.user_admin_service.delete(target_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}
