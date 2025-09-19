use glam::{IVec2, Vec2};
use std::collections::HashMap;

use crate::particle::Particle;

pub struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<IVec2, Vec<usize>>,
    r_max: f32,
}

impl SpatialGrid {
    const DIRS: [IVec2; 9] = [
        IVec2::new(-1, -1),
        IVec2::new(-1, 0),
        IVec2::new(-1, 1),
        IVec2::new(0, -1),
        IVec2::new(0, 0),
        IVec2::new(0, 1),
        IVec2::new(1, -1),
        IVec2::new(1, 0),
        IVec2::new(1, 1),
    ];

    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            r_max: 0.0,
        }
    }

    pub fn set_max_radius(&mut self, r: f32) {
        self.r_max = r;
    }

    pub fn rebuild(&mut self, particles: &[Particle]) {
        self.cells.clear();

        for (i, p) in particles.iter().enumerate() {
            let c = self.cell_coord(p.position);

            self.cells.entry(c).or_default().push(i);
        }
    }

    pub fn cell_list<'a>(&'a self, p: &Particle) -> impl Iterator<Item = usize> + 'a {
        let base = self.cell_coord(p.position);

        Self::DIRS.into_iter().flat_map(move |d| {
            let c = base + d;
            self.cells.get(&c).into_iter().flatten().copied()
        })
    }

    pub fn candidates_along_sweep_with_radius<'a>(
        &'a self,
        particles: &'a [Particle],
        i: usize,
        dt: f32,
    ) -> impl Iterator<Item = usize> + 'a {
        use glam::{IVec2, Vec2};
        use std::collections::{HashSet, VecDeque};

        let p = &particles[i];
        let dir: Vec2 = p.velocity;
        let ray = GridRayIter::new(p.position, dir, dt, self.cell_size);

        let kf = ((p.radius + self.r_max) / self.cell_size).ceil().max(1.0);
        let k = kf as i32;

        let mut seen = HashSet::<usize>::new();
        let mut out = VecDeque::<usize>::new();

        let push_cell = |c: IVec2, out: &mut VecDeque<usize>, seen: &mut HashSet<usize>| {
            if let Some(list) = self.cells.get(&c) {
                for &j in list {
                    if j != i && seen.insert(j) {
                        out.push_back(j);
                    }
                }
            }
        };

        for c in ray {
            for dy in -k..=k {
                for dx in -k..=k {
                    push_cell(IVec2::new(c.x + dx, c.y + dy), &mut out, &mut seen);
                }
            }
        }

        out.into_iter()
    }

    pub fn candidates_swept_aabb<'a>(
        &'a self,
        particles: &'a [Particle],
        i: usize,
        dt: f32,
    ) -> impl Iterator<Item = usize> + 'a {
        use glam::{IVec2, Vec2};
        use std::collections::{HashSet, VecDeque};

        let p = &particles[i];
        let p1 = p.position;
        let p2 = p.position + p.velocity * dt;

        let r = p.radius + self.r_max;
        let mins = p1.min(p2) - Vec2::splat(r);
        let maxs = p1.max(p2) + Vec2::splat(r);

        let cmin = self.cell_coord(mins);
        let cmax = self.cell_coord(maxs);

        let mut seen = HashSet::new();
        let mut out = VecDeque::new();

        for cy in cmin.y..=cmax.y {
            for cx in cmin.x..=cmax.x {
                if let Some(list) = self.cells.get(&IVec2::new(cx, cy)) {
                    for &j in list {
                        if j != i && seen.insert(j) {
                            out.push_back(j);
                        }
                    }
                }
            }
        }

        out.into_iter()
    }

    #[inline]
    fn cell_coord(&self, pos: Vec2) -> IVec2 {
        IVec2::new(
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }
}

struct GridRayIter {
    cur: IVec2,
    step: IVec2,
    t_max: Vec2,
    t_delta: Vec2,
    t_remain: f32,
    done: bool,
}

impl GridRayIter {
    fn new(origin: Vec2, dir: Vec2, tmax: f32, cell_size: f32) -> Self {
        let inv = Vec2::new(
            if dir.x != 0.0 { 1.0 / dir.x } else { 0.0 },
            if dir.y != 0.0 { 1.0 / dir.y } else { 0.0 },
        );

        let cell = IVec2::new(
            (origin.x / cell_size).floor() as i32,
            (origin.y / cell_size).floor() as i32,
        );

        let step_x = if dir.x > 0.0 {
            1
        } else if dir.x < 0.0 {
            -1
        } else {
            0
        };
        let step_y = if dir.y > 0.0 {
            1
        } else if dir.y < 0.0 {
            -1
        } else {
            0
        };

        let next_boundary_x = if step_x > 0 {
            (cell.x as f32 + 1.0) * cell_size
        } else {
            (cell.x as f32) * cell_size
        };
        let next_boundary_y = if step_y > 0 {
            (cell.y as f32 + 1.0) * cell_size
        } else {
            (cell.y as f32) * cell_size
        };

        let tx = if step_x != 0 && dir.x != 0.0 {
            (next_boundary_x - origin.x) * inv.x
        } else {
            f32::INFINITY
        };

        let ty = if step_y != 0 && dir.y != 0.0 {
            (next_boundary_y - origin.y) * inv.y
        } else {
            f32::INFINITY
        };

        let tdx = if step_x != 0 && dir.x != 0.0 {
            (cell_size * step_x as f32) * inv.x
        } else {
            f32::INFINITY
        };

        let tdy = if step_y != 0 && dir.y != 0.0 {
            (cell_size * step_y as f32) * inv.y
        } else {
            f32::INFINITY
        };

        Self {
            cur: cell,
            step: IVec2::new(step_x, step_y),
            t_max: Vec2::new(tx.max(0.0), ty.max(0.0)),
            t_delta: Vec2::new(tdx.abs(), tdy.abs()),
            t_remain: tmax.max(0.0),
            done: false,
        }
    }
}

impl Iterator for GridRayIter {
    type Item = IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let out = self.cur;

        if self.t_remain <= 0.0 || (self.t_max.x.is_infinite() && self.t_max.y.is_infinite()) {
            self.done = true;
            return Some(out);
        }

        if self.t_max.x < self.t_max.y {
            if self.t_max.x > self.t_remain {
                self.done = true;
                return Some(out);
            }

            self.cur.x += self.step.x;
            self.t_max.x += self.t_delta.x;
        } else {
            if self.t_max.y > self.t_remain {
                self.done = true;
                return Some(out);
            }

            self.cur.y += self.step.y;
            self.t_max.y += self.t_delta.y;
        }

        Some(out)
    }
}
