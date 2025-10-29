use serde::Serialize;

#[derive(Debug, Serialize, thiserror::Error)]
pub enum ValidationError {
    #[error("Particle not found at index {0}")]
    ParticleNotFound(usize),
    #[error("Particles are not touching")]
    NotTouching {
        i: usize,
        j: Option<usize>,
        ix: f32,
        iy: f32,
        jx: Option<f32>,
        jy: Option<f32>,
        distance: f32,
        expected: f32,
    },
    #[error("Particles have wrong normal")]
    WrongNormal {
        i: usize,
        j: Option<usize>,
        ix: f32,
        iy: f32,
        jx: Option<f32>,
        jy: Option<f32>,
        nx: f32,
        ny: f32,
    },
    #[error("Particles are not elastic")]
    NotElastic {
        i: usize,
        j: Option<usize>,
        ix: f32,
        iy: f32,
        jx: Option<f32>,
        jy: Option<f32>,
        before: f32,
        after: f32,
        expected: f32,
    },
}
