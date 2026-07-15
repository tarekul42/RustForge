use axum::{
    Json, Router,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::extractors::session;
use crate::state::AppState;

/// Wrap a JSON response with a `Set-Cookie` header for the session token.
fn with_session_cookie<T: Serialize>(
    data: T,
    token: &str,
    expires_at: &chrono::DateTime<chrono::Utc>,
) -> Result<Response, ApiError> {
    let max_age = (*expires_at - chrono::Utc::now()).num_seconds().max(0);
    let cookie = format!(
        "session={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={}",
        token, max_age,
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        "Set-Cookie",
        cookie.parse().map_err(|_| ApiError::Internal)?,
    );
    Ok((headers, Json(data)).into_response())
}

/// Build the auth router — all paths are relative to `/api/v1/auth`.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/request-otp", post(request_otp))
        .route("/verify-otp", post(verify_otp))
        .route("/session", get(session_handler))
        .route("/logout", post(logout))
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

/// Request body for `POST /auth/register`.
#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"email": "user@example.com", "password": "secret123", "display_name": "Alice"}))]
pub struct RegisterRequest {
    /// User email address.
    pub email: String,
    /// Plaintext password.
    pub password: String,
    /// Optional display name.
    pub display_name: Option<String>,
}

/// Response body for `POST /auth/register` and `POST /auth/login`.
#[derive(Serialize, ToSchema)]
#[schema(example = json!({"user_id": "123e4567-e89b-12d3-a456-426614174000", "session_token": "tok_abc123", "session_expires_at": "2026-07-15T12:00:00Z"}))]
pub struct RegisterResponse {
    /// The created user's ID.
    #[schema(value_type = String)]
    pub user_id: String,
    /// Session token for authenticated requests.
    pub session_token: String,
    /// ISO-8601 timestamp when the session expires.
    pub session_expires_at: String,
}

/// Request body for `POST /auth/login`.
#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"email": "user@example.com", "password": "secret123"}))]
pub struct LoginRequest {
    /// User email address.
    pub email: String,
    /// Plaintext password.
    pub password: String,
}

/// Response body for `POST /auth/login`.
#[derive(Serialize, ToSchema)]
#[schema(example = json!({"user_id": "123e4567-e89b-12d3-a456-426614174000", "session_token": "tok_abc123", "session_expires_at": "2026-07-15T12:00:00Z"}))]
pub struct LoginResponse {
    /// The authenticated user's ID.
    #[schema(value_type = String)]
    pub user_id: String,
    /// Session token for authenticated requests.
    pub session_token: String,
    /// ISO-8601 timestamp when the session expires.
    pub session_expires_at: String,
}

/// Request body for `POST /auth/request-otp`.
#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"email": "user@example.com"}))]
pub struct RequestOtpRequest {
    /// Email address to send the OTP to.
    pub email: String,
}

/// Request body for `POST /auth/verify-otp`.
#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"email": "user@example.com", "code": "123456"}))]
pub struct VerifyOtpRequest {
    /// Email address the OTP was sent to.
    pub email: String,
    /// The six-digit OTP code.
    pub code: String,
}

/// Response body for `GET /auth/session`.
#[derive(Serialize, ToSchema)]
#[schema(example = json!({"user_id": "123e4567-e89b-12d3-a456-426614174000", "email": "user@example.com", "role": "student"}))]
pub struct SessionResponse {
    /// The authenticated user's ID.
    #[schema(value_type = String)]
    pub user_id: String,
    /// The user's email address.
    pub email: String,
    /// The user's role (e.g. "student", "admin").
    pub role: String,
}

/// Response body for `POST /auth/logout`.
#[derive(Serialize, ToSchema)]
#[schema(example = json!({"message": "Logged out"}))]
pub struct LogoutResponse {
    /// Success message.
    pub message: &'static str,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Register a new user with email and password.
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Bad request"),
    ),
)]
pub(crate) async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Response, ApiError> {
    let result = state
        .auth_service
        .register(
            &payload.email,
            &payload.password,
            payload.display_name.as_deref(),
        )
        .await?;

    with_session_cookie(
        RegisterResponse {
            user_id: result.user.id().to_string(),
            session_token: result.session_token.clone(),
            session_expires_at: result.session_expires_at.to_rfc3339(),
        },
        &result.session_token,
        &result.session_expires_at,
    )
}

/// Log in with email and password.
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "User logged in successfully", body = LoginResponse),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub(crate) async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Response, ApiError> {
    let result = state
        .auth_service
        .login(&payload.email, &payload.password)
        .await?;

    with_session_cookie(
        LoginResponse {
            user_id: result.user.id().to_string(),
            session_token: result.session_token.clone(),
            session_expires_at: result.session_expires_at.to_rfc3339(),
        },
        &result.session_token,
        &result.session_expires_at,
    )
}

/// Request an OTP code to be sent to the user's email.
#[utoipa::path(
    post,
    path = "/api/v1/auth/request-otp",
    tag = "auth",
    request_body = RequestOtpRequest,
    responses(
        (status = 200, description = "OTP sent successfully"),
        (status = 400, description = "Bad request"),
    ),
)]
pub(crate) async fn request_otp(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RequestOtpRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.auth_service.request_otp(&payload.email).await?;
    Ok(Json(serde_json::json!({ "message": "OTP sent" })))
}

/// Verify an OTP code for the given email.
#[utoipa::path(
    post,
    path = "/api/v1/auth/verify-otp",
    tag = "auth",
    request_body = VerifyOtpRequest,
    responses(
        (status = 200, description = "OTP verified successfully"),
        (status = 400, description = "Bad request"),
    ),
)]
pub(crate) async fn verify_otp(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyOtpRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state
        .auth_service
        .verify_otp(&payload.email, &payload.code)
        .await?;
    Ok(Json(serde_json::json!({ "message": "OTP verified" })))
}

/// Return current session info (user ID, email, role).
#[utoipa::path(
    get,
    path = "/api/v1/auth/session",
    tag = "auth",
    responses(
        (status = 200, description = "Current session info", body = SessionResponse),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub(crate) async fn session_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<SessionResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    Ok(Json(SessionResponse {
        user_id: user_id.to_string(),
        email: user.email().to_string(),
        role: user.role().as_str().to_string(),
    }))
}

/// Invalidate session cookie.
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "auth",
    responses(
        (status = 200, description = "Logged out successfully", body = LogoutResponse),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub(crate) async fn logout(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<LogoutResponse>, ApiError> {
    let (session_id, _user_id) = session::resolve_session(&headers, &state).await?;
    state.auth_service.logout(session_id).await?;
    Ok(Json(LogoutResponse {
        message: "Logged out",
    }))
}
