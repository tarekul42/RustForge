use crate::events::DomainEvent;
use crate::value_objects::ids::{EnrollmentId, PaymentId};
use crate::value_objects::money::Money;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The lifecycle status of a payment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentStatus {
    /// Payment has not yet been processed.
    Unpaid,
    /// Payment completed successfully.
    Paid,
    /// Payment was cancelled before processing.
    Cancelled,
    /// Payment processing failed.
    Failed,
    /// Payment was refunded after being paid.
    Refunded,
}

impl PaymentStatus {
    /// Return the lowercase string representation of this status.
    #[allow(clippy::should_implement_trait)]
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Unpaid => "unpaid",
            PaymentStatus::Paid => "paid",
            PaymentStatus::Cancelled => "cancelled",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Refunded => "refunded",
        }
    }

    /// Parse a status from its lowercase string representation.
    /// Returns `None` for unknown strings.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "unpaid" => Some(PaymentStatus::Unpaid),
            "paid" => Some(PaymentStatus::Paid),
            "cancelled" => Some(PaymentStatus::Cancelled),
            "failed" => Some(PaymentStatus::Failed),
            "refunded" => Some(PaymentStatus::Refunded),
            _ => None,
        }
    }
}

/// Aggregate root for a payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Unique identifier for this payment.
    pub id: PaymentId,
    /// The enrollment this payment is for.
    pub enrollment_id: EnrollmentId,
    /// External transaction ID from the payment gateway.
    pub transaction_id: String,
    /// Payment amount as a Money value.
    pub amount: Money,
    /// Raw response data from the payment gateway.
    pub payment_gateway_data: Option<serde_json::Value>,
    /// URL to the generated invoice.
    pub invoice_url: Option<String>,
    /// Current lifecycle status.
    pub status: PaymentStatus,
    /// Timestamp of creation.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update.
    pub updated_at: DateTime<Utc>,
}

impl Payment {
    /// Create a new unpaid payment for an enrollment.
    ///
    /// Returns the payment along with a `PaymentCreated` domain event.
    pub fn new(
        enrollment_id: EnrollmentId,
        transaction_id: String,
        amount: Money,
    ) -> (Self, DomainEvent) {
        let now = Utc::now();
        let payment = Self {
            id: PaymentId::new(),
            enrollment_id,
            transaction_id,
            amount,
            payment_gateway_data: None,
            invoice_url: None,
            status: PaymentStatus::Unpaid,
            created_at: now,
            updated_at: now,
        };
        let event = DomainEvent::PaymentCreated {
            payment_id: payment.id,
        };
        (payment, event)
    }

    /// Mark the payment as paid, optionally attaching gateway data.
    ///
    /// Returns an error if the current status is not Unpaid.
    pub fn mark_paid(
        &mut self,
        gateway_data: Option<serde_json::Value>,
    ) -> Result<DomainEvent, crate::error::DomainError> {
        match self.status {
            PaymentStatus::Unpaid => {
                self.status = PaymentStatus::Paid;
                self.payment_gateway_data = gateway_data;
                self.updated_at = Utc::now();
                Ok(DomainEvent::PaymentStatusChanged {
                    payment_id: self.id,
                    from: "unpaid",
                    to: "paid",
                })
            }
            _ => Err(crate::error::DomainError::invalid_transition(
                self.status.as_str(),
                "paid",
            )),
        }
    }

    /// Mark the payment as failed.
    ///
    /// Returns an error if the current status is not Unpaid.
    pub fn mark_failed(&mut self) -> Result<DomainEvent, crate::error::DomainError> {
        match self.status {
            PaymentStatus::Unpaid => {
                self.status = PaymentStatus::Failed;
                self.updated_at = Utc::now();
                Ok(DomainEvent::PaymentStatusChanged {
                    payment_id: self.id,
                    from: "unpaid",
                    to: "failed",
                })
            }
            _ => Err(crate::error::DomainError::invalid_transition(
                self.status.as_str(),
                "failed",
            )),
        }
    }

    /// Mark the payment as cancelled.
    ///
    /// Returns an error if the current status is not Unpaid.
    pub fn mark_cancelled(&mut self) -> Result<DomainEvent, crate::error::DomainError> {
        match self.status {
            PaymentStatus::Unpaid => {
                self.status = PaymentStatus::Cancelled;
                self.updated_at = Utc::now();
                Ok(DomainEvent::PaymentStatusChanged {
                    payment_id: self.id,
                    from: "unpaid",
                    to: "cancelled",
                })
            }
            _ => Err(crate::error::DomainError::invalid_transition(
                self.status.as_str(),
                "cancelled",
            )),
        }
    }

    /// Refund a paid payment.
    ///
    /// Returns an error if the current status is not Paid.
    pub fn refund(&mut self, reason: String) -> Result<DomainEvent, crate::error::DomainError> {
        match self.status {
            PaymentStatus::Paid => {
                self.status = PaymentStatus::Refunded;
                self.updated_at = Utc::now();
                Ok(DomainEvent::PaymentRefunded {
                    payment_id: self.id,
                    reason,
                })
            }
            _ => Err(crate::error::DomainError::invalid_transition(
                self.status.as_str(),
                "refunded",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::ids::EnrollmentId;
    use crate::value_objects::money::Money;

    fn make_payment() -> Payment {
        let (payment, _) = Payment::new(
            EnrollmentId::new(),
            "TXN_TEST".to_string(),
            Money::from_cents(5000),
        );
        payment
    }

    #[test]
    fn new_payment_is_unpaid() {
        let payment = make_payment();
        assert_eq!(payment.status, PaymentStatus::Unpaid);
    }

    #[test]
    fn mark_paid_succeeds() {
        let mut payment = make_payment();
        payment.mark_paid(None).unwrap();
        assert_eq!(payment.status, PaymentStatus::Paid);
    }

    #[test]
    fn mark_paid_twice_fails() {
        let mut payment = make_payment();
        payment.mark_paid(None).unwrap();
        assert!(payment.mark_paid(None).is_err());
    }

    #[test]
    fn refund_paid_succeeds() {
        let mut payment = make_payment();
        payment.mark_paid(None).unwrap();
        payment.refund("Customer request".to_string()).unwrap();
        assert_eq!(payment.status, PaymentStatus::Refunded);
    }

    #[test]
    fn refund_unpaid_fails() {
        let mut payment = make_payment();
        assert!(payment.refund("Test".to_string()).is_err());
    }

    #[test]
    fn mark_failed_succeeds() {
        let mut payment = make_payment();
        payment.mark_failed().unwrap();
        assert_eq!(payment.status, PaymentStatus::Failed);
    }

    #[test]
    fn mark_cancelled_succeeds() {
        let mut payment = make_payment();
        payment.mark_cancelled().unwrap();
        assert_eq!(payment.status, PaymentStatus::Cancelled);
    }
}
