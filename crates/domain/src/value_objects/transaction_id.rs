use serde::{Deserialize, Serialize};
use std::fmt;

/// A payment gateway transaction ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionId(String);

impl TransactionId {
    /// Create a new TransactionId.
    pub fn new(value: impl Into<String>) -> Result<Self, crate::error::DomainError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(crate::error::DomainError::validation(
                "Transaction ID cannot be empty",
            ));
        }
        Ok(Self(value.trim().to_string()))
    }

    /// Access the inner transaction ID string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the transaction ID, returning the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transaction_id() {
        let tid = TransactionId::new("TXN123456").unwrap();
        assert_eq!(tid.as_str(), "TXN123456");
    }

    #[test]
    fn empty_fails() {
        assert!(TransactionId::new("").is_err());
    }

    #[test]
    fn trimmed() {
        let tid = TransactionId::new("  TXN123  ").unwrap();
        assert_eq!(tid.as_str(), "TXN123");
    }
}
