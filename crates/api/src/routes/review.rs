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
use sw_domain::value_objects::ids::{ReviewId, WorkshopId};

/// Build the review router — all paths are relative to `/api/v1/reviews`.
pub fn router() -> Router {
    Router::new()
        .route("/", post(create_review))
        .route("/workshop/:workshop_id", get(list_reviews_for_workshop))
        .route("/:id", get(get_review))
        .route("/:id", patch(update_review))
        .route("/:id/approve", patch(approve_review))
        .route("/:id/reject", patch(reject_review))
        .route("/:id", delete(delete_review))
}

#[derive(Deserialize)]
struct CreateReviewRequest {
    workshop_id: String,
    rating: i16,
    title: String,
    content: String,
}

#[derive(Deserialize)]
struct UpdateReviewRequest {
    rating: Option<i16>,
    title: Option<String>,
    content: Option<String>,
}

#[derive(Serialize)]
struct ReviewResponse {
    id: String,
    user_id: String,
    workshop_id: String,
    rating: i16,
    title: String,
    content: String,
    status: String,
    created_at: String,
    updated_at: String,
}

async fn create_review(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateReviewRequest>,
) -> Result<Json<ReviewResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;

    let workshop_id = WorkshopId::parse_str(&payload.workshop_id)
        .map_err(|_| ApiError::BadRequest("Invalid workshop_id".to_string()))?;

    let review = state
        .review_service
        .create(sw_application::services::review::CreateReviewInput {
            user_id,
            workshop_id,
            rating: payload.rating,
            title: payload.title,
            content: payload.content,
        })
        .await?;

    Ok(Json(to_response(&review)))
}

async fn list_reviews_for_workshop(
    Extension(state): Extension<Arc<AppState>>,
    Path(workshop_id): Path<String>,
) -> Result<Json<Vec<ReviewResponse>>, ApiError> {
    let workshop_id = WorkshopId::parse_str(&workshop_id)
        .map_err(|_| ApiError::BadRequest("Invalid workshop_id".to_string()))?;

    // Public endpoint — only return approved reviews.
    let reviews = state
        .review_service
        .find_by_workshop(workshop_id, true)
        .await?;

    Ok(Json(reviews.iter().map(to_response).collect()))
}

async fn get_review(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ReviewResponse>, ApiError> {
    let review_id = ReviewId::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid review id".to_string()))?;

    let review = state
        .review_service
        .find_by_id(review_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(to_response(&review)))
}

async fn update_review(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<UpdateReviewRequest>,
) -> Result<Json<ReviewResponse>, ApiError> {
    let (_session_id, _user_id) = session::resolve_session(&headers, &state).await?;

    let review_id = ReviewId::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid review id".to_string()))?;

    let review = state
        .review_service
        .update(sw_application::services::review::UpdateReviewInput {
            id: review_id,
            rating: payload.rating,
            title: payload.title,
            content: payload.content,
        })
        .await?;

    Ok(Json(to_response(&review)))
}

async fn approve_review(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ReviewResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let caller = state.auth_service.get_user(user_id).await?;
    if !caller.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    let review_id = ReviewId::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid review id".to_string()))?;

    let review = state.review_service.approve(review_id).await?;
    Ok(Json(to_response(&review)))
}

async fn reject_review(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ReviewResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let caller = state.auth_service.get_user(user_id).await?;
    if !caller.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    let review_id = ReviewId::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid review id".to_string()))?;

    let review = state.review_service.reject(review_id).await?;
    Ok(Json(to_response(&review)))
}

async fn delete_review(
    Extension(state): Extension<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let caller = state.auth_service.get_user(user_id).await?;
    if !caller.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    let review_id = ReviewId::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid review id".to_string()))?;

    state.review_service.delete(review_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}

fn to_response(review: &sw_domain::aggregates::review::Review) -> ReviewResponse {
    ReviewResponse {
        id: review.id.to_string(),
        user_id: review.user_id.to_string(),
        workshop_id: review.workshop_id.to_string(),
        rating: review.rating,
        title: review.title.clone(),
        content: review.content.clone(),
        status: review.status.as_str().to_string(),
        created_at: review.created_at.to_rfc3339(),
        updated_at: review.updated_at.to_rfc3339(),
    }
}
