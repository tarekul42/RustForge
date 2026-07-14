use crate::middleware::origin_check::origin_check_mw;
use crate::middleware::rate_limit::rate_limiter_layer;
use crate::middleware::request_id::set_request_id;
use crate::routes;
use crate::state::AppState;
use axum::http::{HeaderValue, Method};
use axum::{middleware, Router};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

/// Build the full Axum application router with middleware and shared state.
pub fn build_app(state: Arc<AppState>) -> Router {
    let cors = {
        let mut cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
            .allow_headers(Any);

        if let Some(ref origins) = state.config.allowed_origins {
            if !origins.is_empty() {
                let allow_origins: Vec<HeaderValue> = origins
                    .iter()
                    .filter_map(|o| HeaderValue::from_str(o).ok())
                    .collect();
                cors = cors.allow_origin(allow_origins);
            }
        }

        cors
    };

    Router::<Arc<AppState>>::new()
        .nest("/api/v1/health", routes::health::liveness_router().with_state(()))
        .nest("/metrics", routes::metrics::router())
        .nest("/api/v1/health", routes::health::readiness_router())
        .nest("/api/v1/health", routes::health::dashboard_router())
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
        .layer(middleware::from_fn_with_state(state.clone(), origin_check_mw))
        .layer(rate_limiter_layer())
        .with_state(state)
        .layer(middleware::from_fn(set_request_id))
        .layer(cors)
}

/// Build a minimal router with only the liveness health endpoint (no DB state needed).
pub fn build_liveness_router() -> Router<()> {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v1/health", routes::health::liveness_router())
        .layer(middleware::from_fn(set_request_id))
        .layer(cors)
}
