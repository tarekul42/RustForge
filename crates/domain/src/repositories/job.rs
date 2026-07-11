use crate::error::DomainError;
use crate::value_objects::ids::JobId;
use chrono::{DateTime, Utc};

/// Repository for managing background jobs.
#[async_trait::async_trait]
pub trait JobRepository: Send + Sync {
    /// Enqueue a new background job.
    async fn enqueue(
        &self,
        job_type: &str,
        payload: &serde_json::Value,
        run_at: Option<DateTime<Utc>>,
    ) -> Result<(), DomainError>;
    /// Claim the next pending job for execution. Returns `None` if no jobs are available.
    async fn claim_next(&self, worker_id: uuid::Uuid) -> Result<Option<JobId>, DomainError>;
    /// Mark a job as completed.
    async fn complete(&self, job_id: JobId) -> Result<(), DomainError>;
    /// Mark a job as failed, recording the error message.
    async fn fail(&self, job_id: JobId, error: &str) -> Result<(), DomainError>;
}
