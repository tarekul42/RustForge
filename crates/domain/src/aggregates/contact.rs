use crate::events::DomainEvent;
use crate::value_objects::Email;
use crate::value_objects::ids::ContactId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Aggregate root for a contact form submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// Unique identifier for this contact record.
    pub id: ContactId,
    /// The submitter's full name (max 100 chars).
    pub name: String,
    /// The submitter's email address.
    pub email: Email,
    /// Subject line (max 200 chars).
    pub subject: String,
    /// Message body (max 5000 chars).
    pub message: String,
    /// Whether an admin has read this submission.
    pub is_read: bool,
    /// Timestamp of creation.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last update.
    pub updated_at: DateTime<Utc>,
}

impl Contact {
    /// Create a new contact form submission.
    ///
    /// Validates field length constraints and returns the contact record
    /// along with a `ContactCreated` domain event.
    pub fn new(
        name: String,
        email: Email,
        subject: String,
        message: String,
    ) -> Result<(Self, DomainEvent), crate::error::DomainError> {
        if name.is_empty() || name.len() > 100 {
            return Err(crate::error::DomainError::validation(
                "Name must be between 1 and 100 characters",
            ));
        }
        if subject.is_empty() || subject.len() > 200 {
            return Err(crate::error::DomainError::validation(
                "Subject must be between 1 and 200 characters",
            ));
        }
        if message.is_empty() || message.len() > 5000 {
            return Err(crate::error::DomainError::validation(
                "Message must be between 1 and 5000 characters",
            ));
        }
        let now = Utc::now();
        let contact = Self {
            id: ContactId::new(),
            name,
            email,
            subject,
            message,
            is_read: false,
            created_at: now,
            updated_at: now,
        };
        let event = DomainEvent::ContactCreated {
            contact_id: contact.id,
        };
        Ok((contact, event))
    }

    /// Mark this contact submission as read.
    pub fn mark_read(&mut self) {
        self.is_read = true;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_contact_unread() {
        let email = Email::new("user@example.com").unwrap();
        let (contact, _) = Contact::new(
            "Alice".to_string(),
            email,
            "Question".to_string(),
            "Great workshop!".to_string(),
        )
        .unwrap();
        assert!(!contact.is_read);
    }

    #[test]
    fn mark_read_sets_flag() {
        let email = Email::new("user@example.com").unwrap();
        let (mut contact, _) = Contact::new(
            "Alice".to_string(),
            email,
            "Question".to_string(),
            "Great workshop!".to_string(),
        )
        .unwrap();
        contact.mark_read();
        assert!(contact.is_read);
    }

    #[test]
    fn name_too_long_fails() {
        let email = Email::new("user@example.com").unwrap();
        let result = Contact::new(
            "x".repeat(101),
            email,
            "Subject".to_string(),
            "Message".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn empty_name_fails() {
        let email = Email::new("user@example.com").unwrap();
        let result = Contact::new(
            "".to_string(),
            email,
            "Subject".to_string(),
            "Message".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn subject_too_long_fails() {
        let email = Email::new("user@example.com").unwrap();
        let result = Contact::new(
            "Alice".to_string(),
            email,
            "x".repeat(201),
            "Message".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn message_too_long_fails() {
        let email = Email::new("user@example.com").unwrap();
        let result = Contact::new(
            "Alice".to_string(),
            email,
            "Subject".to_string(),
            "x".repeat(5001),
        );
        assert!(result.is_err());
    }
}
