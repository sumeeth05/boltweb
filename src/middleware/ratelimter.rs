use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::request::RequestBody;
use crate::response::ResponseWriter;
use crate::types::Middleware;

pub struct RateLimiterConfig {
    pub requests: u32,
    pub per_seconds: u64,
}

#[derive(Clone)]
pub struct RateLimiter {
    config: Arc<RateLimiterConfig>,
    state: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            config: Arc::new(config),
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Middleware for RateLimiter {
    async fn run(&self, req: &mut RequestBody, res: &mut ResponseWriter) {
        let ip = req
            .headers()
            .get("x-forwarded-for")
            .or_else(|| req.headers().get("host"))
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let mut state = self.state.lock().await;
        let now = Instant::now();
        let (count, last_reset) = state.entry(ip.clone()).or_insert((0, now));

        if now.duration_since(*last_reset).as_secs() > self.config.per_seconds {
            *count = 0;
            *last_reset = now;
        }

        if *count >= self.config.requests {
            res.status(429).send("Too Many Requests");
        } else {
            *count += 1;
        }
    }
}
