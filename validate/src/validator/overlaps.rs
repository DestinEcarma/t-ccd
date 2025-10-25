use crate::{
    boundary::Boundary,
    frame_window::FrameWindow,
    validator::{StreamingValidator, report::ValidationReport},
};

pub struct OverlapViolation {
    pub frame: u64,
    pub i: usize,
    pub j: usize,
    pub dist: f32,
    pub min_dist: f32,
}

impl StreamingValidator {
    pub(super) fn check_initial_overlaps(
        &self,
        window: &FrameWindow,
        report: &mut ValidationReport,
    ) {
        let particles: Vec<_> = window.particles.values().collect();

        for i in 0..particles.len() {
            for j in (i + 1)..particles.len() {
                let dist = (particles[i].position - particles[j].position).length();
                let min_dist = particles[i].radius + particles[j].radius;

                if dist < min_dist - self.tolerance {
                    report.initial_overlaps.push(OverlapViolation {
                        frame: window.frame,
                        i,
                        j,
                        dist,
                        min_dist,
                    });
                }
            }
        }
    }
}
