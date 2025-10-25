use std::fs::File;

use anyhow::Result;
use csv::Reader;

use crate::{
    frame_window::FrameWindow,
    miscs::{EventRow, ParticleRow, ParticleState},
};

pub struct BufferedParticleReader {
    reader: Reader<File>,
    peeked: Option<ParticleRow>,
}

impl BufferedParticleReader {
    pub fn new(reader: Reader<File>) -> Self {
        Self {
            reader,
            peeked: None,
        }
    }

    pub fn read_frame(&mut self, frame: u64) -> Result<FrameWindow> {
        let mut window = FrameWindow::new(frame);

        if let Some(particle) = self.peeked.take() {
            if particle.frame == frame {
                window
                    .particles
                    .insert(particle.particle_id, ParticleState::from(particle));
            } else {
                self.peeked = Some(particle);
                return Ok(window);
            }
        }

        for result in self.reader.deserialize::<ParticleRow>() {
            let row = result?;

            if row.frame == frame {
                window.time_s = row.time_s;
                window
                    .particles
                    .insert(row.particle_id, ParticleState::from(row));
            } else if row.frame < frame {
                continue;
            } else {
                self.peeked = Some(row);
                break;
            }
        }

        Ok(window)
    }
}

pub struct BufferedEventReader {
    reader: Reader<File>,
    peeked: Option<EventRow>,
}

impl BufferedEventReader {
    pub fn new(reader: Reader<File>) -> Self {
        Self {
            reader,
            peeked: None,
        }
    }

    pub fn read_frame(&mut self, frame: u64) -> Result<Vec<EventRow>> {
        let mut events = Vec::new();

        if let Some(peeked_row) = self.peeked.take() {
            let peeked_frame = match peeked_row {
                EventRow::Pair { frame, .. } => frame,
                EventRow::Wall { frame, .. } => frame,
            };

            if peeked_frame == frame {
                events.push(peeked_row);
            } else if peeked_frame > frame {
                self.peeked = Some(peeked_row);
                return Ok(events);
            }
        }

        for result in self.reader.deserialize::<EventRow>() {
            let row = result?;

            let row_frame = match row {
                EventRow::Pair { frame, .. } => frame,
                EventRow::Wall { frame, .. } => frame,
            };

            if row_frame == frame {
                events.push(row);
            } else if row_frame < frame {
                continue;
            } else {
                self.peeked = Some(row);
                break;
            }
        }

        Ok(events)
    }
}
