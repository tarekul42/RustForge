#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::doc_markdown,
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::needless_raw_string_hashes,
    clippy::redundant_closure_for_method_calls,
    clippy::return_self_not_must_use,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unnecessary_wraps,
    clippy::wildcard_imports
)]
#![doc = "Infrastructure crate: adapters for all external I/O."]

/// Email sender implementation (lettre + tera).
pub mod email;
/// Background job queue (Postgres-based, with exponential backoff).
pub mod jobs;
/// S3/MinIO object store implementation.
pub mod object_store;
/// Payment gateway adapters (SSLCommerz).
pub mod payment;
/// PDF invoice generation (printpdf).
pub mod pdf;
/// PostgreSQL connection pool and repository implementations.
pub mod postgres;
