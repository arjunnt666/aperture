//! Simple counters for aperture controls.

use parking_lot::Mutex;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Counters {
    pub allowed: u64,
    pub denied: u64,
    pub shed: u64,
    pub failures: u64,
}

pub struct Metrics {
    counters: Mutex<HashMap<String, Counters>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self { counters: Mutex::new(HashMap::new()) }
    }

    pub fn record_allow(&self, name: &str) { self.bump(name, |c| c.allowed += 1); }
    pub fn record_deny(&self, name: &str) { self.bump(name, |c| c.denied += 1); }
    pub fn record_shed(&self, name: &str) { self.bump(name, |c| c.shed += 1); }
    pub fn record_failure(&self, name: &str) { self.bump(name, |c| c.failures += 1); }

    fn bump<F>(&self, name: &str, f: F) where F: FnOnce(&mut Counters) {
        let mut map = self.counters.lock();
        let entry = map.entry(name.to_string()).or_default();
        f(entry);
    }

    pub fn snapshot(&self, name: &str) -> Counters {
        self.counters.lock().get(name).cloned().unwrap_or_default()
    }
}

impl Default for Metrics {
    fn default() -> Self { Self::new() }
}
