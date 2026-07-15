use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::extractors::session;
use crate::state::AppState;
use sw_domain::value_objects::ids::{EnrollmentId, WorkshopId};

/// Build the enrollment router — all paths are relative to `/api/v1/enrollments`.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_enrollment))
        .route("/my", get(my_enrollments))
        .route("/{id}", get(get_enrollment))
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize, ToSchema)]
pub(crate) struct CreateEnrollmentRequest {
    workshop_id: String,
    student_count: Option<i32>,
    cus_name: String,
    cus_email: String,
    cus_phone: String,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct CreateEnrollmentResponse {
    enrollment_id: String,
    payment_id: String,
    gateway_url: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct EnrollmentResponse {
    id: String,
    user_id: String,
    workshop_id: String,
    payment_id: Option<String>,
    student_count: i32,
    status: String,
    created_at: String,
    updated_at: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/api/v1/enrollments",
    tag = "enrollments",
    request_body = CreateEnrollmentRequest,
    responses(
        (status = 201, description = "Enrollment created", body = CreateEnrollmentResponse),
        (status = 400, description = "Bad request"),
    ),
)]
pub(crate) async fn create_enrollment(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateEnrollmentRequest>,
) -> Result<Json<CreateEnrollmentResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;

    let workshop_id = WorkshopId::parse_str(&payload.workshop_id)
        .map_err(|_| ApiError::BadRequest("Invalid workshop_id".to_string()))?;

    let result = state
        .enrollment_service
        .create(
            sw_application::services::enrollment::CreateEnrollmentInput {
                user_id,
                workshop_id,
                student_count: payload.student_count.unwrap_or(1),
                cus_name: payload.cus_name,
                cus_email: payload.cus_email,
                cus_phone: payload.cus_phone,
            },
        )
        .await?;

    Ok(Json(CreateEnrollmentResponse {
        enrollment_id: result.enrollment.id().to_string(),
        payment_id: result.payment.id().to_string(),
        gateway_url: result.gateway_url,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/enrollments/my",
    tag = "enrollments",
    responses(
        (status = 200, description = "List of user enrollments", body = Vec<EnrollmentResponse>),
    ),
)]
pub(crate) async fn my_enrollments(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<EnrollmentResponse>>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;

    let enrollments = state.enrollment_service.list_by_user(user_id).await?;

    Ok(Json(
        enrollments
            .into_iter()
            .map(|e| EnrollmentResponse {
                id: e.id().to_string(),
                user_id: e.user_id().to_string(),
                workshop_id: e.workshop_id().to_string(),
                payment_id: e.payment_id().map(|p| p.to_string()),
                student_count: e.student_count(),
                status: e.status().as_str().to_string(),
                created_at: e.created_at().to_rfc3339(),
                updated_at: e.updated_at().to_rfc3339(),
            })
            .collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/enrollments/{id}",
    tag = "enrollments",
    params(
        ("id" = String, Path, description = "Enrollment ID"),
    ),
    responses(
        (status = 200, description = "Enrollment details", body = EnrollmentResponse),
        (status = 404, description = "Not found"),
    ),
)]
pub(crate) async fn get_enrollment(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<EnrollmentResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;

    let enrollment_id = EnrollmentId::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid enrollment id".to_string()))?;

    let enrollment = state
        .enrollment_service
        .find_by_id(enrollment_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if enrollment.user_id() != user_id {
        return Err(ApiError::NotFound);
    }

    Ok(Json(EnrollmentResponse {
        id: enrollment.id().to_string(),
        user_id: enrollment.user_id().to_string(),
        workshop_id: enrollment.workshop_id().to_string(),
        payment_id: enrollment.payment_id().map(|p| p.to_string()),
        student_count: enrollment.student_count(),
        status: enrollment.status().as_str().to_string(),
        created_at: enrollment.created_at().to_rfc3339(),
        updated_at: enrollment.updated_at().to_rfc3339(),
    }))
}
