use crate::error::ApplicationError;
use sw_domain::aggregates::level::Level;
use sw_domain::events::{DomainEvent, EventStore};
use sw_domain::repositories::level::LevelRepository;
use sw_domain::value_objects::ids::LevelId;
use tracing::instrument;

/// Application service for difficulty level CRUD operations with audit logging.
pub struct LevelService<R: LevelRepository, E: EventStore> {
    repo: R,
    event_store: E,
}

impl<R: LevelRepository, E: EventStore> LevelService<R, E> {
    /// Create a new `LevelService`.
    pub fn new(repo: R, event_store: E) -> Self {
        Self { repo, event_store }
    }

    /// Create a new difficulty level.
    #[instrument(skip(self))]
    pub async fn create(&self, name: String) -> Result<Level, ApplicationError> {
        let (level, event) = Level::new(name);
        self.repo.create(&level).await?;
        self.publish_event(event).await?;
        Ok(level)
    }

    /// Find a level by its ID.
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: LevelId) -> Result<Level, ApplicationError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Level", id))
    }

    /// List all levels.
    #[instrument(skip(self))]
    pub async fn list(&self) -> Result<Vec<Level>, ApplicationError> {
        self.repo.find_all().await.map_err(ApplicationError::from)
    }

    /// Rename a level.
    #[instrument(skip(self))]
    pub async fn rename(&self, id: LevelId, name: String) -> Result<Level, ApplicationError> {
        let mut level = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Level", id))?;

        let event = level.rename(name);
        self.repo.update(&level).await?;
        self.publish_event(event).await?;
        Ok(level)
    }

    /// Delete a level by ID.
    #[instrument(skip(self))]
    pub async fn delete(&self, id: LevelId) -> Result<(), ApplicationError> {
        let _level = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Level", id))?;

        self.repo.delete(id).await?;
        self.publish_event(DomainEvent::LevelDeleted { level_id: id })
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
