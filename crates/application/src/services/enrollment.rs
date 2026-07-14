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
    payment_repo: PR,
    workshop_repo: WR,
    event_store: ES,
    payment_gateway: PG,
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
    pub fn new(
        enrollment_repo: ER,
        payment_repo: PR,
        workshop_repo: WR,
        event_store: ES,
        payment_gateway: PG,
    ) -> Self {
        Self {
            enrollment_repo,
            payment_repo,
            workshop_repo,
            event_store,
            payment_gateway,
        }
    }

    /// Enroll a user in a workshop.
    ///
    /// Flow:
    /// 1. Fetch workshop and verify it exists.
    /// 2. Check user doesn't already have an active enrollment for this workshop.
    /// 3. Atomically reserve a seat (UPDATE with WHERE guard in DB).
    /// 4. Create enrollment and payment domain objects.
    /// 5. Persist enrollment and payment.
    /// 6. Initialize payment with the gateway.
    /// 7. Publish domain events.
    #[instrument(skip(self))]
    pub async fn create(
        &self,
        input: CreateEnrollmentInput,
    ) -> Result<CreateEnrollmentResult, ApplicationError> {
        let _workshop = self
            .workshop_repo
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

        let _updated = self
            .workshop_repo
            .reserve_seat_atomic(input.workshop_id)
            .await?
            .ok_or_else(|| ApplicationError::Unavailable("Workshop is full".to_string()))?;

        let (mut enrollment, enrollment_event) =
            Enrollment::new(input.user_id, input.workshop_id, input.student_count);

        let transaction_id = format!("TXN-{}", uuid::Uuid::now_v7());
        let price = Money::from_cents(_updated.price_cents);
        let (mut payment, payment_event) =
            Payment::new(enrollment.id, transaction_id.clone(), price);

        self.enrollment_repo.create(&enrollment).await?;
        self.payment_repo.create(&payment).await?;

        enrollment.payment_id = Some(payment.id);
        self.enrollment_repo.update(&enrollment).await?;

        let init_result = self
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
            .map_err(|e| ApplicationError::internal(format!("Payment gateway init failed: {e}")))?;

        if let Some(gateway_url) = &init_result.gateway_url {
            payment.payment_gateway_data = Some(serde_json::json!({
                "gateway_url": gateway_url,
                "session_key": init_result.session_key,
            }));
            self.payment_repo.update(&payment).await?;
        }

        self.publish_event(enrollment_event).await?;
        self.publish_event(payment_event).await?;

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

    async fn publish_event(&self, event: DomainEvent) -> Result<(), ApplicationError> {
        self.event_store
            .publish(&event, None)
            .await
            .map_err(|e| ApplicationError::internal(format!("failed to publish event: {e}")))
    }
}
