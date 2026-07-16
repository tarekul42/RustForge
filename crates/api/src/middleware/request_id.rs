use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use std::cell::Cell;
use tracing::Instrument;
use tracing::info_span;
use uuid::Uuid;

thread_local! {
    /// Request ID for the current request, accessible from error handlers.
    static CURRENT_REQUEST_ID: Cell<Option<String>> = const { Cell::new(None) };
}

/// Set the current request ID in the thread-local for error responses.
pub fn set_current_request_id(id: String) {
    CURRENT_REQUEST_ID.set(Some(id));
}

/// Get the current request ID from the thread-local.
#[must_use]
pub fn get_current_request_id() -> Option<String> {
    CURRENT_REQUEST_ID.take()
}

/// Middleware that injects an `X-Request-Id` header into every response.
///
/// Generates a UUID v7 request ID, creates a tracing span scoped to the
/// request with the `request_id` field populated, and attaches the ID
/// to the response as the `X-Request-Id` header.
pub async fn set_request_id(request: Request, next: Next) -> Response {
    let request_id = Uuid::now_v7().to_string();

    set_current_request_id(request_id.clone());

    let span = info_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        path = %request.uri().path(),
    );

    let mut response = next.run(request).instrument(span).await;

    response.headers_mut().insert(
        "X-Request-Id",
        HeaderValue::from_str(&request_id).unwrap_or_else(|_| HeaderValue::from_static("unknown")),
    );

    response
}
