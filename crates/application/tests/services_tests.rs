use std::sync::Mutex;

use sw_application::error::ApplicationError;
use sw_application::services::auth::AuthService;
use sw_application::services::category::CategoryService;
use sw_application::services::contact::{ContactService, SubmitContactInput};
use sw_application::services::level::LevelService;
use sw_application::services::review::{CreateReviewInput, ReviewService, UpdateReviewInput};
use sw_application::services::stats::StatsService;
use sw_application::services::user::{UpdateUserInput, UserAdminService};
use sw_application::services::workshop::{
    CreateWorkshopInput, UpdateWorkshopInput, WorkshopService,
};
use sw_domain::aggregates::category::Category;
use sw_domain::aggregates::contact::Contact;
use sw_domain::aggregates::review::{Review, ReviewStatus};
use sw_domain::aggregates::user::{User, UserRole, UserStatus};
use sw_domain::aggregates::workshop::Workshop;
use sw_domain::error::DomainError;
use sw_domain::events::{AuditContext, DomainEvent, EventStore};
use sw_domain::repositories::category::CategoryRepository;
use sw_domain::repositories::contact::ContactRepository;
use sw_domain::repositories::enrollment::EnrollmentRepository;
use sw_domain::repositories::level::LevelRepository;
use sw_domain::repositories::otp::OtpRepository;
use sw_domain::repositories::review::ReviewRepository;
use sw_domain::repositories::session::SessionRepository;
use sw_domain::repositories::stats::{PlatformStats, StatsRepository};
use sw_domain::repositories::user::UserRepository;
use sw_domain::repositories::workshop::WorkshopRepository;
use sw_domain::value_objects::Email;
use sw_domain::value_objects::ids::JobId;
use sw_domain::value_objects::ids::*;

// ---------------------------------------------------------------------------
// Shared mock helpers
// ---------------------------------------------------------------------------

struct MockEventStore;

struct MockJobRepo;

#[async_trait::async_trait]
impl sw_domain::repositories::job::JobRepository for MockJobRepo {
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

#[async_trait::async_trait]
impl EventStore for MockEventStore {
    async fn publish(
        &self,
        _event: &DomainEvent,
        _context: Option<&AuditContext>,
    ) -> Result<(), DomainError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// MockUserRepo
// ---------------------------------------------------------------------------

struct MockUserRepo {
    users: Mutex<Vec<User>>,
}

impl MockUserRepo {
    fn new() -> Self {
        Self {
            users: Mutex::new(Vec::new()),
        }
    }

    fn with_users(users: Vec<User>) -> Self {
        Self {
            users: Mutex::new(users),
        }
    }
}

#[async_trait::async_trait]
impl UserRepository for MockUserRepo {
    async fn create(&self, user: &User) -> Result<(), DomainError> {
        self.users.lock().unwrap().push(user.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.id() == id)
            .cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.email().as_str() == email)
            .cloned())
    }

    async fn update(&self, _user: &User) -> Result<(), DomainError> {
        Ok(())
    }

