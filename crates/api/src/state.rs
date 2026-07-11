use std::sync::Arc;
use sw_shared::config::Config;

/// Shared application state accessible from all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Application configuration (immutable after startup).
    pub config: Arc<Config>,
}

impl AppState {
    /// Create a new `AppState` from a loaded `Config`.
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}
