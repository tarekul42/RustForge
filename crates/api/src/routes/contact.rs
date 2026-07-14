use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::ApiError;
use crate::extractors::session;
use crate::state::AppState;
use sw_domain::value_objects::ids::ContactId;

/// Build the contact router — all paths are relative to `/api/v1/contacts`.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(submit_contact))
        .route("/", get(list_contacts))
        .route("/{id}/read", patch(mark_read_contact))
        .route("/{id}", delete(delete_contact))
}

#[derive(Deserialize)]
struct SubmitContactRequest {
    name: String,
    email: String,
    subject: String,
    message: String,
}

#[derive(Deserialize)]
struct ListContactsQuery {
    is_read: Option<bool>,
}

#[derive(Serialize)]
struct ContactResponse {
    id: String,
    name: String,
    email: String,
    subject: String,
    message: String,
    is_read: bool,
    created_at: String,
    updated_at: String,
}

async fn submit_contact(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SubmitContactRequest>,
) -> Result<Json<ContactResponse>, ApiError> {
    let contact = state
        .contact_service
        .submit(sw_application::services::contact::SubmitContactInput {
            name: payload.name,
            email: payload.email,
            subject: payload.subject,
            message: payload.message,
        })
        .await?;

    Ok(Json(to_response(&contact)))
}

async fn list_contacts(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(query): Query<ListContactsQuery>,
) -> Result<Json<Vec<ContactResponse>>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let caller = state.auth_service.get_user(user_id).await?;
    if !caller.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    let contacts = state.contact_service.list(query.is_read).await?;
    Ok(Json(contacts.iter().map(to_response).collect()))
}

async fn mark_read_contact(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ContactResponse>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let caller = state.auth_service.get_user(user_id).await?;
    if !caller.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    let contact_id = ContactId::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid contact id".to_string()))?;

    let contact = state.contact_service.mark_read(contact_id).await?;
    Ok(Json(to_response(&contact)))
}

async fn delete_contact(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (_session_id, user_id) = session::resolve_session(&headers, &state).await?;
    let caller = state.auth_service.get_user(user_id).await?;
    if !caller.is_admin() {
        return Err(ApiError::Unauthorized("Admin access required".to_string()));
    }

    let contact_id = ContactId::parse_str(&id)
        .map_err(|_| ApiError::BadRequest("Invalid contact id".to_string()))?;

    state.contact_service.delete(contact_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}

fn to_response(contact: &sw_domain::aggregates::contact::Contact) -> ContactResponse {
    ContactResponse {
        id: contact.id.to_string(),
        name: contact.name.clone(),
        email: contact.email.to_string(),
        subject: contact.subject.clone(),
        message: contact.message.clone(),
        is_read: contact.is_read,
        created_at: contact.created_at.to_rfc3339(),
        updated_at: contact.updated_at.to_rfc3339(),
    }
}
