use glam::Vec2;

use crate::miscs::ParticleState;

#[derive(Default)]
pub struct Boundary {
    pub min: Vec2,
    pub max: Vec2,
}

impl Boundary {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, particle: &ParticleState) -> bool {
        let (x_min, y_min) = self.get_min(particle.radius);
        let (x_max, y_max) = self.get_max(particle.radius);

        x_min <= particle.position.x
            && particle.position.x <= x_max
            && y_min <= particle.position.y
            && particle.position.y <= y_max
    }

    pub fn get_min(&self, offset: f32) -> (f32, f32) {
        (self.min.x + offset, self.min.y + offset)
    }

    pub fn get_max(&self, offset: f32) -> (f32, f32) {
        (self.max.x - offset, self.max.y - offset)
    }
}
