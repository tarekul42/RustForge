use crate::error::ApplicationError;
use sw_domain::aggregates::payment::{Payment, PaymentStatus};
use sw_domain::aggregates::refund_log::RefundLog;
use sw_domain::events::{DomainEvent, EventStore};
use sw_domain::repositories::enrollment::EnrollmentRepository;
use sw_domain::repositories::job::JobRepository;
use sw_domain::repositories::payment::PaymentRepository;
use sw_domain::repositories::refund_log::RefundLogRepository;
use sw_domain::repositories::workshop::WorkshopRepository;
use sw_domain::services::payment_gateway::PaymentGateway;
use sw_domain::value_objects::ids::PaymentId;
use tracing::instrument;

/// Application service for payment operations.
pub struct PaymentService<
    PR: PaymentRepository,
    ER: EnrollmentRepository,
    ES: EventStore,
    PG: PaymentGateway,
    WR: WorkshopRepository,
    JR: JobRepository,
    RL: RefundLogRepository,
> {
    payment_repo: PR,
    enrollment_repo: ER,
    event_store: ES,
    payment_gateway: PG,
    workshop_repo: WR,
    job_repo: JR,
    refund_log_repo: RL,
    // Minimum allowed amount tolerance in BDT (0.5 BDT = 50 paisa).
    amount_tolerance_cents: i64,
    /// Sqlx connection pool used for transactional writes.
    /// `None` in tests (skips transactional wrapping).
    pool: Option<sqlx::PgPool>,
}

impl<
    PR: PaymentRepository,
    ER: EnrollmentRepository,
    ES: EventStore,
    PG: PaymentGateway,
    WR: WorkshopRepository,
    JR: JobRepository,
    RL: RefundLogRepository,
