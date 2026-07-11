use sqlx::PgPool;
use sw_domain::error::DomainError;
use sw_domain::repositories::audit::AuditRepository;
use sw_domain::value_objects::ids::UserId;

/// SQLx-backed implementation of [`AuditRepository`].
pub struct PostgresAuditRepository {
    pool: PgPool,
}

impl PostgresAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AuditRepository for PostgresAuditRepository {
    async fn create(
        &self,
        event_type: &str,
        aggregate_type: &str,
        aggregate_id: &uuid::Uuid,
        actor_id: Option<UserId>,
        changes: &serde_json::Value,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO audit_logs (event_type, aggregate_type, aggregate_id, actor_id, changes, ip_address, user_agent)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(event_type)
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(actor_id.map(|a| a.into_uuid()))
        .bind(changes)
        .bind(ip_address)
        .bind(user_agent)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to create audit log: {e}")))?;
        Ok(())
    }
}
