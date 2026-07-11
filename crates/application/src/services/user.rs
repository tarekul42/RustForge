use crate::error::ApplicationError;
use sw_domain::aggregates::user::User;
use sw_domain::events::{DomainEvent, EventStore};
use sw_domain::repositories::user::UserRepository;
use sw_domain::value_objects::ids::UserId;
use tracing::instrument;

/// Application service for admin user management (list, get, delete).
pub struct UserAdminService<R: UserRepository, E: EventStore> {
    repo: R,
    event_store: E,
}

impl<R: UserRepository, E: EventStore> UserAdminService<R, E> {
    /// Create a new `UserAdminService`.
    pub fn new(repo: R, event_store: E) -> Self {
        Self { repo, event_store }
    }

    /// List all users (admin only).
    #[instrument(skip(self))]
    pub async fn list(&self) -> Result<Vec<User>, ApplicationError> {
        todo!("list users — requires find_all on UserRepository")
    }

    /// Get a user by ID (admin only).
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: UserId) -> Result<User, ApplicationError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("User", id))
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
