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
use sw_domain::value_objects::ids::{ReviewId, WorkshopId};

/// Build the review router — all paths are relative to `/api/v1/reviews`.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_review))
        .route("/workshop/{workshop_id}", get(list_reviews_for_workshop))
        .route("/{id}", get(get_review))
        .route("/{id}", patch(update_review))
        .route("/{id}/approve", patch(approve_review))
        .route("/{id}/reject", patch(reject_review))
        .route("/{id}", delete(delete_review))
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct CreateReviewRequest {
    workshop_id: String,
    rating: i16,
    title: String,
    content: String,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct UpdateReviewRequest {
    rating: Option<i16>,
    title: Option<String>,
    content: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct ReviewResponse {
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

#[utoipa::path(
    post,
    path = "/api/v1/reviews",
    tag = "reviews",
    request_body = CreateReviewRequest,
    responses(
        (status = 201, description = "Review created", body = ReviewResponse),
        (status = 400, description = "Bad request"),
    ),
)]
pub(crate) async fn create_review(
    State(state): State<Arc<AppState>>,
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

#[utoipa::path(
    get,
    path = "/api/v1/reviews/workshop/{workshop_id}",
    tag = "reviews",
    params(
        ("workshop_id" = String, Path, description = "Workshop ID"),
    ),
    responses(
        (status = 200, description = "List of reviews for workshop", body = Vec<ReviewResponse>),
    ),
)]
pub(crate) async fn list_reviews_for_workshop(
    State(state): State<Arc<AppState>>,
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

#[utoipa::path(
    get,
    path = "/api/v1/reviews/{id}",
    tag = "reviews",
    params(
        ("id" = String, Path, description = "Review ID"),
    ),
    responses(
        (status = 200, description = "Review details", body = ReviewResponse),
        (status = 404, description = "Not found"),
    ),
)]
pub(crate) async fn get_review(
    State(state): State<Arc<AppState>>,
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

#[utoipa::path(
    patch,
    path = "/api/v1/reviews/{id}",
    tag = "reviews",
    request_body = UpdateReviewRequest,
    params(
        ("id" = String, Path, description = "Review ID"),
    ),
    responses(
        (status = 200, description = "Review updated", body = ReviewResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
)]
pub(crate) async fn update_review(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<UpdateReviewRequest>,
) -> Result<Json<ReviewResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;

    let review_id = ReviewId::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid review id".to_string()))?;

    let caller = state.auth_service.get_user(user_id).await?;
    let existing = state
        .review_service
        .find_by_id(review_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if existing.user_id() != caller.id() && !caller.is_admin() {
        return Err(ApiError::Unauthorized(
            "Only the review owner or an admin can update this review".to_string(),
        ));
    }

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

#[utoipa::path(
    patch,
    path = "/api/v1/reviews/{id}/approve",
    tag = "reviews",
    params(
        ("id" = String, Path, description = "Review ID"),
    ),
    responses(
        (status = 200, description = "Review approved", body = ReviewResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
)]
pub(crate) async fn approve_review(
    State(state): State<Arc<AppState>>,
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

#[utoipa::path(
    patch,
    path = "/api/v1/reviews/{id}/reject",
    tag = "reviews",
    params(
        ("id" = String, Path, description = "Review ID"),
    ),
    responses(
        (status = 200, description = "Review rejected", body = ReviewResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
)]
pub(crate) async fn reject_review(
    State(state): State<Arc<AppState>>,
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

#[utoipa::path(
    delete,
    path = "/api/v1/reviews/{id}",
    tag = "reviews",
    params(
        ("id" = String, Path, description = "Review ID"),
    ),
    responses(
        (status = 200, description = "Review deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
)]
pub(crate) async fn delete_review(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let caller = state.auth_service.get_user(user_id).await?;
    let review_id = ReviewId::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid review id".to_string()))?;

    let existing = state
        .review_service
        .find_by_id(review_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if existing.user_id() != caller.id() && !caller.is_admin() {
        return Err(ApiError::Unauthorized(
            "Only the review owner or an admin can delete this review".to_string(),
        ));
    }

    state.review_service.delete(review_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}

fn to_response(review: &sw_domain::aggregates::review::Review) -> ReviewResponse {
    ReviewResponse {
        id: review.id().to_string(),
        user_id: review.user_id().to_string(),
        workshop_id: review.workshop_id().to_string(),
        rating: review.rating(),
        title: review.title().to_string(),
        content: review.content().to_string(),
        status: review.status().as_str().to_string(),
        created_at: review.created_at().to_rfc3339(),
        updated_at: review.updated_at().to_rfc3339(),
    }
}
