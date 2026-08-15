//! Thin client wrapper around a local control plane.

use aperture_core::{Outcome, Result};
use aperture_server::ControlPlane;
use std::sync::Arc;

pub struct Client {
    plane: Arc<ControlPlane>,
}

impl Client {
    pub fn new(plane: Arc<ControlPlane>) -> Self { Self { plane } }

    pub fn local() -> Self {
        Self { plane: Arc::new(ControlPlane::new()) }
    }

    pub fn check(&self, name: &str) -> Result<Outcome> { self.plane.check(name) }
    pub fn release(&self) { self.plane.release(); }
    pub fn success(&self) { self.plane.report_success(); }
    pub fn failure(&self) { self.plane.report_failure(); }
}
