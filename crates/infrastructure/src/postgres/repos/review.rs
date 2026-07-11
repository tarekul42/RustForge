use sqlx::PgPool;
use sw_domain::aggregates::review::{Review, ReviewStatus};
use sw_domain::error::DomainError;
use sw_domain::repositories::review::ReviewRepository;
use sw_domain::value_objects::ids::{ReviewId, UserId, WorkshopId};

/// SQLx-backed implementation of [`ReviewRepository`].
pub struct PostgresReviewRepository {
    pool: PgPool,
}

impl PostgresReviewRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ReviewRepository for PostgresReviewRepository {
    async fn create(&self, review: &Review) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO reviews (id, user_id, workshop_id, rating, title, content, status, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(review.id.into_uuid())
        .bind(review.user_id.into_uuid())
        .bind(review.workshop_id.into_uuid())
        .bind(review.rating)
        .bind(&review.title)
        .bind(&review.content)
        .bind(review.status.as_str())
        .bind(review.created_at)
        .bind(review.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to create review: {e}")))?;
        Ok(())
    }

    async fn find_by_id(&self, id: ReviewId) -> Result<Option<Review>, DomainError> {
        let row = sqlx::query_as::<_, ReviewRow>(
            r#"SELECT id, user_id, workshop_id, rating, title, content, status, created_at, updated_at
               FROM reviews WHERE id = $1"#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find review: {e}")))?;
        row.map(ReviewRow::into_domain).transpose()
    }

    async fn find_by_user_and_workshop(
        &self,
        user_id: UserId,
        workshop_id: WorkshopId,
    ) -> Result<Option<Review>, DomainError> {
        let row = sqlx::query_as::<_, ReviewRow>(
            r#"SELECT id, user_id, workshop_id, rating, title, content, status, created_at, updated_at
               FROM reviews WHERE user_id = $1 AND workshop_id = $2"#,
        )
        .bind(user_id.into_uuid())
        .bind(workshop_id.into_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find review: {e}")))?;
        row.map(ReviewRow::into_domain).transpose()
    }

    async fn find_by_workshop(&self, workshop_id: WorkshopId) -> Result<Vec<Review>, DomainError> {
        let rows = sqlx::query_as::<_, ReviewRow>(
            r#"SELECT id, user_id, workshop_id, rating, title, content, status, created_at, updated_at
               FROM reviews WHERE workshop_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(workshop_id.into_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find reviews: {e}")))?;
        rows.into_iter().map(ReviewRow::into_domain).collect()
    }

    async fn update(&self, review: &Review) -> Result<(), DomainError> {
        sqlx::query(
            r#"UPDATE reviews SET rating = $2, title = $3, content = $4, status = $5, updated_at = $6
               WHERE id = $1"#,
        )
        .bind(review.id.into_uuid())
        .bind(review.rating)
        .bind(&review.title)
        .bind(&review.content)
        .bind(review.status.as_str())
        .bind(review.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to update review: {e}")))?;
        Ok(())
    }

    async fn delete(&self, id: ReviewId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM reviews WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to delete review: {e}")))?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ReviewRow {
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    workshop_id: uuid::Uuid,
    rating: i16,
    title: String,
    content: String,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl ReviewRow {
    fn into_domain(self) -> Result<Review, DomainError> {
        let status = ReviewStatus::from_str(&self.status).ok_or_else(|| {
            DomainError::infrastructure(format!("invalid review status: {}", self.status))
        })?;
        Ok(Review {
            id: ReviewId::from_uuid(self.id),
            user_id: UserId::from_uuid(self.user_id),
            workshop_id: WorkshopId::from_uuid(self.workshop_id),
            rating: self.rating,
            title: self.title,
            content: self.content,
            status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
