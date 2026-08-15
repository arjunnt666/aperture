use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitConfig {
    pub rate: f64,
    pub burst: u64,
    pub window_ms: u64,
}

impl Default for LimitConfig {
    fn default() -> Self {
        Self { rate: 10.0, burst: 20, window_ms: 1000 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout_ms: u64,
    pub half_open_max: u32,
}

impl Default for BreakerConfig {
    fn default() -> Self {
        Self { failure_threshold: 5, success_threshold: 2, timeout_ms: 30_000, half_open_max: 1 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkheadConfig {
    pub max_concurrent: u32,
    pub max_queue: u32,
    pub queue_timeout_ms: u64,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self { max_concurrent: 10, max_queue: 50, queue_timeout_ms: 1000 }
    }
}
