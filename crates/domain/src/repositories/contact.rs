use crate::aggregates::contact::Contact;
use crate::error::DomainError;
use crate::value_objects::ids::ContactId;

/// Repository for persisting and retrieving [`Contact`] aggregates.
#[async_trait::async_trait]
pub trait ContactRepository: Send + Sync {
    /// Persist a new contact form submission.
    async fn create(&self, contact: &Contact) -> Result<(), DomainError>;
    /// Find a contact record by its unique ID.
    async fn find_by_id(&self, id: ContactId) -> Result<Option<Contact>, DomainError>;
    /// List all contact records, ordered by creation date descending (most recent first).
    async fn list(&self, is_read: Option<bool>) -> Result<Vec<Contact>, DomainError>;
    /// Persist changes to an existing contact record (e.g. mark as read).
    async fn update(&self, contact: &Contact) -> Result<(), DomainError>;
    /// Delete a contact record by ID.
    async fn delete(&self, id: ContactId) -> Result<(), DomainError>;
}
