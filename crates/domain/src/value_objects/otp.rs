use serde::{Deserialize, Serialize};
use std::fmt;

/// A 6-digit OTP code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtpCode(String);

impl OtpCode {
    /// Create a new OtpCode from a string, validating it's 6 digits.
    pub fn new(value: impl Into<String>) -> Result<Self, crate::error::DomainError> {
        let value = value.into();
        if value.len() != 6 || !value.chars().all(|c| c.is_ascii_digit()) {
            return Err(crate::error::DomainError::validation(
                "OTP code must be exactly 6 digits",
            ));
        }
        Ok(Self(value))
    }

    /// Access the inner code string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the OTP code, returning the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for OtpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never display the actual code - security
        write!(f, "[OTP REDACTED]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_otp() {
        let otp = OtpCode::new("123456").unwrap();
        assert_eq!(otp.as_str(), "123456");
    }

    #[test]
    fn too_short_fails() {
        assert!(OtpCode::new("12345").is_err());
    }

    #[test]
    fn too_long_fails() {
        assert!(OtpCode::new("1234567").is_err());
    }

    #[test]
    fn non_digit_fails() {
        assert!(OtpCode::new("12345a").is_err());
    }

    #[test]
    fn display_is_redacted() {
        let otp = OtpCode::new("123456").unwrap();
        assert_eq!(otp.to_string(), "[OTP REDACTED]");
    }
}
