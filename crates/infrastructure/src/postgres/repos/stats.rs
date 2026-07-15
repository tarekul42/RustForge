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
        let users_row = sqlx::query!(r#"SELECT COUNT(*) as "count!" FROM users"#)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to count users: {e}")))?;
        let total_users = users_row.count;

        let workshops_row = sqlx::query!(r#"SELECT COUNT(*) as "count!" FROM workshops"#)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to count workshops: {e}")))?;
        let total_workshops = workshops_row.count;

        let enrollments_row = sqlx::query!(r#"SELECT COUNT(*) as "count!" FROM enrollments"#)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                DomainError::infrastructure(format!("failed to count enrollments: {e}"))
            })?;
        let total_enrollments = enrollments_row.count;

        let reviews_row = sqlx::query!(r#"SELECT COUNT(*) as "count!" FROM reviews"#)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to count reviews: {e}")))?;
        let total_reviews = reviews_row.count;

        Ok(PlatformStats {
            total_users,
            total_workshops,
            total_enrollments,
            total_reviews,
        })
    }

    async fn workshop_ratings(&self) -> Result<Vec<WorkshopRating>, DomainError> {
        let rows = sqlx::query_as!(
            RatingRow,
            r#"SELECT workshop_id,
                      COALESCE(AVG(rating::numeric), 0.0)::double precision AS "average_rating!",
                      COUNT(*) AS "review_count!"
               FROM reviews
               WHERE status = 'approved'
               GROUP BY workshop_id
               ORDER BY 2 DESC"#,
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
