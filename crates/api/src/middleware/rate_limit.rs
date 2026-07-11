use std::collections::HashMap;
use std::sync::Arc;

use axum::{extract::Request, middleware::Next, response::Response};
use tokio::sync::Mutex;

/// Simple per-IP token-bucket rate limiter.
///
/// This is a minimal in-memory implementation. In production,
/// use a distributed limiter (e.g. `tower-governor` with Redis).
#[derive(Clone)]
pub struct TokenBucket {
    tokens: Arc<Mutex<HashMap<String, InnerBucket>>>,
    capacity: u64,
    refill_rate: f64,
}

#[derive(Clone)]
struct InnerBucket {
    tokens: f64,
    last_refill: std::time::Instant,
}

impl TokenBucket {
    /// Create a new token bucket with the given capacity and refill rate (tokens/second).
    pub fn new(capacity: u64, refill_per_second: u64) -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
            capacity,
            refill_rate: refill_per_second as f64,
        }
    }

    /// Check if a request from `key` is allowed, consuming one token if so.
    pub async fn check_key(&self, key: &str) -> bool {
        let mut map = self.tokens.lock().await;
        let now = std::time::Instant::now();

        if let Some(bucket) = map.get_mut(key) {
            let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
            bucket.tokens = (bucket.tokens + elapsed * self.refill_rate).min(self.capacity as f64);
            bucket.last_refill = now;

            if bucket.tokens >= 1.0 {
                bucket.tokens -= 1.0;
                true
            } else {
                false
            }
        } else {
            map.insert(
                key.to_string(),
                InnerBucket {
                    tokens: self.capacity as f64 - 1.0,
                    last_refill: now,
                },
            );
            true
        }
    }
}

/// Axum middleware that rate-limits by client IP.
pub async fn rate_limit_mw(
    bucket: axum::extract::State<TokenBucket>,
    req: Request,
    next: Next,
) -> Result<Response, (axum::http::StatusCode, &'static str)> {
    let key = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            req.extensions()
                .get::<std::net::SocketAddr>()
                .map(|a| a.ip().to_string())
                .unwrap_or_default()
        });

    if bucket.check_key(&key).await {
        Ok(next.run(req).await)
    } else {
        Err((
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded",
        ))
    }
}
