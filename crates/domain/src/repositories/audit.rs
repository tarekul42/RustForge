use crate::error::DomainError;
use crate::value_objects::ids::UserId;

/// Repository for writing audit log entries.
#[async_trait::async_trait]
pub trait AuditRepository: Send + Sync {
    /// Write a new audit log entry.
    #[allow(clippy::too_many_arguments)]
    async fn create(
        &self,
        event_type: &str,
        aggregate_type: &str,
        aggregate_id: &uuid::Uuid,
        actor_id: Option<UserId>,
        changes: &serde_json::Value,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(), DomainError>;
}
