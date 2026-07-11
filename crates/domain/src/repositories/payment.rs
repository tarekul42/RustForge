use crate::aggregates::payment::Payment;
use crate::error::DomainError;
use crate::value_objects::ids::{EnrollmentId, PaymentId};

/// Repository for persisting and retrieving [`Payment`] aggregates.
#[async_trait::async_trait]
pub trait PaymentRepository: Send + Sync {
    /// Persist a new payment.
    async fn create(&self, payment: &Payment) -> Result<(), DomainError>;
    /// Find a payment by its unique ID.
    async fn find_by_id(&self, id: PaymentId) -> Result<Option<Payment>, DomainError>;
    /// Find the payment associated with an enrollment.
    async fn find_by_enrollment_id(
        &self,
        enrollment_id: EnrollmentId,
    ) -> Result<Option<Payment>, DomainError>;
    /// Find a payment by its external transaction ID.
    async fn find_by_transaction_id(
        &self,
        transaction_id: &str,
    ) -> Result<Option<Payment>, DomainError>;
    /// Persist changes to an existing payment.
    async fn update(&self, payment: &Payment) -> Result<(), DomainError>;
}
