mod cli;
mod detector;
mod miscs;
mod particle;
mod solver;
mod spatial;

use clap::Parser;
use engine::{Bounds, Simulation, SimulationConfig};
use glam::Vec2;
use rand::{Rng, SeedableRng, rngs::StdRng};
use rayon::ThreadPoolBuilder;

use crate::{cli::Cli, particle::Particle, solver::Solver};

const SPEED: f32 = 500.0;

struct TCcdSim {
    particles: Vec<Particle>,
    solver: Solver,

    _seed: Option<u64>,
    _min_radius: f32,
    _max_radius: f32,
}

impl Simulation for TCcdSim {
    type Instance = Particle;

    fn init(&mut self, bounds: Bounds) {
        let (hw, hh) = bounds.half_extents();
        let mut rng = if let Some(seed) = self._seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_os_rng()
        };

        self.particles.iter_mut().for_each(|p| {
            p.position = Vec2::new(
                rng.random_range(-0.9 * hw..0.9 * hw),
                rng.random_range(-0.9 * hh..0.9 * hh),
            );
            p.velocity = Vec2::new(
                rng.random_range(-SPEED..SPEED),
                rng.random_range(-SPEED..SPEED),
            );

            if (self._max_radius - self._min_radius).abs() < f32::EPSILON {
                p.radius = self._min_radius;
            } else {
                p.radius = rng.random_range(self._min_radius..self._max_radius);
            }

            p.mass = std::f32::consts::PI * p.radius * p.radius;
            p.color = [rng.random(), 1.0, 0.5];
        });

        self.solver.recorder.frame += 1;
        self.solver
            .recorder
            .write_particles_snapshot(&self.particles);
    }

    fn step(&mut self, dt: f32, bounds: engine::Bounds) {
        self.solver.solve(&mut self.particles, &bounds, dt);

        self.solver.recorder.frame += 1;
        self.solver.recorder.time_s += dt;
        self.solver
            .recorder
            .write_particles_snapshot(&self.particles);
        self.solver.recorder.flush();
    }

    fn instances(&self) -> &[Self::Instance] {
        &self.particles
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    ThreadPoolBuilder::new()
        .num_threads(num_cpus::get().saturating_sub(4).max(1))
        .build_global()
        .unwrap();

    if cli.min_radius > cli.max_radius {
        log::error!("Min radius cannot be greater than max radius");
        return Ok(());
    }

    if cli.min_radius < 1.0 || cli.max_radius < 1.0 {
        log::error!("Radii must be at least 1.0");
        return Ok(());
    }

    engine::run_with(
        TCcdSim {
            particles: vec![Particle::default(); cli.particle_count as usize],
            solver: Solver::new(cli.cell_size, cli.record, cli.method, cli.particle_count),

            _seed: cli.seed,
            _min_radius: cli.min_radius,
            _max_radius: cli.max_radius,
        },
        SimulationConfig {
            fullscreen: cli.fullscreen,
            fps: cli.fps,
        },
    )?;

    Ok(())
}
