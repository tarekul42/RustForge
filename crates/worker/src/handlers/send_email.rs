use std::sync::Arc;

use sw_domain::services::email_sender::{EmailError, EmailSender};
use sw_infrastructure::email::LettreEmailSender;

/// Payload for the `send_email` job type.
#[derive(serde::Deserialize)]
pub struct SendEmailPayload {
    pub to: String,
    pub subject: String,
    pub template: String,
    pub context: serde_json::Value,
}

/// Process a `send_email` job: render the template and send via SMTP.
pub async fn handle_send_email(
    payload: &serde_json::Value,
    email_sender: &Option<Arc<LettreEmailSender>>,
) -> Result<(), EmailError> {
    let parsed: SendEmailPayload = serde_json::from_value(payload.clone())
        .map_err(|e| EmailError::InvalidEmail(format!("invalid send_email payload: {e}")))?;

    match email_sender {
        Some(sender) => {
            let body = sender.render(&parsed.template, &parsed.context)?;
            sender.send(&parsed.to, &parsed.subject, &body).await?;
            Ok(())
        }
        None => Err(EmailError::SendFailed(
            "Email sender not configured".to_string(),
        )),
    }
}
