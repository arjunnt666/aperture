//! Rate limiters: token bucket and sliding window.

use aperture_core::{Clock, LimitConfig, Outcome, Result, SystemClock};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

pub struct TokenBucket {
    config: LimitConfig,
    tokens: Mutex<f64>,
    last_refill: Mutex<std::time::Instant>,
    clock: Arc<dyn Clock>,
}

impl TokenBucket {
    pub fn new(config: LimitConfig) -> Self {
        Self {
            tokens: Mutex::new(config.burst as f64),
            last_refill: Mutex::new(std::time::Instant::now()),
            config,
            clock: Arc::new(SystemClock),
        }
    }

    pub fn try_acquire(&self, cost: f64) -> Result<Outcome> {
        let _ = self.clock.now();
        let mut tokens = self.tokens.lock();
        let mut last = self.last_refill.lock();
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(*last).as_secs_f64();
        let refill = elapsed * self.config.rate;
        *tokens = (*tokens + refill).min(self.config.burst as f64);
        *last = now;

        if *tokens >= cost {
            *tokens -= cost;
            Ok(Outcome::allow(Some(*tokens as u64)))
        } else {
            let wait = ((cost - *tokens) / self.config.rate * 1000.0) as u64;
            Ok(Outcome::deny(Some(wait)))
        }
    }
}

pub struct SlidingWindow {
    config: LimitConfig,
    hits: Mutex<VecDeque<std::time::Instant>>,
}

impl SlidingWindow {
    pub fn new(config: LimitConfig) -> Self {
        Self {
            config,
            hits: Mutex::new(VecDeque::new()),
        }
    }

    pub fn try_acquire(&self) -> Result<Outcome> {
        let mut hits = self.hits.lock();
        let now = std::time::Instant::now();
        let window = Duration::from_millis(self.config.window_ms);
        while let Some(front) = hits.front() {
            if now.duration_since(*front) > window {
                hits.pop_front();
            } else {
                break;
            }
        }
        if hits.len() < self.config.burst as usize {
            hits.push_back(now);
            let remaining = self.config.burst.saturating_sub(hits.len() as u64);
            Ok(Outcome::allow(Some(remaining)))
        } else {
            Ok(Outcome::deny(Some(self.config.window_ms)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aperture_core::Decision;

    #[test]
    fn bucket_allows_then_denies() {
        let bucket = TokenBucket::new(LimitConfig {
            rate: 1.0,
            burst: 2,
            window_ms: 1000,
        });
        assert_eq!(bucket.try_acquire(1.0).unwrap().decision, Decision::Allow);
        assert_eq!(bucket.try_acquire(1.0).unwrap().decision, Decision::Allow);
        assert_eq!(bucket.try_acquire(1.0).unwrap().decision, Decision::Deny);
    }

    #[test]
    fn window_respects_burst() {
        let w = SlidingWindow::new(LimitConfig {
            rate: 10.0,
            burst: 1,
            window_ms: 60_000,
        });
        assert_eq!(w.try_acquire().unwrap().decision, Decision::Allow);
        assert_eq!(w.try_acquire().unwrap().decision, Decision::Deny);
    }
}