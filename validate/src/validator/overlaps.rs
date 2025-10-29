use serde::Serialize;

use crate::{
    boundary::Boundary,
    frame_window::FrameWindow,
    validator::{StreamingValidator, report::ValidationReport},
};

#[derive(Serialize)]
pub struct OverlapViolation {
    pub frame: u64,
    pub i: usize,
    pub j: usize,
    pub ix: f32,
    pub iy: f32,
    pub jx: f32,
    pub jy: f32,
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
                        ix: particles[i].position.x,
                        iy: particles[i].position.y,
                        jx: particles[j].position.x,
                        jy: particles[j].position.y,
                        dist,
                        min_dist,
                    });
                }
            }
        }
    }
}
