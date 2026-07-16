//! Integration tests against a real Postgres instance spun up by testcontainers.
//!
//! All tests share a single Postgres container but each gets a fresh PgPool.
//! Tests are serialized by a mutex.
//!
//! Requires Docker. Run with: `cargo test --test postgres_integration -- --test-threads=1`

use std::sync::LazyLock;
use tokio::sync::Mutex;

use sqlx::PgPool;
use sw_domain::aggregates::category::Category;
use sw_domain::aggregates::enrollment::Enrollment;
use sw_domain::aggregates::level::Level;
use sw_domain::aggregates::payment::{Payment, PaymentStatus};
use sw_domain::aggregates::user::{User, UserRole, UserStatus};
use sw_domain::aggregates::workshop::Workshop;
use sw_domain::events::{AuditContext, DomainEvent, EventStore};
use sw_domain::repositories::category::CategoryRepository;
use sw_domain::repositories::enrollment::EnrollmentRepository;
use sw_domain::repositories::level::LevelRepository;
use sw_domain::repositories::payment::PaymentRepository;
use sw_domain::repositories::user::UserRepository;
use sw_domain::repositories::workshop::WorkshopRepository;
use sw_domain::value_objects::Email;
use sw_domain::value_objects::ids::*;
use sw_domain::value_objects::money::Money;
use sw_infrastructure::postgres::repos;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

static DB_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

struct ContainerContext {
    _container: testcontainers::ContainerAsync<Postgres>,
    url: String,
}

fn init_ctx() -> ContainerContext {
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let pg = Postgres::default().with_host_auth();
        let node = pg
            .start()
            .await
            .expect("Failed to start Postgres container (is Docker running?)");
        let port = node.get_host_port_ipv4(5432).await.expect("host port");
        let host = node.get_host().await.expect("host");
        let url = format!("postgres://postgres@{host}:{port}/postgres");
        eprintln!("Postgres container ready at {url}");

        let pool = PgPool::connect(&url).await.expect("connect");
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .expect("migrate");
        pool.close().await;

        ContainerContext {
            _container: node,
            url,
        }
    })
}

static CTX: LazyLock<ContainerContext> = LazyLock::new(|| {
    std::thread::spawn(init_ctx)
        .join()
        .expect("failed to initialize container")
});

async fn with_pool<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let _guard = DB_LOCK.lock().await;
    // CTX is already initialized by now; just deref to ensure container is alive.
    let url = &CTX.url;
    let pool = PgPool::connect(url).await.expect("connect pool");

    sqlx::query(
        "TRUNCATE TABLE audit_logs, jobs, refund_logs, payments, enrollments, \
         workshop_images, workshops, reviews, sessions, otp_codes, contacts, users, \
         categories, levels RESTART IDENTITY CASCADE",
    )
    .execute(&pool)
    .await
    .expect("truncate");

    let result = f.await;
    pool.close().await;
    result
}

fn make_user(email: &str) -> User {
    User::from_parts(
        UserId::new(),
        Email::new(email).unwrap(),
        "Test User".into(),
        Some("hash".into()),
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

fn make_workshop(
    slug: &str,
    category_id: CategoryId,
    level_id: LevelId,
    created_by: UserId,
) -> Workshop {
    let (w, _) = Workshop::new(
        format!("Workshop {slug}"),
        slug.into(),
        5000,
        category_id,
        level_id,
        created_by,
    );
    w
}

fn make_level() -> Level {
    Level::new("Beginner".into()).0
}

// ---------------------------------------------------------------------------
// UserRepository integration tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pg_user_create_and_find() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let repo = repos::user::PostgresUserRepository::new(pool);
        let user = make_user("create-find@test.com");
        repo.create(&user).await.expect("insert user");
        let found = repo.find_by_id(user.id()).await.expect("find");
        assert!(found.is_some());
        assert_eq!(found.unwrap().email().as_str(), "create-find@test.com");
    })
    .await
}

#[tokio::test]
async fn pg_user_find_by_email() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let repo = repos::user::PostgresUserRepository::new(pool);
        let user = make_user("find-email@test.com");
        repo.create(&user).await.expect("insert");
        let found = repo
            .find_by_email("find-email@test.com")
            .await
            .expect("find");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), user.id());
    })
    .await
}

