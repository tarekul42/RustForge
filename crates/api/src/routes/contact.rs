use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, patch, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

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

#[derive(Deserialize, ToSchema)]
pub(crate) struct SubmitContactRequest {
    name: String,
    email: String,
    subject: String,
    message: String,
}

#[derive(Deserialize, ToSchema, utoipa::IntoParams)]
pub(crate) struct ListContactsQuery {
    is_read: Option<bool>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct ContactResponse {
    id: String,
    name: String,
    email: String,
    subject: String,
    message: String,
    is_read: bool,
    created_at: String,
    updated_at: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/contacts",
    tag = "contacts",
    request_body = SubmitContactRequest,
    responses(
        (status = 201, description = "Contact submitted", body = ContactResponse),
        (status = 400, description = "Bad request"),
    ),
)]
pub(crate) async fn submit_contact(
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

#[utoipa::path(
    get,
    path = "/api/v1/contacts",
    tag = "contacts",
    params(ListContactsQuery),
    responses(
        (status = 200, description = "List of contacts", body = Vec<ContactResponse>),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub(crate) async fn list_contacts(
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

#[utoipa::path(
    patch,
    path = "/api/v1/contacts/{id}/read",
    tag = "contacts",
    params(
        ("id" = String, Path, description = "Contact ID"),
    ),
    responses(
        (status = 200, description = "Contact marked as read", body = ContactResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
)]
pub(crate) async fn mark_read_contact(
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

#[utoipa::path(
    delete,
    path = "/api/v1/contacts/{id}",
    tag = "contacts",
    params(
        ("id" = String, Path, description = "Contact ID"),
    ),
    responses(
        (status = 200, description = "Contact deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
)]
pub(crate) async fn delete_contact(
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
        id: contact.id().to_string(),
        name: contact.name().to_string(),
        email: contact.email().to_string(),
        subject: contact.subject().to_string(),
        message: contact.message().to_string(),
        is_read: contact.is_read(),
        created_at: contact.created_at().to_rfc3339(),
        updated_at: contact.updated_at().to_rfc3339(),
    }
}
