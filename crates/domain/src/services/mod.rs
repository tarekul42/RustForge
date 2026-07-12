/// Email sender trait for sending emails.
pub mod email_sender;
/// Object store trait for file storage (S3/MinIO).
pub mod object_store;
/// Payment gateway port trait (SSLCommerz).
pub mod payment_gateway;
pub use payment_gateway::*;
