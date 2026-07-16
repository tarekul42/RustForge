use crate::api_doc::ApiDoc;
use crate::middleware::origin_check::origin_check_mw;
use crate::middleware::rate_limit::rate_limiter_layer;
use crate::middleware::request_id::set_request_id;
use crate::routes;
use crate::state::AppState;
use axum::http::{HeaderValue, Method};
use axum::{Json, Router, middleware, response::Html};
use std::sync::Arc;
use std::sync::OnceLock;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;

static OPENAPI_SPEC: OnceLock<String> = OnceLock::new();

fn openapi_spec() -> &'static str {
    OPENAPI_SPEC.get_or_init(|| ApiDoc::openapi().to_json().expect("serialize OpenAPI spec"))
}

async fn serve_openapi_json() -> Json<serde_json::Value> {
    let spec = openapi_spec();
    Json(serde_json::from_str(spec).expect("valid JSON"))
}

async fn serve_swagger_ui() -> Html<String> {
    let spec = openapi_spec();
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>Skill Workshop API — Swagger UI</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    SwaggerUIBundle({{
      spec: {spec},
      dom_id: '#swagger-ui',
      presets: [SwaggerUIBundle.presets.apis],
    }});
  </script>
</body>
</html>"#,
    ))
}

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
        .route("/openapi.json", axum::routing::get(serve_openapi_json))
        .route("/docs", axum::routing::get(serve_swagger_ui))
        .nest(
            "/api/v1/health",
            routes::health::liveness_router().with_state(()),
        )
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
        .layer(middleware::from_fn_with_state(
            state.clone(),
            origin_check_mw,
        ))
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
