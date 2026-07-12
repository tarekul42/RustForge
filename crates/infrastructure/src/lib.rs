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
