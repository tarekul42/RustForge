use sqlx::PgPool;
use sw_domain::error::DomainError;
use sw_domain::repositories::session::SessionRepository;
use sw_domain::value_objects::ids::{SessionId, UserId};

/// SQLx-backed implementation of [`SessionRepository`].
pub struct PostgresSessionRepository {
    pool: PgPool,
}

impl PostgresSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SessionRepository for PostgresSessionRepository {
    async fn create(
        &self,
        session_id: SessionId,
        user_id: UserId,
        token_hash: &str,
        expires_at: &chrono::DateTime<chrono::Utc>,
    ) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO sessions (id, user_id, token_hash, expires_at)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(session_id.into_uuid())
        .bind(user_id.into_uuid())
        .bind(token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to create session: {e}")))?;
        Ok(())
    }

    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<(SessionId, UserId, chrono::DateTime<chrono::Utc>)>, DomainError> {
        let row = sqlx::query_as::<_, SessionRow>(
            r#"SELECT id, user_id, expires_at FROM sessions WHERE token_hash = $1"#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find session: {e}")))?;
        Ok(row.map(|r| {
            (
                SessionId::from_uuid(r.id),
                UserId::from_uuid(r.user_id),
                r.expires_at,
            )
        }))
    }

    async fn delete(&self, session_id: SessionId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(session_id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to delete session: {e}")))?;
        Ok(())
    }

    async fn delete_all_for_user(&self, user_id: UserId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id.into_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to delete sessions: {e}")))?;
        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<u64, DomainError> {
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to cleanup sessions: {e}")))?;
        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    expires_at: chrono::DateTime<chrono::Utc>,
}
