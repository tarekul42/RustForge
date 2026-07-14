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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upload_failed_display() {
        let err = ObjectStoreError::UploadFailed("timeout".into());
        assert_eq!(err.to_string(), "Upload failed: timeout");
    }

    #[test]
    fn delete_failed_display() {
        let err = ObjectStoreError::DeleteFailed("not found".into());
        assert_eq!(err.to_string(), "Delete failed: not found");
    }

    #[test]
    fn presign_failed_display() {
        let err = ObjectStoreError::PresignFailed("expired key".into());
        assert_eq!(
            err.to_string(),
            "Presigned URL generation failed: expired key"
        );
    }

    #[test]
    fn s3_error_display() {
        let err = ObjectStoreError::S3Error("access denied".into());
        assert_eq!(err.to_string(), "S3 error: access denied");
    }

    #[test]
    fn debug_format_includes_variant() {
        let err = ObjectStoreError::UploadFailed("err".into());
        let debug = format!("{err:?}");
        assert!(debug.contains("UploadFailed"));
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ObjectStoreError>();
    }
}
