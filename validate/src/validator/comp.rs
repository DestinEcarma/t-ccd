use std::collections::HashMap;

use crate::{boundary::Boundary, miscs::ParticleState};

pub fn p2p_toi(p1: &ParticleState, p2: &ParticleState, dt: f32) -> Option<f32> {
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

fn boundary_toi(p: &ParticleState, boundary: &Boundary, dt: f32) -> Option<f32> {
    let pos = p.position;
    let vel = p.velocity;
    let r = p.radius;

    let (x_min, y_min) = boundary.get_min(r);
    let (x_max, y_max) = boundary.get_max(r);

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

pub fn compute_totals(particles: &HashMap<usize, ParticleState>) -> (f32, f32, f32) {
    let mut ke = 0.0;
    let mut px = 0.0;
    let mut py = 0.0;

    for p in particles.values() {
        let v_squared = p.velocity.length_squared();
        ke += 0.5 * p.mass * v_squared;

        px += p.mass * p.velocity.x;
        py += p.mass * p.velocity.y;
    }

    (ke, px, py)
}
