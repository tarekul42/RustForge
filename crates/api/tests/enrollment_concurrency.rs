#![allow(dead_code, unused_imports, unused_variables)]
//! Concurrency test: 10 parallel enrollments for a 5-seat workshop.
//!
//! Requires a running Postgres database with migrations applied.
//! Set `DATABASE_URL` env var or run via:
//!   DATABASE_URL=postgres://user:pass@localhost:5432/test cargo test --test enrollment_concurrency -- --ignored

use axum::http::StatusCode;
use sw_api::app::build_app;
use sw_api::state::AppState;
use sw_shared::config::Config;

fn get_db_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/sw_workshop_test".to_string()
    })
}

async fn setup_app() -> AppState {
    let config = Config {
        database: sw_shared::config::DatabaseConfig {
            url: get_db_url(),
            max_connections: 20,
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

    AppState::new(config, pool).await
}

#[tokio::test]
#[ignore]
async fn ten_parallel_enrollments_on_five_seat_workshop() {
    let _state = setup_app().await;

    // TODO: Create a test user, admin session, workshop with 5 max_seats.
    // Then spawn 10 concurrent enrollment requests.
    // Assert exactly 5 succeed with 200, 5 fail with Unavailable.
}
