/// Errors that can occur in the application use-case layer.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ApplicationError {
    /// The requested resource was not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// A conflict (e.g. duplicate email during registration).
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Authentication or authorization failure.
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Validation of input failed.
    #[error("Validation error: {0}")]
    Validation(String),

    /// An unexpected internal error occurred.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// The resource is not available (e.g. workshop full).
    #[error("Unavailable: {0}")]
    Unavailable(String),
}

impl ApplicationError {
    /// Create a `NotFound` error.
    pub fn not_found(entity: &str, id: impl std::fmt::Display) -> Self {
        Self::NotFound(format!("{entity} with id '{id}' not found"))
    }

    /// Create a `Conflict` error.
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    /// Create an `Unauthorized` error.
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    /// Create a `Validation` error.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create an `Internal` error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

use sw_domain::error::DomainError;

impl From<DomainError> for ApplicationError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound(msg) => Self::NotFound(msg),
            DomainError::DuplicateKey(msg) => Self::Conflict(msg),
            DomainError::ForeignKeyViolation(msg) => Self::Conflict(msg),
            DomainError::Conflict(msg) => Self::Conflict(msg),
            DomainError::Validation(msg) => Self::Validation(msg),
            DomainError::Unauthorized(msg) => Self::Unauthorized(msg),
            DomainError::RateLimitExceeded => Self::RateLimitExceeded,
            DomainError::InsufficientSeats { .. } | DomainError::WorkshopFull => {
                Self::Unavailable(err.to_string())
            }
            DomainError::InvalidStatusTransition(msg) | DomainError::Internal(msg) => {
                Self::Internal(msg)
            }
            _ => Self::Internal("Unhandled domain error".to_string()),
        }
    }
}
