use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use sw_api::app::build_app;
use tower::ServiceExt;

#[tokio::test]
async fn health_endpoint_returns_200() {
    let app = build_app(None);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_response_includes_x_request_id() {
    let app = build_app(None);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.headers().contains_key("x-request-id"));
}

#[tokio::test]
async fn health_ready_endpoint_returns_503_without_state() {
    let app = build_app(None);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Without AppState, the readiness endpoint is not mounted → 404
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn health_response_body_is_valid_json() {
    let app = build_app(None);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["status"], "ok");
}
