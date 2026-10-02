//! Playback time shared by model and global sequence tracks.
use super::{Sequence, Track, TrackValue};
use std::ops::RangeInclusive;

/// A borrowed snapshot of playback time, in milliseconds.
///
/// Model tracks loop or clamp within `sequence`; global tracks loop independently
/// using `elapsed_ms` and their indexed duration. A missing sequence only prevents
/// sampling model tracks, so global tracks and static properties still work.
#[derive(Clone, Copy, Debug)]
pub struct AnimationTime<'a> {
    /// Selected model sequence, if available.
    pub sequence: Option<&'a Sequence>,
    /// Elapsed milliseconds since playback started; negative times wrap or clamp.
    pub elapsed_ms: f64,
    /// Global sequence durations in milliseconds, in model order.
    pub global_sequences: &'a [u32],
}

impl AnimationTime<'_> {
    /// Resolves a track's clock and inclusive keyframe interval.
    ///
    /// Returns `None` for a missing model sequence or global sequence ID.
    /// Zero-duration global sequences sample at zero; their keys are unrestricted.
    pub fn track_time<T: TrackValue>(
        &self,
        track: &Track<T>,
    ) -> Option<(f64, RangeInclusive<i32>)> {
        if let Some(global_id) = track.global_sequence_id() {
            let length = *self.global_sequences.get(global_id as usize)?;
            let time = if length == 0 {
                0.0
            } else {
                self.elapsed_ms.rem_euclid(length as f64)
            };
            return Some((time, i32::MIN..=i32::MAX));
        }
        let sequence = self.sequence?;
        let start = sequence.interval[0] as f64;
        let end = sequence.interval[1] as f64;
        let length = (end - start).max(0.0);
        let elapsed = if sequence.flags.non_looping() {
            self.elapsed_ms.clamp(0.0, length)
        } else if length > 0.0 {
            self.elapsed_ms.rem_euclid(length)
        } else {
            0.0
        };
        Some((
            start + elapsed,
            sequence.interval[0] as i32..=sequence.interval[1] as i32,
        ))
    }
}
