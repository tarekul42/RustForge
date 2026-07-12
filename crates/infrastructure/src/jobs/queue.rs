use std::sync::Arc;

use sw_domain::error::DomainError;
use sw_domain::repositories::job::{JobInfo, JobRepository};
use sw_domain::value_objects::ids::JobId;

/// Queue service that wraps a [`JobRepository`] and provides
/// exponential-backoff retry logic for failed jobs.
pub struct QueueService {
    repo: Arc<dyn JobRepository>,
    /// Base backoff in seconds (actual delay = `base_seconds * 2^attempt`).
    base_backoff_seconds: i64,
}

impl QueueService {
    /// Create a new queue service.
    pub fn new(repo: Arc<dyn JobRepository>, base_backoff_seconds: i64) -> Self {
        Self {
            repo,
            base_backoff_seconds,
        }
    }

    /// Enqueue a new background job.
    pub async fn enqueue(
        &self,
        job_type: &str,
        payload: &serde_json::Value,
        run_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), DomainError> {
        self.repo.enqueue(job_type, payload, run_at).await
    }

    /// Claim the next pending job for execution.
    pub async fn claim_next(&self, worker_id: uuid::Uuid) -> Result<Option<JobInfo>, DomainError> {
        self.repo.claim_next(worker_id).await
    }

    /// Mark a job as completed.
    pub async fn complete(&self, job_id: JobId) -> Result<(), DomainError> {
        self.repo.complete(job_id).await
    }

    /// Mark a job as failed and schedule a retry with exponential backoff.
    ///
    /// Returns `true` if the job was rescheduled for retry,
    /// `false` if it exceeded max attempts and stays failed.
    pub async fn fail_with_backoff(&self, job_id: JobId, error: &str) -> Result<bool, DomainError> {
        self.repo.fail(job_id, error).await?;
        self.repo.retry(job_id, self.base_backoff_seconds).await
    }
}
