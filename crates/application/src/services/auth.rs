use crate::error::ApplicationError;
use sw_domain::aggregates::user::{User, UserStatus};
use sw_domain::repositories::job::JobRepository;
use sw_domain::repositories::otp::OtpRepository;
use sw_domain::repositories::session::SessionRepository;
use sw_domain::repositories::user::UserRepository;
use sw_domain::value_objects::email::Email;
use sw_domain::value_objects::ids::{SessionId, UserId};
use sw_shared::crypto;
use tracing::instrument;

/// Application service for authentication flows.
pub struct AuthService<U: UserRepository, S: SessionRepository, O: OtpRepository, J: JobRepository>
{
    user_repo: U,
    session_repo: S,
    otp_repo: O,
    job_repo: J,
}

/// Result of a successful authentication (register or login).
#[derive(Debug)]
pub struct AuthResult {
    /// The authenticated user.
    pub user: User,
    /// The raw session token (to be set as a cookie).
    pub session_token: String,
    /// The session ID.
    pub session_id: SessionId,
    /// When the session expires.
    pub session_expires_at: chrono::DateTime<chrono::Utc>,
}

impl<U: UserRepository, S: SessionRepository, O: OtpRepository, J: JobRepository>
    AuthService<U, S, O, J>
{
    /// Create a new `AuthService`.
    pub fn new(user_repo: U, session_repo: S, otp_repo: O, job_repo: J) -> Self {
        Self {
            user_repo,
            session_repo,
            otp_repo,
            job_repo,
        }
    }

    /// Register a new user with email and password.
    ///
    /// Creates a user record and an initial session.
    #[instrument(skip(self, password), fields(email = %email))]
    pub async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: Option<&str>,
    ) -> Result<AuthResult, ApplicationError> {
        let email = Email::new(email).map_err(ApplicationError::from)?;

        if self
            .user_repo
            .find_by_email(email.as_str())
            .await?
            .is_some()
        {
            return Err(ApplicationError::conflict("Email already registered"));
        }

        let password_hash = {
            let pwd = password.to_string();
            tokio::task::spawn_blocking(move || crypto::hash_password(&pwd))
                .await
                .map_err(|e| ApplicationError::internal(format!("join error: {e}")))?
                .map_err(|e| ApplicationError::internal(format!("hashing error: {e}")))?
        };

        let name = display_name.unwrap_or("").to_string();
        let (user, _event) = User::new(email, name, Some(password_hash));
        self.user_repo.create(&user).await?;

        let session_token = hex::encode(crypto::generate_random_bytes(32));
        let token_hash = crypto::hash_token(&session_token);
        let session_id = SessionId::new();
        let session_expires_at = chrono::Utc::now() + chrono::Duration::days(7);

        self.session_repo
            .create(session_id, user.id(), &token_hash, &session_expires_at)
            .await?;

        Ok(AuthResult {
            user,
            session_token,
            session_id,
            session_expires_at,
        })
    }

    /// Log in with email and password.
    #[instrument(skip(self, password), fields(email = %email))]
    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResult, ApplicationError> {
        let user = self
            .user_repo
            .find_by_email(email)
            .await?
            .ok_or_else(|| ApplicationError::unauthorized("Invalid email or password"))?;

        let password_hash = user.password_hash().ok_or_else(|| {
            ApplicationError::unauthorized("Cannot log in with OAuth-only account")
        })?;

        let hash = password_hash.to_string();
        let pwd = password.to_string();
        let valid = tokio::task::spawn_blocking(move || crypto::verify_password(&pwd, &hash))
            .await
            .map_err(|e| ApplicationError::internal(format!("join error: {e}")))?
            .map_err(|e| ApplicationError::internal(format!("verify error: {e}")))?;

        if !valid {
            return Err(ApplicationError::unauthorized("Invalid email or password"));
        }

        if !user.is_verified() {
            return Err(ApplicationError::unauthorized(
                "Email not verified. Please verify via OTP.",
            ));
        }

        if user.status() != UserStatus::Active {
            return Err(ApplicationError::unauthorized("Account is not active"));
        }

        let session_token = hex::encode(crypto::generate_random_bytes(32));
        let token_hash = crypto::hash_token(&session_token);
        let session_id = SessionId::new();
        let session_expires_at = chrono::Utc::now() + chrono::Duration::days(7);

        self.session_repo
            .create(session_id, user.id(), &token_hash, &session_expires_at)
            .await?;

        Ok(AuthResult {
            user,
            session_token,
            session_id,
            session_expires_at,
        })
    }

    /// Log out by deleting the session.
    #[instrument(skip(self))]
    pub async fn logout(&self, session_id: SessionId) -> Result<(), ApplicationError> {
        self.session_repo.delete(session_id).await?;
        Ok(())
    }

    /// Look up a session by its token hash.
    #[instrument(skip(self))]
    pub async fn lookup_session(
        &self,
        token: &str,
    ) -> Result<Option<(SessionId, UserId)>, ApplicationError> {
        let token_hash = crypto::hash_token(token);
        let result = self.session_repo.find_by_token_hash(&token_hash).await?;

        match result {
            Some((session_id, user_id, expires_at)) if expires_at > chrono::Utc::now() => {
                Ok(Some((session_id, user_id)))
            }
            _ => Ok(None),
        }
    }

    /// Get a user by ID.
    #[instrument(skip(self))]
    pub async fn get_user(&self, user_id: UserId) -> Result<User, ApplicationError> {
        self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("User", user_id))
    }

    /// Update a user's display name and/or avatar URL.
    #[instrument(skip(self))]
    pub async fn update_profile(
        &self,
        user_id: UserId,
        display_name: Option<&str>,
        picture_url: Option<&str>,
    ) -> Result<User, ApplicationError> {
        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ApplicationError::not_found("User", user_id))?;

        if let Some(name) = display_name {
            user.set_name(name.to_string());
        }
        if let Some(url) = picture_url {
            user.set_picture_url(Some(url.to_string()));
        }
        user.touch();

        self.user_repo.update(&user).await?;
        Ok(user)
    }

    /// Request an OTP code to be sent to the user's email.
    #[instrument(skip(self), fields(email = %email))]
    pub async fn request_otp(&self, email: &str) -> Result<(), ApplicationError> {
        let user = self
            .user_repo
            .find_by_email(email)
            .await?
            .ok_or_else(|| ApplicationError::not_found("User", email))?;

        let otp_code = format!("{:06}", rand::random::<u32>() % 1_000_000);
        let code_hash = crypto::hash_token(&otp_code);
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(10);

        self.otp_repo.create(email, &code_hash, &expires_at).await?;

        let payload = serde_json::json!({
            "to": user.email(),
            "subject": "Your OTP Code",
            "template": "otp",
            "context": {
                "otp": otp_code,
            },
        });

        if let Err(e) = self.job_repo.enqueue("send_email", &payload, None).await {
            tracing::error!(error = %e, "Failed to enqueue OTP email job");
        }

        Ok(())
    }

    /// Verify an OTP code for the given email.
    ///
    /// Implements 5-attempt lockout. On success, marks the user as verified.
    #[instrument(skip(self, code), fields(email = %email))]
    pub async fn verify_otp(&self, email: &str, code: &str) -> Result<(), ApplicationError> {
        let stored = self
            .otp_repo
            .find_by_email(email)
            .await?
            .ok_or_else(|| ApplicationError::not_found("OTP code", email))?;

        let (code_hash, attempts, expires_at) = stored;

        if attempts >= 5 {
            return Err(ApplicationError::unauthorized(
                "Too many failed OTP attempts. Request a new code.",
            ));
        }

        if expires_at < chrono::Utc::now() {
            return Err(ApplicationError::unauthorized("OTP code has expired"));
        }

        let input_hash = crypto::hash_token(code);
        if input_hash != code_hash {
            self.otp_repo.increment_attempts(email).await?;
            return Err(ApplicationError::unauthorized("Invalid OTP code"));
        }

        let mut user = self
            .user_repo
            .find_by_email(email)
            .await?
            .ok_or_else(|| ApplicationError::not_found("User", email))?;

        user.set_is_verified(true);
        user.touch();
        self.user_repo.update(&user).await?;

        self.otp_repo.delete(email).await?;
        Ok(())
    }
}
