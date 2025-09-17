use engine::Bounds;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    particle::Particle,
    solver::{Collision, Toi},
    spatial::SpatialGrid,
};

pub trait Detector {
    fn find_min_toi(
        &mut self,
        grid: &mut SpatialGrid,
        particles: &[Particle],
        bounds: &Bounds,
        dt: f32,
    ) -> Option<Toi>;
}

pub struct CellListDetector;
pub struct TccdDetector;
pub struct SweptAabbDetector;

impl Detector for CellListDetector {
    fn find_min_toi(
        &mut self,
        grid: &mut SpatialGrid,
        particles: &[Particle],
        bounds: &Bounds,
        dt: f32,
    ) -> Option<Toi> {
        particles
            .par_iter()
            .enumerate()
            .filter_map(|(i, p)| {
                let mut min_toi = None;
                let mut dt_i = dt;

                for j in grid.cell_list(p) {
                    if j <= i {
                        continue;
                    }

                    if let Some(t) = p2p_toi(p, &particles[j], dt_i)
                        && !min_toi.is_some_and(|toi: Toi| t >= toi.time)
                    {
                        dt_i = t;
                        min_toi = Some(Toi::from((t, Collision::Pair(i, j))));
                    }
                }

                if let Some(t) = boundary_toi(p, bounds, dt_i)
                    && !min_toi.is_some_and(|toi: Toi| t >= toi.time)
                {
                    min_toi = Some(Toi::from((t, Collision::Wall(i))));
                }

                min_toi
            })
            .min_by(|a, b| a.time.partial_cmp(&b.time).unwrap())
    }
}

impl Detector for TccdDetector {
    fn find_min_toi(
        &mut self,
        grid: &mut SpatialGrid,
        particles: &[Particle],
        bounds: &Bounds,
        dt: f32,
    ) -> Option<Toi> {
        particles
            .par_iter()
            .enumerate()
            .with_min_len(64)
            .filter_map(|(i, p1)| {
                let mut min_toi = None;
                let mut dt_i = dt;

                for j in grid.candidates_along_sweep_with_radius(particles, i, dt_i) {
                    if j <= i {
                        continue;
                    }

                    if let Some(t) = p2p_toi(p1, &particles[j], dt_i)
                        && !min_toi.is_some_and(|toi: Toi| t >= toi.time)
                    {
                        dt_i = t;
                        min_toi = Some(Toi::from((t, Collision::Pair(i, j))));
                    }
                }

                if dt_i > 0.0
                    && let Some(t) = boundary_toi(p1, bounds, dt_i)
                    && !min_toi.is_some_and(|toi: Toi| t >= toi.time)
                {
                    min_toi = Some(Toi::from((t, Collision::Wall(i))));
                }

                min_toi
            })
            .min_by(|a, b| a.time.partial_cmp(&b.time).unwrap())
    }
}

impl Detector for SweptAabbDetector {
    fn find_min_toi(
        &mut self,
        grid: &mut SpatialGrid,
        particles: &[Particle],
        bounds: &Bounds,
        dt: f32,
    ) -> Option<Toi> {
        particles
            .par_iter()
            .enumerate()
            .with_min_len(64)
            .filter_map(|(i, p1)| {
                let mut min_toi = None;
                let mut dt_i = dt;

                for j in grid.candidates_swept_aabb(particles, i, dt_i) {
                    if j <= i {
                        continue;
                    }

                    if let Some(t) = p2p_toi(p1, &particles[j], dt_i)
                        && !min_toi.is_some_and(|toi: Toi| t >= toi.time)
                    {
                        dt_i = t;
                        min_toi = Some(Toi::from((t, Collision::Pair(i, j))));
                    }
                }

                if let Some(t) = boundary_toi(p1, bounds, dt_i)
                    && !min_toi.is_some_and(|toi: Toi| t >= toi.time)
                {
                    min_toi = Some(Toi::from((t, Collision::Wall(i))));
                }

                min_toi
            })
            .min_by(|a, b| a.time.partial_cmp(&b.time).unwrap())
    }
}

fn p2p_toi(p1: &Particle, p2: &Particle, dt: f32) -> Option<f32> {
    let dp = p2.position - p1.position;
    let dv = p2.velocity - p1.velocity;
    let r = p1.radius + p2.radius;

    let a = dv.dot(dv);
    let b = 2.0 * dp.dot(dv);
    let c = dp.dot(dp) - r * r;

    if c <= 0.0 {
        return None;
    }

    if a <= 1e-12 {
        return None;
    }

    if b >= 0.0 {
        return None;
    }

    let disc = b * b - 4.0 * a * c;

    if disc < 0.0 {
        return None;
    }

    let sqrt_d = disc.sqrt();
    let t_min = (-b - sqrt_d) / (2.0 * a);

    match t_min >= 0.0 && t_min <= dt {
        true => Some(t_min),
        false => None,
    }
}

fn boundary_toi(p: &Particle, bounds: &Bounds, dt: f32) -> Option<f32> {
    let (hw, hh) = bounds.half_extents();
    let pos = p.position;
    let vel = p.velocity;
    let r = p.radius;

    let (x_min, x_max) = (-hw + r, hw - r);
    let (y_min, y_max) = (-hh + r, hh - r);

    let mut t_min = f32::INFINITY;

    if vel.x > 0.0 {
        let t = (x_max - pos.x) / vel.x;

        if t >= 0.0 && t <= dt {
            t_min = t_min.min(t);
        }
    } else if vel.x < 0.0 {
        let t = (x_min - pos.x) / vel.x;

        if t >= 0.0 && t <= dt {
            t_min = t_min.min(t);
        }
    }

    if vel.y > 0.0 {
        let t = (y_max - pos.y) / vel.y;

        if t >= 0.0 && t <= dt {
            t_min = t_min.min(t);
        }
    } else if vel.y < 0.0 {
        let t = (y_min - pos.y) / vel.y;

        if t >= 0.0 && t <= dt {
            t_min = t_min.min(t);
        }
    }

    match t_min.is_finite() {
        true => Some(t_min),
        false => None,
    }
}
