use crate::aggregates::refund_log::RefundLog;
use crate::error::DomainError;

/// Repository for persisting refund log entries.
#[async_trait::async_trait]
pub trait RefundLogRepository: Send + Sync {
    /// Persist a new refund log entry.
    async fn create(&self, log: &RefundLog) -> Result<(), DomainError>;
}
