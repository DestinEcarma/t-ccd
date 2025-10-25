#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Particle not found at index {0}")]
    ParticleNotFound(usize),
    #[error("Particles are not touching")]
    NotTouching { distance: f32, expected: f32 },
    #[error("Particles have wrong normal")]
    WrongNormal,
    #[error("Particles are not elastic")]
    NotElastic {
        before: f32,
        after: f32,
        expected: f32,
    },
}
