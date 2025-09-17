use engine::circle::Circle;
use glam::Vec2;

#[derive(Default, Copy, Clone, Debug)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub mass: f32,
    pub color: [f32; 3],
}

impl From<Particle> for Circle {
    fn from(p: Particle) -> Self {
        Self {
            position: p.position,
            radius: p.radius,
            color: p.color,
        }
    }
}

impl Particle {
    pub fn new(position: Vec2, velocity: Vec2, radius: f32, mass: f32, color: [f32; 3]) -> Self {
        Self {
            position,
            velocity,
            radius,
            mass,
            color,
        }
    }
}
