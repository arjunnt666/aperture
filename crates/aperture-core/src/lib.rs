//! Core types for aperture traffic controls.

pub mod error;
pub mod clock;
pub mod decision;
pub mod config;

pub use error::{ApertureError, Result};
pub use clock::{Clock, SystemClock};
pub use decision::{Decision, Outcome};
pub use config::{BreakerConfig, BulkheadConfig, LimitConfig};