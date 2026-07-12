use crate::aggregates::workshop::{Workshop, WorkshopImage};
use crate::error::DomainError;
use crate::value_objects::ids::{WorkshopId, WorkshopImageId};

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
    /// Return all images associated with a workshop, ordered by creation time.
    async fn get_images(&self, workshop_id: WorkshopId) -> Result<Vec<WorkshopImage>, DomainError>;
    /// Attach an image record to a workshop.
    async fn add_image(
        &self,
        workshop_id: WorkshopId,
        url: &str,
        s3_key: &str,
    ) -> Result<WorkshopImage, DomainError>;
    /// Remove an image record from a workshop.
    async fn remove_image(&self, image_id: WorkshopImageId) -> Result<(), DomainError>;
    /// Return all workshops, ordered by creation date descending.
    async fn find_all(&self) -> Result<Vec<Workshop>, DomainError>;
}
