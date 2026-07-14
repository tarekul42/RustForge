/// Interface for file storage (S3/MinIO).
#[async_trait::async_trait]
pub trait ObjectStore: Send + Sync {
    /// Check connectivity to the object store.
    async fn health_check(&self) -> Result<(), ObjectStoreError> {
        Ok(())
    }

    /// Upload a file to the store. Returns the public URL.
    async fn upload(
        &self,
        bucket: &str,
        key: &str,
        body: &[u8],
        content_type: &str,
    ) -> Result<String, ObjectStoreError>;

    /// Delete a file from the store by its key.
    async fn delete(&self, bucket: &str, key: &str) -> Result<(), ObjectStoreError>;

    /// Generate a presigned GET URL valid for the given duration in seconds.
    async fn presigned_get_url(
        &self,
        bucket: &str,
        key: &str,
        expires_in_secs: u64,
    ) -> Result<String, ObjectStoreError>;
}

/// Errors returned by [`ObjectStore`] operations.
#[derive(Debug, thiserror::Error)]
pub enum ObjectStoreError {
    /// An upload operation failed.
    #[error("Upload failed: {0}")]
    UploadFailed(String),
    /// A delete operation failed.
    #[error("Delete failed: {0}")]
    DeleteFailed(String),
    /// Presigned URL generation failed.
    #[error("Presigned URL generation failed: {0}")]
    PresignFailed(String),
    /// Internal error from the S3 SDK.
    #[error("S3 error: {0}")]
    S3Error(String),
}
