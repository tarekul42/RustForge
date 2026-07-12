/// Interface for sending emails.
///
/// This trait is defined in the domain layer so that application services
/// can depend on a port without coupling to any specific email provider.
/// The concrete implementation lives in the `infrastructure` crate.
#[async_trait::async_trait]
pub trait EmailSender: Send + Sync {
    /// Send a transactional email.
    async fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), EmailError>;
}

/// Errors returned by [`EmailSender`] operations.
#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    /// The email could not be sent.
    #[error("Failed to send email: {0}")]
    SendFailed(String),
    /// Invalid email address or formatting error.
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
}
