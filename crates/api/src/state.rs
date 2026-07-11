use std::sync::Arc;
use sw_application::services::auth::AuthService;
use sw_infrastructure::postgres::repos::otp::PostgresOtpRepository;
use sw_infrastructure::postgres::repos::session::PostgresSessionRepository;
use sw_infrastructure::postgres::repos::user::PostgresUserRepository;
use sw_shared::config::Config;

/// Shared application state accessible from all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Application configuration (immutable after startup).
    pub config: Arc<Config>,
    /// Auth / user service wired to infrastructure repos.
    pub auth_service:
        Arc<AuthService<PostgresUserRepository, PostgresSessionRepository, PostgresOtpRepository>>,
}

impl AppState {
    /// Create a new `AppState` from a loaded `Config` and a DB pool.
    pub fn new(config: Config, pool: sqlx::PgPool) -> Self {
        let user_repo = PostgresUserRepository::new(pool.clone());
        let session_repo = PostgresSessionRepository::new(pool.clone());
        let otp_repo = PostgresOtpRepository::new(pool);

        Self {
            config: Arc::new(config),
            auth_service: Arc::new(AuthService::new(user_repo, session_repo, otp_repo)),
        }
    }
}
