/// Interface for file storage (S3/MinIO).
///
/// This trait is defined in the domain layer so that application services
/// can depend on it without coupling to any specific infrastructure provider.
/// The concrete implementation lives in the `infrastructure` crate.
pub use sw_shared::object_store::{ObjectStore, ObjectStoreError};