#[tokio::test]
async fn pg_user_find_by_email_not_found() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let repo = repos::user::PostgresUserRepository::new(pool);
        let result = repo.find_by_email("missing@test.com").await.expect("find");
        assert!(result.is_none());
    })
    .await
}

#[tokio::test]
async fn pg_user_update() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let repo = repos::user::PostgresUserRepository::new(pool);
        let mut user = make_user("update@test.com");
        repo.create(&user).await.expect("insert");
        user.set_name("Updated".into());
        repo.update(&user).await.expect("update");
        let found = repo.find_by_id(user.id()).await.expect("find");
        assert_eq!(found.unwrap().name(), "Updated");
    })
    .await
}

#[tokio::test]
async fn pg_user_find_all() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let repo = repos::user::PostgresUserRepository::new(pool);
        repo.create(&make_user("a@test.com")).await.unwrap();
        repo.create(&make_user("b@test.com")).await.unwrap();
        assert_eq!(repo.find_all().await.expect("find all").len(), 2);
    })
    .await
}

#[tokio::test]
async fn pg_user_delete() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let repo = repos::user::PostgresUserRepository::new(pool);
        let user = make_user("delete@test.com");
        repo.create(&user).await.expect("insert");
        repo.delete(user.id()).await.expect("delete");
        assert!(repo.find_by_id(user.id()).await.expect("find").is_none());
    })
    .await
}

// ---------------------------------------------------------------------------
// CategoryRepository integration tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pg_category_create_and_find() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let repo = repos::category::PostgresCategoryRepository::new(pool);
        let (cat, _) = Category::new("Rust".into(), "rust".into(), None, None);
        repo.create(&cat).await.expect("create");
        assert_eq!(
            repo.find_by_id(cat.id())
                .await
                .expect("find")
                .unwrap()
                .name(),
            "Rust"
        );
    })
    .await
}

#[tokio::test]
async fn pg_category_find_by_slug() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let repo = repos::category::PostgresCategoryRepository::new(pool);
        let (cat, _) = Category::new("Go".into(), "go".into(), None, None);
        repo.create(&cat).await.expect("create");
        assert!(repo.find_by_slug("go").await.expect("find").is_some());
    })
    .await
}

#[tokio::test]
async fn pg_category_list() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let repo = repos::category::PostgresCategoryRepository::new(pool);
        repo.create(&Category::new("A".into(), "a".into(), None, None).0)
            .await
            .unwrap();
        repo.create(&Category::new("B".into(), "b".into(), None, None).0)
            .await
            .unwrap();
        assert_eq!(repo.find_all().await.expect("list").len(), 2);
    })
    .await
}

// ---------------------------------------------------------------------------
// WorkshopRepository integration tests
// ---------------------------------------------------------------------------

async fn setup_workshop_deps(pool: &PgPool) -> (CategoryId, LevelId, UserId) {
    let cat_repo = repos::category::PostgresCategoryRepository::new(pool.clone());
    let level_repo = repos::level::PostgresLevelRepository::new(pool.clone());
    let user_repo = repos::user::PostgresUserRepository::new(pool.clone());
    let (cat, _) = Category::new("C".into(), "cw".into(), None, None);
    cat_repo.create(&cat).await.unwrap();
    let level = make_level();
    level_repo.create(&level).await.unwrap();
    let user = make_user("ws-owner@test.com");
    user_repo.create(&user).await.unwrap();
    (cat.id(), level.id(), user.id())
}

#[tokio::test]
async fn pg_workshop_create_and_find() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let (cat_id, level_id, user_id) = setup_workshop_deps(&pool).await;
        let repo = repos::workshop::PostgresWorkshopRepository::new(pool);
        let w = make_workshop("ws-1", cat_id, level_id, user_id);
        repo.create(&w).await.expect("create");
        assert_eq!(
            repo.find_by_id(w.id()).await.expect("find").unwrap().slug(),
            "ws-1"
        );
    })
    .await
}

