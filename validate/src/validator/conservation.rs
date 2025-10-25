use crate::{
    frame_window::FrameWindow,
    miscs::ParticleState,
    validator::{StreamingValidator, comp, report::ValidationReport},
};

pub struct ConservationViolation {
    frame: u64,
    energy_error: f32,
    x_error: f32,
    y_error: f32,
}

impl StreamingValidator {
    pub(super) fn check_conservation(
        &self,
        prev: &FrameWindow,
        curr: &FrameWindow,
        report: &mut ValidationReport,
    ) {
        let (ke_prev, px_prev, py_prev) = comp::compute_totals(&prev.particles);
        let (ke_curr, px_curr, py_curr) = comp::compute_totals(&curr.particles);

        let energy_error = ((ke_curr - ke_prev) / ke_prev).abs();
        let px_error = ((px_curr - px_prev) / px_prev.abs().max(1e-6)).abs();
        let py_error = ((py_curr - py_prev) / py_prev.abs().max(1e-6)).abs();

        if energy_error > 1e-4 || px_error > 1e-4 || py_error > 1e-4 {
            report.conservation_violations.push(ConservationViolation {
                frame: curr.frame,
                energy_error,
                x_error: px_error,
                y_error: py_error,
            });
        }
    }
}
