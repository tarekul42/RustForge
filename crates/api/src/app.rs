use crate::middleware::origin_check::origin_check_mw;
use crate::middleware::rate_limit::rate_limit_mw;
use crate::middleware::rate_limit::TokenBucket;
use crate::middleware::request_id::set_request_id;
use crate::routes;
use crate::state::AppState;
use axum::http::Method;
use axum::{middleware, Router};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

/// Build the full Axum application router with middleware and shared state.
///
/// Pass `Some(state)` to enable stateful endpoints (auth, users, categories, workshops).
/// Pass `None` to build a minimal router for testing health endpoints.
pub fn build_app(state: Option<Arc<AppState>>) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    let rate_limiter = TokenBucket::new(120, 60);

    let mut router = Router::new().nest("/api/v1/health", routes::health::liveness_router());

    if let Some(state) = state {
        router = router
            .nest("/api/v1/health", routes::health::readiness_router())
            .nest("/api/v1/auth", routes::auth::router())
            .nest("/api/v1/users", routes::user::router())
            .nest("/api/v1/categories", routes::category::router())
            .nest("/api/v1/contacts", routes::contact::router())
            .nest("/api/v1/enrollments", routes::enrollment::router())
            .nest("/api/v1/payments", routes::payment::router())
            .nest("/api/v1/reviews", routes::review::router())
            .nest("/api/v1/stats", routes::stats::router())
            .nest("/api/v1/workshops", routes::workshop::router())
            .nest(
                "/api/v1/workshops/levels",
                routes::workshop::levels::router(),
            )
            .layer(middleware::from_fn(origin_check_mw))
            .layer(axum::Extension(state));
    }

    router
        .layer(middleware::from_fn(set_request_id))
        .layer(middleware::from_fn_with_state(rate_limiter, rate_limit_mw))
        .layer(cors)
}
