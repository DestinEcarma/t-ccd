use serde::Serialize;

use crate::{
    frame_window::FrameWindow,
    miscs::EventRow,
    validator::{StreamingValidator, comp},
};

#[derive(Serialize)]
pub struct MissedCollision {
    pub frame: u64,
    pub i: usize,
    pub j: usize,
    pub ix: f32,
    pub iy: f32,
    pub jx: f32,
    pub jy: f32,
    pub toi: f32,
}

impl StreamingValidator {
    pub(super) fn find_missed_collisions(
        &self,
        curr: &FrameWindow,
        next: &FrameWindow,
        events: &[EventRow],
        dt: f32,
    ) -> Vec<MissedCollision> {
        let mut missed = Vec::new();
        let keys = curr.particles.keys().copied().collect::<Vec<_>>();

        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                let i = keys[i];
                let j = keys[j];

                let p1 = &curr.particles[&i];
                let p2 = &curr.particles[&j];

                if let Some(toi) = comp::p2p_toi(p1, p2, dt) {
                    let was_reported = events.iter().any(|e| {
                        matches!(e, EventRow::Pair { i: ei, j: ej, .. }
                                 if (*ei == i && *ej == j) || (*ei == j && *ej == i))
                    });

                    if !was_reported {
                        missed.push(MissedCollision {
                            frame: next.frame,
                            i,
                            j,
                            ix: p1.position.x,
                            iy: p1.position.y,
                            jx: p2.position.x,
                            jy: p2.position.y,
                            toi,
                        });
                    }
                }
            }
        }

        missed
    }
}
