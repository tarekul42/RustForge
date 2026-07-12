use std::sync::Arc;
use sw_application::services::auth::AuthService;
use sw_application::services::category::CategoryService;
use sw_application::services::level::LevelService;
use sw_application::services::user::UserAdminService;
use sw_application::services::workshop::WorkshopService;
use sw_infrastructure::postgres::repos::category::PostgresCategoryRepository;
use sw_infrastructure::postgres::repos::event_store::PostgresEventStore;
use sw_infrastructure::postgres::repos::level::PostgresLevelRepository;
use sw_infrastructure::postgres::repos::otp::PostgresOtpRepository;
use sw_infrastructure::postgres::repos::session::PostgresSessionRepository;
use sw_infrastructure::postgres::repos::user::PostgresUserRepository;
use sw_infrastructure::postgres::repos::workshop::PostgresWorkshopRepository;
use sw_shared::config::Config;

/// Shared application state accessible from all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Application configuration (immutable after startup).
    pub config: Arc<Config>,
    /// Auth / user service wired to infrastructure repos.
    pub auth_service:
        Arc<AuthService<PostgresUserRepository, PostgresSessionRepository, PostgresOtpRepository>>,
    /// Category management service.
    pub category_service: Arc<CategoryService<PostgresCategoryRepository, PostgresEventStore>>,
    /// Level management service.
    pub level_service: Arc<LevelService<PostgresLevelRepository, PostgresEventStore>>,
    /// Workshop management service.
    pub workshop_service: Arc<
        WorkshopService<
            PostgresWorkshopRepository,
            PostgresCategoryRepository,
            PostgresLevelRepository,
            PostgresEventStore,
        >,
    >,
    /// Admin user management service.
    pub user_admin_service:
        Arc<UserAdminService<PostgresUserRepository, PostgresEventStore>>,
}

impl AppState {
    /// Create a new `AppState` from a loaded `Config` and a DB pool.
    pub fn new(config: Config, pool: sqlx::PgPool) -> Self {
        let user_repo = PostgresUserRepository::new(pool.clone());
        let session_repo = PostgresSessionRepository::new(pool.clone());
        let otp_repo = PostgresOtpRepository::new(pool.clone());
        let category_repo = PostgresCategoryRepository::new(pool.clone());
        let level_repo = PostgresLevelRepository::new(pool.clone());
        let workshop_repo = PostgresWorkshopRepository::new(pool.clone());

        Self {
            config: Arc::new(config),
            auth_service: Arc::new(AuthService::new(user_repo, session_repo, otp_repo)),
            category_service: Arc::new(CategoryService::new(
                category_repo,
                PostgresEventStore::new(pool.clone()),
            )),
            level_service: Arc::new(LevelService::new(
                level_repo,
                PostgresEventStore::new(pool.clone()),
            )),
            workshop_service: Arc::new(WorkshopService::new(
                workshop_repo,
                PostgresCategoryRepository::new(pool.clone()),
                PostgresLevelRepository::new(pool.clone()),
                PostgresEventStore::new(pool.clone()),
            )),
            user_admin_service: Arc::new(UserAdminService::new(
                PostgresUserRepository::new(pool.clone()),
                PostgresEventStore::new(pool),
            )),
        }
    }
}
