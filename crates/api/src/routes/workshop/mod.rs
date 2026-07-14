/// Level CRUD sub-routes (mounted at `/api/v1/workshops/levels`).
pub mod levels;

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
use sw_application::services::workshop::{CreateWorkshopInput, UpdateWorkshopInput};
use sw_application::slug::generate_slug;
use sw_domain::value_objects::ids::{CategoryId, LevelId, WorkshopId, WorkshopImageId};

/// Build the workshop router — all paths are relative to `/api/v1/workshops`.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_workshops))
        .route("/", post(create_workshop))
        .route("/{slug}", get(get_workshop_by_slug))
        .route("/{id}", patch(update_workshop))
        .route("/{id}", delete(delete_workshop))
        .route("/{id}/images", post(add_workshop_image))
        .route("/{id}/images/{image_id}", delete(remove_workshop_image))
}

#[derive(Serialize)]
struct WorkshopResponse {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub price_cents: i64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub max_seats: Option<i32>,
    pub current_enrollments: i32,
    pub min_age: Option<i16>,
    pub category_id: String,
    pub level_id: String,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub images: Vec<WorkshopImageResponse>,
}

#[derive(Serialize)]
struct WorkshopImageResponse {
    pub id: String,
    pub url: String,
    pub created_at: String,
}

#[derive(Deserialize)]
struct CreateWorkshopRequest {
    pub title: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub price_cents: i64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub max_seats: Option<i32>,
    pub min_age: Option<i16>,
    pub category_id: String,
    pub level_id: String,
}

#[derive(Deserialize)]
struct UpdateWorkshopRequest {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub price_cents: Option<i64>,
    pub category_id: Option<String>,
    pub level_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub max_seats: Option<i32>,
    pub min_age: Option<i16>,
}

#[derive(Deserialize)]
struct AddImageRequest {
    pub url: String,
    pub s3_key: String,
}

async fn create_workshop(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateWorkshopRequest>,
) -> Result<Json<WorkshopResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    if !user.can_manage_workshops() {
        return Err(ApiError::Unauthorized(
            "Insufficient permissions to create workshops".to_string(),
        ));
    }

    let slug = payload
        .slug
        .unwrap_or_else(|| generate_slug(&payload.title));
    let category_id = CategoryId::parse_str(&payload.category_id)
        .map_err(|_| ApiError::BadRequest("Invalid category_id".to_string()))?;
    let level_id = LevelId::parse_str(&payload.level_id)
        .map_err(|_| ApiError::BadRequest("Invalid level_id".to_string()))?;

    let start_date = payload
        .start_date
        .map(|s| chrono::DateTime::parse_from_rfc3339(&s))
        .transpose()
        .map_err(|e| ApiError::BadRequest(format!("Invalid start_date: {e}")))?
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let end_date = payload
        .end_date
        .map(|s| chrono::DateTime::parse_from_rfc3339(&s))
        .transpose()
        .map_err(|e| ApiError::BadRequest(format!("Invalid end_date: {e}")))?
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let workshop = state
        .workshop_service
        .create(CreateWorkshopInput {
            title: payload.title,
            slug,
            price_cents: payload.price_cents,
            category_id,
            level_id,
            created_by: user_id,
            description: payload.description,
            location: payload.location,
            start_date,
            end_date,
            max_seats: payload.max_seats,
            min_age: payload.min_age,
        })
        .await?;

    let images = state.workshop_service.get_images(workshop.id).await?;
    Ok(Json(to_workshop_response(workshop, images)))
}

