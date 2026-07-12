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
    /// Find all enrollments for a specific user.
    async fn find_by_user(&self, user_id: UserId) -> Result<Vec<Enrollment>, DomainError>;
    /// Find all enrollments for a specific user and workshop.
    async fn find_by_user_and_workshop(
        &self,
        user_id: UserId,
        workshop_id: WorkshopId,
    ) -> Result<Vec<Enrollment>, DomainError>;
    /// Update enrollment status only if the current status matches `from_status`.
    /// Returns `true` if the row was updated, `false` if no row matched (CAS miss).
    async fn update_status_cas(
        &self,
        id: EnrollmentId,
        from_status: &str,
        to_status: &str,
    ) -> Result<bool, DomainError>;
    /// Count enrollments with active status (pending, complete) for a workshop.
    async fn count_active_for_workshop(&self, workshop_id: WorkshopId) -> Result<i64, DomainError>;
    /// Persist changes to an existing enrollment.
    async fn update(&self, enrollment: &Enrollment) -> Result<(), DomainError>;
    /// Delete an enrollment by ID.
    async fn delete(&self, id: EnrollmentId) -> Result<(), DomainError>;
}
