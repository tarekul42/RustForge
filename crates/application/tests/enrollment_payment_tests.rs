use std::collections::HashMap;
use std::sync::Mutex;
use sw_application::services::enrollment::{CreateEnrollmentInput, EnrollmentService};
use sw_application::services::payment::PaymentService;
use sw_domain::aggregates::enrollment::{Enrollment, EnrollmentStatus};
use sw_domain::aggregates::payment::{Payment, PaymentStatus};
use sw_domain::aggregates::refund_log::RefundLog;
use sw_domain::aggregates::workshop::Workshop;
use sw_domain::error::DomainError;
use sw_domain::events::{DomainEvent, EventStore};
use sw_domain::repositories::enrollment::EnrollmentRepository;
use sw_domain::repositories::job::JobRepository;
use sw_domain::repositories::payment::PaymentRepository;
use sw_domain::repositories::refund_log::RefundLogRepository;
use sw_domain::repositories::workshop::WorkshopRepository;
use sw_domain::services::payment_gateway::{
    GatewayInitResponse, GatewayValidationResponse, PaymentGateway, PaymentGatewayError,
};
use sw_domain::value_objects::ids::*;
use sw_domain::value_objects::money::Money;

// ---------------------------------------------------------------------------
// Mock repositories
// ---------------------------------------------------------------------------

struct MockEnrollmentRepo {
    enrollments: Mutex<Vec<Enrollment>>,
}

