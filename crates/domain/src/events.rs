use crate::value_objects::ids::*;
use serde::{Deserialize, Serialize};

/// Events emitted by aggregate methods.
///
/// These are the single source of truth for all state changes
/// and are used to build the audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DomainEvent {
    /// A new user registered.
    UserRegistered {
        /// The ID of the registered user.
        user_id: UserId,
        /// The email address used at registration.
        email: crate::value_objects::Email,
    },
    /// A user's email was verified.
    UserVerified {
        /// The ID of the verified user.
        user_id: UserId,
    },
    /// A user changed their password.
    PasswordChanged {
        /// The ID of the user whose password changed.
        user_id: UserId,
    },
    /// A user's profile was updated.
    UserUpdated {
        /// The ID of the updated user.
        user_id: UserId,
    },
    /// A user account was deleted.
    UserDeleted {
        /// The ID of the deleted user.
        user_id: UserId,
    },
    /// A new category was created.
    CategoryCreated {
        /// The ID of the created category.
        category_id: CategoryId,
    },
    /// A category's details were updated.
    CategoryUpdated {
        /// The ID of the updated category.
        category_id: CategoryId,
    },
    /// A category was deleted.
    CategoryDeleted {
        /// The ID of the deleted category.
        category_id: CategoryId,
    },
    /// A new level was created.
    LevelCreated {
        /// The ID of the created level.
        level_id: LevelId,
    },
    /// A level was renamed.
    LevelUpdated {
        /// The ID of the updated level.
        level_id: LevelId,
    },
    /// A level was deleted.
    LevelDeleted {
        /// The ID of the deleted level.
        level_id: LevelId,
    },
    /// A new workshop was created.
    WorkshopCreated {
        /// The ID of the created workshop.
        workshop_id: WorkshopId,
    },
    /// A workshop's details were updated.
    WorkshopUpdated {
        /// The ID of the updated workshop.
        workshop_id: WorkshopId,
    },
    /// A workshop was deleted.
    WorkshopDeleted {
        /// The ID of the deleted workshop.
        workshop_id: WorkshopId,
    },
    /// A new enrollment was created.
    EnrollmentCreated {
        /// The ID of the created enrollment.
        enrollment_id: EnrollmentId,
    },
    /// An enrollment's status changed (e.g. pending → complete).
    EnrollmentStatusChanged {
        /// The ID of the enrollment.
        enrollment_id: EnrollmentId,
        /// The previous status.
        from: &'static str,
        /// The new status.
        to: &'static str,
    },
    /// An enrollment was cancelled.
    EnrollmentCancelled {
        /// The ID of the cancelled enrollment.
        enrollment_id: EnrollmentId,
    },
    /// A new payment was created.
    PaymentCreated {
        /// The ID of the created payment.
        payment_id: PaymentId,
    },
    /// A payment's status changed (e.g. unpaid → paid).
    PaymentStatusChanged {
        /// The ID of the payment.
        payment_id: PaymentId,
        /// The previous status.
        from: &'static str,
        /// The new status.
        to: &'static str,
    },
    /// A payment was refunded.
    PaymentRefunded {
        /// The ID of the refunded payment.
        payment_id: PaymentId,
        /// The reason for the refund.
        reason: String,
    },
    /// A new review was created.
    ReviewCreated {
        /// The ID of the created review.
        review_id: ReviewId,
    },
    /// A review was moderated (approved or rejected).
    ReviewModerated {
        /// The ID of the moderated review.
        review_id: ReviewId,
        /// The previous moderation status.
        from: &'static str,
        /// The new moderation status.
        to: &'static str,
    },
    /// A new contact form submission was received.
    ContactCreated {
        /// The ID of the created contact record.
        contact_id: ContactId,
    },
}

