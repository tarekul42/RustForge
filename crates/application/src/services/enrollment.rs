use crate::error::ApplicationError;
use sw_domain::aggregates::enrollment::{Enrollment, EnrollmentStatus};
use sw_domain::aggregates::payment::Payment;
use sw_domain::events::{DomainEvent, EventStore};
use sw_domain::repositories::enrollment::EnrollmentRepository;
use sw_domain::repositories::payment::PaymentRepository;
use sw_domain::repositories::workshop::WorkshopRepository;
use sw_domain::services::payment_gateway::PaymentGateway;
use sw_domain::value_objects::ids::{EnrollmentId, UserId, WorkshopId};
use sw_domain::value_objects::money::Money;
use tracing::instrument;

/// Input for creating a new enrollment (student registers for a workshop).
#[derive(Debug)]
pub struct CreateEnrollmentInput {
    /// Student enrolling.
    pub user_id: UserId,
    /// Workshop to enroll in.
    pub workshop_id: WorkshopId,
    /// Number of students (defaults to 1).
    pub student_count: i32,
    /// Customer name for payment gateway.
    pub cus_name: String,
    /// Customer email for payment gateway.
    pub cus_email: String,
    /// Customer phone for payment gateway.
    pub cus_phone: String,
}

/// Result of a successful enrollment creation.
#[derive(Debug)]
pub struct CreateEnrollmentResult {
    /// The created enrollment.
    pub enrollment: Enrollment,
    /// The created payment.
    pub payment: Payment,
    /// URL to redirect the user to for payment.
    pub gateway_url: Option<String>,
}

/// Application service for enrollment operations.
pub struct EnrollmentService<
    ER: EnrollmentRepository,
    PR: PaymentRepository,
    WR: WorkshopRepository,
    ES: EventStore,
    PG: PaymentGateway,
> {
    enrollment_repo: ER,
    #[allow(dead_code)]
    payment_repo: PR,
    workshop_repo: WR,
    #[allow(dead_code)]
    event_store: ES,
    payment_gateway: PG,
    /// Sqlx connection pool used for transactional writes.
    /// `None` in tests (skips transactional wrapping).
    pool: Option<sqlx::PgPool>,
}

impl<
    ER: EnrollmentRepository,
    PR: PaymentRepository,
    WR: WorkshopRepository,
    ES: EventStore,
    PG: PaymentGateway,
