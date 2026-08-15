//! Circuit breaker state machine.

use aperture_core::{BreakerConfig, Outcome, Result};
use parking_lot::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State { Closed, Open, HalfOpen }

pub struct CircuitBreaker {
    config: BreakerConfig,
    state: Mutex<Inner>,
}

struct Inner {
    state: State,
    failures: u32,
    successes: u32,
    opened_at: Option<Instant>,
    half_open_inflight: u32,
}

impl CircuitBreaker {
    pub fn new(config: BreakerConfig) -> Self {
        Self {
            config,
            state: Mutex::new(Inner {
                state: State::Closed,
                failures: 0,
                successes: 0,
                opened_at: None,
                half_open_inflight: 0,
            }),
        }
    }

    pub fn allow(&self) -> Result<Outcome> {
        let mut inner = self.state.lock();
        match inner.state {
            State::Closed => Ok(Outcome::allow(None)),
            State::Open => {
                if let Some(opened) = inner.opened_at {
                    if opened.elapsed() >= Duration::from_millis(self.config.timeout_ms) {
                        inner.state = State::HalfOpen;
                        inner.half_open_inflight = 0;
                        inner.successes = 0;
                        return Ok(Outcome::allow(None));
                    }
                }
                Ok(Outcome::deny(Some(self.config.timeout_ms)))
            }
            State::HalfOpen => {
                if inner.half_open_inflight < self.config.half_open_max {
                    inner.half_open_inflight += 1;
                    Ok(Outcome::allow(None))
                } else {
                    Ok(Outcome::deny(None))
                }
            }
        }
    }

    pub fn record_success(&self) {
        let mut inner = self.state.lock();
        match inner.state {
            State::HalfOpen => {
                inner.successes += 1;
                if inner.successes >= self.config.success_threshold {
                    inner.state = State::Closed;
                    inner.failures = 0;
                    inner.half_open_inflight = 0;
                }
            }
            State::Closed => { inner.failures = 0; }
            State::Open => {}
        }
    }

    pub fn record_failure(&self) {
        let mut inner = self.state.lock();
        match inner.state {
            State::Closed => {
                inner.failures += 1;
                if inner.failures >= self.config.failure_threshold {
                    inner.state = State::Open;
                    inner.opened_at = Some(Instant::now());
                }
            }
            State::HalfOpen => {
                inner.state = State::Open;
                inner.opened_at = Some(Instant::now());
                inner.half_open_inflight = 0;
            }
            State::Open => {}
        }
    }

    pub fn state(&self) -> State { self.state.lock().state }
}
