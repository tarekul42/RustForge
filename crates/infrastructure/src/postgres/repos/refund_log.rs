use sqlx::PgPool;
use sw_domain::aggregates::refund_log::RefundLog;
use sw_domain::error::DomainError;
use sw_domain::repositories::refund_log::RefundLogRepository;

/// SQLx-backed implementation of [`RefundLogRepository`].
pub struct PostgresRefundLogRepository {
    pool: PgPool,
}

impl PostgresRefundLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RefundLogRepository for PostgresRefundLogRepository {
    async fn create(&self, log: &RefundLog) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO refund_logs (id, payment_id, amount_cents, reason, created_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(log.id.into_uuid())
        .bind(log.payment_id.into_uuid())
        .bind(log.amount_cents)
        .bind(&log.reason)
        .bind(log.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to create refund_log: {e}")))?;
        Ok(())
    }
}
