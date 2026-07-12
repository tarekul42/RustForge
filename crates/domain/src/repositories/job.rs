use crate::error::DomainError;
use crate::value_objects::ids::JobId;
use chrono::{DateTime, Utc};

/// Information about a claimed job returned by [`JobRepository::claim_next`].
#[derive(Debug, Clone)]
pub struct JobInfo {
    /// The job's unique identifier.
    pub id: JobId,
    /// The job type (e.g. "send_email", "generate_invoice").
    pub job_type: String,
    /// The JSON payload associated with the job.
    pub payload: serde_json::Value,
}

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
    async fn claim_next(&self, worker_id: uuid::Uuid) -> Result<Option<JobInfo>, DomainError>;
    /// Mark a job as completed.
    async fn complete(&self, job_id: JobId) -> Result<(), DomainError>;
    /// Mark a job as failed, recording the error message.
    async fn fail(&self, job_id: JobId, error: &str) -> Result<(), DomainError>;
    /// Retry a failed job with exponential backoff.
    ///
    /// Resets the job status to `pending` and sets `run_at` to the current time
    /// plus a backoff duration: `base_seconds * 2^attempts`.
    /// Returns `true` if the job was rescheduled, `false` if max attempts exceeded.
    async fn retry(&self, job_id: JobId, base_seconds: i64) -> Result<bool, DomainError>;
}
