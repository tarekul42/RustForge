use sqlx::PgPool;
use sw_domain::error::DomainError;
use sw_domain::repositories::job::{JobInfo, JobRepository};
use sw_domain::value_objects::ids::JobId;

/// SQLx-backed implementation of [`JobRepository`].
pub struct PostgresJobRepository {
    pool: PgPool,
}

impl PostgresJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl JobRepository for PostgresJobRepository {
    async fn enqueue(
        &self,
        job_type: &str,
        payload: &serde_json::Value,
        run_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO jobs (job_type, payload, run_at)
               VALUES ($1, $2, COALESCE($3, NOW()))"#,
        )
        .bind(job_type)
        .bind(payload)
        .bind(run_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to enqueue job: {e}")))?;
        Ok(())
    }

    async fn claim_next(&self, worker_id: uuid::Uuid) -> Result<Option<JobInfo>, DomainError> {
        let row = sqlx::query_as::<_, JobRow>(
            r#"UPDATE jobs SET status = 'running', locked_by = $1, locked_at = NOW(), attempts = attempts + 1
               WHERE id = (
                   SELECT id FROM jobs
                   WHERE status = 'pending' AND run_at <= NOW()
                   AND (attempts < max_attempts OR max_attempts = 0)
                   ORDER BY run_at ASC
                   LIMIT 1 FOR UPDATE SKIP LOCKED
               )
               RETURNING id, job_type, payload"#,
        )
        .bind(worker_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to claim job: {e}")))?;
        Ok(row.map(|r| JobInfo {
            id: JobId::from_uuid(r.id),
            job_type: r.job_type,
            payload: r.payload,
        }))
    }

    async fn complete(&self, job_id: JobId) -> Result<(), DomainError> {
        sqlx::query(
            r#"UPDATE jobs SET status = 'completed', completed_at = NOW()
               WHERE id = $1"#,
        )
        .bind(job_id.into_uuid())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to complete job: {e}")))?;
        Ok(())
    }

    async fn fail(&self, job_id: JobId, error: &str) -> Result<(), DomainError> {
        sqlx::query(
            r#"UPDATE jobs SET status = 'failed', error = $2
               WHERE id = $1"#,
        )
        .bind(job_id.into_uuid())
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to fail job: {e}")))?;
        Ok(())
    }

    async fn retry(&self, job_id: JobId, base_seconds: i64) -> Result<bool, DomainError> {
        let result = sqlx::query(
            r#"UPDATE jobs
               SET status = 'pending',
                   run_at = NOW() + ($2 * GREATEST(1, 2 ^ (attempts - 1)) * INTERVAL '1 second'),
                   locked_by = NULL,
                   locked_at = NULL,
                   error = NULL
               WHERE id = $1
                 AND status = 'failed'
                 AND attempts < max_attempts"#,
        )
        .bind(job_id.into_uuid())
        .bind(base_seconds)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to retry job: {e}")))?;
        Ok(result.rows_affected() > 0)
    }
}

#[derive(sqlx::FromRow)]
struct JobRow {
    id: uuid::Uuid,
    job_type: String,
    payload: serde_json::Value,
}
