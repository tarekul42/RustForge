use serde::{Deserialize, Serialize};
use std::fmt;

/// A validated email address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    /// Create a new Email after basic validation.
    pub fn new(value: impl Into<String>) -> Result<Self, crate::error::DomainError> {
        let value = value.into();
        let trimmed = value.trim().to_lowercase();
        if trimmed.is_empty() {
            return Err(crate::error::DomainError::validation(
                "Email cannot be empty",
            ));
        }
        if !trimmed.contains('@') {
            return Err(crate::error::DomainError::validation(
                "Email must contain '@'",
            ));
        }
        let parts: Vec<&str> = trimmed.splitn(2, '@').collect();
        if parts[0].is_empty() || parts[1].is_empty() {
            return Err(crate::error::DomainError::validation(
                "Email must have both local and domain parts",
            ));
        }
        Ok(Self(trimmed))
    }

    /// Access the inner email string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the email, returning the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_email() {
        let email = Email::new("test@example.com").unwrap();
        assert_eq!(email.as_str(), "test@example.com");
    }

    #[test]
    fn email_is_lowercased() {
        let email = Email::new("Test@Example.COM").unwrap();
        assert_eq!(email.as_str(), "test@example.com");
    }

    #[test]
    fn empty_email_fails() {
        assert!(Email::new("").is_err());
    }

    #[test]
    fn no_at_sign_fails() {
        assert!(Email::new("notanemail").is_err());
    }

    #[test]
    fn missing_local_part_fails() {
        assert!(Email::new("@domain.com").is_err());
    }

    #[test]
    fn missing_domain_fails() {
        assert!(Email::new("user@").is_err());
    }

    #[test]
    fn valid_email_with_plus_sign() {
        let email = Email::new("user+tag@example.com").unwrap();
        assert_eq!(email.as_str(), "user+tag@example.com");
    }

    #[test]
    fn valid_email_with_subdomain() {
        let email = Email::new("user@sub.example.com").unwrap();
        assert_eq!(email.as_str(), "user@sub.example.com");
    }

    #[test]
    fn email_display_shows_normalized() {
        let email = Email::new("User@Example.COM").unwrap();
        assert_eq!(email.to_string(), "user@example.com");
    }

    #[test]
    fn email_equality_is_case_insensitive() {
        let a = Email::new("A@B.com").unwrap();
        let b = Email::new("a@b.com").unwrap();
        assert_eq!(a, b);
    }
}
