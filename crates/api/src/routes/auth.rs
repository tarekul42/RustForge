use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
#[derive(Deserialize)]
pub struct RegisterRequest {
    /// User email address.
    pub email: String,
    /// Plaintext password.
    pub password: String,
    /// Optional display name.
    pub display_name: Option<String>,
}

/// Response body for `POST /auth/register` and `POST /auth/login`.
#[derive(Serialize)]
pub struct RegisterResponse {
    /// The created user's ID.
    pub user_id: String,
    /// Session token for authenticated requests.
    pub session_token: String,
    /// ISO-8601 timestamp when the session expires.
    pub session_expires_at: String,
}

/// Request body for `POST /auth/login`.
#[derive(Deserialize)]
pub struct LoginRequest {
    /// User email address.
    pub email: String,
    /// Plaintext password.
    pub password: String,
}

/// Response body for `POST /auth/login`.
#[derive(Serialize)]
pub struct LoginResponse {
    /// The authenticated user's ID.
    pub user_id: String,
    /// Session token for authenticated requests.
    pub session_token: String,
    /// ISO-8601 timestamp when the session expires.
    pub session_expires_at: String,
}

/// Request body for `POST /auth/request-otp`.
#[derive(Deserialize)]
pub struct RequestOtpRequest {
    /// Email address to send the OTP to.
    pub email: String,
}

/// Request body for `POST /auth/verify-otp`.
#[derive(Deserialize)]
pub struct VerifyOtpRequest {
    /// Email address the OTP was sent to.
    pub email: String,
    /// The six-digit OTP code.
    pub code: String,
}

/// Response body for `GET /auth/session`.
#[derive(Serialize)]
pub struct SessionResponse {
    /// The authenticated user's ID.
    pub user_id: String,
    /// The user's email address.
    pub email: String,
    /// The user's role (e.g. "student", "admin").
    pub role: String,
}

/// Response body for `POST /auth/logout`.
#[derive(Serialize)]
pub struct LogoutResponse {
    /// Success message.
    pub message: &'static str,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Register a new user with email and password.
async fn register(
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
            user_id: result.user.id.to_string(),
            session_token: result.session_token.clone(),
            session_expires_at: result.session_expires_at.to_rfc3339(),
        },
        &result.session_token,
        &result.session_expires_at,
    )
}

/// Log in with email and password.
async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Response, ApiError> {
    let result = state
        .auth_service
        .login(&payload.email, &payload.password)
        .await?;

    with_session_cookie(
        LoginResponse {
            user_id: result.user.id.to_string(),
            session_token: result.session_token.clone(),
            session_expires_at: result.session_expires_at.to_rfc3339(),
        },
        &result.session_token,
        &result.session_expires_at,
    )
}

/// Request an OTP code to be sent to the user's email.
async fn request_otp(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RequestOtpRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.auth_service.request_otp(&payload.email).await?;
    Ok(Json(serde_json::json!({ "message": "OTP sent" })))
}

/// Verify an OTP code for the given email.
async fn verify_otp(
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
async fn session_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<SessionResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;
    Ok(Json(SessionResponse {
        user_id: user_id.to_string(),
        email: user.email.to_string(),
        role: user.role.as_str().to_string(),
    }))
}

/// Invalidate session cookie.
async fn logout(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<LogoutResponse>, ApiError> {
    let (session_id, _user_id) = session::resolve_session(&headers, &state).await?;
    state.auth_service.logout(session_id).await?;
    Ok(Json(LogoutResponse {
        message: "Logged out",
    }))
}
