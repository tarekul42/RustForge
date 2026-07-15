use std::sync::Arc;

use governor::middleware::StateInformationMiddleware;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;

/// Build a rate limiter layer using `Governor` with the `SmartIpKeyExtractor`.
///
/// `SmartIpKeyExtractor` checks common proxy headers (X-Forwarded-For, X-Real-IP,
/// Forwarded) before falling back to the peer IP from `SocketAddr`.
pub fn rate_limiter_layer<RespBody>()
-> GovernorLayer<SmartIpKeyExtractor, StateInformationMiddleware, RespBody> {
    let mut b = GovernorConfigBuilder::default();
    b.per_second(60);
    b.burst_size(120);
    let mut b = b.key_extractor(SmartIpKeyExtractor);
    let config = Arc::new(
        b.use_headers()
            .finish()
            .unwrap_or_else(|| panic!("invalid rate limiter config, check governor parameters")),
    );
    GovernorLayer::new(config)
}
