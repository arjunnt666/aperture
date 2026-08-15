//! Adaptive concurrency based on latency and error signals.

use aperture_core::{Outcome, Result};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AdaptiveConfig {
    pub min_limit: u32,
    pub max_limit: u32,
    pub initial_limit: u32,
    pub target_latency_ms: u64,
    pub smoothing: f64,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            min_limit: 1,
            max_limit: 200,
            initial_limit: 20,
            target_latency_ms: 50,
            smoothing: 0.2,
        }
    }
}

pub struct AdaptiveLimiter {
    config: AdaptiveConfig,
    limit: Mutex<f64>,
    inflight: Mutex<u32>,
    samples: Mutex<VecDeque<(Duration, bool)>>,
}

impl AdaptiveLimiter {
    pub fn new(config: AdaptiveConfig) -> Self {
        let initial = config.initial_limit as f64;
        Self {
            config,
            limit: Mutex::new(initial),
            inflight: Mutex::new(0),
            samples: Mutex::new(VecDeque::with_capacity(64)),
        }
    }

    pub fn try_acquire(&self) -> Result<Outcome> {
        let mut inflight = self.inflight.lock();
        let limit = *self.limit.lock();
        if (*inflight as f64) < limit {
            *inflight += 1;
            Ok(Outcome::allow(Some((limit as u32).saturating_sub(*inflight) as u64)))
        } else {
            Ok(Outcome::shed())
        }
    }

    pub fn release(&self, latency: Duration, success: bool) {
        {
            let mut inflight = self.inflight.lock();
            if *inflight > 0 { *inflight -= 1; }
        }
        let mut samples = self.samples.lock();
        samples.push_back((latency, success));
        if samples.len() > 32 { samples.pop_front(); }
        self.adjust(&samples);
    }

    fn adjust(&self, samples: &VecDeque<(Duration, bool)>) {
        if samples.len() < 8 { return; }
        let avg_ms: f64 = samples.iter().map(|(d, _)| d.as_secs_f64() * 1000.0).sum::<f64>() / samples.len() as f64;
        let error_rate = samples.iter().filter(|(_, ok)| !*ok).count() as f64 / samples.len() as f64;
        let mut limit = self.limit.lock();
        let target = self.config.target_latency_ms as f64;
        let alpha = self.config.smoothing;
        if avg_ms > target * 1.2 || error_rate > 0.1 {
            *limit = (*limit * (1.0 - alpha)).max(self.config.min_limit as f64);
        } else if avg_ms < target * 0.8 && error_rate < 0.02 {
            *limit = (*limit * (1.0 + alpha)).min(self.config.max_limit as f64);
        }
    }

    pub fn current_limit(&self) -> u32 { *self.limit.lock() as u32 }
}
