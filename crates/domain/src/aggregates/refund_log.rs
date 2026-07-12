use crate::value_objects::ids::{PaymentId, RefundLogId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A record of a refund processed for a payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundLog {
    /// Unique identifier for this refund record.
    pub id: RefundLogId,
    /// The payment that was refunded.
    pub payment_id: PaymentId,
    /// The amount refunded, in cents.
    pub amount_cents: i64,
    /// Reason for the refund.
    pub reason: String,
    /// When the refund was processed.
    pub created_at: DateTime<Utc>,
}

impl RefundLog {
    /// Create a new refund log entry.
    pub fn new(payment_id: PaymentId, amount_cents: i64, reason: String) -> Self {
        Self {
            id: RefundLogId::new(),
            payment_id,
            amount_cents,
            reason,
            created_at: Utc::now(),
        }
    }
}