#[tokio::test]
async fn pg_workshop_find_by_slug() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let (cat_id, level_id, user_id) = setup_workshop_deps(&pool).await;
        let repo = repos::workshop::PostgresWorkshopRepository::new(pool);
        let w = make_workshop("slug-test", cat_id, level_id, user_id);
        repo.create(&w).await.unwrap();
        assert!(
            repo.find_by_slug("slug-test")
                .await
                .expect("find")
                .is_some()
        );
    })
    .await
}

#[tokio::test]
async fn pg_workshop_seat_reserve_and_release() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let (cat_id, level_id, user_id) = setup_workshop_deps(&pool).await;
        let repo = repos::workshop::PostgresWorkshopRepository::new(pool);
        let mut w = make_workshop("seat", cat_id, level_id, user_id);
        w.set_max_seats(Some(1));
        repo.create(&w).await.expect("create");
        assert!(
            repo.reserve_seat_atomic(w.id())
                .await
                .expect("reserve1")
                .is_some()
        );
        assert!(
            repo.reserve_seat_atomic(w.id())
                .await
                .expect("reserve2")
                .is_none()
        );
        repo.release_seat_atomic(w.id()).await.expect("release");
        assert!(
            repo.reserve_seat_atomic(w.id())
                .await
                .expect("reserve3")
                .is_some()
        );
    })
    .await
}

// ---------------------------------------------------------------------------
// Enrollment + Payment
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pg_enrollment_and_payment_flow() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let user_repo = repos::user::PostgresUserRepository::new(pool.clone());
        let cat_repo = repos::category::PostgresCategoryRepository::new(pool.clone());
        let level_repo = repos::level::PostgresLevelRepository::new(pool.clone());
        let ws_repo = repos::workshop::PostgresWorkshopRepository::new(pool.clone());
        let enroll_repo = repos::enrollment::PostgresEnrollmentRepository::new(pool.clone());
        let pay_repo = repos::payment::PostgresPaymentRepository::new(pool.clone());

        let user = make_user("ep@test.com");
        user_repo.create(&user).await.unwrap();
        let (cat, _) = Category::new("C".into(), "c4".into(), None, None);
        cat_repo.create(&cat).await.unwrap();
        let level = make_level();
        level_repo.create(&level).await.unwrap();
        let mut w = make_workshop("ep", cat.id(), level.id(), user.id());
        w.set_max_seats(Some(5));
        ws_repo.create(&w).await.unwrap();

        let (mut enrollment, _) = Enrollment::new(user.id(), w.id(), 1);
        enroll_repo
            .create(&enrollment)
            .await
            .expect("create enrollment");
        let (payment, _) = Payment::new(enrollment.id(), "TXN-EP".into(), Money::from_cents(5000));
        pay_repo.create(&payment).await.expect("create payment");
        enrollment.set_payment_id(Some(payment.id()));
        enroll_repo.update(&enrollment).await.unwrap();

        assert_eq!(
            enroll_repo
                .find_by_id(enrollment.id())
                .await
                .unwrap()
                .unwrap()
                .payment_id(),
            Some(payment.id())
        );
        assert_eq!(
            pay_repo
                .find_by_transaction_id("TXN-EP")
                .await
                .unwrap()
                .unwrap()
                .status(),
            PaymentStatus::Unpaid
        );
    })
    .await
}

// ---------------------------------------------------------------------------
// EventStore
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pg_event_store_publish() {
    with_pool(async {
        let pool = PgPool::connect(&CTX.url).await.unwrap();
        let user_repo = repos::user::PostgresUserRepository::new(pool.clone());
        let actor = make_user("actor@test.com");
        user_repo.create(&actor).await.unwrap();
        let store = repos::event_store::PostgresEventStore::new(pool);
        let event = DomainEvent::UserRegistered {
            user_id: UserId::new(),
            email: Email::new("evt@test.com").unwrap(),
        };
        let ctx = AuditContext {
            actor_id: Some(actor.id()),
            ip_address: Some("127.0.0.1".into()),
            user_agent: Some("test".into()),
        };
        store.publish(&event, Some(&ctx)).await.expect("publish");
    })
    .await
}
