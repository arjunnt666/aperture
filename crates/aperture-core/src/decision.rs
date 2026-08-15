use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision { Allow, Deny, Shed }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub decision: Decision,
    pub remaining: Option<u64>,
    pub retry_after_ms: Option<u64>,
}

impl Outcome {
    pub fn allow(remaining: Option<u64>) -> Self {
        Self { decision: Decision::Allow, remaining, retry_after_ms: None }
    }
    pub fn deny(retry_after_ms: Option<u64>) -> Self {
        Self { decision: Decision::Deny, remaining: Some(0), retry_after_ms }
    }
    pub fn shed() -> Self {
        Self { decision: Decision::Shed, remaining: Some(0), retry_after_ms: None }
    }
}