    async fn delete(&self, _id: UserId) -> Result<(), DomainError> {
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<User>, DomainError> {
        Ok(self.users.lock().unwrap().clone())
    }
}

// ---------------------------------------------------------------------------
// MockSessionRepo
// ---------------------------------------------------------------------------

struct MockSessionRepo;

#[async_trait::async_trait]
impl SessionRepository for MockSessionRepo {
    async fn create(
        &self,
        _session_id: SessionId,
        _user_id: UserId,
        _token_hash: &str,
        _expires_at: &chrono::DateTime<chrono::Utc>,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn find_by_token_hash(
        &self,
        _token_hash: &str,
    ) -> Result<Option<(SessionId, UserId, chrono::DateTime<chrono::Utc>)>, DomainError> {
        Ok(None)
    }

    async fn delete(&self, _session_id: SessionId) -> Result<(), DomainError> {
        Ok(())
    }

    async fn delete_all_for_user(&self, _user_id: UserId) -> Result<(), DomainError> {
        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<u64, DomainError> {
        Ok(0)
    }
}

// ---------------------------------------------------------------------------
// MockOtpRepo
// ---------------------------------------------------------------------------

struct MockOtpRepo {
    hash: Mutex<String>,
    attempts: Mutex<i32>,
}

impl MockOtpRepo {
    fn new() -> Self {
        Self {
            hash: Mutex::new(String::new()),
            attempts: Mutex::new(0),
        }
    }
}

#[async_trait::async_trait]
impl OtpRepository for MockOtpRepo {
    async fn create(
        &self,
        _email: &str,
        code_hash: &str,
        _expires_at: &chrono::DateTime<chrono::Utc>,
    ) -> Result<(), DomainError> {
        *self.hash.lock().unwrap() = code_hash.to_string();
        *self.attempts.lock().unwrap() = 0;
        Ok(())
    }

    async fn find_by_email(
        &self,
        _email: &str,
    ) -> Result<Option<(String, i32, chrono::DateTime<chrono::Utc>)>, DomainError> {
        let hash = self.hash.lock().unwrap().clone();
        let attempts = *self.attempts.lock().unwrap();
        if hash.is_empty() {
            Ok(None)
        } else {
            Ok(Some((
                hash,
                attempts,
                chrono::Utc::now() + chrono::Duration::minutes(10),
            )))
        }
    }

    async fn increment_attempts(&self, _email: &str) -> Result<(), DomainError> {
        *self.attempts.lock().unwrap() += 1;
        Ok(())
    }

    async fn delete(&self, _email: &str) -> Result<(), DomainError> {
        *self.hash.lock().unwrap() = String::new();
        *self.attempts.lock().unwrap() = 0;
        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<u64, DomainError> {
        Ok(0)
    }
}

// ---------------------------------------------------------------------------
// MockCategoryRepo
// ---------------------------------------------------------------------------

struct MockCategoryRepo {
    categories: Mutex<Vec<Category>>,
}

impl MockCategoryRepo {
    fn new() -> Self {
        Self {
            categories: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl CategoryRepository for MockCategoryRepo {
    async fn create(&self, category: &Category) -> Result<(), DomainError> {
        self.categories.lock().unwrap().push(category.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: CategoryId) -> Result<Option<Category>, DomainError> {
        Ok(self
            .categories
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.id() == id)
            .cloned())
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Category>, DomainError> {
        Ok(self
            .categories
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.slug() == slug)
            .cloned())
    }

    async fn find_all(&self) -> Result<Vec<Category>, DomainError> {
        Ok(self.categories.lock().unwrap().clone())
    }

    async fn update(&self, _category: &Category) -> Result<(), DomainError> {
        Ok(())
    }

    async fn delete(&self, _id: CategoryId) -> Result<(), DomainError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// MockLevelRepo
// ---------------------------------------------------------------------------

struct MockLevelRepo;

#[async_trait::async_trait]
impl LevelRepository for MockLevelRepo {
    async fn create(
        &self,
        _level: &sw_domain::aggregates::level::Level,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn find_by_id(
        &self,
        _id: LevelId,
    ) -> Result<Option<sw_domain::aggregates::level::Level>, DomainError> {
        Ok(Some(
            sw_domain::aggregates::level::Level::new("Beginner".into()).0,
        ))
    }

    async fn find_all(&self) -> Result<Vec<sw_domain::aggregates::level::Level>, DomainError> {
        Ok(vec![])
    }

    async fn update(
        &self,
        _level: &sw_domain::aggregates::level::Level,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn delete(&self, _id: LevelId) -> Result<(), DomainError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// MockReviewRepo
// ---------------------------------------------------------------------------

struct MockReviewRepo {
    reviews: Mutex<Vec<Review>>,
}

impl MockReviewRepo {
    fn new() -> Self {
        Self {
            reviews: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl ReviewRepository for MockReviewRepo {
    async fn create(&self, review: &Review) -> Result<(), DomainError> {
        self.reviews.lock().unwrap().push(review.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: ReviewId) -> Result<Option<Review>, DomainError> {
        Ok(self
            .reviews
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.id() == id)
            .cloned())
    }

    async fn find_by_user_and_workshop(
        &self,
        user_id: UserId,
        workshop_id: WorkshopId,
    ) -> Result<Option<Review>, DomainError> {
        Ok(self
            .reviews
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.user_id() == user_id && r.workshop_id() == workshop_id)
            .cloned())
    }

    async fn find_by_workshop(&self, workshop_id: WorkshopId) -> Result<Vec<Review>, DomainError> {
        Ok(self
            .reviews
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.workshop_id() == workshop_id)
            .cloned()
            .collect())
    }

    async fn update(&self, _review: &Review) -> Result<(), DomainError> {
        Ok(())
    }

    async fn delete(&self, _id: ReviewId) -> Result<(), DomainError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// MockEnrollmentRepo (minimal)
// ---------------------------------------------------------------------------

struct MockEnrollmentRepoEmpty;

#[async_trait::async_trait]
impl EnrollmentRepository for MockEnrollmentRepoEmpty {
    async fn create(
        &self,
        _e: &sw_domain::aggregates::enrollment::Enrollment,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn find_by_id(
        &self,
        _id: EnrollmentId,
    ) -> Result<Option<sw_domain::aggregates::enrollment::Enrollment>, DomainError> {
        Ok(None)
    }

    async fn find_by_user(
        &self,
        _user_id: UserId,
    ) -> Result<Vec<sw_domain::aggregates::enrollment::Enrollment>, DomainError> {
        Ok(vec![])
    }

    async fn find_by_user_and_workshop(
        &self,
        _user_id: UserId,
        _workshop_id: WorkshopId,
    ) -> Result<Vec<sw_domain::aggregates::enrollment::Enrollment>, DomainError> {
        Ok(vec![sw_domain::aggregates::enrollment::Enrollment::from_parts(
            EnrollmentId::new(),
            UserId::new(),
            WorkshopId::new(),
            None,
            1,
            sw_domain::aggregates::enrollment::EnrollmentStatus::Complete,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )])
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

    async fn update(
        &self,
        _e: &sw_domain::aggregates::enrollment::Enrollment,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn delete(&self, _id: EnrollmentId) -> Result<(), DomainError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_user(email: &str) -> User {
    User::from_parts(
        UserId::new(),
        Email::new(email).unwrap(),
        String::new(),
        None,
        None,
        None,
        None,
        None,
        UserRole::Student,
        UserStatus::Active,
        true,
        None,
        None,
        chrono::Utc::now(),
        chrono::Utc::now(),
    )
}

fn make_workshop() -> (WorkshopId, Workshop) {
    let id = WorkshopId::new();
    let (w, _) = Workshop::new(
        "Test".into(),
        "test".into(),
        10000,
        CategoryId::new(),
        LevelId::new(),
        UserId::new(),
    );
    (id, w)
}

// ---------------------------------------------------------------------------
// AuthService tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_register_success() {
    let user_repo = MockUserRepo::new();
    let service = AuthService::new(user_repo, MockSessionRepo, MockOtpRepo::new(), MockJobRepo);

    let result = service
        .register("new@test.com", "password123", Some("Alice"))
        .await;
    assert!(
        result.is_ok(),
        "register should succeed: {:?}",
        result.err()
    );
    let auth = result.unwrap();
    assert_eq!(auth.user.email().as_str(), "new@test.com");
    assert_eq!(auth.user.name(), "Alice");
}

#[tokio::test]
async fn auth_register_duplicate_email_fails() {
    let user_repo = MockUserRepo::with_users(vec![make_user("dup@test.com")]);
    let service = AuthService::new(user_repo, MockSessionRepo, MockOtpRepo::new(), MockJobRepo);

    let result = service.register("dup@test.com", "password123", None).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ApplicationError::Conflict(_)));
}

#[tokio::test]
async fn auth_register_invalid_email_fails() {
    let user_repo = MockUserRepo::new();
    let service = AuthService::new(user_repo, MockSessionRepo, MockOtpRepo::new(), MockJobRepo);

    let result = service.register("not-an-email", "password123", None).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ApplicationError::Validation(_)
    ));
}

#[tokio::test]
async fn auth_lookup_session_expired_returns_none() {
    let user_repo = MockUserRepo::new();
    let service = AuthService::new(user_repo, MockSessionRepo, MockOtpRepo::new(), MockJobRepo);

    let result = service.lookup_session("some-token").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn auth_logout_succeeds() {
    let user_repo = MockUserRepo::new();
    let service = AuthService::new(user_repo, MockSessionRepo, MockOtpRepo::new(), MockJobRepo);

    let result = service.logout(SessionId::new()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn auth_get_user_not_found_fails() {
    let user_repo = MockUserRepo::new();
    let service = AuthService::new(user_repo, MockSessionRepo, MockOtpRepo::new(), MockJobRepo);

    let result = service.get_user(UserId::new()).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ApplicationError::NotFound(_)));
}

#[tokio::test]
async fn auth_update_profile_success() {
    let user = make_user("profile@test.com");
    let user_id = user.id();
    let user_repo = MockUserRepo::with_users(vec![user]);
    let service = AuthService::new(user_repo, MockSessionRepo, MockOtpRepo::new(), MockJobRepo);

    let result = service
        .update_profile(user_id, Some("New Name"), Some("https://pic.url"))
        .await;
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.name(), "New Name");
    assert_eq!(updated.picture_url(), Some("https://pic.url"));
}

#[tokio::test]
async fn auth_update_profile_not_found_fails() {
    let user_repo = MockUserRepo::new();
    let service = AuthService::new(user_repo, MockSessionRepo, MockOtpRepo::new(), MockJobRepo);

    let result = service
        .update_profile(UserId::new(), Some("Name"), None)
        .await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ApplicationError::NotFound(_)));
}

// ---------------------------------------------------------------------------
// OTP tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn otp_request_succeeds() {
    let user = make_user("otp@test.com");
    let user_repo = MockUserRepo::with_users(vec![user]);
    let otp_repo = MockOtpRepo::new();
    let service = AuthService::new(user_repo, MockSessionRepo, otp_repo, MockJobRepo);

    let result = service.request_otp("otp@test.com").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn otp_request_user_not_found_fails() {
    let user_repo = MockUserRepo::new();
    let service = AuthService::new(user_repo, MockSessionRepo, MockOtpRepo::new(), MockJobRepo);

    let result = service.request_otp("missing@test.com").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ApplicationError::NotFound(_)));
}

#[tokio::test]
async fn otp_verify_success() {
    let user = make_user("verify@test.com");
    let user_id = user.id();
    let user_repo = MockUserRepo::with_users(vec![user]);

    let otp_repo = MockOtpRepo::new();
    let code = "123456";
    let code_hash = sw_shared::crypto::hash_token(code);
    otp_repo
        .create(
            "verify@test.com",
            &code_hash,
            &(chrono::Utc::now() + chrono::Duration::minutes(10)),
        )
        .await
        .unwrap();

    let service = AuthService::new(user_repo, MockSessionRepo, otp_repo, MockJobRepo);

    let result = service.verify_otp("verify@test.com", code).await;
    assert!(
        result.is_ok(),
        "verify_otp should succeed: {:?}",
        result.err()
    );

    let user = service.get_user(user_id).await.unwrap();
    assert!(user.is_verified());
}

#[tokio::test]
async fn otp_verify_wrong_code_fails() {
    let user = make_user("wrong-otp@test.com");
    let user_repo = MockUserRepo::with_users(vec![user]);

    let otp_repo = MockOtpRepo::new();
    let code_hash = sw_shared::crypto::hash_token("correct-code");
    otp_repo
        .create(
            "wrong-otp@test.com",
            &code_hash,
            &(chrono::Utc::now() + chrono::Duration::minutes(10)),
        )
        .await
        .unwrap();

    let service = AuthService::new(user_repo, MockSessionRepo, otp_repo, MockJobRepo);

    let result = service.verify_otp("wrong-otp@test.com", "wrong-code").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn otp_verify_too_many_attempts_fails() {
    let user = make_user("locked@test.com");
    let user_repo = MockUserRepo::with_users(vec![user]);

    let otp_repo = MockOtpRepo::new();
    let code_hash = sw_shared::crypto::hash_token("secret");
    otp_repo
        .create(
            "locked@test.com",
            &code_hash,
            &(chrono::Utc::now() + chrono::Duration::minutes(10)),
        )
        .await
        .unwrap();

    for _ in 0..5 {
        let _ = otp_repo.increment_attempts("locked@test.com").await;
    }

    let service = AuthService::new(user_repo, MockSessionRepo, otp_repo, MockJobRepo);

    let result = service.verify_otp("locked@test.com", "secret").await;
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// WorkshopService tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn workshop_create_success() {
    let workshop_repo = MockWorkshopRepoEmpty;
    let cat_repo = MockCategoryRepo::new();
    let _ = cat_repo
        .create(&Category::new("Rust".into(), "rust".into(), None, None).0)
        .await;
    let cat_list = CategoryRepository::find_all(&cat_repo).await.unwrap();
    let category_id = cat_list[0].id();

    let service = WorkshopService::new(workshop_repo, cat_repo, MockLevelRepo, MockEventStore);

    let input = CreateWorkshopInput {
        title: "New Workshop".into(),
        slug: "new-workshop".into(),
        price_cents: 5000,
        category_id,
        level_id: LevelId::new(),
        created_by: UserId::new(),
        description: Some("A great workshop".into()),
        location: None,
        start_date: None,
        end_date: None,
        max_seats: Some(20),
        min_age: Some(16),
    };

    let result = service.create(input).await;
    assert!(result.is_ok(), "create workshop: {:?}", result.err());
    let w = result.unwrap();
    assert_eq!(w.title(), "New Workshop");
    assert_eq!(w.price_cents(), 5000);
}

#[tokio::test]
async fn workshop_get_not_found_fails() {
    let workshop_repo = MockWorkshopRepoEmpty;
    let service = WorkshopService::new(
        workshop_repo,
        MockCategoryRepo::new(),
        MockLevelRepo,
        MockEventStore,
    );

    let result = service.get_by_id(WorkshopId::new()).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ApplicationError::NotFound(_)));
}

#[tokio::test]
async fn workshop_update_partial() {
    let (_wid, workshop) = make_workshop();
    let workshop_id = workshop.id();
    let repo = MockWorkshopRepoWithOne::new(workshop);
    let service =
        WorkshopService::new(repo, MockCategoryRepo::new(), MockLevelRepo, MockEventStore);

    let input = UpdateWorkshopInput {
        id: workshop_id,
        title: Some("Updated Title".into()),
        slug: None,
        description: Some("Updated desc".into()),
        location: None,
        price_cents: Some(15000),
        category_id: None,
        level_id: None,
        start_date: None,
        end_date: None,
        max_seats: None,
        min_age: None,
    };

    let result = service.update(input).await;
    assert!(result.is_ok(), "update workshop: {:?}", result.err());
    let w = result.unwrap();
    assert_eq!(w.title(), "Updated Title");
}

#[tokio::test]
async fn workshop_delete_success() {
    let (_wid, workshop) = make_workshop();
    let workshop_id = workshop.id();
    let repo = MockWorkshopRepoWithOne::new(workshop);
    let service =
        WorkshopService::new(repo, MockCategoryRepo::new(), MockLevelRepo, MockEventStore);

    let result = service.delete(workshop_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn workshop_delete_not_found_fails() {
    let service = WorkshopService::new(
        MockWorkshopRepoEmpty,
        MockCategoryRepo::new(),
        MockLevelRepo,
        MockEventStore,
    );

    let result = service.delete(WorkshopId::new()).await;
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// CategoryService tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn category_create_success() {
    let service = CategoryService::new(MockCategoryRepo::new(), MockEventStore);

    let result = service
        .create("Rust".into(), "rust".into(), None, None)
        .await;
    assert!(result.is_ok());
    let cat = result.unwrap();
    assert_eq!(cat.name(), "Rust");
    assert_eq!(cat.slug(), "rust");
}

#[tokio::test]
async fn category_create_duplicate_slug_fails() {
    let repo = MockCategoryRepo::new();
    let service = CategoryService::new(repo, MockEventStore);

    let _ = service
        .create("Rust".into(), "rust".into(), None, None)
        .await
        .unwrap();
    let result = service
        .create("Rust Again".into(), "rust".into(), None, None)
        .await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ApplicationError::Conflict(_)));
}

#[tokio::test]
async fn category_list() {
    let service = CategoryService::new(MockCategoryRepo::new(), MockEventStore);
    let list = service.list().await.unwrap();
    assert!(list.is_empty());
}

// ---------------------------------------------------------------------------
// ReviewService tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn review_create_success() {
    let review_repo = MockReviewRepo::new();
    let (_wid, workshop) = make_workshop();
    let workshop_id = workshop.id();
    let workshop_repo = MockWorkshopRepoWithOne::new(workshop);
    let service = ReviewService::new(
        review_repo,
        MockEnrollmentRepoEmpty,
        workshop_repo,
        MockEventStore,
    );

    let input = CreateReviewInput {
        user_id: UserId::new(),
        workshop_id,
        rating: 5,
        title: "Great!".into(),
        content: "Awesome workshop".into(),
    };

    let result = service.create(input).await;
    assert!(result.is_ok(), "create review: {:?}", result.err());
    let r = result.unwrap();
    assert_eq!(r.rating(), 5);
    assert_eq!(r.title(), "Great!");
}

#[tokio::test]
async fn review_create_invalid_rating_fails() {
    let (_wid, workshop) = make_workshop();
    let workshop_id = workshop.id();
    let review_repo = MockReviewRepo::new();
    let service = ReviewService::new(
        review_repo,
        MockEnrollmentRepoEmpty,
        MockWorkshopRepoWithOne::new(workshop),
        MockEventStore,
    );

    let input = CreateReviewInput {
        user_id: UserId::new(),
        workshop_id,
        rating: 6,
        title: "Bad".into(),
        content: "Too high".into(),
    };

    let result = service.create(input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn review_approve_success() {
    let review_repo = MockReviewRepo::new();
    let (review, _) = Review::new(
        UserId::new(),
        WorkshopId::new(),
        4,
        "Nice".into(),
        "Content".into(),
    )
    .unwrap();
    let review_id = review.id();
    review_repo.reviews.lock().unwrap().push(review);

    let service = ReviewService::new(
        review_repo,
        MockEnrollmentRepoEmpty,
        MockWorkshopRepoEmpty,
        MockEventStore,
    );

    let result = service.approve(review_id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status(), ReviewStatus::Approved);
}

#[tokio::test]
async fn review_reject_success() {
    let review_repo = MockReviewRepo::new();
    let (review, _) = Review::new(
        UserId::new(),
        WorkshopId::new(),
        3,
        "Meh".into(),
        "OK".into(),
    )
    .unwrap();
    let review_id = review.id();
    review_repo.reviews.lock().unwrap().push(review);

    let service = ReviewService::new(
        review_repo,
        MockEnrollmentRepoEmpty,
        MockWorkshopRepoEmpty,
        MockEventStore,
    );

    let result = service.reject(review_id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status(), ReviewStatus::Rejected);
}

#[tokio::test]
async fn review_update_success() {
    let review_repo = MockReviewRepo::new();
    let (review, _) = Review::new(
        UserId::new(),
        WorkshopId::new(),
        3,
        "Orig".into(),
        "Orig".into(),
    )
    .unwrap();
    let review_id = review.id();
    review_repo.reviews.lock().unwrap().push(review);

    let service = ReviewService::new(
        review_repo,
        MockEnrollmentRepoEmpty,
        MockWorkshopRepoEmpty,
        MockEventStore,
    );

    let result = service
        .update(UpdateReviewInput {
            id: review_id,
            rating: Some(5),
            title: Some("Updated".into()),
            content: Some("Updated content".into()),
        })
        .await;
    assert!(result.is_ok());
    let r = result.unwrap();
    assert_eq!(r.rating(), 5);
    assert_eq!(r.title(), "Updated");
}

#[tokio::test]
async fn review_update_after_approve_fails() {
    let review_repo = MockReviewRepo::new();
    let (mut review, _) = Review::new(
        UserId::new(),
        WorkshopId::new(),
        4,
        "Good".into(),
        "Good".into(),
    )
    .unwrap();
    let _ = review.approve();
    let review_id = review.id();
    review_repo.reviews.lock().unwrap().push(review);

    let service = ReviewService::new(
        review_repo,
        MockEnrollmentRepoEmpty,
        MockWorkshopRepoEmpty,
        MockEventStore,
    );

    let result = service
        .update(UpdateReviewInput {
            id: review_id,
            rating: None,
            title: Some("Can't".into()),
            content: None,
        })
        .await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ApplicationError::Validation(_)
    ));
}

#[tokio::test]
async fn review_detail_not_found_fails() {
    let service = ReviewService::new(
        MockReviewRepo::new(),
        MockEnrollmentRepoEmpty,
        MockWorkshopRepoEmpty,
        MockEventStore,
    );
    let result = service.find_by_id(ReviewId::new()).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn review_find_by_workshop() {
    let user_id = UserId::new();
    let workshop_id = WorkshopId::new();
    let (mut r1, _) = Review::new(user_id, workshop_id, 5, "Great".into(), "A+".into()).unwrap();
    let _ = r1.approve();
    let (mut r2, _) =
        Review::new(UserId::new(), workshop_id, 2, "Bad".into(), "Meh".into()).unwrap();
    let _ = r2.approve();

    let review_repo = MockReviewRepo::new();
    review_repo.reviews.lock().unwrap().push(r1);
    review_repo.reviews.lock().unwrap().push(r2);

    let service = ReviewService::new(
        review_repo,
        MockEnrollmentRepoEmpty,
        MockWorkshopRepoEmpty,
        MockEventStore,
    );

    let all = service.find_by_workshop(workshop_id, false).await.unwrap();
    assert_eq!(all.len(), 2);

    let approved = service.find_by_workshop(workshop_id, true).await.unwrap();
    assert_eq!(approved.len(), 2);
}

#[tokio::test]
async fn review_delete_success() {
    let review_repo = MockReviewRepo::new();
    let (review, _) = Review::new(
        UserId::new(),
        WorkshopId::new(),
        4,
        "Title".into(),
        "Content".into(),
    )
    .unwrap();
    let review_id = review.id();
    review_repo.reviews.lock().unwrap().push(review);

    let service = ReviewService::new(
        review_repo,
        MockEnrollmentRepoEmpty,
        MockWorkshopRepoEmpty,
        MockEventStore,
    );
    let result = service.delete(review_id).await;
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// ApplicationError tests
// ---------------------------------------------------------------------------

#[test]
fn app_error_factories() {
    let err = ApplicationError::not_found("User", "id-1");
    assert!(err.to_string().contains("User"));

    let err = ApplicationError::conflict("email taken");
    assert_eq!(err.to_string(), "Conflict: email taken");

    let err = ApplicationError::unauthorized("bad password");
    assert!(err.to_string().contains("bad password"));

    let err = ApplicationError::validation("too long");
    assert!(err.to_string().contains("too long"));

    let err = ApplicationError::internal("db crash");
    assert!(err.to_string().contains("db crash"));

    assert_eq!(
        ApplicationError::RateLimitExceeded.to_string(),
        "Rate limit exceeded"
    );
}

#[test]
fn app_error_equality() {
    let a = ApplicationError::not_found("User", "1");
    let b = ApplicationError::not_found("User", "1");
    assert_eq!(a, b);

    let c = ApplicationError::conflict("dup");
    let d = ApplicationError::conflict("dup");
    assert_eq!(c, d);
}

// ---------------------------------------------------------------------------
// MockWorkshopRepo variants needed above
// ---------------------------------------------------------------------------

struct MockWorkshopRepoEmpty;

#[async_trait::async_trait]
impl WorkshopRepository for MockWorkshopRepoEmpty {
    async fn create(&self, _w: &Workshop) -> Result<(), DomainError> {
        Ok(())
    }
    async fn find_by_id(&self, _id: WorkshopId) -> Result<Option<Workshop>, DomainError> {
        Ok(None)
    }
    async fn find_by_slug(&self, _slug: &str) -> Result<Option<Workshop>, DomainError> {
        Ok(None)
    }
    async fn update(&self, _w: &Workshop) -> Result<(), DomainError> {
        Ok(())
    }
    async fn delete(&self, _id: WorkshopId) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_images(
        &self,
        _id: WorkshopId,
    ) -> Result<Vec<sw_domain::aggregates::workshop::WorkshopImage>, DomainError> {
        Ok(vec![])
    }
    async fn add_image(
        &self,
        _wid: WorkshopId,
        _url: &str,
        _key: &str,
    ) -> Result<sw_domain::aggregates::workshop::WorkshopImage, DomainError> {
        Ok(sw_domain::aggregates::workshop::WorkshopImage::from_parts(
            WorkshopImageId::new(),
            _wid,
            _url.to_string(),
            _key.to_string(),
            chrono::Utc::now(),
        ))
    }
    async fn remove_image(&self, _iid: WorkshopImageId) -> Result<(), DomainError> {
        Ok(())
    }
    async fn find_all(&self) -> Result<Vec<Workshop>, DomainError> {
        Ok(vec![])
    }
    async fn reserve_seat_atomic(&self, _wid: WorkshopId) -> Result<Option<Workshop>, DomainError> {
        Ok(None)
    }
    async fn release_seat_atomic(&self, _wid: WorkshopId) -> Result<(), DomainError> {
        Ok(())
    }
}

struct MockWorkshopRepoWithOne {
    workshop: Mutex<Option<Workshop>>,
}

impl MockWorkshopRepoWithOne {
    fn new(w: Workshop) -> Self {
        Self {
            workshop: Mutex::new(Some(w)),
        }
    }
}

#[async_trait::async_trait]
impl WorkshopRepository for MockWorkshopRepoWithOne {
    async fn create(&self, _w: &Workshop) -> Result<(), DomainError> {
        Ok(())
    }
    async fn find_by_id(&self, id: WorkshopId) -> Result<Option<Workshop>, DomainError> {
        Ok(self.workshop.lock().unwrap().clone().filter(|w| w.id() == id))
    }
    async fn find_by_slug(&self, _slug: &str) -> Result<Option<Workshop>, DomainError> {
        Ok(self.workshop.lock().unwrap().clone())
    }
    async fn update(&self, _w: &Workshop) -> Result<(), DomainError> {
        Ok(())
    }
    async fn delete(&self, _id: WorkshopId) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_images(
        &self,
        _id: WorkshopId,
    ) -> Result<Vec<sw_domain::aggregates::workshop::WorkshopImage>, DomainError> {
        Ok(vec![])
    }
    async fn add_image(
        &self,
        _wid: WorkshopId,
        _url: &str,
        _key: &str,
    ) -> Result<sw_domain::aggregates::workshop::WorkshopImage, DomainError> {
        Ok(sw_domain::aggregates::workshop::WorkshopImage::from_parts(
            WorkshopImageId::new(),
            _wid,
            _url.to_string(),
            _key.to_string(),
            chrono::Utc::now(),
        ))
    }
    async fn remove_image(&self, _iid: WorkshopImageId) -> Result<(), DomainError> {
        Ok(())
    }
    async fn find_all(&self) -> Result<Vec<Workshop>, DomainError> {
        Ok(vec![])
    }
    async fn reserve_seat_atomic(&self, _wid: WorkshopId) -> Result<Option<Workshop>, DomainError> {
        Ok(self.workshop.lock().unwrap().clone())
    }
    async fn release_seat_atomic(&self, _wid: WorkshopId) -> Result<(), DomainError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// MockContactRepo
// ---------------------------------------------------------------------------

struct MockContactRepo {
    contacts: Mutex<Vec<Contact>>,
}

impl MockContactRepo {
    fn new() -> Self {
        Self {
            contacts: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl ContactRepository for MockContactRepo {
    async fn create(&self, contact: &Contact) -> Result<(), DomainError> {
        self.contacts.lock().unwrap().push(contact.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: ContactId) -> Result<Option<Contact>, DomainError> {
        Ok(self
            .contacts
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.id() == id)
            .cloned())
    }

    async fn list(&self, _is_read: Option<bool>) -> Result<Vec<Contact>, DomainError> {
        Ok(self.contacts.lock().unwrap().clone())
    }

    async fn update(&self, _contact: &Contact) -> Result<(), DomainError> {
        Ok(())
    }

    async fn delete(&self, _id: ContactId) -> Result<(), DomainError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// MockStatsRepo
// ---------------------------------------------------------------------------

struct MockStatsRepo;

#[async_trait::async_trait]
impl StatsRepository for MockStatsRepo {
    async fn platform_stats(&self) -> Result<PlatformStats, DomainError> {
        Ok(PlatformStats {
            total_users: 42,
            total_workshops: 10,
            total_enrollments: 100,
            total_reviews: 25,
        })
    }

    async fn workshop_ratings(
        &self,
    ) -> Result<Vec<sw_domain::repositories::stats::WorkshopRating>, DomainError> {
        Ok(vec![sw_domain::repositories::stats::WorkshopRating {
            workshop_id: WorkshopId::new(),
            average_rating: 4.5,
            review_count: 10,
        }])
    }
}

// ---------------------------------------------------------------------------
// ContactService tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contact_submit_success() {
    let repo = MockContactRepo::new();
    let service = ContactService::new(repo, MockEventStore);

    let result = service
        .submit(SubmitContactInput {
            name: "Alice".into(),
            email: "alice@test.com".into(),
            subject: "Question".into(),
            message: "Great workshop!".into(),
        })
        .await;
    assert!(result.is_ok(), "submit contact: {:?}", result.err());
    let c = result.unwrap();
    assert_eq!(c.name(), "Alice");
    assert_eq!(c.subject(), "Question");
}

#[tokio::test]
async fn contact_submit_invalid_email_fails() {
    let repo = MockContactRepo::new();
    let service = ContactService::new(repo, MockEventStore);

    let result = service
        .submit(SubmitContactInput {
            name: "Bob".into(),
            email: "not-email".into(),
            subject: "Hi".into(),
            message: "Test".into(),
        })
        .await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ApplicationError::Validation(_)
    ));
}

#[tokio::test]
async fn contact_find_by_id_success() {
    let repo = MockContactRepo::new();
    let email = Email::new("find@test.com").unwrap();
    let (contact, _) = Contact::new("Alice".into(), email, "Sub".into(), "Msg".into()).unwrap();
    let id = contact.id();
    repo.contacts.lock().unwrap().push(contact);

    let service = ContactService::new(repo, MockEventStore);
    let result = service.find_by_id(id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "Alice");
}

#[tokio::test]
async fn contact_find_by_id_not_found() {
    let service = ContactService::new(MockContactRepo::new(), MockEventStore);
    let result = service.find_by_id(ContactId::new()).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ApplicationError::NotFound(_)));
}

#[tokio::test]
async fn contact_list() {
    let repo = MockContactRepo::new();
    let email = Email::new("a@b.com").unwrap();
    repo.contacts.lock().unwrap().push(
        Contact::new("A".into(), email, "S".into(), "M".into())
            .unwrap()
            .0,
    );
    let service = ContactService::new(repo, MockEventStore);
    let result = service.list(None).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
}

#[tokio::test]
async fn contact_mark_read() {
    let repo = MockContactRepo::new();
    let email = Email::new("read@test.com").unwrap();
    let (contact, _) = Contact::new("Carol".into(), email, "Sub".into(), "Msg".into()).unwrap();
    let id = contact.id();
    repo.contacts.lock().unwrap().push(contact);

    let service = ContactService::new(repo, MockEventStore);
    let result = service.mark_read(id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_read());
}

#[tokio::test]
async fn contact_delete_success() {
    let repo = MockContactRepo::new();
    let email = Email::new("del@test.com").unwrap();
    let (contact, _) = Contact::new("Del".into(), email, "Sub".into(), "Msg".into()).unwrap();
    let id = contact.id();
    repo.contacts.lock().unwrap().push(contact);

    let service = ContactService::new(repo, MockEventStore);
    let result = service.delete(id).await;
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// UserAdminService tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn user_admin_list() {
    let repo = MockUserRepo::with_users(vec![make_user("a@test.com"), make_user("b@test.com")]);
    let service = UserAdminService::new(repo, MockEventStore);
    let result = service.list().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

#[tokio::test]
async fn user_admin_get_by_id_success() {
    let user = make_user("get@test.com");
    let id = user.id();
    let repo = MockUserRepo::with_users(vec![user]);
    let service = UserAdminService::new(repo, MockEventStore);
    let result = service.get_by_id(id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().email().as_str(), "get@test.com");
}

#[tokio::test]
async fn user_admin_get_by_id_not_found() {
    let service = UserAdminService::new(MockUserRepo::new(), MockEventStore);
    let result = service.get_by_id(UserId::new()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn user_admin_update() {
    let user = make_user("admin-update@test.com");
    let id = user.id();
    let repo = MockUserRepo::with_users(vec![user]);
    let service = UserAdminService::new(repo, MockEventStore);
    let result = service
        .update(UpdateUserInput {
            user_id: id,
            name: Some("Admin Updated".into()),
            role: Some("admin".into()),
            status: Some("active".into()),
            phone: Some("+123".into()),
            age: Some(30),
            address: Some("123 Street".into()),
            expertise: Some("Rust".into()),
            bio: Some("Bio here".into()),
        })
        .await;
    assert!(result.is_ok(), "admin update: {:?}", result.err());
    assert_eq!(result.unwrap().name(), "Admin Updated");
}

#[tokio::test]
async fn user_admin_delete_success() {
    let user = make_user("admin-del@test.com");
    let id = user.id();
    let repo = MockUserRepo::with_users(vec![user]);
    let service = UserAdminService::new(repo, MockEventStore);
    let result = service.delete(id).await;
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// LevelService tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn level_create_success() {
    let service = LevelService::new(MockLevelRepo, MockEventStore);
    let result = service.create("Advanced".into()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "Advanced");
}

#[tokio::test]
async fn level_get_by_id_success() {
    let service = LevelService::new(MockLevelRepo, MockEventStore);
    let result = service.get_by_id(LevelId::new()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name(), "Beginner");
}

#[tokio::test]
async fn level_list() {
    let service = LevelService::new(MockLevelRepo, MockEventStore);
    let result = service.list().await;
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// StatsService tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stats_platform_stats() {
    let service = StatsService::new(MockStatsRepo);
    let result = service.platform_stats().await;
    assert!(result.is_ok());
    let stats = result.unwrap();
    assert_eq!(stats.total_users, 42);
    assert_eq!(stats.total_workshops, 10);
}

#[tokio::test]
async fn stats_workshop_ratings() {
    let service = StatsService::new(MockStatsRepo);
    let result = service.workshop_ratings().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
}
