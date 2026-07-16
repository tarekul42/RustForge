use sqlx::PgPool;

/// Run all periodic cleanup tasks.
pub async fn run_periodic_cleanup(pool: &PgPool) {
    tracing::info!("Running periodic cleanup");

    if let Err(e) = cleanup_expired_otps(pool).await {
        tracing::error!(error = %e, "OTP cleanup failed");
    }

    if let Err(e) = cleanup_expired_sessions(pool).await {
        tracing::error!(error = %e, "Session cleanup failed");
    }

    if let Err(e) = cleanup_stale_locked_jobs(pool).await {
        tracing::error!(error = %e, "Stale job cleanup failed");
    }
}

/// Delete OTP codes that have expired.
async fn cleanup_expired_otps(pool: &PgPool) -> Result<(), sqlx::Error> {
    let deleted = sqlx::query("DELETE FROM otp_codes WHERE expires_at < NOW()")
        .execute(pool)
        .await?;
    if deleted.rows_affected() > 0 {
        tracing::info!(count = deleted.rows_affected(), "Deleted expired OTP codes");
    }
    Ok(())
}

/// Delete expired sessions.
async fn cleanup_expired_sessions(pool: &PgPool) -> Result<(), sqlx::Error> {
    let deleted = sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
        .execute(pool)
        .await?;
    if deleted.rows_affected() > 0 {
        tracing::info!(count = deleted.rows_affected(), "Deleted expired sessions");
    }
    Ok(())
}

/// Reset jobs stuck in `running` for more than 5 minutes (worker crash recovery).
async fn cleanup_stale_locked_jobs(pool: &PgPool) -> Result<(), sqlx::Error> {
    let updated = sqlx::query(
        r#"UPDATE jobs
           SET status = 'pending', locked_by = NULL, locked_at = NULL
           WHERE status = 'running' AND locked_at < NOW() - INTERVAL '5 minutes'"#,
    )
    .execute(pool)
    .await?;
    if updated.rows_affected() > 0 {
        tracing::info!(count = updated.rows_affected(), "Reset stale locked jobs");
    }
    Ok(())
}
