use engine::Bounds;
use glam::Vec2;

use crate::{
    detector::{CellListDetector, Detector, SweptAabbDetector, TccdDetector},
    miscs::{DetectionType, Recorder, RecorderType},
    particle::Particle,
    spatial::SpatialGrid,
};

const EPS_T: f32 = 1e-5;
const MAX_ITER: usize = 100;

#[derive(Debug, Clone, Copy)]
pub enum Collision {
    Pair(usize, usize),
    Wall(usize),
}

#[derive(Debug, Clone, Copy)]
pub struct Toi {
    pub time: f32,
    pub collision: Collision,
}

impl From<(f32, Collision)> for Toi {
    fn from(value: (f32, Collision)) -> Self {
        Toi {
            time: value.0,
            collision: value.1,
        }
    }
}

pub struct Solver {
    pub recorder: Recorder,

    grid: SpatialGrid,
    detector: Box<dyn Detector>,
}

impl Solver {
    pub fn new(
        cell_size: f32,
        r_type: Option<RecorderType>,
        d_type: DetectionType,
        particle_count: u64,
    ) -> Self {
        Self {
            grid: SpatialGrid::new(cell_size),
            recorder: Recorder::new(r_type, d_type, particle_count),
            detector: match d_type {
                DetectionType::CellList => Box::new(CellListDetector),
                DetectionType::Tccd => Box::new(TccdDetector),
                DetectionType::SweptAabb => Box::new(SweptAabbDetector),
            },
        }
    }

    pub fn solve(&mut self, particles: &mut [Particle], bounds: &Bounds, mut dt: f32) {
        for _ in 0..MAX_ITER {
            if dt <= EPS_T {
                Self::advance_all(particles, dt);
                break;
            }

            self.grid.rebuild(particles);

            let min_toi = self
                .detector
                .find_min_toi(&mut self.grid, particles, bounds, dt);

            match min_toi {
                Some(toi) => {
                    Self::advance_all(particles, toi.time);
                    self.resolve_collision(particles, bounds, toi);

                    dt -= toi.time;
                }
                None => {
                    Self::advance_all(particles, dt);
                    break;
                }
            }
        }

        Self::clamp_particles(particles, bounds);
    }

    fn resolve_collision(&mut self, particles: &mut [Particle], bounds: &Bounds, toi: Toi) {
        match toi.collision {
            Collision::Pair(i, j) => {
                let p1 = &particles[i];
                let p2 = &particles[j];

                let n = p2.position - p1.position;
                let dist2 = n.dot(n);

                if dist2 == 0.0 {
                    return;
                }

                let n_hat = n / dist2.sqrt();
                let v_rel_n = (p2.velocity - p1.velocity).dot(n_hat);

                if v_rel_n >= 0.0 {
                    return;
                }

                let (m1, m2) = (p1.mass, p2.mass);
                let impulse = (2.0 * m1 * m2 / (m1 + m2)) * v_rel_n * n_hat;

                particles[i].velocity += impulse / m1;
                particles[j].velocity -= impulse / m2;

                let v_rel_n_after = (particles[j].velocity - particles[i].velocity).dot(n_hat);

                self.recorder.write_event_pair((
                    toi.time,
                    i,
                    j,
                    n_hat.x,
                    n_hat.y,
                    v_rel_n,
                    v_rel_n_after,
                ));
            }
            Collision::Wall(i) => {
                let p = &mut particles[i];
                let (hw, hh) = bounds.half_extents();

                let (x_min, x_max) = (-hw + p.radius, hw - p.radius);
                let (y_min, y_max) = (-hh + p.radius, hh - p.radius);

                let n = if p.position.x <= x_min {
                    Vec2::new(-1.0, 0.0)
                } else if p.position.x >= x_max {
                    Vec2::new(1.0, 0.0)
                } else if p.position.y <= y_min {
                    Vec2::new(0.0, -1.0)
                } else {
                    Vec2::new(0.0, 1.0)
                };

                let vn_before = p.velocity.dot(n);

                if p.position.x <= x_min && p.velocity.x < 0.0 {
                    p.position.x = x_min;
                    p.velocity.x *= -1.0;
                } else if p.position.x >= x_max && p.velocity.x > 0.0 {
                    p.position.x = x_max;
                    p.velocity.x *= -1.0;
                }

                if p.position.y <= y_min && p.velocity.y < 0.0 {
                    p.position.y = y_min;
                    p.velocity.y *= -1.0;
                } else if p.position.y >= y_max && p.velocity.y > 0.0 {
                    p.position.y = y_max;
                    p.velocity.y *= -1.0;
                }

                let vn_after = p.velocity.dot(n);
                let wall = if p.position.x <= x_min {
                    "left"
                } else if p.position.x >= x_max {
                    "right"
                } else if p.position.y <= y_min {
                    "bottom"
                } else {
                    "top"
                };

                self.recorder
                    .write_event_wall((toi.time, i, wall, n.x, n.y, vn_before, vn_after));
            }
        }
    }

    #[inline]
    fn advance_all(particles: &mut [Particle], dt: f32) {
        for p in particles {
            p.position += p.velocity * dt;
        }
    }

    fn clamp_particles(particles: &mut [Particle], bounds: &Bounds) {
        let (hw, hh) = bounds.half_extents();

        for p in particles {
            let (x_min, x_max) = (-hw + p.radius, hw - p.radius);
            let (y_min, y_max) = (-hh + p.radius, hh - p.radius);

            if p.position.x < x_min {
                p.position.x = x_min;
                p.velocity.x *= -1.0;
            } else if p.position.x > x_max {
                p.position.x = x_max;
                p.velocity.x *= -1.0;
            }

            if p.position.y < y_min {
                p.position.y = y_min;
                p.velocity.y *= -1.0;
            } else if p.position.y > y_max {
                p.position.y = y_max;
                p.velocity.y *= -1.0;
            }
        }
    }
}
