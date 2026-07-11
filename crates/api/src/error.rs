use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

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
            ApiError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.as_str()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", "Resource not found"),
            ApiError::NotImplemented => (
                StatusCode::NOT_IMPLEMENTED,
                "NOT_IMPLEMENTED",
                "Not implemented",
            ),
        };

        let body = json!({
            "success": false,
            "error": {
                "code": code,
                "message": message,
                "details": null
            },
            "requestId": null
        });

        (status, Json(body)).into_response()
    }
}
