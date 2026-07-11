use sqlx::PgPool;
use sw_domain::error::DomainError;
use sw_domain::events::{DomainEvent, EventStore};

/// SQLx-backed implementation of [`EventStore`].
///
/// Persists domain events to the `audit_logs` table for the audit trail.
pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EventStore for PostgresEventStore {
    async fn publish(&self, event: &DomainEvent) -> Result<(), DomainError> {
        let event_type = event.event_type();
        let aggregate_type = event.aggregate_type();
        let aggregate_id = aggregate_id_from_event(event);

        sqlx::query(
            r#"INSERT INTO audit_logs (event_type, aggregate_type, aggregate_id, changes)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(event_type)
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(serde_json::to_value(event).unwrap_or_default())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to publish event: {e}")))?;
        Ok(())
    }
}

fn aggregate_id_from_event(event: &DomainEvent) -> uuid::Uuid {
    match event {
        DomainEvent::UserRegistered { user_id, .. } => (*user_id).into(),
        DomainEvent::UserVerified { user_id } => (*user_id).into(),
        DomainEvent::PasswordChanged { user_id } => (*user_id).into(),
        DomainEvent::UserUpdated { user_id } => (*user_id).into(),
        DomainEvent::UserDeleted { user_id } => (*user_id).into(),
        DomainEvent::CategoryCreated { category_id } => (*category_id).into(),
        DomainEvent::CategoryUpdated { category_id } => (*category_id).into(),
        DomainEvent::CategoryDeleted { category_id } => (*category_id).into(),
        DomainEvent::LevelCreated { level_id } => (*level_id).into(),
        DomainEvent::LevelUpdated { level_id } => (*level_id).into(),
        DomainEvent::LevelDeleted { level_id } => (*level_id).into(),
        DomainEvent::WorkshopCreated { workshop_id } => (*workshop_id).into(),
        DomainEvent::WorkshopUpdated { workshop_id } => (*workshop_id).into(),
        DomainEvent::WorkshopDeleted { workshop_id } => (*workshop_id).into(),
        DomainEvent::EnrollmentCreated { enrollment_id } => (*enrollment_id).into(),
        DomainEvent::EnrollmentStatusChanged { enrollment_id, .. } => (*enrollment_id).into(),
        DomainEvent::EnrollmentCancelled { enrollment_id } => (*enrollment_id).into(),
        DomainEvent::PaymentCreated { payment_id } => (*payment_id).into(),
        DomainEvent::PaymentStatusChanged { payment_id, .. } => (*payment_id).into(),
        DomainEvent::PaymentRefunded { payment_id, .. } => (*payment_id).into(),
        DomainEvent::ReviewCreated { review_id } => (*review_id).into(),
        DomainEvent::ReviewModerated { review_id, .. } => (*review_id).into(),
        DomainEvent::ContactCreated { contact_id } => (*contact_id).into(),
        _ => unreachable!("unknown DomainEvent variant"),
    }
}
