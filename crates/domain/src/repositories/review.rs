use crate::aggregates::review::Review;
use crate::error::DomainError;
use crate::value_objects::ids::{ReviewId, UserId, WorkshopId};

/// Repository for persisting and retrieving [`Review`] aggregates.
#[async_trait::async_trait]
pub trait ReviewRepository: Send + Sync {
    /// Persist a new review.
    async fn create(&self, review: &Review) -> Result<(), DomainError>;
    /// Find a review by its unique ID.
    async fn find_by_id(&self, id: ReviewId) -> Result<Option<Review>, DomainError>;
    /// Find a review by user and workshop (at most one per pair).
    async fn find_by_user_and_workshop(
        &self,
        user_id: UserId,
        workshop_id: WorkshopId,
    ) -> Result<Option<Review>, DomainError>;
    /// Find all reviews for a workshop.
    async fn find_by_workshop(&self, workshop_id: WorkshopId) -> Result<Vec<Review>, DomainError>;
    /// Persist changes to an existing review.
    async fn update(&self, review: &Review) -> Result<(), DomainError>;
    /// Delete a review by ID.
    async fn delete(&self, id: ReviewId) -> Result<(), DomainError>;
}
