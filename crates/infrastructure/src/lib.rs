#![doc = "Infrastructure crate: adapters for all external I/O."]

/// S3/MinIO object store implementation.
pub mod object_store;
/// PostgreSQL connection pool and repository implementations.
pub mod postgres;
