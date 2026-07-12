use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use sw_domain::services::email_sender::{EmailError, EmailSender};
use sw_shared::config::EmailConfig;
use tera::Tera;

/// Lettre + Tera implementation of [`EmailSender`].
pub struct LettreEmailSender {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    templates: Tera,
    from: Mailbox,
}

impl LettreEmailSender {
    /// Create a new `LettreEmailSender`.
    ///
    /// Loads all `.html` templates from the given directory at startup.
    /// Panics if the template directory cannot be read or if required
    /// templates are missing.
    pub fn new(config: &EmailConfig) -> Result<Self, EmailError> {
        let creds = Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());

        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
            .map_err(|e| EmailError::SendFailed(format!("SMTP relay setup failed: {e}")))?
            .port(config.smtp_port)
            .credentials(creds)
            .build();

        let pattern = format!("{}/**/*.html", config.template_dir);
        let templates = Tera::new(&pattern)
            .map_err(|e| EmailError::InvalidEmail(format!("template load failed: {e}")))?;

        let from: Mailbox = format!("{} <{}>", config.from_name, config.from_email)
            .parse()
            .map_err(|e| EmailError::InvalidEmail(format!("invalid from address: {e}")))?;

        Ok(Self {
            transport,
            templates,
            from,
        })
    }

    /// Render a template with the given context.
    pub fn render(
        &self,
        template: &str,
        context: &serde_json::Value,
    ) -> Result<String, EmailError> {
        let template_name = format!("{template}.html");
        let mut tera_ctx = tera::Context::new();
        if let Some(obj) = context.as_object() {
            for (key, val) in obj {
                tera_ctx.insert(key, val);
            }
        }
        self.templates
            .render(&template_name, &tera_ctx)
            .map_err(|e| EmailError::SendFailed(format!("template render failed: {e}")))
    }
}

#[async_trait::async_trait]
impl EmailSender for LettreEmailSender {
    async fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), EmailError> {
        let to_mailbox: Mailbox = to
            .parse()
            .map_err(|e| EmailError::InvalidEmail(format!("invalid to address: {e}")))?;

        let email = Message::builder()
            .from(self.from.clone())
            .to(to_mailbox)
            .subject(subject)
            .body(body.to_string())
            .map_err(|e| EmailError::SendFailed(format!("message build failed: {e}")))?;

        self.transport
            .send(email)
            .await
            .map_err(|e| EmailError::SendFailed(format!("send failed: {e}")))?;

        Ok(())
    }
}
