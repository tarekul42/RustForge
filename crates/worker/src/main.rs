#![deny(missing_docs)]
#![allow(
    clippy::cast_precision_loss,
    clippy::doc_markdown,
    clippy::if_not_else,
    clippy::map_unwrap_or,
    clippy::missing_panics_doc,
    clippy::needless_raw_string_hashes,
    clippy::ref_option
)]
#![doc = "Worker crate: background job runner for async tasks (email, PDF, cleanup)."]

mod handlers;

use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use sw_domain::repositories::enrollment::EnrollmentRepository;
use sw_domain::repositories::job::JobRepository;
use sw_domain::repositories::payment::PaymentRepository;
use sw_domain::repositories::user::UserRepository;
use sw_domain::repositories::workshop::WorkshopRepository;
use sw_infrastructure::email::LettreEmailSender;
use sw_infrastructure::jobs::queue::QueueService;
use sw_infrastructure::object_store::s3::S3ObjectStore;
use sw_infrastructure::postgres::repos::enrollment::PostgresEnrollmentRepository;
use sw_infrastructure::postgres::repos::job::PostgresJobRepository;
use sw_infrastructure::postgres::repos::payment::PostgresPaymentRepository;
use sw_infrastructure::postgres::repos::user::PostgresUserRepository;
use sw_infrastructure::postgres::repos::workshop::PostgresWorkshopRepository;
use sw_shared::config::Config;
use sw_shared::object_store::ObjectStore;

use crate::handlers::{handle_generate_invoice, handle_send_email, run_periodic_cleanup};

/// Maximum time to wait for a current job to finish during shutdown.
const SHUTDOWN_JOB_TIMEOUT: Duration = Duration::from_secs(60);
/// Interval between periodic cleanup runs.
const CLEANUP_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes

struct WorkerContext {
    queue: QueueService,
    payment_repo: Arc<dyn PaymentRepository>,
    enrollment_repo: Arc<dyn EnrollmentRepository>,
    user_repo: Arc<dyn UserRepository>,
    workshop_repo: Arc<dyn WorkshopRepository>,
    object_store: Arc<dyn ObjectStore>,
    email_sender: Option<Arc<LettreEmailSender>>,
    invoices_bucket: String,
    pool: PgPool,
    worker_id: uuid::Uuid,
    poll_interval: Duration,
}

#[tokio::main]
async fn main() {
    let config = Config::load();
    sw_shared::logging::init(&config.observability);

    let pool = PgPool::connect(&config.database.url)
        .await
        .expect("failed to connect to database");

    let job_repo = Arc::new(PostgresJobRepository::new(pool.clone()));
    let queue = QueueService::new(
        job_repo as Arc<dyn JobRepository>,
        config.worker.base_backoff_seconds,
    );

    let payment_repo =
        Arc::new(PostgresPaymentRepository::new(pool.clone())) as Arc<dyn PaymentRepository>;
    let enrollment_repo =
        Arc::new(PostgresEnrollmentRepository::new(pool.clone())) as Arc<dyn EnrollmentRepository>;
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone())) as Arc<dyn UserRepository>;
    let workshop_repo =
        Arc::new(PostgresWorkshopRepository::new(pool.clone())) as Arc<dyn WorkshopRepository>;

    let email_sender: Option<Arc<LettreEmailSender>> = if !config.email.smtp_host.is_empty() {
        match LettreEmailSender::new(&config.email) {
            Ok(sender) => Some(Arc::new(sender)),
            Err(e) => {
                tracing::error!(error = %e, "failed to initialize email sender, email jobs will fail");
                None
            }
        }
    } else {
        tracing::warn!("SMTP not configured, email jobs will fail");
        None
    };

    let object_store: Option<Arc<S3ObjectStore>> = if config.s3.invoices_bucket.is_empty() {
        tracing::warn!("S3 invoices bucket not configured, PDF upload will fail");
        None
    } else {
        match S3ObjectStore::from_env().await {
            Ok(store) => Some(Arc::new(store)),
            Err(e) => {
                tracing::error!(error = %e, "failed to initialize S3 object store");
                None
            }
        }
    };

    let worker_id = if config.worker.worker_id.is_empty() {
        uuid::Uuid::new_v4()
    } else {
        uuid::Uuid::parse_str(&config.worker.worker_id).expect("invalid worker UUID in config")
    };

    let ctx = WorkerContext {
        queue,
        payment_repo,
        enrollment_repo,
        user_repo,
        workshop_repo,
        object_store: object_store
            .map(|o| o as Arc<dyn ObjectStore>)
            .unwrap_or_else(|| Arc::new(NoopObjectStore)),
        email_sender,
        invoices_bucket: config.s3.invoices_bucket.clone(),
        pool: pool.clone(),
        worker_id,
        poll_interval: Duration::from_millis(config.worker.poll_interval_ms),
    };

    tracing::info!(
        worker_id = %ctx.worker_id,
        poll_interval_ms = config.worker.poll_interval_ms,
        "Worker starting"
    );

    run_worker(&ctx).await;
}

