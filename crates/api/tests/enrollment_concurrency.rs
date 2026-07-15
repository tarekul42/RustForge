//! Concurrency tests for enrollment seat reservation.
//!
//! Tests the core invariant: no overselling of workshop seats under concurrency.
//!
//! Requires a running Postgres database with migrations applied.
//! Set `DATABASE_URL` env var or run via:
//!   DATABASE_URL=postgres://user:pass@localhost:5432/test cargo test --test enrollment_concurrency -- --ignored

use sw_domain::aggregates::workshop::Workshop;
use sw_domain::repositories::workshop::WorkshopRepository;
use sw_domain::value_objects::ids::{CategoryId, LevelId, UserId, WorkshopId};
use sw_infrastructure::postgres::repos::workshop::PostgresWorkshopRepository;
use sw_shared::config::Config;

fn get_db_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/sw_workshop_test".to_string()
    })
}

async fn setup_pool() -> sqlx::PgPool {
    let config = Config {
        database: sw_shared::config::DatabaseConfig {
            url: get_db_url(),
            max_connections: 25,
            ..Default::default()
        },
        ..Default::default()
    };
    let pool = sqlx::PgPool::connect(&config.database.url)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Seed a user, category, and level; return a workshop with the given max seats.
async fn create_test_workshop(
    pool: &sqlx::PgPool,
    max_seats: Option<i32>,
) -> WorkshopId {
    let user_id = UserId::new();
    let category_id = CategoryId::new();
    let level_id = LevelId::new();

    // Insert a minimal admin user
    sqlx::query(
        r#"INSERT INTO users (id, email, name, password_hash, role, status, is_verified, created_at, updated_at)
           VALUES ($1, $2, $3, $4, 'admin', 'active', true, NOW(), NOW())"#,
    )
    .bind(user_id.into_uuid())
    .bind("concurrency-test-admin@example.com")
    .bind("Admin User")
    .bind::<Option<String>>(None)
    .execute(pool)
    .await
    .expect("Failed to insert test user");

    // Insert a category
    sqlx::query(
        r#"INSERT INTO categories (id, name, slug, description, created_at, updated_at)
           VALUES ($1, $2, $3, $4, NOW(), NOW())
           ON CONFLICT (slug) DO NOTHING"#,
    )
    .bind(category_id.into_uuid())
    .bind("Concurrency Test Category")
    .bind("concurrency-test-category")
    .bind::<Option<String>>(None)
    .execute(pool)
    .await
    .expect("Failed to insert test category");

    // Insert a level
    sqlx::query(
        r#"INSERT INTO levels (id, name, created_at, updated_at)
           VALUES ($1, $2, NOW(), NOW())
           ON CONFLICT (name) DO NOTHING"#,
    )
    .bind(level_id.into_uuid())
    .bind("Concurrency Test Level")
    .execute(pool)
    .await
    .expect("Failed to insert test level");

    // Create the workshop
    let (workshop, _event) = Workshop::new(
        "Concurrency Test Workshop".to_string(),
        "concurrency-test-workshop".to_string(),
        10000,
        category_id,
        level_id,
        user_id,
    );

    // Override max_seats after construction
    let workshop = Workshop {
        max_seats,
        ..workshop
    };

    let id = workshop.id;
    let repo = PostgresWorkshopRepository::new(pool.clone());
    repo.create(&workshop)
        .await
        .expect("Failed to create workshop");
    id
}

// ---------------------------------------------------------------------------
// Test 1: Repository-level atomic seat reservation
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn ten_parallel_seat_reservations_on_five_seat_workshop() {
    let pool = setup_pool().await;
    let repo = PostgresWorkshopRepository::new(pool.clone());

    let workshop_id = create_test_workshop(&pool, Some(5)).await;

    let mut handles = Vec::new();
    for _ in 0..10 {
        let repo = PostgresWorkshopRepository::new(pool.clone());
        handles.push(tokio::spawn(async move {
            repo.reserve_seat_atomic(workshop_id).await
        }));
    }

    let mut success_count = 0;
    let mut none_count = 0;
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        match result {
            Ok(Some(_)) => success_count += 1,
            Ok(None) => none_count += 1,
            Err(e) => panic!("Seat reservation failed with error: {e}"),
        }
    }

    let final_workshop = repo
        .find_by_id(workshop_id)
        .await
        .expect("Failed to fetch workshop")
        .expect("Workshop not found");

    assert_eq!(
        success_count, 5,
        "Exactly 5 reservations should succeed (5 seats, 10 attempts)"
    );
    assert_eq!(
        none_count, 5,
        "Exactly 5 reservations should fail (no seats left)"
    );
    assert_eq!(
        final_workshop.current_enrollments, 5,
        "Workshop should have exactly 5 enrollments after 10 concurrent attempts"
    );
}

// ---------------------------------------------------------------------------
// Test 2: Repository-level atomic seat release
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn ten_parallel_seat_releases_never_negative() {
    let pool = setup_pool().await;
    let repo = PostgresWorkshopRepository::new(pool.clone());

    let workshop_id = create_test_workshop(&pool, None).await;

    // First reserve one seat
    repo.reserve_seat_atomic(workshop_id)
        .await
        .expect("reserve failed")
        .expect("no seat available");

    // Fire 10 concurrent releases — the seat count should never go below 0
    let mut handles = Vec::new();
    for _ in 0..10 {
        let repo = PostgresWorkshopRepository::new(pool.clone());
        handles.push(tokio::spawn(async move {
            repo.release_seat_atomic(workshop_id).await
        }));
    }

    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(
            result.is_ok(),
            "Seat release should always succeed (floors at 0)"
        );
    }

    let final_workshop = repo
        .find_by_id(workshop_id)
        .await
        .expect("Failed to fetch workshop")
        .expect("Workshop not found");

    assert_eq!(
        final_workshop.current_enrollments, 0,
        "Seat count should floor at 0, even with more releases than reservations"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Unlimited workshop always succeeds
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn concurrent_reservations_on_unlimited_workshop_all_succeed() {
    let pool = setup_pool().await;
    let workshop_id = create_test_workshop(&pool, None).await;

    let mut handles = Vec::new();
    for _ in 0..20 {
        let repo = PostgresWorkshopRepository::new(pool.clone());
        handles.push(tokio::spawn(async move {
            repo.reserve_seat_atomic(workshop_id).await
        }));
    }

    let mut success_count = 0;
    for h in handles {
        if h.await.expect("Task panicked").ok().flatten().is_some() {
            success_count += 1;
        }
    }

    assert_eq!(
        success_count, 20,
        "All 20 concurrent reservations should succeed on unlimited workshop"
    );
}
