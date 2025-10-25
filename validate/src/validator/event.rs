use glam::Vec2;

use crate::{
    frame_window::FrameWindow,
    miscs::EventRow,
    validator::{StreamingValidator, error::ValidationError},
};

impl StreamingValidator {
    pub(super) fn validate_event(
        &self,
        event: &EventRow,
        window: &FrameWindow,
    ) -> Result<(), ValidationError> {
        match event {
            EventRow::Pair {
                frame,
                toi,
                i,
                j,
                ix,
                iy,
                jx,
                jy,
                nx,
                ny,
                vrel_n_before,
                vrel_n_after,
                ..
            } => {
                let pi = window
                    .particles
                    .get(i)
                    .ok_or(ValidationError::ParticleNotFound(*i))?;
                let pj = window
                    .particles
                    .get(j)
                    .ok_or(ValidationError::ParticleNotFound(*j))?;

                let i_pos = Vec2::new(*ix, *iy);
                let j_pos = Vec2::new(*jx, *jy);

                let n_hat = (i_pos - j_pos).normalize();
                let dist = (i_pos - j_pos).length();
                let contact_dist = pi.radius + pj.radius;

                let relative_error = ((dist - contact_dist) / contact_dist).abs();
                if relative_error > self.tolerance {
                    return Err(ValidationError::NotTouching {
                        distance: dist,
                        expected: contact_dist,
                    });
                }

                let expected_n = Vec2::new(*nx, *ny);
                let dot = n_hat.dot(expected_n);
                if dot.abs() < (1.0 - self.tolerance) {
                    return Err(ValidationError::WrongNormal);
                }

                let expected_after = -vrel_n_before;
                let diff = (vrel_n_after - expected_after).abs();
                let magnitude = vrel_n_before.abs().max(1.0);
                if diff > self.tolerance * magnitude {
                    return Err(ValidationError::NotElastic {
                        before: *vrel_n_before,
                        after: *vrel_n_after,
                        expected: expected_after,
                    });
                }
                Ok(())
            }
            EventRow::Wall {
                frame,
                toi,
                i,
                vn_before,
                vn_after,
                ..
            } => {
                let expected_after = -vn_before;
                if (vn_after - expected_after).abs() > self.tolerance {
                    return Err(ValidationError::NotElastic {
                        before: *vn_before,
                        after: *vn_after,
                        expected: expected_after,
                    });
                }
                Ok(())
            }
        }
    }
}