/// A no-op ObjectStore used when S3 is not configured.
struct NoopObjectStore;

#[async_trait::async_trait]
impl ObjectStore for NoopObjectStore {
    async fn upload(
        &self,
        _bucket: &str,
        _key: &str,
        _body: &[u8],
        _content_type: &str,
    ) -> Result<String, sw_shared::object_store::ObjectStoreError> {
        Err(sw_shared::object_store::ObjectStoreError::UploadFailed(
            "S3 not configured".to_string(),
        ))
    }

    async fn delete(
        &self,
        _bucket: &str,
        _key: &str,
    ) -> Result<(), sw_shared::object_store::ObjectStoreError> {
        Ok(())
    }

    async fn presigned_get_url(
        &self,
        _bucket: &str,
        _key: &str,
        _expires_in_secs: u64,
    ) -> Result<String, sw_shared::object_store::ObjectStoreError> {
        Err(sw_shared::object_store::ObjectStoreError::PresignFailed(
            "S3 not configured".to_string(),
        ))
    }
}

async fn run_worker(ctx: &WorkerContext) {
    let mut shutdown = false;

    #[cfg(unix)]
    let mut term_signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("failed to register SIGTERM handler");
    #[cfg(unix)]
    let mut int_signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .expect("failed to register SIGINT handler");

    // Spawn periodic cleanup task
    let cleanup_pool = ctx.pool.clone();
    let cleanup_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
        interval.tick().await; // skip first immediate tick
        loop {
            interval.tick().await;
            run_periodic_cleanup(&cleanup_pool).await;
        }
    });

    loop {
        #[cfg(unix)]
        if term_signal.recv().await.is_some() || int_signal.recv().await.is_some() {
            if shutdown {
                tracing::info!("Forcing immediate shutdown");
                break;
            }
            tracing::info!("Shutdown signal received, finishing current job");
            shutdown = true;
        }

        if shutdown {
            break;
        }

        match ctx.queue.claim_next(ctx.worker_id).await {
            Ok(Some(job_info)) => {
                let job_id = job_info.id;
                tracing::info!(%job_id, job_type = %job_info.job_type, "Processing job");

                // Run job with a timeout for graceful shutdown
                let result =
                    tokio::time::timeout(SHUTDOWN_JOB_TIMEOUT, process_job(ctx, &job_info)).await;

                match result {
                    Ok(Ok(())) => {
                        if let Err(e) = ctx.queue.complete(job_id).await {
                            tracing::error!(%job_id, error = %e, "Failed to mark job completed");
                        } else {
                            tracing::info!(%job_id, "Job completed");
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::error!(%job_id, error = %e, "Job failed");
                        if let Ok(true) = ctx.queue.fail_with_backoff(job_id, &e.to_string()).await
                        {
                            tracing::info!(%job_id, "Job scheduled for retry");
                        }
                    }
                    Err(_elapsed) => {
                        tracing::warn!(%job_id, "Job timed out during shutdown");
                        if let Err(e) = ctx.queue.fail_with_backoff(job_id, "job timeout").await {
                            tracing::error!(%job_id, error = %e, "Failed to handle job timeout");
                        }
                    }
                }
            }
            Ok(None) => {
                tokio::time::sleep(ctx.poll_interval).await;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to claim job");
                tokio::time::sleep(ctx.poll_interval).await;
            }
        }
    }

    // Signal cleanup task to stop
    cleanup_handle.abort();

    tracing::info!("Worker shut down");
}

async fn process_job(
    ctx: &WorkerContext,
    job_info: &sw_domain::repositories::job::JobInfo,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match job_info.job_type.as_str() {
        "send_email" => Ok(handle_send_email(&job_info.payload, &ctx.email_sender).await?),
        "generate_invoice" => {
            let object_store: &dyn ObjectStore = &*ctx.object_store;
            Ok(handle_generate_invoice(
                &job_info.payload,
                &*ctx.payment_repo,
                &*ctx.enrollment_repo,
                &*ctx.user_repo,
                &*ctx.workshop_repo,
                object_store,
                &ctx.queue,
                &ctx.invoices_bucket,
            )
            .await?)
        }
        other => Err(format!("Unknown job type: {other}").into()),
    }
}