impl MockEnrollmentRepo {
    fn new() -> Self {
        Self {
            enrollments: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl EnrollmentRepository for MockEnrollmentRepo {
    async fn create(&self, enrollment: &Enrollment) -> Result<(), DomainError> {
        self.enrollments.lock().unwrap().push(enrollment.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: EnrollmentId) -> Result<Option<Enrollment>, DomainError> {
        Ok(self
            .enrollments
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.id() == id)
            .cloned())
    }

    async fn find_by_user(&self, user_id: UserId) -> Result<Vec<Enrollment>, DomainError> {
        Ok(self
            .enrollments
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.user_id() == user_id)
            .cloned()
            .collect())
    }

    async fn find_by_user_and_workshop(
        &self,
        user_id: UserId,
        _workshop_id: WorkshopId,
    ) -> Result<Vec<Enrollment>, DomainError> {
        Ok(self
            .enrollments
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.user_id() == user_id)
            .cloned()
            .collect())
    }

    async fn update_status_cas(
        &self,
        _id: EnrollmentId,
        _from: &str,
        _to: &str,
    ) -> Result<bool, DomainError> {
        Ok(true)
    }

    async fn count_active_for_workshop(
        &self,
        _workshop_id: WorkshopId,
    ) -> Result<i64, DomainError> {
        Ok(0)
    }

    async fn update(&self, _enrollment: &Enrollment) -> Result<(), DomainError> {
        Ok(())
    }

    async fn delete(&self, _id: EnrollmentId) -> Result<(), DomainError> {
        Ok(())
    }
}

struct MockPaymentRepo {
    payments: Mutex<Vec<Payment>>,
}

impl MockPaymentRepo {
    fn new() -> Self {
        Self {
            payments: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl PaymentRepository for MockPaymentRepo {
    async fn create(&self, payment: &Payment) -> Result<(), DomainError> {
        self.payments.lock().unwrap().push(payment.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: PaymentId) -> Result<Option<Payment>, DomainError> {
        Ok(self
            .payments
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.id() == id)
            .cloned())
    }

    async fn find_by_enrollment_id(
        &self,
        enrollment_id: EnrollmentId,
    ) -> Result<Option<Payment>, DomainError> {
        Ok(self
            .payments
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.enrollment_id() == enrollment_id)
            .cloned())
    }

    async fn find_by_transaction_id(
        &self,
        transaction_id: &str,
    ) -> Result<Option<Payment>, DomainError> {
        Ok(self
            .payments
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.transaction_id() == transaction_id)
            .cloned())
    }

    async fn update_status_cas(
        &self,
        _id: PaymentId,
        _from: &str,
        _to: &str,
    ) -> Result<bool, DomainError> {
        Ok(true)
    }

    async fn acquire_advisory_lock(&self, _key: &str) -> Result<(), DomainError> {
        Ok(())
    }

    async fn update(&self, _payment: &Payment) -> Result<(), DomainError> {
        Ok(())
    }
}

struct MockWorkshopRepo {
    workshops: Mutex<Vec<Workshop>>,
}

impl MockWorkshopRepo {
    fn new(workshops: Vec<Workshop>) -> Self {
        Self {
            workshops: Mutex::new(workshops),
        }
    }
}

#[async_trait::async_trait]
impl WorkshopRepository for MockWorkshopRepo {
    async fn create(&self, _workshop: &Workshop) -> Result<(), DomainError> {
        Ok(())
    }
    async fn find_by_id(&self, id: WorkshopId) -> Result<Option<Workshop>, DomainError> {
        Ok(self
            .workshops
            .lock()
            .unwrap()
            .iter()
            .find(|w| w.id() == id)
            .cloned())
    }
    async fn find_by_slug(&self, _slug: &str) -> Result<Option<Workshop>, DomainError> {
        Ok(None)
    }
    async fn update(&self, _workshop: &Workshop) -> Result<(), DomainError> {
        Ok(())
    }
    async fn delete(&self, _id: WorkshopId) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_images(
        &self,
        _workshop_id: WorkshopId,
    ) -> Result<Vec<sw_domain::aggregates::workshop::WorkshopImage>, DomainError> {
        Ok(vec![])
    }
    async fn add_image(
        &self,
        _workshop_id: WorkshopId,
        _url: &str,
        _s3_key: &str,
    ) -> Result<sw_domain::aggregates::workshop::WorkshopImage, DomainError> {
        Ok(sw_domain::aggregates::workshop::WorkshopImage::from_parts(
            WorkshopImageId::new(),
            _workshop_id,
            String::new(),
            String::new(),
            chrono::Utc::now(),
        ))
    }
    async fn remove_image(&self, _image_id: WorkshopImageId) -> Result<(), DomainError> {
        Ok(())
    }
    async fn find_all(&self) -> Result<Vec<Workshop>, DomainError> {
        Ok(vec![])
    }
    async fn reserve_seat_atomic(
        &self,
        _workshop_id: WorkshopId,
    ) -> Result<Option<Workshop>, DomainError> {
        let w = self.workshops.lock().unwrap().first().cloned();
        Ok(w)
    }
    async fn release_seat_atomic(&self, _workshop_id: WorkshopId) -> Result<(), DomainError> {
        Ok(())
    }
}

struct MockEventStore;

#[async_trait::async_trait]
impl EventStore for MockEventStore {
    async fn publish(
        &self,
        _event: &DomainEvent,
        _context: Option<&sw_domain::events::AuditContext>,
    ) -> Result<(), DomainError> {
        Ok(())
    }
}

struct MockJobRepo;

#[async_trait::async_trait]
impl JobRepository for MockJobRepo {
    async fn enqueue(
        &self,
        _job_type: &str,
        _payload: &serde_json::Value,
        _run_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn claim_next(
        &self,
        _worker_id: uuid::Uuid,
    ) -> Result<Option<sw_domain::repositories::job::JobInfo>, DomainError> {
        Ok(None)
    }

    async fn complete(&self, _job_id: JobId) -> Result<(), DomainError> {
        Ok(())
    }

    async fn fail(&self, _job_id: JobId, _error: &str) -> Result<(), DomainError> {
        Ok(())
    }

    async fn retry(&self, _job_id: JobId, _base_seconds: i64) -> Result<bool, DomainError> {
        Ok(true)
    }
}

struct MockRefundLogRepo;

#[async_trait::async_trait]
impl RefundLogRepository for MockRefundLogRepo {
    async fn create(&self, _log: &RefundLog) -> Result<(), DomainError> {
        Ok(())
    }
}

struct MockPaymentGateway {
    should_succeed: bool,
}

#[async_trait::async_trait]
impl PaymentGateway for MockPaymentGateway {
    async fn init_payment(
        &self,
        _txn: &str,
        _amount: i64,
        _cur: &str,
        _name: &str,
        _email: &str,
        _phone: &str,
    ) -> Result<GatewayInitResponse, PaymentGatewayError> {
        Ok(GatewayInitResponse {
            success: self.should_succeed,
            gateway_url: if self.should_succeed {
                Some("https://gateway.test/pay".to_string())
            } else {
                None
            },
            session_key: Some("sess_key".to_string()),
            error_message: if self.should_succeed {
                None
            } else {
                Some("Init failed".to_string())
            },
        })
    }

    async fn validate_payment(
        &self,
        _val_id: &str,
    ) -> Result<GatewayValidationResponse, PaymentGatewayError> {
        Ok(GatewayValidationResponse {
            is_valid: true,
            amount: Some("100.00".to_string()),
            currency: Some("BDT".to_string()),
            transaction_id: Some("txn_123".to_string()),
            raw_data: serde_json::json!({"status": "VALID"}),
        })
    }

