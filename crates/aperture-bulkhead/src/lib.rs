//! Bulkhead isolation via semaphore style budgets.

use aperture_core::{BulkheadConfig, Outcome, Result};
use parking_lot::Mutex;
use std::sync::Arc;

pub struct Bulkhead {
    config: BulkheadConfig,
    active: Mutex<u32>,
    queued: Mutex<u32>,
}

impl Bulkhead {
    pub fn new(config: BulkheadConfig) -> Self {
        Self {
            config,
            active: Mutex::new(0),
            queued: Mutex::new(0),
        }
    }

    pub fn try_enter(&self) -> Result<Outcome> {
        let mut active = self.active.lock();
        if *active < self.config.max_concurrent {
            *active += 1;
            return Ok(Outcome::allow(Some((self.config.max_concurrent - *active) as u64)));
        }
        let mut queued = self.queued.lock();
        if *queued < self.config.max_queue {
            *queued += 1;
            *queued -= 1;
            Ok(Outcome::deny(Some(self.config.queue_timeout_ms)))
        } else {
            Ok(Outcome::shed())
        }
    }

    pub fn exit(&self) {
        let mut active = self.active.lock();
        if *active > 0 { *active -= 1; }
    }

    pub fn active_count(&self) -> u32 { *self.active.lock() }
}

pub struct BulkheadGuard {
    bulkhead: Arc<Bulkhead>,
}

impl BulkheadGuard {
    pub fn new(bulkhead: Arc<Bulkhead>) -> Result<Self> {
        bulkhead.try_enter()?;
        Ok(Self { bulkhead })
    }
}

impl Drop for BulkheadGuard {
    fn drop(&mut self) { self.bulkhead.exit(); }
}
