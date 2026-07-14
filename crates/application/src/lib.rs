#![deny(missing_docs)]
#![allow(
    clippy::doc_markdown,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::redundant_closure_for_method_calls,
    clippy::return_self_not_must_use,
    clippy::struct_field_names,
    clippy::uninlined_format_args,
    clippy::unused_async,
    clippy::used_underscore_binding
)]
#![doc = "Application crate: use-case orchestration — no I/O dependencies."]

/// Application-layer error types.
pub mod error;
/// Use-case services organized by domain.
pub mod services;
/// Slug generation and uniqueness utilities.
pub mod slug;
