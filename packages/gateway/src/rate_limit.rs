use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<RwLock<RateLimiterInner>>,
}

struct RateLimiterInner {
    hits: HashMap<IpAddr, Vec<Instant>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(RateLimiterInner {
                hits: HashMap::new(),
                max_requests,
                window,
            })),
        }
    }

    pub async fn check(&self, ip: IpAddr) -> bool {
        let mut inner = self.inner.write().await;
        let now = Instant::now();
        let window = inner.window;
        let max_requests = inner.max_requests;

        let hits = inner.hits.entry(ip).or_insert_with(Vec::new);

        // Remove old entries outside the window
        hits.retain(|t| now.duration_since(*t) < window);

        if hits.len() >= max_requests {
            return false; // Rate limited
        }

        hits.push(now);
        true
    }

    /// Periodically clean up old entries to prevent memory growth
    #[allow(dead_code)]
    pub async fn cleanup(&self) {
        let mut inner = self.inner.write().await;
        let now = Instant::now();
        let window = inner.window;
        inner.hits.retain(|_, hits| {
            hits.retain(|t| now.duration_since(*t) < window);
            !hits.is_empty()
        });
    }
}
