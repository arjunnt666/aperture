//! Embeddable control plane that wires limiters, breakers, and bulkheads.

use aperture_adaptive::{AdaptiveConfig, AdaptiveLimiter};
use aperture_breaker::CircuitBreaker;
use aperture_bulkhead::Bulkhead;
use aperture_core::{BreakerConfig, BulkheadConfig, Decision, LimitConfig, Outcome, Result};
use aperture_limiter::{SlidingWindow, TokenBucket};
use aperture_metrics::Metrics;
use std::sync::Arc;
use tracing::info;

pub struct ControlPlane {
    pub limiter: TokenBucket,
    pub window: SlidingWindow,
    pub breaker: CircuitBreaker,
    pub bulkhead: Arc<Bulkhead>,
    pub adaptive: AdaptiveLimiter,
    pub metrics: Metrics,
}

impl ControlPlane {
    pub fn new() -> Self {
        Self {
            limiter: TokenBucket::new(LimitConfig::default()),
            window: SlidingWindow::new(LimitConfig::default()),
            breaker: CircuitBreaker::new(BreakerConfig::default()),
            bulkhead: Arc::new(Bulkhead::new(BulkheadConfig::default())),
            adaptive: AdaptiveLimiter::new(AdaptiveConfig::default()),
            metrics: Metrics::new(),
        }
    }

    pub fn check(&self, name: &str) -> Result<Outcome> {
        let o = self.breaker.allow()?;
        if o.decision != Decision::Allow {
            self.metrics.record_deny(name);
            return Ok(o);
        }
        let o = self.limiter.try_acquire(1.0)?;
        if o.decision != Decision::Allow {
            self.metrics.record_deny(name);
            return Ok(o);
        }
        let o = self.bulkhead.try_enter()?;
        match o.decision {
            Decision::Allow => { self.metrics.record_allow(name); Ok(o) }
            Decision::Shed => { self.metrics.record_shed(name); Ok(o) }
            Decision::Deny => { self.metrics.record_deny(name); Ok(o) }
        }
    }

    pub fn release(&self) { self.bulkhead.exit(); }
    pub fn report_success(&self) { self.breaker.record_success(); }
    pub fn report_failure(&self) {
        self.breaker.record_failure();
        self.metrics.record_failure("default");
    }
}

impl Default for ControlPlane {
    fn default() -> Self {
        let plane = Self::new();
        info!("control plane ready");
        plane
    }
}
