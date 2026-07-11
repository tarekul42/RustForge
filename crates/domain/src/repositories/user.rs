use crate::aggregates::user::User;
use crate::error::DomainError;
use crate::value_objects::ids::UserId;

/// Repository for persisting and retrieving [`User`] aggregates.
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    /// Persist a new user.
    async fn create(&self, user: &User) -> Result<(), DomainError>;
    /// Find a user by their unique ID.
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError>;
    /// Find a user by their email address (case-insensitive).
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    /// Persist changes to an existing user.
    async fn update(&self, user: &User) -> Result<(), DomainError>;
    /// Delete a user by ID.
    async fn delete(&self, id: UserId) -> Result<(), DomainError>;
}
