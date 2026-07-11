use crate::aggregates::enrollment::Enrollment;
use crate::error::DomainError;
use crate::value_objects::ids::{EnrollmentId, UserId, WorkshopId};

/// Repository for persisting and retrieving [`Enrollment`] aggregates.
#[async_trait::async_trait]
pub trait EnrollmentRepository: Send + Sync {
    /// Persist a new enrollment.
    async fn create(&self, enrollment: &Enrollment) -> Result<(), DomainError>;
    /// Find an enrollment by its unique ID.
    async fn find_by_id(&self, id: EnrollmentId) -> Result<Option<Enrollment>, DomainError>;
    /// Find all enrollments for a specific user and workshop.
    async fn find_by_user_and_workshop(
        &self,
        user_id: UserId,
        workshop_id: WorkshopId,
    ) -> Result<Vec<Enrollment>, DomainError>;
    /// Persist changes to an existing enrollment.
    async fn update(&self, enrollment: &Enrollment) -> Result<(), DomainError>;
    /// Delete an enrollment by ID.
    async fn delete(&self, id: EnrollmentId) -> Result<(), DomainError>;
}
