use sqlx::PgPool;
use sw_domain::error::DomainError;
use sw_domain::repositories::otp::OtpRepository;

/// SQLx-backed implementation of [`OtpRepository`].
pub struct PostgresOtpRepository {
    pool: PgPool,
}

impl PostgresOtpRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OtpRepository for PostgresOtpRepository {
    async fn create(
        &self,
        email: &str,
        code_hash: &str,
        expires_at: &chrono::DateTime<chrono::Utc>,
    ) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO otp_codes (id, email, code_hash, expires_at)
               VALUES (gen_random_uuid(), $1, $2, $3)"#,
        )
        .bind(email)
        .bind(code_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to create otp code: {e}")))?;
        Ok(())
    }

    async fn find_by_email(
        &self,
        email: &str,
    ) -> Result<Option<(String, i32, chrono::DateTime<chrono::Utc>)>, DomainError> {
        let row = sqlx::query_as::<_, OtpRow>(
            r#"SELECT code_hash, attempts, expires_at
               FROM otp_codes WHERE email = $1
               ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find otp code: {e}")))?;
        Ok(row.map(|r| (r.code_hash, r.attempts, r.expires_at)))
    }

    async fn increment_attempts(&self, email: &str) -> Result<(), DomainError> {
        sqlx::query("UPDATE otp_codes SET attempts = attempts + 1 WHERE email = $1")
            .bind(email)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                DomainError::infrastructure(format!("failed to increment otp attempts: {e}"))
            })?;
        Ok(())
    }

    async fn delete(&self, email: &str) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM otp_codes WHERE email = $1")
            .bind(email)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::infrastructure(format!("failed to delete otp code: {e}")))?;
        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<u64, DomainError> {
        let result = sqlx::query("DELETE FROM otp_codes WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                DomainError::infrastructure(format!("failed to cleanup otp codes: {e}"))
            })?;
        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct OtpRow {
    code_hash: String,
    attempts: i32,
    expires_at: chrono::DateTime<chrono::Utc>,
}
