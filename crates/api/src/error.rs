use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use sw_application::error::ApplicationError;

use crate::middleware::request_id::get_current_request_id;

/// Top-level API error type that implements `IntoResponse`.
///
/// Every handler should return `Result<impl IntoResponse, ApiError>`.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Unexpected internal error.
    #[error("Internal server error")]
    Internal,

    /// Invalid request (validation, malformed body, etc.).
    #[error("{0}")]
    BadRequest(String),

    /// Resource not found.
    #[error("Not found")]
    NotFound,

    /// Conflict (duplicate, foreign-key violation, etc.).
    #[error("{0}")]
    Conflict(String),

    /// Authorization failure.
    #[error("{0}")]
    Unauthorized(String),

    /// Resource unavailable (workshop full, insufficient seats, etc.).
    #[error("{0}")]
    Unavailable(String),

    /// Rate limit exceeded.
    #[error("Too many requests")]
    RateLimitExceeded,

    /// Feature not yet implemented.
    #[error("Not implemented")]
    NotImplemented,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            ApiError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL",
                "Internal server error",
            ),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.as_str()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", "Resource not found"),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.as_str()),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.as_str()),
            ApiError::Unavailable(msg) => {
                (StatusCode::SERVICE_UNAVAILABLE, "UNAVAILABLE", msg.as_str())
            }
            ApiError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
                "Too many requests",
            ),
            ApiError::NotImplemented => (
                StatusCode::NOT_IMPLEMENTED,
                "NOT_IMPLEMENTED",
                "Not implemented",
            ),
        };

        let request_id = get_current_request_id();

        let body = json!({
            "success": false,
            "error": {
                "code": code,
                "message": message,
                "details": null
            },
            "requestId": request_id
        });

        (status, Json(body)).into_response()
    }
}

impl From<(axum::http::StatusCode, &str)> for ApiError {
    fn from((_status, msg): (axum::http::StatusCode, &str)) -> Self {
        ApiError::Unauthorized(msg.to_string())
    }
}

impl From<ApplicationError> for ApiError {
    fn from(err: ApplicationError) -> Self {
        match err {
            ApplicationError::NotFound(_s) => ApiError::NotFound,
            ApplicationError::Validation(s) => ApiError::BadRequest(s),
            ApplicationError::Conflict(s) => ApiError::Conflict(s),
            ApplicationError::Unauthorized(s) => ApiError::Unauthorized(s),
            ApplicationError::Unavailable(s) => ApiError::Unavailable(s),
            ApplicationError::RateLimitExceeded => ApiError::RateLimitExceeded,
            ApplicationError::Internal(s) => {
                tracing::error!(error = %s, "Internal application error");
                ApiError::Internal
            }
        }
    }
}
