#![deny(missing_docs)]
#![doc = "Worker crate: background job runner stub for Phase 0."]

use std::time::Duration;
use sw_shared::config::Config;

/// Stub entry point — loads config, initializes logging, then sleeps.
/// Will be replaced with actual job processing in Phase 6.
#[tokio::main]
async fn main() {
    let config = Config::load();

    sw_shared::logging::init(&config.observability);

    tracing::info!("Worker starting — no jobs configured yet. Sleeping...");
    tokio::time::sleep(Duration::from_secs(3600)).await;
}
