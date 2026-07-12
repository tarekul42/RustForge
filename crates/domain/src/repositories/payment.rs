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
    /// Update payment status only if the current status matches `from_status`.
    /// Returns `true` if the row was updated, `false` if no row matched (CAS miss).
    async fn update_status_cas(
        &self,
        id: crate::value_objects::ids::PaymentId,
        from_status: &str,
        to_status: &str,
    ) -> Result<bool, DomainError>;
    /// Acquire a Postgres advisory lock (transaction-scoped) for the given key.
    /// This serializes concurrent payment validations (e.g., IPN + success-URL race).
    async fn acquire_advisory_lock(&self, key: &str) -> Result<(), DomainError>;
    /// Persist changes to an existing payment.
    async fn update(&self, payment: &Payment) -> Result<(), DomainError>;
}
