#![deny(missing_docs)]
#![allow(
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::uninlined_format_args,
    clippy::wildcard_imports
)]
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
