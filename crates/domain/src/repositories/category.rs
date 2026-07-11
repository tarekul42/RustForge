use crate::aggregates::category::Category;
use crate::error::DomainError;
use crate::value_objects::ids::CategoryId;

/// Repository for persisting and retrieving [`Category`] aggregates.
#[async_trait::async_trait]
pub trait CategoryRepository: Send + Sync {
    /// Persist a new category.
    async fn create(&self, category: &Category) -> Result<(), DomainError>;
    /// Find a category by its unique ID.
    async fn find_by_id(&self, id: CategoryId) -> Result<Option<Category>, DomainError>;
    /// Find a category by its URL slug.
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Category>, DomainError>;
    /// Return all categories, ordered by name.
    async fn find_all(&self) -> Result<Vec<Category>, DomainError>;
    /// Persist changes to an existing category.
    async fn update(&self, category: &Category) -> Result<(), DomainError>;
    /// Delete a category by ID.
    async fn delete(&self, id: CategoryId) -> Result<(), DomainError>;
}
