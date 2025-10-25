use crate::{
    boundary::Boundary,
    frame_window::FrameWindow,
    validator::{StreamingValidator, report::ValidationReport},
};

pub struct BoundaryViolation {
    frame: u64,
    id: usize,
    x: f32,
    y: f32,
    radius: f32,
}

impl StreamingValidator {
    pub(super) fn check_boundaries(&self, window: &FrameWindow, report: &mut ValidationReport) {
        for (key, particle) in &window.particles {
            if !self.boundary.contains(particle) {
                report.boundary_violations.push(BoundaryViolation {
                    frame: window.frame,
                    id: *key,
                    x: particle.position.x,
                    y: particle.position.y,
                    radius: particle.radius,
                });
            }
        }
    }
}
