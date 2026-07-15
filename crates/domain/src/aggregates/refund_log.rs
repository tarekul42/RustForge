use crate::value_objects::ids::{PaymentId, RefundLogId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A record of a refund processed for a payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundLog {
    /// Unique identifier for this refund record.
    pub(crate) id: RefundLogId,
    /// The payment that was refunded.
    pub(crate) payment_id: PaymentId,
    /// The amount refunded, in cents.
    pub(crate) amount_cents: i64,
    /// Reason for the refund.
    pub(crate) reason: String,
    /// When the refund was processed.
    pub(crate) created_at: DateTime<Utc>,
}

impl RefundLog {
    /// Unique identifier for this refund record.
    pub fn id(&self) -> RefundLogId {
        self.id
    }

    /// The payment that was refunded.
    pub fn payment_id(&self) -> PaymentId {
        self.payment_id
    }

    /// The amount refunded, in cents.
    pub fn amount_cents(&self) -> i64 {
        self.amount_cents
    }

    /// Reason for the refund.
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// When the refund was processed.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

impl RefundLog {
    /// Restore a refund log from persisted data (used by infrastructure repos).
    pub fn from_parts(
        id: RefundLogId,
        payment_id: PaymentId,
        amount_cents: i64,
        reason: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            payment_id,
            amount_cents,
            reason,
            created_at,
        }
    }
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
