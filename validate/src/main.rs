#![allow(dead_code, unused)]

mod boundary;
mod buffered_reader;
mod cli;
mod frame_window;
mod miscs;
mod validator;

use std::path::Path;

use anyhow::Result;
use clap::Parser;
use csv::Reader;
use glam::Vec2;

use crate::{
    boundary::Boundary,
    cli::Cli,
    miscs::{EventRow, ParticleRow},
    validator::StreamingValidator,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let boundary = {
        let size = cli
            .size
            .split('x')
            .map(|s| s.parse::<f32>().unwrap())
            .collect::<Vec<_>>();
        let hw = size[0] / 2.0;
        let hh = size[1] / 2.0;

        Boundary::new(Vec2::new(-hw, -hh), Vec2::new(hw, hh))
    };

    let mut streaming_validator = StreamingValidator::default()
        .with_boundary(boundary)
        .with_max_frame(cli.max_frame)
        .with_tolerance(cli.tolerance);

    let result = streaming_validator.validate(&cli.particles, &cli.events)?;

    result.summary();

    if let Some(output) = &cli.output {
        result.write_to_csv(output)?;
    }

    Ok(())
}