async fn list_workshops(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<WorkshopResponse>>, ApiError> {
    let workshops = state.workshop_service.list().await?;
    let mut responses = Vec::with_capacity(workshops.len());
    for w in workshops {
        let images = state.workshop_service.get_images(w.id).await?;
        responses.push(to_workshop_response(w, images));
    }
    Ok(Json(responses))
}

async fn get_workshop_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<WorkshopResponse>, ApiError> {
    let workshop = state.workshop_service.get_by_slug(&slug).await?;
    let images = state.workshop_service.get_images(workshop.id).await?;
    Ok(Json(to_workshop_response(workshop, images)))
}

async fn update_workshop(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<UpdateWorkshopRequest>,
) -> Result<Json<WorkshopResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    if !user.can_manage_workshops() {
        return Err(ApiError::Unauthorized(
            "Insufficient permissions to update workshops".to_string(),
        ));
    }

    let workshop_id = WorkshopId::from_uuid(id);
    let category_id = payload
        .category_id
        .map(|s| CategoryId::parse_str(&s))
        .transpose()
        .map_err(|_| ApiError::BadRequest("Invalid category_id".to_string()))?;
    let level_id = payload
        .level_id
        .map(|s| LevelId::parse_str(&s))
        .transpose()
        .map_err(|_| ApiError::BadRequest("Invalid level_id".to_string()))?;

    let start_date = payload
        .start_date
        .map(|s| chrono::DateTime::parse_from_rfc3339(&s))
        .transpose()
        .map_err(|e| ApiError::BadRequest(format!("Invalid start_date: {e}")))?
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let end_date = payload
        .end_date
        .map(|s| chrono::DateTime::parse_from_rfc3339(&s))
        .transpose()
        .map_err(|e| ApiError::BadRequest(format!("Invalid end_date: {e}")))?
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let workshop = state
        .workshop_service
        .update(UpdateWorkshopInput {
            id: workshop_id,
            title: payload.title,
            slug: payload.slug,
            description: payload.description,
            location: payload.location,
            price_cents: payload.price_cents,
            category_id,
            level_id,
            start_date: start_date.map(Some),
            end_date: end_date.map(Some),
            max_seats: payload.max_seats.map(Some),
            min_age: payload.min_age.map(Some),
        })
        .await?;

    let images = state.workshop_service.get_images(workshop.id).await?;
    Ok(Json(to_workshop_response(workshop, images)))
}

async fn delete_workshop(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    if !user.can_manage_workshops() {
        return Err(ApiError::Unauthorized(
            "Insufficient permissions to delete workshops".to_string(),
        ));
    }

    let workshop_id = WorkshopId::from_uuid(id);
    state.workshop_service.delete(workshop_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}

async fn add_workshop_image(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<AddImageRequest>,
) -> Result<Json<WorkshopImageResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    if !user.can_manage_workshops() {
        return Err(ApiError::Unauthorized(
            "Insufficient permissions to manage workshop images".to_string(),
        ));
    }

    let workshop_id = WorkshopId::from_uuid(id);
    let image = state
        .workshop_service
        .add_image(workshop_id, &payload.url, &payload.s3_key)
        .await?;

    Ok(Json(WorkshopImageResponse {
        id: image.id.to_string(),
        url: image.url,
        created_at: image.created_at.to_rfc3339(),
    }))
}

async fn remove_workshop_image(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path((_workshop_id, image_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    if !user.can_manage_workshops() {
        return Err(ApiError::Unauthorized(
            "Insufficient permissions to manage workshop images".to_string(),
        ));
    }

    state
        .workshop_service
        .remove_image(WorkshopImageId::from_uuid(image_id))
        .await?;
    Ok(Json(serde_json::json!({"success": true})))
}

fn to_workshop_response(
    w: sw_domain::aggregates::workshop::Workshop,
    images: Vec<sw_domain::aggregates::workshop::WorkshopImage>,
) -> WorkshopResponse {
    WorkshopResponse {
        id: w.id.to_string(),
        title: w.title,
        slug: w.slug,
        description: w.description,
        location: w.location,
        price_cents: w.price_cents,
        start_date: w.start_date.map(|d| d.to_rfc3339()),
        end_date: w.end_date.map(|d| d.to_rfc3339()),
        max_seats: w.max_seats,
        current_enrollments: w.current_enrollments,
        min_age: w.min_age,
        category_id: w.category_id.to_string(),
        level_id: w.level_id.to_string(),
        created_by: w.created_by.to_string(),
        created_at: w.created_at.to_rfc3339(),
        updated_at: w.updated_at.to_rfc3339(),
        images: images
            .into_iter()
            .map(|i| WorkshopImageResponse {
                id: i.id.to_string(),
                url: i.url,
                created_at: i.created_at.to_rfc3339(),
            })
            .collect(),
    }
}
