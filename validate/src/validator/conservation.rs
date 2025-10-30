use serde::Serialize;

use crate::{
    frame_window::FrameWindow,
    miscs::ParticleState,
    validator::{StreamingValidator, comp, report::ValidationReport},
};

#[derive(Serialize)]
pub struct ConservationViolation {
    pub frame: u64,
    pub energy_error: f32,
    pub x_error: f32,
    pub y_error: f32,
}

impl StreamingValidator {
    pub(super) fn check_conservation(
        &self,
        curr: &FrameWindow,
        next: &FrameWindow,
        report: &mut ValidationReport,
    ) {
        let (ke_prev, px_prev, py_prev) = comp::compute_totals(&curr.particles);
        let (ke_curr, px_curr, py_curr) = comp::compute_totals(&next.particles);

        let energy_error = ((ke_curr - ke_prev) / ke_prev).abs();
        let px_error = ((px_curr - px_prev) / px_prev.abs().max(1e-6)).abs();
        let py_error = ((py_curr - py_prev) / py_prev.abs().max(1e-6)).abs();

        if energy_error > self.tolerance || px_error > self.tolerance || py_error > self.tolerance {
            report.conservation_violations.push(ConservationViolation {
                frame: next.frame,
                energy_error,
                x_error: px_error,
                y_error: py_error,
            });
        }
    }
}
