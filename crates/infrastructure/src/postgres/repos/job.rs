use sqlx::PgPool;
use sw_domain::error::DomainError;
use sw_domain::repositories::job::JobRepository;
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

    async fn claim_next(&self, worker_id: uuid::Uuid) -> Result<Option<JobId>, DomainError> {
        let row = sqlx::query_as::<_, JobIdRow>(
            r#"UPDATE jobs SET status = 'running', locked_by = $1, locked_at = NOW(), attempts = attempts + 1
               WHERE id = (
                   SELECT id FROM jobs
                   WHERE status = 'pending' AND run_at <= NOW()
                   AND (attempts < max_attempts OR max_attempts = 0)
                   ORDER BY run_at ASC
                   LIMIT 1 FOR UPDATE SKIP LOCKED
               )
               RETURNING id"#,
        )
        .bind(worker_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to claim job: {e}")))?;
        Ok(row.map(|r| JobId::from_uuid(r.id)))
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
}

#[derive(sqlx::FromRow)]
struct JobIdRow {
    id: uuid::Uuid,
}
