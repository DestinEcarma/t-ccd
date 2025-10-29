use std::{
    fmt::{self},
    path::PathBuf,
};

use anyhow::Result;
use csv::Writer;

use crate::validator::{
    boundary::BoundaryViolation, conservation::ConservationViolation, error::ValidationError,
    event::FalsePositive, missed_collision::MissedCollision, overlaps::OverlapViolation,
};

#[derive(Default)]
pub struct ValidationReport {
    pub initial_overlaps: Vec<OverlapViolation>,
    pub valid_collisions: usize,
    pub false_positives: Vec<FalsePositive>,
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
            for fp in self.false_positives.iter().take(10) {
                println!("  Frame {}: {:?}", fp.frame, fp.error);
            }
        }

        if !self.conservation_violations.is_empty() {
            println!("\nFirst 10 conservation violations:");
            for cv in self.conservation_violations.iter().take(10) {
                println!(
                    "  Frame {}: energy {} x={:.6} y={:.6}",
                    cv.frame, cv.energy_error, cv.x_error, cv.y_error
                );
            }
        }

        if !self.boundary_violations.is_empty() {
            println!("\nFirst 10 boundary violations:");
            for bv in self.boundary_violations.iter().take(10) {
                println!(
                    "  Frame {}: particle {} at x={:.6}, y={:.6}",
                    bv.frame, bv.id, bv.x, bv.y
                );
            }
        }
    }

    pub fn write_to_csv(&self, base_path: &PathBuf) -> Result<()> {
        if !self.initial_overlaps.is_empty() {
            let mut writer = Writer::from_path(base_path.clone().join("initial_overlaps.csv"))?;

            for overlap in &self.initial_overlaps {
                writer.serialize(overlap)?;
            }

            writer.flush()?;
        }

        if !self.false_positives.is_empty() {
            let mut writer = Writer::from_path(base_path.clone().join("false_positives.csv"))?;

            for fp in &self.false_positives {
                writer.serialize(fp)?;
            }

            writer.flush()?;
        }

        if !self.missed_collisions.is_empty() {
            let mut writer = Writer::from_path(base_path.clone().join("missed_collisions.csv"))?;

            for mc in &self.missed_collisions {
                writer.serialize(mc)?;
            }

            writer.flush()?;
        }

        if !self.conservation_violations.is_empty() {
            let mut writer =
                Writer::from_path(base_path.clone().join("conservation_violations.csv"))?;

            for cv in &self.conservation_violations {
                writer.serialize(cv)?;
            }

            writer.flush()?;
        }

        if !self.boundary_violations.is_empty() {
            let mut writer = Writer::from_path(base_path.clone().join("boundary_violations.csv"))?;

            for bv in &self.boundary_violations {
                writer.serialize(bv)?;
            }

            writer.flush()?;
        }

        Ok(())
    }
}
