use crate::aggregates::level::Level;
use crate::error::DomainError;
use crate::value_objects::ids::LevelId;

/// Repository for persisting and retrieving [`Level`] aggregates.
#[async_trait::async_trait]
pub trait LevelRepository: Send + Sync {
    /// Persist a new difficulty level.
    async fn create(&self, level: &Level) -> Result<(), DomainError>;
    /// Find a level by its unique ID.
    async fn find_by_id(&self, id: LevelId) -> Result<Option<Level>, DomainError>;
    /// Return all levels, ordered by name.
    async fn find_all(&self) -> Result<Vec<Level>, DomainError>;
    /// Persist changes to an existing level.
    async fn update(&self, level: &Level) -> Result<(), DomainError>;
    /// Delete a level by ID.
    async fn delete(&self, id: LevelId) -> Result<(), DomainError>;
}
