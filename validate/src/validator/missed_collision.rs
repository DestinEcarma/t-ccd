use glam::Vec2;
use serde::Serialize;

use crate::{
    frame_window::FrameWindow,
    miscs::{EventRow, ParticleState},
    validator::{StreamingValidator, comp, report},
};

const MAX_ITER: usize = 100;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum MissedCollision {
    Pair {
        frame: u64,
        toi: f32,
        iter: usize,
        i: usize,
        ix: f32,
        iy: f32,
        j: usize,
        jx: f32,
        jy: f32,
    },
    Wall {
        frame: u64,
        toi: f32,
        iter: usize,
        i: usize,
        x: f32,
        y: f32,
        _j: Option<usize>,
        _x: Option<f32>,
        _y: Option<f32>,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Toi {
    time: f32,
    collision: Collision,
}

#[derive(Debug, Clone, Copy)]
pub enum Collision {
    Pair(usize, usize),
    Wall(usize),
}

impl From<(f32, Collision)> for Toi {
    fn from(value: (f32, Collision)) -> Self {
        Toi {
            time: value.0,
            collision: value.1,
        }
    }
}

impl StreamingValidator {
    pub(super) fn find_missed_collisions(
        &self,
        window: &mut FrameWindow,
        events: &[EventRow],
        mut dt: f32,
    ) -> Vec<MissedCollision> {
        let mut missed = Vec::new();
        let keys = window.particles.keys().copied().collect::<Vec<_>>();

        for iter in 0..MAX_ITER {
            if dt <= self.tolerance {
                break;
            }

            let mut min_toi = None::<(Toi, bool)>;

            for i in 0..keys.len() {
                let i = keys[i];
                let p1 = &window.particles[&i];

                for j in (i + 1)..keys.len() {
                    let j = keys[j];

                    let p2 = &window.particles[&j];

                    if let Some(t) = comp::p2p_toi(p1, p2, dt, self.tolerance) {
                        let toi = events.iter().find(|e| {
                            matches!(e, EventRow::Pair { frame, iter: eiter, i: ei, j: ej, .. }
                                if *frame == window.frame && ((*ei == i && *ej == j) || (*ei == j && *ej == i)))
                        }).map(|e| match e {
                            EventRow::Pair {toi, ..} => toi,
                            EventRow::Wall {toi, ..} => toi,
                        });

                        if !min_toi.is_some_and(|(toi, ..)| t >= toi.time) {
                            min_toi = Some((
                                Toi::from((*toi.unwrap_or(&t), Collision::Pair(i, j))),
                                toi.is_some(),
                            ));
                        }
                    }
                }

                if let Some(t) = comp::boundary_toi(p1, &self.boundary, dt) {
                    let toi = events
                        .iter()
                        .find(|e| {
                            matches!(e, EventRow::Wall { frame, iter: eiter, i: ei,  .. }
                                if *frame == window.frame &&  *ei == i)
                        })
                        .map(|e| match e {
                            EventRow::Pair { toi, .. } => toi,
                            EventRow::Wall { toi, .. } => toi,
                        });

                    if !min_toi.is_some_and(|(toi, ..)| t >= toi.time) {
                        min_toi = Some((
                            Toi::from((*toi.unwrap_or(&t), Collision::Wall(i))),
                            toi.is_some(),
                        ));
                    }
                }
            }

            if let Some((toi, was_reported)) = min_toi {
                if (!was_reported) {
                    match toi.collision {
                        Collision::Pair(i, j) => {
                            let p1 = &window.particles[&i];
                            let p2 = &window.particles[&j];

                            missed.push(MissedCollision::Pair {
                                frame: window.frame,
                                toi: toi.time,
                                iter,
                                i,
                                ix: p1.position.x,
                                iy: p1.position.y,
                                j,
                                jx: p2.position.x,
                                jy: p2.position.y,
                            });
                        }
                        Collision::Wall(i) => {
                            let p = &window.particles[&i];

                            missed.push(MissedCollision::Wall {
                                frame: window.frame,
                                toi: toi.time,
                                iter,
                                i,
                                x: p.position.x,
                                y: p.position.y,
                                _j: None,
                                _x: None,
                                _y: None,
                            });
                        }
                    }
                }

                self.resolve_collision(window, &toi);

                dt -= toi.time;
            }
        }

        missed
    }

    fn resolve_collision(&self, window: &mut FrameWindow, toi: &Toi) {
        match toi.collision {
            Collision::Pair(i, j) => {
                let p1 = &window.particles[&i];
                let p2 = &window.particles[&j];

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

                window.particles.get_mut(&i).unwrap().velocity += impulse / m1;
                window.particles.get_mut(&j).unwrap().velocity -= impulse / m2;
            }
            Collision::Wall(i) => {
                let p = window.particles.get_mut(&i).unwrap();

                let (x_min, y_min) = self.boundary.get_min(p.radius);
                let (x_max, y_max) = self.boundary.get_max(p.radius);

                let n = if p.position.x <= x_min {
                    Vec2::new(-1.0, 0.0)
                } else if p.position.x >= x_max {
                    Vec2::new(1.0, 0.0)
                } else if p.position.y <= y_min {
                    Vec2::new(0.0, -1.0)
                } else {
                    Vec2::new(0.0, 1.0)
                };

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
            }
        }
    }
}
