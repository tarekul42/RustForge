use crate::aggregates::workshop::Workshop;
use crate::error::DomainError;
use crate::value_objects::ids::WorkshopId;

/// Repository for persisting and retrieving [`Workshop`] aggregates.
#[async_trait::async_trait]
pub trait WorkshopRepository: Send + Sync {
    /// Persist a new workshop.
    async fn create(&self, workshop: &Workshop) -> Result<(), DomainError>;
    /// Find a workshop by its unique ID.
    async fn find_by_id(&self, id: WorkshopId) -> Result<Option<Workshop>, DomainError>;
    /// Find a workshop by its URL slug.
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Workshop>, DomainError>;
    /// Persist changes to an existing workshop.
    async fn update(&self, workshop: &Workshop) -> Result<(), DomainError>;
    /// Delete a workshop by ID.
    async fn delete(&self, id: WorkshopId) -> Result<(), DomainError>;
}
