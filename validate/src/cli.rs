use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    /// Path to particles CSV
    #[arg(short, long, required = true)]
    pub particles: PathBuf,

    /// Path to events CSV
    #[arg(short, long, required = true)]
    pub events: PathBuf,

    /// Maximum frame to validate
    #[arg(short, long, required = true)]
    pub max_frame: u64,

    /// Tolerance for floating point comparisons
    #[arg(short, long, default_value_t = 1e-5)]
    pub tolerance: f32,

    /// Size: 800x400
    #[arg(short, long, default_value = "800x400")]
    pub size: String,
}
