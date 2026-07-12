use crate::error::ApplicationError;
use sw_domain::aggregates::category::Category;
use sw_domain::events::{DomainEvent, EventStore};
use sw_domain::repositories::category::CategoryRepository;
use sw_domain::value_objects::ids::CategoryId;
use tracing::instrument;

/// Application service for category CRUD operations with audit logging.
pub struct CategoryService<R: CategoryRepository, E: EventStore> {
    repo: R,
    event_store: E,
}

impl<R: CategoryRepository, E: EventStore> CategoryService<R, E> {
    /// Create a new `CategoryService`.
    pub fn new(repo: R, event_store: E) -> Self {
        Self { repo, event_store }
    }

    /// Create a new category.
    #[instrument(skip(self))]
    pub async fn create(&self, name: String, slug: String) -> Result<Category, ApplicationError> {
        if self.repo.find_by_slug(&slug).await?.is_some() {
            return Err(ApplicationError::conflict(format!(
                "Category with slug '{slug}' already exists"
            )));
        }

        let (category, event) = Category::new(name, slug);
        self.repo.create(&category).await?;
        self.publish_event(event).await?;
        Ok(category)
    }

    /// Find a category by its ID.
    #[instrument(skip(self))]
    pub async fn get_by_id(&self, id: CategoryId) -> Result<Category, ApplicationError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Category", id))
    }

    /// Find a category by its slug.
    #[instrument(skip(self))]
    pub async fn get_by_slug(&self, slug: &str) -> Result<Category, ApplicationError> {
        self.repo
            .find_by_slug(slug)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Category", slug))
    }

    /// List all categories.
    #[instrument(skip(self))]
    pub async fn list(&self) -> Result<Vec<Category>, ApplicationError> {
        self.repo.find_all().await.map_err(ApplicationError::from)
    }

    /// Update a category.
    #[instrument(skip(self))]
    pub async fn update(
        &self,
        id: CategoryId,
        name: Option<String>,
        description: Option<String>,
        thumbnail_url: Option<String>,
    ) -> Result<Category, ApplicationError> {
        let mut category = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Category", id))?;

        let event = category.update(name, description, thumbnail_url);
        self.repo.update(&category).await?;
        self.publish_event(event).await?;
        Ok(category)
    }

    /// Delete a category by ID.
    #[instrument(skip(self))]
    pub async fn delete(&self, id: CategoryId) -> Result<(), ApplicationError> {
        let _category = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Category", id))?;

        self.repo.delete(id).await?;
        self.publish_event(DomainEvent::CategoryDeleted { category_id: id })
            .await?;
        Ok(())
    }

    async fn publish_event(&self, event: DomainEvent) -> Result<(), ApplicationError> {
        self.event_store
            .publish(&event, None)
            .await
            .map_err(|e| ApplicationError::internal(format!("failed to publish event: {e}")))
    }
}
