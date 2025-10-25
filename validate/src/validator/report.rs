use std::fmt::{self};

use crate::validator::{
    boundary::BoundaryViolation, conservation::ConservationViolation, error::ValidationError,
    missed_collision::MissedCollision, overlaps::OverlapViolation,
};

#[derive(Default)]
pub struct ValidationReport {
    pub initial_overlaps: Vec<OverlapViolation>,
    pub valid_collisions: usize,
    pub false_positives: Vec<(u64, ValidationError)>,
    pub missed_collisions: Vec<MissedCollision>,
    pub conservation_violations: Vec<ConservationViolation>,
    pub boundary_violations: Vec<BoundaryViolation>,
}

impl fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n=== VALIDATION REPORT ===");
        writeln!(f, "Initial overlaps: {}", self.initial_overlaps.len());
        writeln!(f, "Valid collisions: {}", self.valid_collisions);
        writeln!(f, "False positives: {}", self.false_positives.len());
        writeln!(f, "Missed collisions: {}", self.missed_collisions.len());
        println!(
            "Conservation violations: {}",
            self.conservation_violations.len()
        );
        write!(f, "Boundary violations: {}", self.boundary_violations.len())
    }
}

impl ValidationReport {
    pub fn summary(&self) {
        println!("{self}");

        if !self.initial_overlaps.is_empty() {
            println!("\nInitial overlaps:");
            for overlap in &self.initial_overlaps {
                println!(
                    "  Frame {}: particles {} ↔ {} overlap (dist={:.4}, min={:.4})",
                    overlap.frame, overlap.i, overlap.j, overlap.dist, overlap.min_dist
                )
            }
        }

        if !self.missed_collisions.is_empty() {
            println!("\nFirst 10 missed collisions:");
            for mc in self.missed_collisions.iter().take(10) {
                println!(
                    "  Frame {}: particles {} ↔ {} at t={:.6}s",
                    mc.frame, mc.i, mc.j, mc.toi
                );
            }
        }

        if !self.false_positives.is_empty() {
            println!("\nFirst 10 false positives:");
            for (frame, err) in self.false_positives.iter().take(10) {
                println!("  Frame {frame}: {err:?}");
            }
        }
    }

    pub fn conservation_violations(&self) -> &Vec<ConservationViolation> {
        &self.conservation_violations
    }
}
