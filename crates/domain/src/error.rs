/// Errors that can occur in the domain layer.
///
/// This enum is `#[non_exhaustive]` — new variants may be added in future
/// versions without a breaking change for consumers.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum DomainError {
    /// The requested resource was not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// An invalid state transition was attempted.
    #[error("Invalid status transition: {0}")]
    InvalidStatusTransition(String),

    /// A duplicate key violation occurred.
    #[error("Duplicate key: {0}")]
    DuplicateKey(String),

    /// A foreign key constraint was violated.
    #[error("Foreign key violation: {0}")]
    ForeignKeyViolation(String),

    /// A concurrency conflict (optimistic locking) occurred.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// The provided value failed validation.
    #[error("Validation error: {0}")]
    Validation(String),

    /// An unexpected internal error occurred.
    #[error("Internal error: {0}")]
    Internal(String),

    /// The user is not authorized to perform this action.
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// The rate limit was exceeded.
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Insufficient seats available for enrollment.
    #[error("Not enough available seats: requested {requested}, available {available}")]
    InsufficientSeats {
        /// Number of seats requested.
        requested: u32,
        /// Number of seats currently available.
        available: u32,
    },

    /// The enrollment is full.
    #[error("Workshop is full")]
    WorkshopFull,
}

impl DomainError {
    /// Create a NotFound error for a given entity type and ID.
    pub fn not_found(entity: &str, id: impl std::fmt::Display) -> Self {
        Self::NotFound(format!("{entity} with id '{id}' not found"))
    }

    /// Create an InvalidStatusTransition error.
    pub fn invalid_transition(from: &str, to: &str) -> Self {
        Self::InvalidStatusTransition(format!("Cannot transition from '{from}' to '{to}'"))
    }

    /// Create a Validation error.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create an Internal error from an infrastructure (I/O) failure.
    pub fn infrastructure(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
