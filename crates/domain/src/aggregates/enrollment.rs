use crate::events::DomainEvent;
use crate::value_objects::ids::{EnrollmentId, UserId, WorkshopId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The lifecycle status of an enrollment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnrollmentStatus {
    /// Awaiting payment or confirmation.
    Pending,
    /// Enrollment completed and confirmed.
    Complete,
    /// Enrollment cancelled by student or admin.
    Cancelled,
    /// Enrollment failed (e.g. payment declined).
    Failed,
}

impl EnrollmentStatus {
    /// Return the lowercase string representation of this status.
    #[allow(clippy::should_implement_trait)]
    pub fn as_str(&self) -> &'static str {
        match self {
            EnrollmentStatus::Pending => "pending",
            EnrollmentStatus::Complete => "complete",
            EnrollmentStatus::Cancelled => "cancelled",
            EnrollmentStatus::Failed => "failed",
        }
    }

    /// Parse a status from its lowercase string representation.
    /// Returns `None` for unknown strings.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(EnrollmentStatus::Pending),
            "complete" => Some(EnrollmentStatus::Complete),
            "cancelled" => Some(EnrollmentStatus::Cancelled),
            "failed" => Some(EnrollmentStatus::Failed),
            _ => None,
        }
    }
}

/// Aggregate root for an enrollment (links a user to a workshop).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enrollment {
    /// Unique identifier for this enrollment.
    pub id: EnrollmentId,
    /// The user who enrolled.
    pub user_id: UserId,
    /// The workshop being enrolled in.
    pub workshop_id: WorkshopId,
    /// Optional payment associated with this enrollment.
    pub payment_id: Option<crate::value_objects::ids::PaymentId>,
    /// Number of students covered by this enrollment (1 for individual).
    pub student_count: i32,
    /// Current lifecycle status.
    pub status: EnrollmentStatus,
    /// Timestamp of creation.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update.
    pub updated_at: DateTime<Utc>,
}

impl Enrollment {
    /// Create a new pending enrollment for a user in a workshop.
    ///
    /// Returns the enrollment along with an `EnrollmentCreated` domain event.
    pub fn new(
        user_id: UserId,
        workshop_id: WorkshopId,
        student_count: i32,
    ) -> (Self, DomainEvent) {
        let now = Utc::now();
        let enrollment = Self {
            id: EnrollmentId::new(),
            user_id,
            workshop_id,
            payment_id: None,
            student_count,
            status: EnrollmentStatus::Pending,
            created_at: now,
            updated_at: now,
        };
        let event = DomainEvent::EnrollmentCreated {
            enrollment_id: enrollment.id,
        };
        (enrollment, event)
    }

    /// Transition the enrollment from Pending to Complete.
    ///
    /// Returns an error if the current status is not Pending.
    pub fn complete(&mut self) -> Result<DomainEvent, crate::error::DomainError> {
        match self.status {
            EnrollmentStatus::Pending => {
                self.status = EnrollmentStatus::Complete;
                self.updated_at = Utc::now();
                Ok(DomainEvent::EnrollmentStatusChanged {
                    enrollment_id: self.id,
                    from: "pending",
                    to: "complete",
                })
            }
            _ => Err(crate::error::DomainError::invalid_transition(
                self.status.as_str(),
                "complete",
            )),
        }
    }

    /// Transition the enrollment from Pending to Cancelled.
    ///
    /// Returns an error if the current status is not Pending.
    pub fn cancel(&mut self) -> Result<DomainEvent, crate::error::DomainError> {
        match self.status {
            EnrollmentStatus::Pending => {
                self.status = EnrollmentStatus::Cancelled;
                self.updated_at = Utc::now();
                Ok(DomainEvent::EnrollmentCancelled {
                    enrollment_id: self.id,
                })
            }
            _ => Err(crate::error::DomainError::invalid_transition(
                self.status.as_str(),
                "cancelled",
            )),
        }
    }

