use axum::{
    extract::{Form, Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::ApiError;
use crate::extractors::session;
use crate::state::AppState;
use sw_domain::value_objects::ids::PaymentId;

/// Build the payment router — all paths are relative to `/api/v1/payments`.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/success", get(success_get).post(success_post))
        .route("/fail", get(fail_get).post(fail_post))
        .route("/cancel", get(cancel_get).post(cancel_post))
        .route("/ipn", post(ipn_handler))
        .route("/refund", post(refund_handler))
        .route("/invoice/{id}", get(get_invoice_url))
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct PaymentCallback {
    tran_id: Option<String>,
    val_id: Option<String>,
}

#[derive(Deserialize)]
struct RefundRequest {
    payment_id: String,
    reason: String,
}

#[derive(Serialize)]
struct PaymentResponse {
    id: String,
    enrollment_id: String,
    transaction_id: String,
    amount_cents: i64,
    status: String,
    invoice_url: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct IpnResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct InvoiceResponse {
    url: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn success_get(
    State(state): State<Arc<AppState>>,
    Query(payload): Query<PaymentCallback>,
) -> Result<Json<PaymentResponse>, ApiError> {
    let transaction_id = payload
        .tran_id
        .ok_or_else(|| ApiError::BadRequest("Missing tran_id".to_string()))?;
    let val_id = payload
        .val_id
        .ok_or_else(|| ApiError::BadRequest("Missing val_id".to_string()))?;

    let payment = state
        .payment_service
        .success_payment(&transaction_id, &val_id)
        .await?;

    Ok(Json(PaymentResponse {
        id: payment.id.to_string(),
        enrollment_id: payment.enrollment_id.to_string(),
        transaction_id: payment.transaction_id,
        amount_cents: payment.amount.cents(),
        status: payment.status.as_str().to_string(),
        invoice_url: payment.invoice_url,
        created_at: payment.created_at.to_rfc3339(),
        updated_at: payment.updated_at.to_rfc3339(),
    }))
}

async fn success_post(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<PaymentCallback>,
) -> Result<Json<PaymentResponse>, ApiError> {
    success_get(State(state), Query(payload)).await
}

async fn fail_get(
    State(state): State<Arc<AppState>>,
    Query(payload): Query<PaymentCallback>,
) -> Result<Json<PaymentResponse>, ApiError> {
    let transaction_id = payload
        .tran_id
        .ok_or_else(|| ApiError::BadRequest("Missing tran_id".to_string()))?;

    let payment = state.payment_service.fail_payment(&transaction_id).await?;

    Ok(Json(PaymentResponse {
        id: payment.id.to_string(),
        enrollment_id: payment.enrollment_id.to_string(),
        transaction_id: payment.transaction_id,
        amount_cents: payment.amount.cents(),
        status: payment.status.as_str().to_string(),
        invoice_url: payment.invoice_url,
        created_at: payment.created_at.to_rfc3339(),
        updated_at: payment.updated_at.to_rfc3339(),
    }))
}

async fn fail_post(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<PaymentCallback>,
) -> Result<Json<PaymentResponse>, ApiError> {
    fail_get(State(state), Query(payload)).await
}

async fn cancel_get(
    State(state): State<Arc<AppState>>,
    Query(payload): Query<PaymentCallback>,
) -> Result<Json<PaymentResponse>, ApiError> {
    let transaction_id = payload
        .tran_id
        .ok_or_else(|| ApiError::BadRequest("Missing tran_id".to_string()))?;

    let payment = state
        .payment_service
        .cancel_payment(&transaction_id)
        .await?;

    Ok(Json(PaymentResponse {
        id: payment.id.to_string(),
        enrollment_id: payment.enrollment_id.to_string(),
        transaction_id: payment.transaction_id,
        amount_cents: payment.amount.cents(),
        status: payment.status.as_str().to_string(),
        invoice_url: payment.invoice_url,
        created_at: payment.created_at.to_rfc3339(),
        updated_at: payment.updated_at.to_rfc3339(),
    }))
}

async fn cancel_post(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<PaymentCallback>,
) -> Result<Json<PaymentResponse>, ApiError> {
    cancel_get(State(state), Query(payload)).await
}

async fn ipn_handler(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<HashMap<String, String>>,
) -> Result<Json<IpnResponse>, ApiError> {
    state.payment_service.handle_ipn(&payload).await?;

    Ok(Json(IpnResponse {
        success: true,
        message: "IPN processed".to_string(),
    }))
}

async fn refund_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<RefundRequest>,
) -> Result<Json<PaymentResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let user = state.auth_service.get_user(user_id).await?;

    if !matches!(
        user.role,
        sw_domain::aggregates::user::UserRole::Admin
            | sw_domain::aggregates::user::UserRole::SuperAdmin
    ) {
        return Err(ApiError::Unauthorized(
            "Only admins can process refunds".to_string(),
        ));
    }

    let payment_id = PaymentId::parse_str(&payload.payment_id)
        .map_err(|_| ApiError::BadRequest("Invalid payment_id".to_string()))?;

    let payment = state
        .payment_service
        .refund(payment_id, payload.reason)
        .await?;

    Ok(Json(PaymentResponse {
        id: payment.id.to_string(),
        enrollment_id: payment.enrollment_id.to_string(),
        transaction_id: payment.transaction_id,
        amount_cents: payment.amount.cents(),
        status: payment.status.as_str().to_string(),
        invoice_url: payment.invoice_url,
        created_at: payment.created_at.to_rfc3339(),
        updated_at: payment.updated_at.to_rfc3339(),
    }))
}

async fn get_invoice_url(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<InvoiceResponse>, ApiError> {
    let (_session_id, _user_id) = session::resolve_session(&headers, &state).await?;

    let payment_id = PaymentId::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid payment id".to_string()))?;

    let payment = state
        .payment_service
        .find_by_id(payment_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(InvoiceResponse {
        url: payment.invoice_url,
    }))
}
