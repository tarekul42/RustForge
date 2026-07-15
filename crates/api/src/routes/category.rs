use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, patch, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::extractors::session;
use crate::state::AppState;
use sw_application::slug::generate_slug;
use sw_domain::value_objects::ids::CategoryId;

/// Build the category router — all paths are relative to `/api/v1/categories`.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_categories))
        .route("/", post(create_category))
        .route("/{slug}", get(get_category_by_slug))
        .route("/{id}", patch(update_category))
        .route("/{id}", delete(delete_category))
}

#[derive(Serialize, ToSchema)]
pub(crate) struct CategoryResponse {
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
            id: c.id().to_string(),
            name: c.name().to_string(),
            slug: c.slug().to_string(),
            description: c.description().map(|s| s.to_string()),
            thumbnail_url: c.thumbnail_url().map(|s| s.to_string()),
            created_at: c.created_at().to_rfc3339(),
            updated_at: c.updated_at().to_rfc3339(),
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct CreateCategoryRequest {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/categories",
    tag = "categories",
    request_body = CreateCategoryRequest,
    responses(
        (status = 200, description = "Category created", body = CategoryResponse),
    ),
)]
pub(crate) async fn create_category(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    if !user.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    let slug = payload.slug.unwrap_or_else(|| generate_slug(&payload.name));

    let category = state
        .category_service
        .create(
            payload.name,
            slug,
            payload.description,
            payload.thumbnail_url,
        )
        .await?;
    Ok(Json(CategoryResponse::from(category)))
}

#[utoipa::path(
    get,
    path = "/api/v1/categories",
    tag = "categories",
    responses(
        (status = 200, description = "List of categories", body = Vec<CategoryResponse>),
    ),
)]
pub(crate) async fn list_categories(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CategoryResponse>>, ApiError> {
    let categories = state.category_service.list().await?;
    Ok(Json(
        categories.into_iter().map(CategoryResponse::from).collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/categories/{slug}",
    tag = "categories",
    params(
        ("slug" = String, Path, description = "Category slug"),
    ),
    responses(
        (status = 200, description = "Category found", body = CategoryResponse),
    ),
)]
pub(crate) async fn get_category_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let category = state.category_service.get_by_slug(&slug).await?;
    Ok(Json(CategoryResponse::from(category)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/categories/{id}",
    tag = "categories",
    params(
        ("id" = Uuid, Path, description = "Category ID"),
    ),
    request_body = UpdateCategoryRequest,
    responses(
        (status = 200, description = "Category updated", body = CategoryResponse),
    ),
)]
pub(crate) async fn update_category(
    State(state): State<Arc<AppState>>,
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

#[utoipa::path(
    delete,
    path = "/api/v1/categories/{id}",
    tag = "categories",
    params(
        ("id" = Uuid, Path, description = "Category ID"),
    ),
    responses(
        (status = 200, description = "Category deleted", body = serde_json::Value),
    ),
)]
pub(crate) async fn delete_category(
    State(state): State<Arc<AppState>>,
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
