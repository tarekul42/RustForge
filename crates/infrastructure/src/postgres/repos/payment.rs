use sqlx::PgPool;
use sw_domain::aggregates::payment::{Payment, PaymentStatus};
use sw_domain::error::DomainError;
use sw_domain::repositories::payment::PaymentRepository;
use sw_domain::value_objects::ids::{EnrollmentId, PaymentId};
use sw_domain::value_objects::money::Money;

/// SQLx-backed implementation of [`PaymentRepository`].
pub struct PostgresPaymentRepository {
    pool: PgPool,
}

impl PostgresPaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl PaymentRepository for PostgresPaymentRepository {
    async fn create(&self, payment: &Payment) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO payments (id, enrollment_id, transaction_id, amount_cents,
               payment_gateway_data, invoice_url, status, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(payment.id.into_uuid())
        .bind(payment.enrollment_id.into_uuid())
        .bind(&payment.transaction_id)
        .bind(payment.amount.cents())
        .bind(&payment.payment_gateway_data)
        .bind(&payment.invoice_url)
        .bind(payment.status.as_str())
        .bind(payment.created_at)
        .bind(payment.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to create payment: {e}")))?;
        Ok(())
    }

    async fn find_by_id(&self, id: PaymentId) -> Result<Option<Payment>, DomainError> {
        let row = sqlx::query_as::<_, PaymentRow>(
            r#"SELECT id, enrollment_id, transaction_id, amount_cents,
                      payment_gateway_data, invoice_url, status, created_at, updated_at
               FROM payments WHERE id = $1"#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to find payment: {e}")))?;
        row.map(PaymentRow::into_domain).transpose()
    }

    async fn find_by_enrollment_id(
        &self,
        enrollment_id: EnrollmentId,
    ) -> Result<Option<Payment>, DomainError> {
        let row = sqlx::query_as::<_, PaymentRow>(
            r#"SELECT id, enrollment_id, transaction_id, amount_cents,
                      payment_gateway_data, invoice_url, status, created_at, updated_at
               FROM payments WHERE enrollment_id = $1"#,
        )
        .bind(enrollment_id.into_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::infrastructure(format!("failed to find payment by enrollment: {e}"))
        })?;
        row.map(PaymentRow::into_domain).transpose()
    }

    async fn find_by_transaction_id(
        &self,
        transaction_id: &str,
    ) -> Result<Option<Payment>, DomainError> {
        let row = sqlx::query_as::<_, PaymentRow>(
            r#"SELECT id, enrollment_id, transaction_id, amount_cents,
                      payment_gateway_data, invoice_url, status, created_at, updated_at
               FROM payments WHERE transaction_id = $1"#,
        )
        .bind(transaction_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DomainError::infrastructure(format!("failed to find payment by transaction: {e}"))
        })?;
        row.map(PaymentRow::into_domain).transpose()
    }

    async fn update(&self, payment: &Payment) -> Result<(), DomainError> {
        sqlx::query(
            r#"UPDATE payments SET amount_cents = $2, payment_gateway_data = $3,
               invoice_url = $4, status = $5, updated_at = $6
               WHERE id = $1"#,
        )
        .bind(payment.id.into_uuid())
        .bind(payment.amount.cents())
        .bind(&payment.payment_gateway_data)
        .bind(&payment.invoice_url)
        .bind(payment.status.as_str())
        .bind(payment.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::infrastructure(format!("failed to update payment: {e}")))?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct PaymentRow {
    id: uuid::Uuid,
    enrollment_id: uuid::Uuid,
    transaction_id: String,
    amount_cents: i64,
    payment_gateway_data: Option<serde_json::Value>,
    invoice_url: Option<String>,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl PaymentRow {
    fn into_domain(self) -> Result<Payment, DomainError> {
        let status = PaymentStatus::from_str(&self.status).ok_or_else(|| {
            DomainError::infrastructure(format!("invalid payment status: {}", self.status))
        })?;
        Ok(Payment {
            id: PaymentId::from_uuid(self.id),
            enrollment_id: EnrollmentId::from_uuid(self.enrollment_id),
            transaction_id: self.transaction_id,
            amount: Money::from_cents(self.amount_cents),
            payment_gateway_data: self.payment_gateway_data,
            invoice_url: self.invoice_url,
            status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