> PaymentService<PR, ER, ES, PG, WR, JR, RL>
{
    /// Create a new `PaymentService`.
    /// Create a new `PaymentService`.
    #[allow(missing_docs, clippy::too_many_arguments)]
    pub fn new(
        payment_repo: PR,
        enrollment_repo: ER,
        event_store: ES,
        payment_gateway: PG,
        workshop_repo: WR,
        job_repo: JR,
        refund_log_repo: RL,
        pool: Option<sqlx::PgPool>,
    ) -> Self {
        Self {
            payment_repo,
            enrollment_repo,
            event_store,
            payment_gateway,
            workshop_repo,
            job_repo,
            refund_log_repo,
            amount_tolerance_cents: 50,
            pool,
        }
    }

    /// Handle a successful payment callback from the gateway.
    ///
    /// CAS: UNPAID → PAID
    /// Updates enrollment to Complete.
    /// Uses Postgres advisory lock to serialize concurrent calls (IPN + success-URL race).
    #[instrument(skip(self))]
    pub async fn success_payment(
        &self,
        transaction_id: &str,
        val_id: &str,
    ) -> Result<Payment, ApplicationError> {
        self.payment_repo
            .acquire_advisory_lock(val_id)
            .await
            .map_err(|e| ApplicationError::internal(format!("Failed to acquire lock: {e}")))?;

        let mut payment = self
            .payment_repo
            .find_by_transaction_id(transaction_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Payment", transaction_id))?;

        if payment.status() != PaymentStatus::Unpaid {
            return Ok(payment);
        }

        let validation = self
            .payment_gateway
            .validate_payment(val_id)
            .await
            .map_err(|e| ApplicationError::internal(format!("Payment validation failed: {e}")))?;

        if !validation.is_valid {
            return Err(ApplicationError::internal(
                "Payment validation returned invalid status",
            ));
        }

        if let Some(ref gw_amount) = validation.amount {
            let gw_cents = parse_amount_cents(gw_amount);
            let diff = (payment.amount().cents() - gw_cents).abs();
            if diff > self.amount_tolerance_cents {
                return Err(ApplicationError::internal(format!(
                    "Amount mismatch: expected {} cents, gateway returned {} cents",
                    payment.amount().cents(),
                    gw_cents
                )));
            }
        }

        let mut enrollment;

        if let Some(ref pool) = self.pool {
            let mut tx = match pool.begin().await {
                Ok(tx) => tx,
                Err(e) => {
                    return Err(ApplicationError::internal(format!(
                        "failed to begin transaction: {e}"
                    )));
                }
            };

            let updated = sqlx::query_scalar::<_, bool>(
                "UPDATE payments SET status = 'paid', updated_at = NOW() WHERE id = $1 AND status = 'unpaid'",
            )
            .bind(payment.id().into_uuid())
            .fetch_one(&mut *tx)
            .await
            .unwrap_or(false);

            if !updated {
                let _ = tx.rollback().await;
                return Ok(payment);
            }

            payment.set_status(PaymentStatus::Paid);
            payment.set_payment_gateway_data(Some(validation.raw_data));

            if let Err(e) = sqlx::query!(
                "UPDATE payments SET payment_gateway_data = $2, status = 'paid', updated_at = NOW() WHERE id = $1",
                payment.id().into_uuid(),
                payment.payment_gateway_data(),
            )
            .execute(&mut *tx)
            .await
            {
                let _ = tx.rollback().await;
                return Err(ApplicationError::internal(format!("failed to update payment: {e}")));
            }

            enrollment = self
                .enrollment_repo
                .find_by_id(payment.enrollment_id())
                .await?
                .ok_or_else(|| ApplicationError::not_found("Enrollment", payment.enrollment_id()))?;

            let event = enrollment
                .complete()
                .map_err(|e| ApplicationError::internal(e.to_string()))?;

            let enrollment_updated = sqlx::query_scalar::<_, bool>(
                "UPDATE enrollments SET status = 'complete', updated_at = NOW() WHERE id = $1 AND status = 'pending'",
            )
            .bind(enrollment.id().into_uuid())
            .fetch_one(&mut *tx)
            .await
            .unwrap_or(false);

            if !enrollment_updated {
                let _ = tx.rollback().await;
                return Err(ApplicationError::conflict(
                    "Enrollment status changed before payment completion",
                ));
            }

            self.publish_event_in_tx(&mut tx, event).await?;
            self.publish_event_in_tx(
                &mut tx,
                DomainEvent::PaymentStatusChanged {
                    payment_id: payment.id(),
                    from: "unpaid",
                    to: "paid",
                },
            )
            .await?;

            match tx.commit().await {
                Ok(_) => {}
                Err(e) => {
                    return Err(ApplicationError::internal(format!(
                        "failed to commit transaction: {e}"
                    )));
                }
            }
        } else {
            let updated = self
                .payment_repo
                .update_status_cas(payment.id(), "unpaid", "paid")
                .await?;
            if !updated {
                return Ok(payment);
            }

            payment.set_status(PaymentStatus::Paid);
            payment.set_payment_gateway_data(Some(validation.raw_data));

            enrollment = self
                .enrollment_repo
                .find_by_id(payment.enrollment_id())
                .await?
                .ok_or_else(|| ApplicationError::not_found("Enrollment", payment.enrollment_id()))?;

            let event = enrollment
                .complete()
                .map_err(|e| ApplicationError::internal(e.to_string()))?;

            let enrollment_updated = self
                .enrollment_repo
                .update_status_cas(enrollment.id(), "pending", enrollment.status().as_str())
                .await?;
            if !enrollment_updated {
                return Err(ApplicationError::conflict(
                    "Enrollment status changed before payment completion",
                ));
            }

            self.payment_repo.update(&payment).await?;
            self.publish_event(event).await?;
            self.publish_event(            DomainEvent::PaymentStatusChanged {
                payment_id: payment.id(),
                from: "unpaid",
                to: "paid",
            })
            .await?;
        }

        let invoice_payload = self.build_invoice_payload(&enrollment, &payment).await?;
        if let Err(e) = self
            .job_repo
            .enqueue("generate_invoice", &invoice_payload, None)
            .await
        {
            tracing::error!(error = %e, "Failed to enqueue invoice job");
        }

        Ok(payment)
    }

    /// Handle a failed payment callback.
    ///
    /// CAS: UNPAID → FAILED
    /// Updates enrollment to Failed, releases seat.
    #[instrument(skip(self))]
    pub async fn fail_payment(&self, transaction_id: &str) -> Result<Payment, ApplicationError> {
        let payment = self
            .payment_repo
            .find_by_transaction_id(transaction_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Payment", transaction_id))?;

        if payment.status() != PaymentStatus::Unpaid {
            return Ok(payment);
        }

        let updated = self
            .payment_repo
            .update_status_cas(payment.id(), "unpaid", "failed")
            .await?;
        if !updated {
            return Ok(payment);
        }

        let mut payment = payment;
        payment.set_status(PaymentStatus::Failed);

        let enrollment = self
            .enrollment_repo
            .find_by_id(payment.enrollment_id())
            .await?
            .ok_or_else(|| ApplicationError::not_found("Enrollment", payment.enrollment_id()))?;

        let mut enrollment = enrollment;
        let event = enrollment
            .fail()
            .map_err(|e| ApplicationError::internal(e.to_string()))?;

        let enrollment_updated = self
            .enrollment_repo
            .update_status_cas(enrollment.id(), "pending", "failed")
            .await?;
        if !enrollment_updated {
            return Err(ApplicationError::conflict(
                "Enrollment status changed before payment failure",
            ));
        }

        self.workshop_repo
            .release_seat_atomic(enrollment.workshop_id())
            .await?;
        self.publish_event(event).await?;
        Ok(payment)
    }

    /// Handle a cancelled payment callback.
    ///
    /// CAS: UNPAID → CANCELLED
    /// Updates enrollment to Cancelled, releases seat.
    #[instrument(skip(self))]
    pub async fn cancel_payment(&self, transaction_id: &str) -> Result<Payment, ApplicationError> {
        let payment = self
            .payment_repo
            .find_by_transaction_id(transaction_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Payment", transaction_id))?;

        if payment.status() != PaymentStatus::Unpaid {
            return Ok(payment);
        }

        let updated = self
            .payment_repo
            .update_status_cas(payment.id(), "unpaid", "cancelled")
            .await?;
        if !updated {
            return Ok(payment);
        }

        let mut payment = payment;
        payment.set_status(PaymentStatus::Cancelled);

        let enrollment = self
            .enrollment_repo
            .find_by_id(payment.enrollment_id())
            .await?
            .ok_or_else(|| ApplicationError::not_found("Enrollment", payment.enrollment_id()))?;

        let mut enrollment = enrollment;
        let event = enrollment
            .cancel()
            .map_err(|e| ApplicationError::internal(e.to_string()))?;

        let enrollment_updated = self
            .enrollment_repo
            .update_status_cas(enrollment.id(), "pending", "cancelled")
            .await?;
        if !enrollment_updated {
            return Err(ApplicationError::conflict(
                "Enrollment status changed before payment cancellation",
            ));
        }

        self.workshop_repo
            .release_seat_atomic(enrollment.workshop_id())
            .await?;
        self.publish_event(event).await?;
        Ok(payment)
    }

    /// Process an IPN notification from the gateway.
    ///
    /// Verifies the IPN signature, then applies the appropriate status
    /// transition based on the gateway's status field.
    #[instrument(skip(self))]
    pub async fn handle_ipn(
        &self,
        data: &std::collections::HashMap<String, String>,
    ) -> Result<(), ApplicationError> {
        if !self.payment_gateway.verify_ipn_signature(data) {
            return Err(ApplicationError::internal(
                "IPN signature verification failed",
            ));
        }

        let transaction_id = data
            .get("tran_id")
            .or_else(|| data.get("transaction_id"))
            .ok_or_else(|| ApplicationError::validation("Missing transaction_id in IPN data"))?;

        let val_id = data
            .get("val_id")
            .ok_or_else(|| ApplicationError::validation("Missing val_id in IPN data"))?;

        let status = data.get("status").map(|s| s.as_str()).unwrap_or("");

        match status {
            "VALID" | "VALIDATED" => {
                self.success_payment(transaction_id, val_id).await?;
            }
            "FAILED" => {
                self.fail_payment(transaction_id).await?;
            }
            "CANCELLED" => {
                self.cancel_payment(transaction_id).await?;
            }
            _ => {
                return Err(ApplicationError::internal(format!(
                    "Unknown IPN status: {status}"
                )));
            }
        }

        Ok(())
    }

    /// Process a refund for a paid payment.
    ///
    /// CAS: PAID → REFUNDED
    /// Cancels enrollment, releases seat.
    #[instrument(skip(self))]
    pub async fn refund(
        &self,
        payment_id: PaymentId,
        reason: String,
    ) -> Result<Payment, ApplicationError> {
        let payment = self
            .payment_repo
            .find_by_id(payment_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Payment", payment_id))?;

        if payment.status() != PaymentStatus::Paid {
            return Err(ApplicationError::conflict(format!(
                "Cannot refund payment with status '{}'",
                payment.status().as_str()
            )));
        }

        let updated = self
            .payment_repo
            .update_status_cas(payment.id(), "paid", "refunded")
            .await?;
        if !updated {
            return Err(ApplicationError::conflict(
                "Payment already refunded or status changed",
            ));
        }

        let mut payment = payment;
        payment.set_status(PaymentStatus::Refunded);

        let enrollment = self
            .enrollment_repo
            .find_by_id(payment.enrollment_id())
            .await?
            .ok_or_else(|| ApplicationError::not_found("Enrollment", payment.enrollment_id()))?;

        let mut enrollment = enrollment;
        let event = enrollment
            .cancel_refund()
            .map_err(|e| ApplicationError::internal(e.to_string()))?;

        let enrollment_updated = self
            .enrollment_repo
            .update_status_cas(enrollment.id(), "complete", "cancelled")
            .await?;
        if !enrollment_updated {
            return Err(ApplicationError::conflict(
                "Enrollment status changed before refund",
            ));
        }

        self.workshop_repo
            .release_seat_atomic(enrollment.workshop_id())
            .await?;
        self.publish_event(event).await?;
        self.publish_event(DomainEvent::PaymentRefunded {
            payment_id: payment.id(),
            reason: reason.clone(),
        })
        .await?;

        let refund_log = RefundLog::new(payment.id(), payment.amount().cents(), reason);
        self.refund_log_repo.create(&refund_log).await?;

        Ok(payment)
    }

    /// Find a payment by ID.
    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: PaymentId) -> Result<Option<Payment>, ApplicationError> {
        self.payment_repo
            .find_by_id(id)
            .await
            .map_err(ApplicationError::from)
    }

    async fn publish_event(&self, event: DomainEvent) -> Result<(), ApplicationError> {
        self.event_store
            .publish(&event, None)
            .await
            .map_err(|e| ApplicationError::internal(format!("failed to publish event: {e}")))
    }

    async fn publish_event_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        event: DomainEvent,
    ) -> Result<(), ApplicationError> {
        let event_type = event.event_type();
        let aggregate_type = event.aggregate_type();
        let aggregate_id = super::enrollment::aggregate_id_from_event(&event);
        let changes = serde_json::to_value(&event).unwrap_or_default();

        sqlx::query(
            r#"INSERT INTO audit_logs (event_type, aggregate_type, aggregate_id, actor_id, ip_address, user_agent, changes)
               VALUES ($1, $2, $3, NULL, NULL::inet, NULL, $4)"#,
        )
        .bind(event_type)
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(changes)
        .execute(&mut **tx)
        .await
        .map_err(|e| ApplicationError::internal(format!("failed to publish event: {e}")))?;
        Ok(())
    }

    async fn build_invoice_payload(
        &self,
        _enrollment: &sw_domain::aggregates::enrollment::Enrollment,
        payment: &Payment,
    ) -> Result<serde_json::Value, ApplicationError> {
        Ok(serde_json::json!({
            "payment_id": payment.id(),
        }))
    }
}

/// Parse an amount string from the gateway (e.g. "500.00") into cents.
fn parse_amount_cents(amount: &str) -> i64 {
    let parts: Vec<&str> = amount.split('.').collect();
    let dollars: i64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let cents: i64 = if parts.len() > 1 {
        let c = &parts[1][..parts[1].len().min(2)];
        format!("{:<02}", c)[..2].parse().unwrap_or(0)
    } else {
        0
    };
    dollars * 100 + cents
}
