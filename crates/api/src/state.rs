use std::sync::Arc;
use sw_application::services::auth::AuthService;
use sw_application::services::category::CategoryService;
use sw_application::services::contact::ContactService;
use sw_application::services::enrollment::EnrollmentService;
use sw_application::services::level::LevelService;
use sw_application::services::payment::PaymentService;
use sw_application::services::review::ReviewService;
use sw_application::services::stats::StatsService;
use sw_application::services::user::UserAdminService;
use sw_application::services::workshop::WorkshopService;
use sw_infrastructure::payment::SslCommerzClient;
use sw_infrastructure::postgres::repos::category::PostgresCategoryRepository;
use sw_infrastructure::postgres::repos::contact::PostgresContactRepository;
use sw_infrastructure::postgres::repos::enrollment::PostgresEnrollmentRepository;
use sw_infrastructure::postgres::repos::event_store::PostgresEventStore;
use sw_infrastructure::postgres::repos::level::PostgresLevelRepository;
use sw_infrastructure::postgres::repos::otp::PostgresOtpRepository;
use sw_infrastructure::postgres::repos::payment::PostgresPaymentRepository;
use sw_infrastructure::postgres::repos::review::PostgresReviewRepository;
use sw_infrastructure::postgres::repos::session::PostgresSessionRepository;
use sw_infrastructure::postgres::repos::stats::PostgresStatsRepository;
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
    /// Contact form submission service.
    pub contact_service: Arc<ContactService<PostgresContactRepository, PostgresEventStore>>,
    /// Level management service.
    pub level_service: Arc<LevelService<PostgresLevelRepository, PostgresEventStore>>,
    /// Enrollment service.
    pub enrollment_service: Arc<
        EnrollmentService<
            PostgresEnrollmentRepository,
            PostgresPaymentRepository,
            PostgresWorkshopRepository,
            PostgresEventStore,
            SslCommerzClient,
        >,
    >,
    /// Payment service.
    pub payment_service: Arc<
        PaymentService<
            PostgresPaymentRepository,
            PostgresEnrollmentRepository,
            PostgresEventStore,
            SslCommerzClient,
            PostgresWorkshopRepository,
        >,
    >,
    /// Review service.
    pub review_service: Arc<
        ReviewService<
            PostgresReviewRepository,
            PostgresEnrollmentRepository,
            PostgresWorkshopRepository,
            PostgresEventStore,
        >,
    >,
    /// Stats service with moka cache.
    pub stats_service: Arc<StatsService<PostgresStatsRepository>>,
    /// Admin user management service.
    pub user_admin_service: Arc<UserAdminService<PostgresUserRepository, PostgresEventStore>>,
    /// Workshop management service.
    pub workshop_service: Arc<
        WorkshopService<
            PostgresWorkshopRepository,
            PostgresCategoryRepository,
            PostgresLevelRepository,
            PostgresEventStore,
        >,
    >,
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
        let enrollment_repo = PostgresEnrollmentRepository::new(pool.clone());
        let payment_repo = PostgresPaymentRepository::new(pool.clone());
        let event_store = PostgresEventStore::new(pool.clone());
        let review_repo = PostgresReviewRepository::new(pool.clone());
        let contact_repo = PostgresContactRepository::new(pool.clone());

        use sw_infrastructure::payment::SslCommerzConfig;
        let sslcommerz_config = SslCommerzConfig {
            store_id: config.payment.store_id.clone(),
            store_passwd: config.payment.store_passwd.clone(),
            base_url: config.payment.base_url.clone(),
            success_url: config.payment.success_url.clone(),
            fail_url: config.payment.fail_url.clone(),
            cancel_url: config.payment.cancel_url.clone(),
            ipn_url: config.payment.ipn_url.clone(),
        };
        let sslcommerz_client = SslCommerzClient::new(sslcommerz_config);

        Self {
            config: Arc::new(config.clone()),
            auth_service: Arc::new(AuthService::new(user_repo, session_repo, otp_repo)),
            category_service: Arc::new(CategoryService::new(
                category_repo,
                PostgresEventStore::new(pool.clone()),
            )),
            contact_service: Arc::new(ContactService::new(
                contact_repo,
                PostgresEventStore::new(pool.clone()),
            )),
            level_service: Arc::new(LevelService::new(
                level_repo,
                PostgresEventStore::new(pool.clone()),
            )),
            enrollment_service: Arc::new(EnrollmentService::new(
                enrollment_repo,
                payment_repo,
                PostgresWorkshopRepository::new(pool.clone()),
                event_store,
                sslcommerz_client,
            )),
            payment_service: Arc::new(PaymentService::new(
                PostgresPaymentRepository::new(pool.clone()),
                PostgresEnrollmentRepository::new(pool.clone()),
                PostgresEventStore::new(pool.clone()),
                SslCommerzClient::new(SslCommerzConfig {
                    store_id: config.payment.store_id.clone(),
                    store_passwd: config.payment.store_passwd.clone(),
                    base_url: config.payment.base_url.clone(),
                    success_url: config.payment.success_url.clone(),
                    fail_url: config.payment.fail_url.clone(),
                    cancel_url: config.payment.cancel_url.clone(),
                    ipn_url: config.payment.ipn_url.clone(),
                }),
                PostgresWorkshopRepository::new(pool.clone()),
            )),
            review_service: Arc::new(ReviewService::new(
                review_repo,
                PostgresEnrollmentRepository::new(pool.clone()),
                PostgresWorkshopRepository::new(pool.clone()),
                PostgresEventStore::new(pool.clone()),
            )),
            stats_service: Arc::new(StatsService::new(PostgresStatsRepository::new(
                pool.clone(),
            ))),
            user_admin_service: Arc::new(UserAdminService::new(
                PostgresUserRepository::new(pool.clone()),
                PostgresEventStore::new(pool.clone()),
            )),
            workshop_service: Arc::new(WorkshopService::new(
                workshop_repo,
                PostgresCategoryRepository::new(pool.clone()),
                PostgresLevelRepository::new(pool.clone()),
                PostgresEventStore::new(pool),
            )),
        }
    }
}
