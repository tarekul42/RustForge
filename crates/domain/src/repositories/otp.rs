use crate::error::DomainError;

/// Repository for managing one-time-password (OTP) codes.
#[async_trait::async_trait]
pub trait OtpRepository: Send + Sync {
    /// Store a new OTP code hash for an email address.
    async fn create(
        &self,
        email: &str,
        code_hash: &str,
        expires_at: &chrono::DateTime<chrono::Utc>,
    ) -> Result<(), DomainError>;
    /// Look up the most recent OTP code for an email address.
    async fn find_by_email(
        &self,
        email: &str,
    ) -> Result<Option<(String, i32, chrono::DateTime<chrono::Utc>)>, DomainError>;
    /// Increment the attempt counter for an email's OTP code.
    async fn increment_attempts(&self, email: &str) -> Result<(), DomainError>;
    /// Delete all OTP codes for an email address.
    async fn delete(&self, email: &str) -> Result<(), DomainError>;
    /// Remove all expired OTP codes. Returns the number of rows deleted.
    async fn cleanup_expired(&self) -> Result<u64, DomainError>;
}
