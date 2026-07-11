use crate::error::DomainError;
use crate::value_objects::ids::{SessionId, UserId};

/// Repository for managing user sessions.
#[async_trait::async_trait]
pub trait SessionRepository: Send + Sync {
    /// Create a new session for a user.
    async fn create(
        &self,
        session_id: SessionId,
        user_id: UserId,
        token_hash: &str,
        expires_at: &chrono::DateTime<chrono::Utc>,
    ) -> Result<(), DomainError>;
    /// Look up a session by its token hash.
    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<(SessionId, UserId, chrono::DateTime<chrono::Utc>)>, DomainError>;
    /// Delete a single session.
    async fn delete(&self, session_id: SessionId) -> Result<(), DomainError>;
    /// Delete all sessions for a given user.
    async fn delete_all_for_user(&self, user_id: UserId) -> Result<(), DomainError>;
    /// Remove all expired sessions. Returns the number of rows deleted.
    async fn cleanup_expired(&self) -> Result<u64, DomainError>;
}
