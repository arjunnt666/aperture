//! Embeddable control plane that wires limiters, breakers, and bulkheads.

use aperture_adaptive::{AdaptiveConfig, AdaptiveLimiter};
use aperture_breaker::CircuitBreaker;
use aperture_bulkhead::Bulkhead;
use aperture_core::{BreakerConfig, BulkheadConfig, Decision, LimitConfig, Outcome, Result};
use aperture_limiter::{SlidingWindow, TokenBucket};
use aperture_metrics::Metrics;
use std::sync::Arc;
use std::time::Duration;
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
        Self::with_limits(
            LimitConfig::default(),
            BreakerConfig::default(),
            BulkheadConfig::default(),
        )
    }

    pub fn with_limits(
        limit: LimitConfig,
        breaker: BreakerConfig,
        bulkhead: BulkheadConfig,
    ) -> Self {
        Self::with_all(limit, breaker, bulkhead, AdaptiveConfig::default())
    }

    pub fn with_all(
        limit: LimitConfig,
        breaker: BreakerConfig,
        bulkhead: BulkheadConfig,
        adaptive: AdaptiveConfig,
    ) -> Self {
        Self {
            limiter: TokenBucket::new(limit.clone()),
            window: SlidingWindow::new(limit),
            breaker: CircuitBreaker::new(breaker),
            bulkhead: Arc::new(Bulkhead::new(bulkhead)),
            adaptive: AdaptiveLimiter::new(adaptive),
            metrics: Metrics::new(),
        }
    }

    /// Stacked admission: breaker, adaptive concurrency, token bucket,
    /// sliding window, then bulkhead. Later layers roll back earlier
    /// inflight reservations on deny.
    pub fn check(&self, name: &str) -> Result<Outcome> {
        let o = self.breaker.allow()?;
        if o.decision != Decision::Allow {
            self.metrics.record_deny(name);
            return Ok(o);
        }

        let o = self.adaptive.try_acquire()?;
        if o.decision != Decision::Allow {
            self.metrics.record_shed(name);
            return Ok(o);
        }

        let o = self.limiter.try_acquire(1.0)?;
        if o.decision != Decision::Allow {
            self.adaptive.release(Duration::from_millis(0), false);
            self.metrics.record_deny(name);
            return Ok(o);
        }

        let o = self.window.try_acquire()?;
        if o.decision != Decision::Allow {
            self.adaptive.release(Duration::from_millis(0), false);
            self.metrics.record_deny(name);
            return Ok(o);
        }

        let o = self.bulkhead.try_enter()?;
        match o.decision {
            Decision::Allow => {
                self.metrics.record_allow(name);
                Ok(o)
            }
            Decision::Shed => {
                self.adaptive.release(Duration::from_millis(0), false);
                self.metrics.record_shed(name);
                Ok(o)
            }
            Decision::Deny => {
                self.adaptive.release(Duration::from_millis(0), false);
                self.metrics.record_deny(name);
                Ok(o)
            }
        }
    }

    pub fn release(&self) {
        self.bulkhead.exit();
        self.adaptive.release(Duration::from_millis(1), true);
    }

    pub fn report_success(&self) {
        self.breaker.record_success();
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use aperture_adaptive::AdaptiveConfig;
    use aperture_core::{BreakerConfig, BulkheadConfig, Decision, LimitConfig};

    fn wide_limit() -> LimitConfig {
        LimitConfig {
            rate: 1000.0,
            burst: 50,
            window_ms: 60_000,
        }
    }

    fn roomy_bulkhead() -> BulkheadConfig {
        BulkheadConfig {
            max_concurrent: 50,
            max_queue: 0,
            queue_timeout_ms: 1,
        }
    }

    #[test]
    fn breaker_open_denies_even_with_tokens() {
        let plane = ControlPlane::with_limits(
            wide_limit(),
            BreakerConfig {
                failure_threshold: 1,
                success_threshold: 1,
                timeout_ms: 60_000,
                half_open_max: 1,
            },
            roomy_bulkhead(),
        );
        assert_eq!(plane.check("x").unwrap().decision, Decision::Allow);
        plane.release();
        plane.report_failure();
        assert_eq!(plane.breaker.state(), aperture_breaker::State::Open);
        assert_eq!(plane.check("x").unwrap().decision, Decision::Deny);
    }

    #[test]
    fn bulkhead_sheds_when_full() {
        let plane = ControlPlane::with_limits(
            wide_limit(),
            BreakerConfig::default(),
            BulkheadConfig {
                max_concurrent: 1,
                max_queue: 0,
                queue_timeout_ms: 1,
            },
        );
        assert_eq!(plane.check("x").unwrap().decision, Decision::Allow);
        assert_eq!(plane.check("x").unwrap().decision, Decision::Shed);
        plane.release();
        assert_eq!(plane.check("x").unwrap().decision, Decision::Allow);
    }

    #[test]
    fn bucket_exhausts_in_the_stack() {
        let plane = ControlPlane::with_limits(
            LimitConfig {
                rate: 0.0001,
                burst: 2,
                window_ms: 60_000,
            },
            BreakerConfig::default(),
            roomy_bulkhead(),
        );
        assert_eq!(plane.check("x").unwrap().decision, Decision::Allow);
        plane.release();
        assert_eq!(plane.check("x").unwrap().decision, Decision::Allow);
        plane.release();
        assert_eq!(plane.check("x").unwrap().decision, Decision::Deny);
    }

    #[test]
    fn adaptive_sheds_when_inflight_hits_limit() {
        let plane = ControlPlane::with_all(
            wide_limit(),
            BreakerConfig::default(),
            roomy_bulkhead(),
            AdaptiveConfig {
                min_limit: 1,
                max_limit: 4,
                initial_limit: 1,
                target_latency_ms: 50,
                smoothing: 0.2,
            },
        );
        assert_eq!(plane.check("x").unwrap().decision, Decision::Allow);
        assert_eq!(plane.check("x").unwrap().decision, Decision::Shed);
        plane.release();
        assert_eq!(plane.check("x").unwrap().decision, Decision::Allow);
    }
}