    /// Cancel a completed enrollment (for refunds).
    /// Only allowed when the current status is `Complete`.
    pub fn cancel_refund(&mut self) -> Result<DomainEvent, crate::error::DomainError> {
        match self.status {
            EnrollmentStatus::Complete => {
                self.status = EnrollmentStatus::Cancelled;
                self.updated_at = Utc::now();
                Ok(DomainEvent::EnrollmentCancelled {
                    enrollment_id: self.id,
                })
            }
            _ => Err(crate::error::DomainError::invalid_transition(
                self.status.as_str(),
                "cancelled",
            )),
        }
    }

    /// Transition the enrollment from Pending to Failed.
    ///
    /// Returns an error if the current status is not Pending.
    pub fn fail(&mut self) -> Result<DomainEvent, crate::error::DomainError> {
        match self.status {
            EnrollmentStatus::Pending => {
                self.status = EnrollmentStatus::Failed;
                self.updated_at = Utc::now();
                Ok(DomainEvent::EnrollmentStatusChanged {
                    enrollment_id: self.id,
                    from: "pending",
                    to: "failed",
                })
            }
            _ => Err(crate::error::DomainError::invalid_transition(
                self.status.as_str(),
                "failed",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::ids::{UserId, WorkshopId};

    fn make_enrollment() -> Enrollment {
        let (enrollment, _) = Enrollment::new(UserId::new(), WorkshopId::new(), 1);
        enrollment
    }

    #[test]
    fn new_enrollment_is_pending() {
        let enrollment = make_enrollment();
        assert_eq!(enrollment.status, EnrollmentStatus::Pending);
    }

    #[test]
    fn complete_pending_succeeds() {
        let mut enrollment = make_enrollment();
        enrollment.complete().unwrap();
        assert_eq!(enrollment.status, EnrollmentStatus::Complete);
    }

    #[test]
    fn complete_cancelled_fails() {
        let mut enrollment = make_enrollment();
        enrollment.cancel().unwrap();
        assert!(enrollment.complete().is_err());
    }

    #[test]
    fn cancel_pending_succeeds() {
        let mut enrollment = make_enrollment();
        enrollment.cancel().unwrap();
        assert_eq!(enrollment.status, EnrollmentStatus::Cancelled);
    }

    #[test]
    fn cancel_complete_fails() {
        let mut enrollment = make_enrollment();
        enrollment.complete().unwrap();
        assert!(enrollment.cancel().is_err());
    }

    #[test]
    fn fail_pending_succeeds() {
        let mut enrollment = make_enrollment();
        enrollment.fail().unwrap();
        assert_eq!(enrollment.status, EnrollmentStatus::Failed);
    }

    #[test]
    fn cancel_refund_complete_succeeds() {
        let mut enrollment = make_enrollment();
        enrollment.complete().unwrap();
        enrollment.cancel_refund().unwrap();
        assert_eq!(enrollment.status, EnrollmentStatus::Cancelled);
    }

    #[test]
    fn cancel_refund_pending_fails() {
        let mut enrollment = make_enrollment();
        assert!(enrollment.cancel_refund().is_err());
    }

    #[test]
    fn fail_twice_fails() {
        let mut enrollment = make_enrollment();
        enrollment.fail().unwrap();
        assert!(enrollment.fail().is_err());
    }

    #[test]
    fn complete_already_complete_fails() {
        let mut enrollment = make_enrollment();
        enrollment.complete().unwrap();
        assert!(enrollment.complete().is_err());
    }

    #[test]
    fn fail_cancelled_fails() {
        let mut enrollment = make_enrollment();
        enrollment.cancel().unwrap();
        assert!(enrollment.fail().is_err());
    }

    #[test]
    fn fail_complete_fails() {
        let mut enrollment = make_enrollment();
        enrollment.complete().unwrap();
        assert!(enrollment.fail().is_err());
    }

    #[test]
    fn new_enrollment_has_no_payment() {
        let enrollment = make_enrollment();
        assert!(enrollment.payment_id.is_none());
    }

    #[test]
    fn new_enrollment_returns_created_event() {
        let (_, event) = Enrollment::new(UserId::new(), WorkshopId::new(), 1);
        assert!(matches!(event, DomainEvent::EnrollmentCreated { .. }));
    }
}