    fn verify_ipn_signature(&self, _data: &HashMap<String, String>) -> bool {
        true
    }
}

fn make_workshop() -> Workshop {
    let (w, _) = Workshop::new(
        "Test".into(),
        "test".into(),
        10000,
        CategoryId::new(),
        LevelId::new(),
        UserId::new(),
    );
    w
}

fn make_enrollment_input(user_id: UserId, workshop_id: WorkshopId) -> CreateEnrollmentInput {
    CreateEnrollmentInput {
        user_id,
        workshop_id,
        student_count: 1,
        cus_name: "Test User".into(),
        cus_email: "test@test.com".into(),
        cus_phone: "1234567890".into(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn enrollment_create_success() {
    let workshop = make_workshop();
    let workshop_id = workshop.id();
    let user_id = UserId::new();

    let service = EnrollmentService::new(
        MockEnrollmentRepo::new(),
        MockPaymentRepo::new(),
        MockWorkshopRepo::new(vec![workshop]),
        MockEventStore,
        MockPaymentGateway {
            should_succeed: true,
        },
        None,
    );

    let result = service
        .create(make_enrollment_input(user_id, workshop_id))
        .await
        .expect("enrollment should succeed");

    assert!(result.gateway_url.is_some());
    assert_eq!(result.gateway_url.unwrap(), "https://gateway.test/pay");
}

#[tokio::test]
async fn enrollment_create_workshop_not_found() {
    let user_id = UserId::new();
    let workshop_id = WorkshopId::new();

    let service = EnrollmentService::new(
        MockEnrollmentRepo::new(),
        MockPaymentRepo::new(),
        MockWorkshopRepo::new(vec![]), // no workshop
        MockEventStore,
        MockPaymentGateway {
            should_succeed: true,
        },
        None,
    );

    let err = service
        .create(make_enrollment_input(user_id, workshop_id))
        .await
        .expect_err("should fail with NotFound");

    assert!(matches!(
        err,
        sw_application::error::ApplicationError::NotFound(_)
    ));
}

#[tokio::test]
async fn payment_success_cas_idempotent() {
    let payment_id = PaymentId::new();
    let enrollment_id = EnrollmentId::new();
    let user_id = UserId::new();
    let workshop = make_workshop();

    let payment = Payment::from_parts(
        payment_id,
        enrollment_id,
        "txn_123".into(),
        Money::from_cents(10000),
        None,
        None,
        PaymentStatus::Unpaid,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let pay_repo = MockPaymentRepo::new();
    pay_repo.payments.lock().unwrap().push(payment.clone());

    let enrollment = Enrollment::from_parts(
        enrollment_id,
        user_id,
        workshop.id(),
        Some(payment_id),
        1,
        EnrollmentStatus::Pending,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let enroll_repo = MockEnrollmentRepo::new();
    enroll_repo.enrollments.lock().unwrap().push(enrollment);

    let workshop_repo = MockWorkshopRepo::new(vec![workshop]);

    let service = PaymentService::new(
        pay_repo,
        enroll_repo,
        MockEventStore,
        MockPaymentGateway {
            should_succeed: true,
        },
        workshop_repo,
        MockJobRepo,
        MockRefundLogRepo,
        None,
    );

    // First call should succeed
    let result = service.success_payment("txn_123", "val_123").await;
    assert!(
        result.is_ok(),
        "first success_payment should succeed: {:?}",
        result.err()
    );

    // Second call should also succeed (idempotent) — payment is already PAID
    let result2 = service.success_payment("txn_123", "val_123").await;
    assert!(
        result2.is_ok(),
        "idempotent call should succeed: {:?}",
        result2.err()
    );
}

#[tokio::test]
async fn refund_paid_payment_succeeds() {
    let payment_id = PaymentId::new();
    let enrollment_id = EnrollmentId::new();
    let payment = Payment::from_parts(
        payment_id,
        enrollment_id,
        "txn_refund".into(),
        Money::from_cents(10000),
        None,
        None,
        PaymentStatus::Paid,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let pay_repo = MockPaymentRepo::new();
    pay_repo.payments.lock().unwrap().push(payment);

    let enrollment = Enrollment::from_parts(
        enrollment_id,
        UserId::new(),
        WorkshopId::new(),
        Some(payment_id),
        1,
        EnrollmentStatus::Complete,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let enroll_repo = MockEnrollmentRepo::new();
    enroll_repo.enrollments.lock().unwrap().push(enrollment);
    let workshop = make_workshop();
    let workshop_repo = MockWorkshopRepo::new(vec![workshop]);

    let service = PaymentService::new(
        pay_repo,
        enroll_repo,
        MockEventStore,
        MockPaymentGateway {
            should_succeed: true,
        },
        workshop_repo,
        MockJobRepo,
        MockRefundLogRepo,
        None,
    );

    let result = service
        .refund(payment_id, "Customer requested".into())
        .await;
    assert!(result.is_ok(), "refund should succeed: {:?}", result.err());
}

#[tokio::test]
async fn refund_unpaid_payment_fails() {
    let payment_id = PaymentId::new();
    let enrollment_id = EnrollmentId::new();
    let payment = Payment::from_parts(
        payment_id,
        enrollment_id,
        "txn_unpaid".into(),
        Money::from_cents(10000),
        None,
        None,
        PaymentStatus::Unpaid,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let pay_repo = MockPaymentRepo::new();
    pay_repo.payments.lock().unwrap().push(payment);
    let workshop = make_workshop();

    let service = PaymentService::new(
        pay_repo,
        MockEnrollmentRepo::new(),
        MockEventStore,
        MockPaymentGateway {
            should_succeed: true,
        },
        MockWorkshopRepo::new(vec![workshop]),
        MockJobRepo,
        MockRefundLogRepo,
        None,
    );

    let result = service.refund(payment_id, "Test".into()).await;
    assert!(result.is_err(), "refunding unpaid should fail");
    assert!(matches!(
        result.unwrap_err(),
        sw_application::error::ApplicationError::Conflict(_)
    ));
}