impl DomainEvent {
    /// Return the dot-separated event type string (e.g. `"user.registered"`).
    pub fn event_type(&self) -> &'static str {
        match self {
            DomainEvent::UserRegistered { .. } => "user.registered",
            DomainEvent::UserVerified { .. } => "user.verified",
            DomainEvent::PasswordChanged { .. } => "user.password_changed",
            DomainEvent::UserUpdated { .. } => "user.updated",
            DomainEvent::UserDeleted { .. } => "user.deleted",
            DomainEvent::CategoryCreated { .. } => "category.created",
            DomainEvent::CategoryUpdated { .. } => "category.updated",
            DomainEvent::CategoryDeleted { .. } => "category.deleted",
            DomainEvent::LevelCreated { .. } => "level.created",
            DomainEvent::LevelUpdated { .. } => "level.updated",
            DomainEvent::LevelDeleted { .. } => "level.deleted",
            DomainEvent::WorkshopCreated { .. } => "workshop.created",
            DomainEvent::WorkshopUpdated { .. } => "workshop.updated",
            DomainEvent::WorkshopDeleted { .. } => "workshop.deleted",
            DomainEvent::EnrollmentCreated { .. } => "enrollment.created",
            DomainEvent::EnrollmentStatusChanged { .. } => "enrollment.status_changed",
            DomainEvent::EnrollmentCancelled { .. } => "enrollment.cancelled",
            DomainEvent::PaymentCreated { .. } => "payment.created",
            DomainEvent::PaymentStatusChanged { .. } => "payment.status_changed",
            DomainEvent::PaymentRefunded { .. } => "payment.refunded",
            DomainEvent::ReviewCreated { .. } => "review.created",
            DomainEvent::ReviewModerated { .. } => "review.moderated",
            DomainEvent::ContactCreated { .. } => "contact.created",
        }
    }

    /// Return the aggregate type name this event belongs to (e.g. `"User"`).
    pub fn aggregate_type(&self) -> &'static str {
        match self {
            DomainEvent::UserRegistered { .. }
            | DomainEvent::UserVerified { .. }
            | DomainEvent::PasswordChanged { .. }
            | DomainEvent::UserUpdated { .. }
            | DomainEvent::UserDeleted { .. } => "User",
            DomainEvent::CategoryCreated { .. }
            | DomainEvent::CategoryUpdated { .. }
            | DomainEvent::CategoryDeleted { .. } => "Category",
            DomainEvent::LevelCreated { .. }
            | DomainEvent::LevelUpdated { .. }
            | DomainEvent::LevelDeleted { .. } => "Level",
            DomainEvent::WorkshopCreated { .. }
            | DomainEvent::WorkshopUpdated { .. }
            | DomainEvent::WorkshopDeleted { .. } => "Workshop",
            DomainEvent::EnrollmentCreated { .. }
            | DomainEvent::EnrollmentStatusChanged { .. }
            | DomainEvent::EnrollmentCancelled { .. } => "Enrollment",
            DomainEvent::PaymentCreated { .. }
            | DomainEvent::PaymentStatusChanged { .. }
            | DomainEvent::PaymentRefunded { .. } => "Payment",
            DomainEvent::ReviewCreated { .. } | DomainEvent::ReviewModerated { .. } => "Review",
            DomainEvent::ContactCreated { .. } => "Contact",
        }
    }
}

/// Contextual metadata captured alongside a domain event for audit trail purposes.
///
/// This information is extracted from the HTTP request at the API layer and
/// passed down through application services to the [`EventStore`].
#[derive(Debug, Clone, Default)]
pub struct AuditContext {
    /// The user who performed the action (if authenticated).
    pub actor_id: Option<crate::value_objects::ids::UserId>,
    /// The client's IP address.
    pub ip_address: Option<String>,
    /// The User-Agent header from the request.
    pub user_agent: Option<String>,
}

/// Repository trait for persisting domain events.
///
/// Implementations write events to durable storage (e.g. the `audit_logs` table)
/// to build an append-only event log. Optional [`AuditContext`] provides
/// actor, IP, and user-agent metadata for the audit trail.
#[async_trait::async_trait]
pub trait EventStore: Send + Sync {
    /// Persist a domain event with optional audit context.
    async fn publish(
        &self,
        event: &DomainEvent,
        context: Option<&AuditContext>,
    ) -> Result<(), crate::error::DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_user_registered() {
        let event = DomainEvent::UserRegistered {
            user_id: UserId::new(),
            email: crate::value_objects::Email::new("test@example.com").unwrap(),
        };
        assert_eq!(event.event_type(), "user.registered");
    }

    #[test]
    fn event_type_user_verified() {
        let event = DomainEvent::UserVerified {
            user_id: UserId::new(),
        };
        assert_eq!(event.event_type(), "user.verified");
    }

    #[test]
    fn event_type_password_changed() {
        let event = DomainEvent::PasswordChanged {
            user_id: UserId::new(),
        };
        assert_eq!(event.event_type(), "user.password_changed");
    }

    #[test]
    fn event_type_enrollment_created() {
        let event = DomainEvent::EnrollmentCreated {
            enrollment_id: EnrollmentId::new(),
        };
        assert_eq!(event.event_type(), "enrollment.created");
    }

    #[test]
    fn event_type_payment_created() {
        let event = DomainEvent::PaymentCreated {
            payment_id: PaymentId::new(),
        };
        assert_eq!(event.event_type(), "payment.created");
    }

    #[test]
    fn aggregate_type_user_events() {
        let event = DomainEvent::UserVerified {
            user_id: UserId::new(),
        };
        assert_eq!(event.aggregate_type(), "User");
    }

    #[test]
    fn aggregate_type_payment_events() {
        let event = DomainEvent::PaymentRefunded {
            payment_id: PaymentId::new(),
            reason: "test".to_string(),
        };
        assert_eq!(event.aggregate_type(), "Payment");
    }

    #[test]
    fn aggregate_type_contact_events() {
        let event = DomainEvent::ContactCreated {
            contact_id: ContactId::new(),
        };
        assert_eq!(event.aggregate_type(), "Contact");
    }

    #[test]
    fn event_debug_format() {
        let event = DomainEvent::UserRegistered {
            user_id: UserId::new(),
            email: crate::value_objects::Email::new("test@example.com").unwrap(),
        };
        let debug = format!("{event:?}");
        assert!(debug.contains("UserRegistered"));
    }

    #[test]
    fn audit_context_default() {
        let ctx = AuditContext::default();
        assert!(ctx.actor_id.is_none());
        assert!(ctx.ip_address.is_none());
        assert!(ctx.user_agent.is_none());
    }
}
