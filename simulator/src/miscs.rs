use std::{fs::File, io::BufWriter};

use clap::ValueEnum;
use serde::Serialize;

use crate::particle::Particle;

pub struct Recorder {
    pub frame: u64,
    pub time_s: f32,

    particles_csv: Option<CsvSink>,
    events_csv: Option<CsvSink>,
    checks_csv: Option<CsvSink>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum RecorderType {
    Snapshots,
    Events,
    Checks,
    All,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DetectionType {
    CellList,
    Tccd,
    SweptAabb,
}

impl DetectionType {
    fn tag(&self) -> &'static str {
        match self {
            DetectionType::CellList => "cell_list",
            DetectionType::Tccd => "tccd",
            DetectionType::SweptAabb => "swept_aabb",
        }
    }

    fn make_sink(prefix: &str, tag: &str, count: u64) -> CsvSink {
        CsvSink::new(format!("{prefix}_{tag}_{count}.csv"))
    }
}

impl Recorder {
    pub fn new(r_type: Option<RecorderType>, d_type: DetectionType, particle_count: u64) -> Self {
        let (particles_csv, events_csv, checks_csv) = match r_type {
            None => (None, None, None),
            Some(r) => {
                let tag = d_type.tag();
                let has_particles = matches!(r, RecorderType::Snapshots | RecorderType::All);
                let has_events = matches!(r, RecorderType::Events | RecorderType::All);
                let has_checks = matches!(r, RecorderType::Checks | RecorderType::All);

                let p = if has_particles {
                    Some(DetectionType::make_sink("particles", tag, particle_count))
                } else {
                    None
                };
                let e = if has_events {
                    Some(DetectionType::make_sink("events", tag, particle_count))
                } else {
                    None
                };

                let c = if has_checks {
                    Some(DetectionType::make_sink("checks", tag, particle_count))
                } else {
                    None
                };

                (p, e, c)
            }
        };

        Self {
            frame: 0,
            time_s: 0.0,

            particles_csv,
            events_csv,
            checks_csv,
        }
    }

    pub fn write_particles_snapshot(&mut self, particles: &[Particle]) {
        if let Some(pw) = &mut self.particles_csv {
            for (i, p) in particles.iter().enumerate() {
                if let Err(e) = pw.writer_mut().serialize(ParticleRow {
                    frame: self.frame,
                    time_s: self.time_s,
                    particle_id: i,
                    x: p.position.x,
                    y: p.position.y,
                    vx: p.velocity.x,
                    vy: p.velocity.y,
                    radius: p.radius,
                    mass: p.mass,
                }) {
                    log::error!("Failed to write particle snapshot: {}", e);
                    break;
                }
            }
        }
    }

    pub fn write_event_pair(
        &mut self,
        (toi, i, j, nx, ny, vrel_n_before, vrel_n_after): (f32, usize, usize, f32, f32, f32, f32),
    ) {
        if let Some(ew) = &mut self.events_csv
            && let Err(e) = ew.writer_mut().serialize(EventRow::Pair {
                frame: self.frame,
                time_s: self.time_s + toi,
                toi,
                i,
                j,
                nx,
                ny,
                vrel_n_before,
                vrel_n_after,
            })
        {
            log::error!("Failed to write pair event: {}", e);
        }
    }

    pub fn write_event_wall(
        &mut self,
        (toi, i, wall, nx, ny, vn_before, vn_after): (f32, usize, &'static str, f32, f32, f32, f32),
    ) {
        if let Some(ew) = &mut self.events_csv
            && let Err(e) = ew.writer_mut().serialize(EventRow::Wall {
                frame: self.frame,
                time_s: self.time_s + toi,
                toi,
                i,
                wall,
                nx,
                ny,
                vn_before,
                vn_after,
            })
        {
            log::error!("Failed to write wall event: {}", e);
        }
    }

    pub fn write_check(&mut self, iter: usize, count: u64) {
        if let Some(cw) = &mut self.checks_csv
            && let Err(e) = cw.writer_mut().serialize(CheckRow {
                frame: self.frame,
                time_s: self.time_s,
                count,
                iter,
            })
        {
            log::error!("Failed to write check row: {}", e);
        }
    }

    pub fn flush(&mut self) {
        if self.frame % 60 == 0 {
            if let Some(pw) = &mut self.particles_csv {
                pw.flush();
            }

            if let Some(ew) = &mut self.events_csv {
                ew.flush();
            }

            if let Some(cw) = &mut self.checks_csv {
                cw.flush();
            }
        };
    }
}

pub struct CsvSink {
    name: String,
    writer: csv::Writer<BufWriter<File>>,
}

impl CsvSink {
    fn new(path: String) -> Self {
        let file = File::create(&path).expect("create csv");
        let buf = BufWriter::new(file);
        let writer = csv::WriterBuilder::new().from_writer(buf);

        Self { name: path, writer }
    }

    fn flush(&mut self) {
        if let Err(e) = self.writer.flush() {
            log::error!("Failed to flush {}: {}", self.name, e);
        }
    }

    fn writer_mut(&mut self) -> &mut csv::Writer<BufWriter<File>> {
        &mut self.writer
    }
}

#[derive(Serialize)]
pub struct ParticleRow {
    pub frame: u64,
    pub time_s: f32,
    pub particle_id: usize,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub mass: f32,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum EventRow {
    Pair {
        frame: u64,
        time_s: f32,
        toi: f32,
        i: usize,
        j: usize,
        nx: f32,
        ny: f32,
        vrel_n_before: f32,
        vrel_n_after: f32,
    },
    Wall {
        frame: u64,
        time_s: f32,
        toi: f32,
        i: usize,
        wall: &'static str,
        nx: f32,
        ny: f32,
        vn_before: f32,
        vn_after: f32,
    },
}

#[derive(Serialize)]
pub struct CheckRow {
    pub frame: u64,
    pub time_s: f32,
    pub iter: usize,
    pub count: u64,
}
