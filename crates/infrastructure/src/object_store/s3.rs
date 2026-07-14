use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use std::time::Duration;
use sw_shared::object_store::{ObjectStore, ObjectStoreError};

/// S3-compatible object store implementation using `aws-sdk-s3`.
///
/// Works with both AWS S3 and MinIO (configure via `AWS_ENDPOINT_URL`).
pub struct S3ObjectStore {
    client: aws_sdk_s3::Client,
}

impl S3ObjectStore {
    /// Create a new `S3ObjectStore` from an existing S3 client.
    pub fn new(client: aws_sdk_s3::Client) -> Self {
        Self { client }
    }

    /// Create a new `S3ObjectStore`, loading AWS config from the environment.
    pub async fn from_env() -> Result<Self, ObjectStoreError> {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&config);
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl ObjectStore for S3ObjectStore {
    async fn health_check(&self) -> Result<(), ObjectStoreError> {
        self.client
            .list_buckets()
            .send()
            .await
            .map_err(|e| ObjectStoreError::S3Error(e.to_string()))?;
        Ok(())
    }

    async fn upload(
        &self,
        bucket: &str,
        key: &str,
        body: &[u8],
        content_type: &str,
    ) -> Result<String, ObjectStoreError> {
        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(ByteStream::from(body.to_vec()))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| ObjectStoreError::UploadFailed(e.to_string()))?;

        let url = format!("https://{bucket}.s3.amazonaws.com/{key}");
        Ok(url)
    }

    async fn delete(&self, bucket: &str, key: &str) -> Result<(), ObjectStoreError> {
        self.client
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| ObjectStoreError::DeleteFailed(e.to_string()))?;
        Ok(())
    }

    async fn presigned_get_url(
        &self,
        bucket: &str,
        key: &str,
        expires_in_secs: u64,
    ) -> Result<String, ObjectStoreError> {
        let presign_config = PresigningConfig::builder()
            .expires_in(Duration::from_secs(expires_in_secs))
            .build()
            .map_err(|e| ObjectStoreError::PresignFailed(e.to_string()))?;

        let resp = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .presigned(presign_config)
            .await
            .map_err(|e| ObjectStoreError::PresignFailed(e.to_string()))?;

        Ok(resp.uri().to_string())
    }
}
