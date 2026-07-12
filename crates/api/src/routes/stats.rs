use axum::{extract::Extension, routing::get, Json, Router};
use serde::Serialize;
use std::sync::Arc;

use crate::state::AppState;

/// Build the stats router — all paths are relative to `/api/v1/stats`.
pub fn router() -> Router {
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
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<PlatformStatsResponse>, crate::error::ApiError> {
    let stats = state.stats_service.platform_stats().await?;
    Ok(Json(PlatformStatsResponse {
        total_users: stats.total_users,
        total_workshops: stats.total_workshops,
        total_enrollments: stats.total_enrollments,
        total_reviews: stats.total_reviews,
    }))
}

async fn workshop_ratings(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<Vec<WorkshopRatingResponse>>, crate::error::ApiError> {
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
