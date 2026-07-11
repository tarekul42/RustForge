#![deny(missing_docs)]
#![doc = "Shared crate: cross-cutting primitives used by all other crates."]

/// Application configuration loading and types.
pub mod config;
/// Boxed error type for wrapping Send + Sync errors.
pub mod error;
/// Tracing/logging subscriber initialization.
pub mod logging;
/// Prometheus metrics exporter and descriptions.
pub mod metrics;
