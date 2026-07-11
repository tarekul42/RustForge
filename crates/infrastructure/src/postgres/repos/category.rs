use sqlx::PgPool;
use sw_domain::aggregates::category::Category;
use sw_domain::error::DomainError;
use sw_domain::repositories::category::CategoryRepository;
use sw_domain::value_objects::ids::CategoryId;

pub struct PostgresCategoryRepository {
    pool: PgPool,
}

impl PostgresCategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CategoryRepository for PostgresCategoryRepository {
    async fn create(&self, category: &Category) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO categories (id, name, slug, description, thumbnail_url, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(category.id.into_uuid())
        .bind(&category.name)
        .bind(&category.slug)
        .bind(&category.description)
        .bind(&category.thumbnail_url)
        .bind(category.created_at)
        .bind(category.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to create category: {e}")))?;
        Ok(())
    }

    async fn find_by_id(&self, id: CategoryId) -> Result<Option<Category>, DomainError> {
        let row = sqlx::query_as::<_, CategoryRow>(
            r#"SELECT id, name, slug, description, thumbnail_url, created_at, updated_at
               FROM categories WHERE id = $1"#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find category: {e}")))?;
        row.map(CategoryRow::into_domain).transpose()
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Category>, DomainError> {
        let row = sqlx::query_as::<_, CategoryRow>(
            r#"SELECT id, name, slug, description, thumbnail_url, created_at, updated_at
               FROM categories WHERE slug = $1"#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::infrastructure(format!("failed to find category by slug: {e}"))
        })?;
        row.map(CategoryRow::into_domain).transpose()
    }

    async fn find_all(&self) -> Result<Vec<Category>, DomainError> {
        let rows = sqlx::query_as::<_, CategoryRow>(
            r#"SELECT id, name, slug, description, thumbnail_url, created_at, updated_at
               FROM categories ORDER BY name"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to list categories: {e}")))?;
        rows.into_iter().map(CategoryRow::into_domain).collect()
    }

    async fn update(&self, category: &Category) -> Result<(), DomainError> {
        sqlx::query(
            r#"UPDATE categories SET name = $2, slug = $3, description = $4, thumbnail_url = $5, updated_at = $6
               WHERE id = $1"#,
        )
        .bind(category.id.into_uuid())
        .bind(&category.name)
        .bind(&category.slug)
        .bind(&category.description)
        .bind(&category.thumbnail_url)
        .bind(category.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to update category: {e}")))?;
        Ok(())
    }

    async fn delete(&self, id: CategoryId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM categories WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to delete category: {e}")))?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct CategoryRow {
    id: uuid::Uuid,
    name: String,
    slug: String,
    description: Option<String>,
    thumbnail_url: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl CategoryRow {
    fn into_domain(self) -> Result<Category, DomainError> {
        Ok(Category {
            id: CategoryId::from_uuid(self.id),
            name: self.name,
            slug: self.slug,
            description: self.description,
            thumbnail_url: self.thumbnail_url,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
