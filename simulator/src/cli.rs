use clap::Parser;

use crate::miscs::{DetectionType, RecorderType};

#[derive(Parser)]
#[command(version, about, long_about)]
pub struct Cli {
    /// Detection method to use
    #[arg(default_value_t = DetectionType::Tccd,  value_enum)]
    pub method: DetectionType,

    /// Number of particles to simulate
    #[arg(short, long, default_value_t = 500)]
    pub particle_count: u64,

    /// Random seed for reproducibility
    #[arg(short, long)]
    pub seed: Option<u64>,

    /// Record simulation data to CSV files
    #[arg(short, long, value_enum)]
    pub record: Option<RecorderType>,

    /// Cell size for spatial partitioning
    #[arg(short, long, default_value_t = 20.0)]
    pub cell_size: f32,

    /// Frame rate for the simulation
    #[arg(short, long, default_value_t = 30)]
    pub fps: u64,

    /// Open in fullscreen mode
    #[arg(short = 'F', long, default_value_t = false)]
    pub fullscreen: bool,

    /// Min radius of particles
    #[arg(short, long, default_value_t = 3.0)]
    pub min_radius: f32,

    /// Max radius of particles
    #[arg(short = 'M', long, default_value_t = 10.0)]
    pub max_radius: f32,

    /// Max velocity of particles
    #[arg(short = 'v', long, default_value_t = 500.0)]
    pub max_velocity: f32,
}
