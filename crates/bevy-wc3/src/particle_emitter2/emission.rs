//! PRE2 squirt scheduling is independent of continuous emission phase.
use wc3::model::animation::Animatable;

use crate::animation::{track_time, Wc3Animation};

pub(super) struct SquirtTracker {
    last_cycle: i64,
    last_key: Option<i32>,
}
impl Default for SquirtTracker {
    fn default() -> Self {
        Self {
            last_cycle: -1,
            last_key: None,
        }
    }
}
impl SquirtTracker {
    pub(super) fn observe(
        &mut self,
        rate: &Animatable<f32>,
        animation: &Wc3Animation,
        reset: bool,
    ) {
        let cycle =
            if let Some(global_id) = rate.track().and_then(|track| track.global_sequence_id()) {
                let length = animation
                    .global_sequences
                    .get(global_id as usize)
                    .copied()
                    .unwrap_or(0);
                if length == 0 {
                    0
                } else {
                    (animation.elapsed_ms / f64::from(length)).floor() as i64
                }
            } else {
                animation
                    .sequences
                    .get(animation.sequence)
                    .map(|sequence| {
                        let length = sequence.interval[1].saturating_sub(sequence.interval[0]);
                        if sequence.flags.non_looping() || length == 0 {
                            0
                        } else {
                            (animation.elapsed_ms / f64::from(length)).floor() as i64
                        }
                    })
                    .unwrap_or(0)
            };
        if reset || cycle != self.last_cycle {
            self.last_key = None;
        }
        self.last_cycle = cycle;
    }

    /// Return the age of a new burst, or None if its key already fired.
    pub(super) fn burst_age(
        &mut self,
        rate: &Animatable<f32>,
        animation: &Wc3Animation,
        dt: f64,
    ) -> Option<f64> {
        let key = emission_key(rate, animation).or(Some(0));
        if self.last_key == key {
            return None;
        }
        self.last_key = key;
        Some(
            rate.track()
                .and_then(|track| track_time(track, animation))
                .map(|(time, _)| (time - f64::from(key.unwrap_or(0))).max(0.0) * 0.001)
                .unwrap_or(dt)
                .min(dt),
        )
    }
}

fn emission_key(value: &Animatable<f32>, animation: &Wc3Animation) -> Option<i32> {
    let track = value.track()?;
    let (time, interval) = track_time(track, animation)?;
    track.key_frame_at_or_before_in(time, interval)
}
