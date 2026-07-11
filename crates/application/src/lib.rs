#![deny(missing_docs)]
#![doc = "Application crate: use-case orchestration — no I/O dependencies."]

/// Application-layer error types.
pub mod error;
/// Use-case services organized by domain.
pub mod services;
/// Slug generation and uniqueness utilities.
pub mod slug;
