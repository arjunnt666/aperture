//! Bulkhead isolation via semaphore style budgets.

use aperture_core::{ApertureError, BulkheadConfig, Decision, Outcome, Result};
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
            return Ok(Outcome::allow(Some(
                (self.config.max_concurrent - *active) as u64,
            )));
        }
        drop(active);
        let queued = self.queued.lock();
        if *queued < self.config.max_queue {
            Ok(Outcome::deny(Some(self.config.queue_timeout_ms)))
        } else {
            Ok(Outcome::shed())
        }
    }

    pub fn exit(&self) {
        let mut active = self.active.lock();
        if *active > 0 {
            *active -= 1;
        }
    }

    pub fn active_count(&self) -> u32 {
        *self.active.lock()
    }
}

/// Guard that releases the bulkhead slot on drop.
pub struct BulkheadGuard {
    bulkhead: Arc<Bulkhead>,
}

impl BulkheadGuard {
    pub fn new(bulkhead: Arc<Bulkhead>) -> Result<Self> {
        let o = bulkhead.try_enter()?;
        if o.decision != Decision::Allow {
            return Err(ApertureError::BulkheadFull);
        }
        Ok(Self { bulkhead })
    }
}

impl Drop for BulkheadGuard {
    fn drop(&mut self) {
        self.bulkhead.exit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aperture_core::{BulkheadConfig, Decision};

    #[test]
    fn fills_then_sheds() {
        let b = Bulkhead::new(BulkheadConfig {
            max_concurrent: 1,
            max_queue: 0,
            queue_timeout_ms: 1,
        });
        assert_eq!(b.try_enter().unwrap().decision, Decision::Allow);
        assert_eq!(b.try_enter().unwrap().decision, Decision::Shed);
        b.exit();
        assert_eq!(b.try_enter().unwrap().decision, Decision::Allow);
    }

    #[test]
    fn guard_releases_on_drop() {
        let b = Arc::new(Bulkhead::new(BulkheadConfig {
            max_concurrent: 1,
            max_queue: 0,
            queue_timeout_ms: 1,
        }));
        {
            let _g = BulkheadGuard::new(Arc::clone(&b)).unwrap();
            assert!(BulkheadGuard::new(Arc::clone(&b)).is_err());
        }
        assert!(BulkheadGuard::new(Arc::clone(&b)).is_ok());
    }
}
