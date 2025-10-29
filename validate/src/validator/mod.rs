mod boundary;
mod comp;
mod conservation;
mod error;
mod event;
mod missed_collision;
mod overlaps;
mod report;

use std::path::Path;

use anyhow::{Result, bail};
use csv::Reader;
use glam::Vec2;

use crate::{
    boundary::Boundary,
    buffered_reader::{BufferedEventReader, BufferedParticleReader},
    frame_window::FrameWindow,
    miscs::{EventRow, ParticleRow, ParticleState},
    validator::{error::ValidationError, event::FalsePositive, report::ValidationReport},
};

#[derive(Default)]
pub struct StreamingValidator {
    dt: f32,
    tolerance: f32,
    max_frame: u64,
    boundary: Boundary,
}

impl StreamingValidator {
    pub fn with_tolerance(mut self, tolerance: f32) -> Self {
        self.tolerance = tolerance;
        self
    }

    pub fn with_max_frame(mut self, max_frame: u64) -> Self {
        self.max_frame = max_frame;
        self
    }

    pub fn with_boundary(mut self, boundary: Boundary) -> Self {
        self.boundary = boundary;
        self
    }

    pub fn validate(
        &mut self,
        particles_csv: &Path,
        events_csv: &Path,
    ) -> Result<ValidationReport> {
        let mut particle_stream = BufferedParticleReader::new(Reader::from_path(particles_csv)?);
        let mut event_stream = BufferedEventReader::new(Reader::from_path(events_csv)?);

        let mut prev_window = particle_stream.read_frame(1)?;

        if !prev_window.particles.is_empty() {
            println!(
                "Particles loaded with length {0} at the first frame",
                prev_window.particles.len()
            );
        } else {
            bail!("No particles found at the first frame");
        }

        let mut frame = 2;

        let mut report = ValidationReport::default();

        self.check_initial_overlaps(&prev_window, &mut report);

        while frame <= self.max_frame {
            let curr_window = particle_stream.read_frame(frame)?;

            if curr_window.particles.is_empty() {
                println!("No more particles found at frame {frame}");
                self.max_frame = frame - 1;
                println!("Setting max frames to {}", self.max_frame);
                break;
            }

            let dt = curr_window.time_s - prev_window.time_s;

            println!(
                "Processing frame {} / {} ({:.2}%) (t={:.6}s, dt={:.6}s)",
                frame,
                self.max_frame,
                frame / self.max_frame * 100,
                curr_window.time_s,
                dt,
            );

            let events = event_stream.read_frame(frame)?;

            self.validate_frame(&prev_window, &curr_window, &events, dt, &mut report);

            prev_window = curr_window;
            frame += 1;
        }
        println!("\nValidation complete! Processed {} frames", frame - 1);
        println!("{report}");

        Ok(report)
    }

    fn validate_frame(
        &self,
        prev_window: &FrameWindow,
        curr_window: &FrameWindow,
        events: &[EventRow],
        dt: f32,
        report: &mut ValidationReport,
    ) {
        for event in events {
            if let Err(e) = self.validate_event(event, prev_window) {
                report
                    .false_positives
                    .push(FalsePositive::new(prev_window.frame, e));
            } else {
                report.valid_collisions += 1;
            }
        }

        report.missed_collisions.extend(self.find_missed_collisions(
            prev_window,
            curr_window,
            events,
            dt,
        ));

        self.check_boundaries(curr_window, report);
        self.check_conservation(prev_window, curr_window, report);
    }
}
