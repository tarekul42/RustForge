use axum::{
    extract::{Extension, Path},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::ApiError;
use crate::extractors::session;
use crate::state::AppState;
use sw_application::slug::generate_slug;
use sw_domain::value_objects::ids::CategoryId;

/// Build the category router — all paths are relative to `/api/v1/categories`.
pub fn router() -> Router {
    Router::new()
        .route("/", get(list_categories))
        .route("/", post(create_category))
        .route("/:slug", get(get_category_by_slug))
        .route("/:id", patch(update_category))
        .route("/:id", delete(delete_category))
}

#[derive(Serialize)]
struct CategoryResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<sw_domain::aggregates::category::Category> for CategoryResponse {
    fn from(c: sw_domain::aggregates::category::Category) -> Self {
        Self {
            id: c.id.to_string(),
            name: c.name,
            slug: c.slug,
            description: c.description,
            thumbnail_url: c.thumbnail_url,
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CreateCategoryRequest {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
}

#[derive(Deserialize)]
struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
}

async fn create_category(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    if !user.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    let slug = payload.slug.unwrap_or_else(|| generate_slug(&payload.name));

    let category = state.category_service.create(payload.name, slug).await?;
    Ok(Json(CategoryResponse::from(category)))
}

async fn list_categories(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Vec<CategoryResponse>>, ApiError> {
    let categories = state.category_service.list().await?;
    Ok(Json(
        categories.into_iter().map(CategoryResponse::from).collect(),
    ))
}

async fn get_category_by_slug(
    Extension(state): Extension<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let category = state.category_service.get_by_slug(&slug).await?;
    Ok(Json(CategoryResponse::from(category)))
}

async fn update_category(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    if !user.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    let category_id = CategoryId::from_uuid(id);
    let category = state
        .category_service
        .update(
            category_id,
            payload.name,
            payload.description,
            payload.thumbnail_url,
        )
        .await?;
    Ok(Json(CategoryResponse::from(category)))
}

async fn delete_category(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    if !user.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    let category_id = CategoryId::from_uuid(id);
    state.category_service.delete(category_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}
