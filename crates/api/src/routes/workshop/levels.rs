use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, patch, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::ApiError;
use crate::extractors::session;
use crate::state::AppState;
use sw_domain::value_objects::ids::LevelId;

/// Build the levels sub-router — mounted at `/api/v1/workshops/levels`.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_levels))
        .route("/", post(create_level))
        .route("/{id}", get(get_level))
        .route("/{id}", patch(rename_level))
        .route("/{id}", delete(delete_level))
}

#[derive(Serialize)]
struct LevelResponse {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<sw_domain::aggregates::level::Level> for LevelResponse {
    fn from(l: sw_domain::aggregates::level::Level) -> Self {
        Self {
            id: l.id().to_string(),
            name: l.name().to_string(),
            created_at: l.created_at().to_rfc3339(),
            updated_at: l.updated_at().to_rfc3339(),
        }
    }
}

#[derive(Deserialize)]
struct CreateLevelRequest {
    pub name: String,
}

#[derive(Deserialize)]
struct RenameLevelRequest {
    pub name: String,
}

async fn create_level(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateLevelRequest>,
) -> Result<Json<LevelResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    if !user.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }
    let level = state.level_service.create(payload.name).await?;
    Ok(Json(LevelResponse::from(level)))
}

async fn list_levels(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<LevelResponse>>, ApiError> {
    let levels = state.level_service.list().await?;
    Ok(Json(levels.into_iter().map(LevelResponse::from).collect()))
}

async fn get_level(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<LevelResponse>, ApiError> {
    let level = state
        .level_service
        .get_by_id(LevelId::from_uuid(id))
        .await?;
    Ok(Json(LevelResponse::from(level)))
}

async fn rename_level(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<RenameLevelRequest>,
) -> Result<Json<LevelResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    if !user.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }
    let level = state
        .level_service
        .rename(LevelId::from_uuid(id), payload.name)
        .await?;
    Ok(Json(LevelResponse::from(level)))
}

async fn delete_level(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    if !user.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }
    state.level_service.delete(LevelId::from_uuid(id)).await?;
    Ok(Json(serde_json::json!({"success": true})))
}
