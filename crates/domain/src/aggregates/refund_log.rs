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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::ids::PaymentId;

    #[test]
    fn new_refund_log_stores_fields() {
        let log = RefundLog::new(PaymentId::new(), 5000, "Customer request".to_string());
        assert_eq!(log.amount_cents, 5000);
        assert_eq!(log.reason, "Customer request");
    }

    #[test]
    fn new_refund_log_has_id() {
        let log = RefundLog::new(PaymentId::new(), 1000, "refund".to_string());
        assert_ne!(log.id, RefundLogId::default());
    }

    #[test]
    fn refund_log_zero_amount() {
        let log = RefundLog::new(PaymentId::new(), 0, "test".to_string());
        assert_eq!(log.amount_cents, 0);
    }
}
