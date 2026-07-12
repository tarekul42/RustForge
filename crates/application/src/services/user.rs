use crate::error::ApplicationError;
use sw_domain::aggregates::user::User;
use sw_domain::events::{DomainEvent, EventStore};
use sw_domain::repositories::user::UserRepository;
use sw_domain::value_objects::ids::UserId;
use tracing::instrument;

/// Application service for admin user management (list, get, update, delete).
pub struct UserAdminService<R: UserRepository, E: EventStore> {
    repo: R,
    event_store: E,
}

/// Input for updating a user (admin operation). `None` fields are left unchanged.
#[derive(Debug)]
pub struct UpdateUserInput {
    /// User to update.
    pub user_id: UserId,
    /// New display name.
    pub name: Option<String>,
    /// New role (e.g. "admin", "instructor", "student").
    pub role: Option<String>,
    /// New status (e.g. "active", "inactive", "blocked").
    pub status: Option<String>,
    /// New phone number.
    pub phone: Option<String>,
    /// New age.
    pub age: Option<i16>,
    /// New address.
    pub address: Option<String>,
    /// New expertise description.
    pub expertise: Option<String>,
    /// New biography.
    pub bio: Option<String>,
}

impl<R: UserRepository, E: EventStore> UserAdminService<R, E> {
    /// Create a new `UserAdminService`.
    pub fn new(repo: R, event_store: E) -> Self {
        Self { repo, event_store }
    }

    /// List all users (admin only).
    #[instrument(skip(self))]
    pub async fn list(&self) -> Result<Vec<User>, ApplicationError> {
        self.repo.find_all().await.map_err(ApplicationError::from)
    }

    /// Get a user by ID (admin only).
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: UserId) -> Result<User, ApplicationError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("User", id))
    }

    /// Update a user's fields (admin only).
    #[instrument(skip(self))]
    pub async fn update(&self, input: UpdateUserInput) -> Result<User, ApplicationError> {
        let mut user = self
            .repo
            .find_by_id(input.user_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("User", input.user_id))?;

        if let Some(name) = input.name {
            user.name = name;
        }
        if let Some(role_str) = input.role {
            let role = sw_domain::aggregates::user::UserRole::from_str(&role_str)
                .ok_or_else(|| ApplicationError::validation(format!("Invalid role: {role_str}")))?;
            user.role = role;
        }
        if let Some(status_str) = input.status {
            let status = sw_domain::aggregates::user::UserStatus::from_str(&status_str)
                .ok_or_else(|| {
                    ApplicationError::validation(format!("Invalid status: {status_str}"))
                })?;
            user.status = status;
        }
        if let Some(phone) = input.phone {
            user.phone = Some(phone);
        }
        if let Some(age) = input.age {
            user.age = Some(age);
        }
        if let Some(address) = input.address {
            user.address = Some(address);
        }
        if let Some(expertise) = input.expertise {
            user.expertise = Some(expertise);
        }
        if let Some(bio) = input.bio {
            user.bio = Some(bio);
        }

        user.updated_at = chrono::Utc::now();
        self.repo.update(&user).await?;
        self.publish_event(DomainEvent::UserUpdated {
            user_id: input.user_id,
        })
        .await?;
        Ok(user)
    }

    /// Delete a user by ID (admin only).
    #[instrument(skip(self))]
    pub async fn delete(&self, id: UserId) -> Result<(), ApplicationError> {
        let _user = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("User", id))?;

        self.repo.delete(id).await?;
        self.publish_event(DomainEvent::UserDeleted { user_id: id })
            .await?;
        Ok(())
    }

    async fn publish_event(&self, event: DomainEvent) -> Result<(), ApplicationError> {
        self.event_store
            .publish(&event)
            .await
            .map_err(|e| ApplicationError::internal(format!("failed to publish event: {e}")))
    }
}
