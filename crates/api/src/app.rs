use crate::middleware::request_id::set_request_id;
use crate::routes;
use axum::http::Method;
use axum::{middleware, Router};
use tower_http::cors::{Any, CorsLayer};

/// Build the full Axum application router with middleware.
pub fn build_app() -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v1", routes::health::router())
        .layer(middleware::from_fn(set_request_id))
        .layer(cors)
}