> EnrollmentService<ER, PR, WR, ES, PG>
{
    /// Create a new `EnrollmentService`.
    #[allow(missing_docs)]
    pub fn new(
        enrollment_repo: ER,
        payment_repo: PR,
        workshop_repo: WR,
        event_store: ES,
        payment_gateway: PG,
        pool: Option<sqlx::PgPool>,
    ) -> Self {
        Self {
            enrollment_repo,
            payment_repo,
            workshop_repo,
            event_store,
            payment_gateway,
            pool,
        }
    }

    /// Enroll a user in a workshop.
    ///
    /// Flow:
    /// 1. Fetch workshop and verify it exists.
    /// 2. Check user doesn't already have an active enrollment.
    /// 3. Atomically reserve a seat.
    /// 4. Create enrollment and payment domain objects.
    /// 5. Initialize payment with the gateway (BEFORE writes).
    /// 6. Persist enrollment, payment, and events in a single DB transaction.
    #[instrument(skip(self))]
    pub async fn create(
        &self,
        input: CreateEnrollmentInput,
    ) -> Result<CreateEnrollmentResult, ApplicationError> {
        self.workshop_repo
            .find_by_id(input.workshop_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Workshop", input.workshop_id))?;

        let existing = self
            .enrollment_repo
            .find_by_user_and_workshop(input.user_id, input.workshop_id)
            .await?;
        if existing.iter().any(|e| {
            matches!(
                e.status,
                EnrollmentStatus::Pending | EnrollmentStatus::Complete
            )
        }) {
            return Err(ApplicationError::conflict(
                "User already has an active enrollment for this workshop",
            ));
        }

        let workshop = self
            .workshop_repo
            .reserve_seat_atomic(input.workshop_id)
            .await?
            .ok_or_else(|| ApplicationError::Unavailable("Workshop is full".to_string()))?;

        let (mut enrollment, enrollment_event) =
            Enrollment::new(input.user_id, input.workshop_id, input.student_count);

        let transaction_id = format!("TXN-{}", uuid::Uuid::now_v7());
        let price = Money::from_cents(workshop.price_cents);
        let (mut payment, payment_event) =
            Payment::new(enrollment.id, transaction_id.clone(), price);

        enrollment.payment_id = Some(payment.id);

        // Initialize gateway BEFORE any DB writes.
        // If this fails, the seat is released atomically and no DB state changed.
        let init_result = match self
            .payment_gateway
            .init_payment(
                &transaction_id,
                price.cents(),
                "BDT",
                &input.cus_name,
                &input.cus_email,
                &input.cus_phone,
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let _ = self
                    .workshop_repo
                    .release_seat_atomic(input.workshop_id)
                    .await;
                return Err(ApplicationError::internal(format!(
                    "Payment gateway init failed: {e}"
                )));
            }
        };

        if let Some(gateway_url) = &init_result.gateway_url {
            payment.payment_gateway_data = Some(serde_json::json!({
                "gateway_url": gateway_url,
                "session_key": init_result.session_key,
            }));
        }

        // All writes in a single transaction (or use repos directly when no pool is available, e.g. tests)
        if let Some(ref pool) = self.pool {
            let mut tx = match pool.begin().await {
                Ok(tx) => tx,
                Err(e) => {
                    let _ = self
                        .workshop_repo
                        .release_seat_atomic(input.workshop_id)
                        .await;
                    return Err(ApplicationError::internal(format!(
                        "failed to begin transaction: {e}"
                    )));
                }
            };

            if let Err(e) = sqlx::query(
                r#"INSERT INTO enrollments (id, user_id, workshop_id, payment_id, student_count, status, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, 'pending', NOW(), NOW())"#,
            )
            .bind(enrollment.id.into_uuid())
            .bind(enrollment.user_id.into_uuid())
            .bind(enrollment.workshop_id.into_uuid())
            .bind(enrollment.payment_id.map(|id| id.into_uuid()))
            .bind(enrollment.student_count)
            .execute(&mut *tx)
            .await
            {
                let _ = tx.rollback().await;
                let _ = self.workshop_repo.release_seat_atomic(input.workshop_id).await;
                return Err(ApplicationError::internal(format!("failed to create enrollment: {e}")));
            }

            if let Err(e) = sqlx::query(
                r#"INSERT INTO payments (id, enrollment_id, transaction_id, amount_cents, currency, payment_gateway_data, invoice_url, status, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, 'BDT', $5, NULL, 'unpaid', NOW(), NOW())"#,
            )
            .bind(payment.id.into_uuid())
            .bind(payment.enrollment_id.into_uuid())
            .bind(&payment.transaction_id)
            .bind(payment.amount.cents())
            .bind(&payment.payment_gateway_data)
            .execute(&mut *tx)
            .await
            {
                let _ = tx.rollback().await;
                let _ = self.workshop_repo.release_seat_atomic(input.workshop_id).await;
                return Err(ApplicationError::internal(format!("failed to create payment: {e}")));
            }

            if let Some(ref data) = payment.payment_gateway_data {
                if let Err(e) = sqlx::query(
                    "UPDATE payments SET payment_gateway_data = $2, updated_at = NOW() WHERE id = $1",
                )
                .bind(payment.id.into_uuid())
                .bind(data)
                .execute(&mut *tx)
                .await
                {
                    let _ = tx.rollback().await;
                    let _ = self.workshop_repo.release_seat_atomic(input.workshop_id).await;
                    return Err(ApplicationError::internal(format!("failed to update payment gateway data: {e}")));
                }
            }

            for event in [enrollment_event, payment_event] {
                if let Err(e) = self.publish_event_in_tx(&mut tx, event).await {
                    let _ = tx.rollback().await;
                    let _ = self
                        .workshop_repo
                        .release_seat_atomic(input.workshop_id)
                        .await;
                    return Err(e);
                }
            }

            if let Err(e) = tx.commit().await {
                let _ = self
                    .workshop_repo
                    .release_seat_atomic(input.workshop_id)
                    .await;
                return Err(ApplicationError::internal(format!(
                    "failed to commit transaction: {e}"
                )));
            }
        } else {
            // Non-transactional path for tests with mock repos
            self.enrollment_repo.create(&enrollment).await?;
            self.payment_repo.create(&payment).await?;
            self.enrollment_repo.update(&enrollment).await?;
            if payment.payment_gateway_data.is_some() {
                self.payment_repo.update(&payment).await?;
            }
            self.publish_event(enrollment_event).await?;
            self.publish_event(payment_event).await?;
        }

        Ok(CreateEnrollmentResult {
            enrollment,
            payment,
            gateway_url: init_result.gateway_url,
        })
    }

    /// Find an enrollment by ID.
    #[instrument(skip(self))]
    pub async fn find_by_id(
        &self,
        id: EnrollmentId,
    ) -> Result<Option<Enrollment>, ApplicationError> {
        self.enrollment_repo
            .find_by_id(id)
            .await
            .map_err(ApplicationError::from)
    }

    /// List all enrollments for a user.
    #[instrument(skip(self))]
    pub async fn list_by_user(&self, user_id: UserId) -> Result<Vec<Enrollment>, ApplicationError> {
        self.enrollment_repo
            .find_by_user(user_id)
            .await
            .map_err(ApplicationError::from)
    }

    #[allow(dead_code)]
    async fn publish_event(&self, event: DomainEvent) -> Result<(), ApplicationError> {
        self.event_store
            .publish(&event, None)
            .await
            .map_err(|e| ApplicationError::internal(format!("failed to publish event: {e}")))
    }

    async fn publish_event_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        event: DomainEvent,
    ) -> Result<(), ApplicationError> {
        let event_type = event.event_type();
        let aggregate_type = event.aggregate_type();
        let aggregate_id = aggregate_id_from_event(&event);
        let changes = serde_json::to_value(&event).unwrap_or_default();

        sqlx::query(
            r#"INSERT INTO audit_logs (event_type, aggregate_type, aggregate_id, actor_id, ip_address, user_agent, changes)
               VALUES ($1, $2, $3, NULL, NULL::inet, NULL, $4)"#,
        )
        .bind(event_type)
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(changes)
        .execute(&mut **tx)
        .await
        .map_err(|e| ApplicationError::internal(format!("failed to publish event: {e}")))?;
        Ok(())
    }
}

pub(super) fn aggregate_id_from_event(event: &sw_domain::events::DomainEvent) -> uuid::Uuid {
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
