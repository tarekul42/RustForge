use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use std::sync::Arc;

use crate::error::ApiError;
use crate::extractors::session;
use crate::state::AppState;

/// Build the stats router — all paths are relative to `/api/v1/stats`.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(platform_stats))
        .route("/ratings", get(workshop_ratings))
}

#[derive(Serialize)]
struct PlatformStatsResponse {
    total_users: i64,
    total_workshops: i64,
    total_enrollments: i64,
    total_reviews: i64,
}

#[derive(Serialize)]
struct WorkshopRatingResponse {
    workshop_id: String,
    average_rating: f64,
    review_count: i64,
}

async fn platform_stats(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<PlatformStatsResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let caller = state.auth_service.get_user(user_id).await?;
    if !caller.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }
    let stats = state.stats_service.platform_stats().await?;
    Ok(Json(PlatformStatsResponse {
        total_users: stats.total_users,
        total_workshops: stats.total_workshops,
        total_enrollments: stats.total_enrollments,
        total_reviews: stats.total_reviews,
    }))
}

async fn workshop_ratings(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<WorkshopRatingResponse>>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let caller = state.auth_service.get_user(user_id).await?;
    if !caller.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }
    let ratings = state.stats_service.workshop_ratings().await?;
    Ok(Json(
        ratings
            .into_iter()
            .map(|r| WorkshopRatingResponse {
                workshop_id: r.workshop_id.to_string(),
                average_rating: r.average_rating,
                review_count: r.review_count,
            })
            .collect(),
    ))
}
