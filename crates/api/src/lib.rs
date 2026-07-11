#![deny(missing_docs)]
#![doc = "API crate: HTTP server using Axum."]

/// Axum application router and middleware setup.
pub mod app;
/// API error types with IntoResponse implementation.
pub mod error;
/// Request extractors (session, auth_user, etc.).
pub mod extractors;
/// Request-processing middleware (request_id, etc.).
pub mod middleware;
/// Route handlers organized by feature.
pub mod routes;
/// Shared application state accessible from handlers.
pub mod state;
/// File upload validation and S3 helper utilities.
pub mod upload;
