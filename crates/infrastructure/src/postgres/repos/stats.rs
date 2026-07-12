use sqlx::PgPool;
use sw_domain::error::DomainError;
use sw_domain::repositories::stats::{PlatformStats, StatsRepository, WorkshopRating};
use sw_domain::value_objects::ids::WorkshopId;

/// SQLx-backed implementation of [`StatsRepository`].
pub struct PostgresStatsRepository {
    pool: PgPool,
}

impl PostgresStatsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl StatsRepository for PostgresStatsRepository {
    async fn platform_stats(&self) -> Result<PlatformStats, DomainError> {
        let (total_users,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to count users: {e}")))?;

        let (total_workshops,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM workshops")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to count workshops: {e}")))?;

        let (total_enrollments,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM enrollments")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                DomainError::infrastructure(format!("failed to count enrollments: {e}"))
            })?;

        let (total_reviews,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM reviews")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to count reviews: {e}")))?;

        Ok(PlatformStats {
            total_users,
            total_workshops,
            total_enrollments,
            total_reviews,
        })
    }

    async fn workshop_ratings(&self) -> Result<Vec<WorkshopRating>, DomainError> {
        let rows = sqlx::query_as::<_, RatingRow>(
            r#"SELECT workshop_id,
                      COALESCE(AVG(rating::numeric), 0.0)::double precision AS average_rating,
                      COUNT(*) AS review_count
               FROM reviews
               WHERE status = 'approved'
               GROUP BY workshop_id
               ORDER BY average_rating DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to get ratings: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| WorkshopRating {
                workshop_id: WorkshopId::from_uuid(r.workshop_id),
                average_rating: r.average_rating,
                review_count: r.review_count,
            })
            .collect())
    }
}

#[derive(sqlx::FromRow)]
struct RatingRow {
    workshop_id: uuid::Uuid,
    average_rating: f64,
    review_count: i64,
}
