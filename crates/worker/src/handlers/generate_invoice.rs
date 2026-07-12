use sw_domain::error::DomainError;
use sw_domain::repositories::enrollment::EnrollmentRepository;
use sw_domain::repositories::payment::PaymentRepository;
use sw_domain::repositories::user::UserRepository;
use sw_domain::repositories::workshop::WorkshopRepository;
use sw_domain::value_objects::ids::PaymentId;
use sw_infrastructure::jobs::queue::QueueService;
use sw_infrastructure::pdf::{self, InvoiceData};
use sw_shared::object_store::ObjectStore;

/// Payload for the `generate_invoice` job type.
#[derive(serde::Deserialize)]
pub struct GenerateInvoicePayload {
    pub payment_id: PaymentId,
}

/// Errors that can occur during invoice generation.
#[derive(Debug, thiserror::Error)]
pub enum InvoiceError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
    #[error("PDF error: {0}")]
    Pdf(#[from] sw_infrastructure::pdf::PdfError),
    #[error("Object store error: {0}")]
    ObjectStore(#[from] sw_shared::object_store::ObjectStoreError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invoice already exists at {0}")]
    AlreadyExists(String),
}

/// Process a `generate_invoice` job.
///
/// 1. Load payment by ID (idempotency check: skip if `invoice_url` is set)
/// 2. Load enrollment, user, and workshop
/// 3. Generate PDF via `spawn_blocking` (CPU-bound printpdf)
/// 4. Upload PDF to S3
/// 5. Update `payment.invoice_url`
/// 6. Enqueue a `send_email` job with the invoice link
#[allow(clippy::too_many_arguments)]
pub async fn handle_generate_invoice(
    payload: &serde_json::Value,
    payment_repo: &dyn PaymentRepository,
    enrollment_repo: &dyn EnrollmentRepository,
    user_repo: &dyn UserRepository,
    workshop_repo: &dyn WorkshopRepository,
    object_store: &dyn ObjectStore,
    queue: &QueueService,
    invoices_bucket: &str,
) -> Result<(), InvoiceError> {
    let parsed: GenerateInvoicePayload = serde_json::from_value(payload.clone())?;

    // --- Idempotency check ---
    let payment = payment_repo
        .find_by_id(parsed.payment_id)
        .await?
        .ok_or_else(|| {
            DomainError::infrastructure(format!(
                "payment {} not found for invoice generation",
                parsed.payment_id
            ))
        })?;

    if let Some(ref url) = payment.invoice_url {
        tracing::info!(%parsed.payment_id, invoice_url = %url, "Invoice already exists, skipping");
        return Err(InvoiceError::AlreadyExists(url.clone()));
    }

    // --- Load related data ---
    let enrollment = enrollment_repo
        .find_by_id(payment.enrollment_id)
        .await?
        .ok_or_else(|| {
            DomainError::infrastructure(format!("enrollment {} not found", payment.enrollment_id))
        })?;

    let user = user_repo
        .find_by_id(enrollment.user_id)
        .await?
        .ok_or_else(|| {
            DomainError::infrastructure(format!("user {} not found", enrollment.user_id))
        })?;

    let workshop = workshop_repo
        .find_by_id(enrollment.workshop_id)
        .await?
        .ok_or_else(|| {
            DomainError::infrastructure(format!("workshop {} not found", enrollment.workshop_id))
        })?;

    // --- Generate PDF (CPU-bound, offload to blocking pool) ---
    let invoice_data = InvoiceData {
        transaction_id: payment.transaction_id.clone(),
        payment_id: payment.id,
        user_name: user.name.clone(),
        user_email: user.email.to_string(),
        workshop_title: workshop.title.clone(),
        amount: payment.amount,
    };

    let pdf_bytes = tokio::task::spawn_blocking(move || pdf::generate_invoice(&invoice_data))
        .await
        .map_err(|e| DomainError::infrastructure(format!("spawn_blocking failed: {e}")))?
        .map_err(InvoiceError::from)?;

    // --- Upload to S3 ---
    let key = format!("invoices/{}.pdf", payment.transaction_id);
    let invoice_url = object_store
        .upload(invoices_bucket, &key, &pdf_bytes, "application/pdf")
        .await?;

    // --- Update payment.invoice_url ---
    let mut payment = payment;
    payment.invoice_url = Some(invoice_url.clone());
    payment.updated_at = chrono::Utc::now();
    payment_repo.update(&payment).await?;

    tracing::info!(
        payment_id = %payment.id,
        invoice_url = %invoice_url,
        bucket = %invoices_bucket,
        "Invoice uploaded and payment updated"
    );

    // --- Enqueue follow-up email ---
    let email_payload = serde_json::json!({
        "to": user.email,
        "subject": format!("Invoice for {}", workshop.title),
        "template": "invoice",
        "context": {
            "user_name": user.name,
            "workshop_title": workshop.title,
            "transaction_id": payment.transaction_id,
            "amount": format!("{:.2}", payment.amount.cents() as f64 / 100.0),
            "invoice_url": invoice_url,
        },
    });

    if let Err(e) = queue.enqueue("send_email", &email_payload, None).await {
        tracing::error!(error = %e, %parsed.payment_id, "Failed to enqueue invoice email");
    }

    Ok(())
}
