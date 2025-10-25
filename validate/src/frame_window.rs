use std::collections::HashMap;

use crate::miscs::{EventRow, ParticleState};

#[derive(Default)]
pub struct FrameWindow {
    pub frame: u64,
    pub time_s: f32,
    pub particles: HashMap<usize, ParticleState>,
    pub events: Vec<EventRow>,
}

impl FrameWindow {
    pub fn new(frame: u64) -> Self {
        Self {
            frame,
            ..Default::default()
        }
    }
}
