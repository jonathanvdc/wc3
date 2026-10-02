use super::Wc3Animation;
use std::ops::RangeInclusive;
use wc3::model::animation::{Animatable, Interpolate, Track, TrackValue};

pub(crate) fn track_time<T: TrackValue>(
    track: &Track<T>,
    animation: &Wc3Animation,
) -> Option<(f64, RangeInclusive<i32>)> {
    if let Some(global_id) = track.global_sequence_id() {
        let length = *animation.global_sequences.get(global_id as usize)?;
        if length == 0 {
            return Some((0.0, i32::MIN..=i32::MAX));
        }
        return Some((
            animation.elapsed_ms.rem_euclid(length as f64),
            i32::MIN..=i32::MAX,
        ));
    }
    let sequence = animation.sequences.get(animation.sequence)?;
    let start = sequence.interval[0] as f64;
    let end = sequence.interval[1] as f64;
    let length = (end - start).max(0.0);
    let elapsed = if sequence.flags.non_looping() {
        animation.elapsed_ms.clamp(0.0, length)
    } else if length > 0.0 {
        animation.elapsed_ms.rem_euclid(length)
    } else {
        0.0
    };
    Some((
        start + elapsed,
        sequence.interval[0] as i32..=sequence.interval[1] as i32,
    ))
}

pub(crate) fn sample<T: Interpolate>(track: &Track<T>, animation: &Wc3Animation) -> Option<T> {
    let (time, interval) = track_time(track, animation)?;
    track.evaluate_in(time, interval)
}

pub(crate) fn sample_value<T: Interpolate>(value: &Animatable<T>, animation: &Wc3Animation) -> T {
    value
        .track()
        .and_then(|track| sample(track, animation))
        .or_else(|| value.value().copied())
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "sampling_tests.rs"]
mod tests;
