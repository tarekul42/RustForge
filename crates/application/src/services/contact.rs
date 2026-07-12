use crate::error::ApplicationError;
use sw_domain::aggregates::contact::Contact;
use sw_domain::events::{DomainEvent, EventStore};
use sw_domain::repositories::contact::ContactRepository;
use sw_domain::value_objects::ids::ContactId;
use sw_domain::value_objects::Email;
use tracing::instrument;

/// Input for submitting a contact form.
#[derive(Debug)]
pub struct SubmitContactInput {
    /// The submitter's full name (max 100 chars).
    pub name: String,
    /// The submitter's email address.
    pub email: String,
    /// Subject line (max 200 chars).
    pub subject: String,
    /// Message body (max 5000 chars).
    pub message: String,
}

/// Application service for contact form submissions.
pub struct ContactService<CR: ContactRepository, ES: EventStore> {
    repo: CR,
    event_store: ES,
}

impl<CR: ContactRepository, ES: EventStore> ContactService<CR, ES> {
    /// Create a new `ContactService`.
    pub fn new(repo: CR, event_store: ES) -> Self {
        Self { repo, event_store }
    }

    /// Submit a contact form.
    #[instrument(skip(self))]
    pub async fn submit(&self, input: SubmitContactInput) -> Result<Contact, ApplicationError> {
        let email = Email::new(&input.email)?;

        let (contact, event) = Contact::new(input.name, email, input.subject, input.message)?;

        self.repo.create(&contact).await?;
        self.publish_event(event).await?;
        Ok(contact)
    }

    /// Find a contact record by ID.
    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: ContactId) -> Result<Contact, ApplicationError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Contact", id))
    }

    /// List contact submissions, optionally filtered by read status.
    #[instrument(skip(self))]
    pub async fn list(&self, is_read: Option<bool>) -> Result<Vec<Contact>, ApplicationError> {
        self.repo
            .list(is_read)
            .await
            .map_err(ApplicationError::from)
    }

    /// Mark a contact submission as read.
    #[instrument(skip(self))]
    pub async fn mark_read(&self, id: ContactId) -> Result<Contact, ApplicationError> {
        let mut contact = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Contact", id))?;

        contact.mark_read();
        self.repo.update(&contact).await?;
        Ok(contact)
    }

    /// Delete a contact record.
    #[instrument(skip(self))]
    pub async fn delete(&self, id: ContactId) -> Result<(), ApplicationError> {
        let _contact = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("Contact", id))?;

        self.repo.delete(id).await?;
        Ok(())
    }

    async fn publish_event(&self, event: DomainEvent) -> Result<(), ApplicationError> {
        self.event_store
            .publish(&event, None)
            .await
            .map_err(|e| ApplicationError::internal(format!("failed to publish event: {e}")))
    }
}
