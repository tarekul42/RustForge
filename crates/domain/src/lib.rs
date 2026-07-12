#![deny(missing_docs)]
#![doc = "Domain crate: pure business logic — no I/O dependencies."]

/// Domain aggregates (User, Workshop, Enrollment, Payment, Review).
pub mod aggregates;
/// Domain error types.
pub mod error;
/// Domain events emitted by aggregate methods.
pub mod events;
/// Repository traits for persisting and retrieving aggregates.
pub mod repositories;
/// Domain service port traits (ObjectStore, EmailSender).
pub mod services;
/// Value objects (ids, email, money, OTP, transaction ID).
pub mod value_objects;
