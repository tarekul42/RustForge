#![deny(missing_docs)]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::doc_markdown,
    clippy::filter_map_next,
    clippy::ignored_unit_patterns,
    clippy::items_after_statements,
    clippy::map_unwrap_or,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::redundant_closure_for_method_calls,
    clippy::return_self_not_must_use,
    clippy::similar_names,
    clippy::struct_field_names,
    clippy::uninlined_format_args
)]
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
