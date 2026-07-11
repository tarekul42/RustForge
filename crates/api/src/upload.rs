use axum::extract::multipart::Field;
use sw_shared::object_store::{ObjectStore, ObjectStoreError};
use uuid::Uuid;

/// Validated image upload result.
pub struct UploadedImage {
    /// The S3 key used to store the file.
    pub s3_key: String,
    /// The public URL of the uploaded file.
    pub url: String,
}

/// Validated content types for workshop images.
const ALLOWED_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp"];
/// Maximum file size in bytes (5 MB).
const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;

/// Validate a single image file from a multipart field.
///
/// Checks file type (magic bytes via `infer`) and size.
pub fn validate_image(data: &[u8], content_type: Option<&str>) -> Result<(), String> {
    if data.is_empty() {
        return Err("Empty file".to_string());
    }

    if data.len() > MAX_FILE_SIZE {
        return Err(format!(
            "File too large: {} bytes (max {MAX_FILE_SIZE})",
            data.len()
        ));
    }

    let kind = infer::get(data).ok_or_else(|| "Unable to detect file type".to_string())?;

    let is_allowed = matches!(kind.mime_type(), "image/jpeg" | "image/png" | "image/webp");

    if !is_allowed {
        return Err(format!(
            "Invalid file type '{}'. Allowed: jpeg, png, webp",
            kind.mime_type()
        ));
    }

    if let Some(ct) = content_type {
        if !ALLOWED_MIME_TYPES.contains(&ct) {
            return Err(format!(
                "Invalid Content-Type '{ct}'. Allowed: jpeg, png, webp"
            ));
        }
    }

    Ok(())
}

/// Generate a unique S3 key for an uploaded image.
pub fn generate_s3_key(content_type: &str) -> String {
    let ext = match content_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        _ => "bin",
    };
    let id = Uuid::now_v7();
    format!("workshops/images/{id}.{ext}")
}

/// Upload a validated image to the object store and return the S3 key + public URL.
pub async fn upload_image(
    store: &dyn ObjectStore,
    bucket: &str,
    data: &[u8],
    content_type: &str,
) -> Result<UploadedImage, ObjectStoreError> {
    let s3_key = generate_s3_key(content_type);
    let url = store.upload(bucket, &s3_key, data, content_type).await?;
    Ok(UploadedImage { s3_key, url })
}

/// Extract a text field value from a multipart field.
pub async fn field_text(field: Field<'_>) -> Result<String, String> {
    let name = field.name().unwrap_or("?").to_string();
    let data = field
        .text()
        .await
        .map_err(|e| format!("Failed to read field '{name}': {e}"))?;
    Ok(data)
}

/// Extract a file (bytes + content type) from a multipart field.
pub async fn field_bytes(field: Field<'_>) -> Result<(Vec<u8>, String), String> {
    let name = field.name().unwrap_or("?").to_string();
    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();
    let data = field
        .bytes()
        .await
        .map_err(|e| format!("Failed to read file field '{name}': {e}"))?
        .to_vec();
    Ok((data, content_type))
}
