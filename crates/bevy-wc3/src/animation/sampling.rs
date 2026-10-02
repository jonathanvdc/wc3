use super::Wc3Animation;
use std::ops::RangeInclusive;
use wc3::model::animation::{Animatable, Interpolate, Track, TrackValue};

pub(crate) fn track_time<T: TrackValue>(
    track: &Track<T>,
    animation: &Wc3Animation,
) -> Option<(f64, RangeInclusive<i32>)> {
    animation.time().track_time(track)
}

pub(crate) fn sample<T: Interpolate>(track: &Track<T>, animation: &Wc3Animation) -> Option<T> {
    track.sample(&animation.time())
}

pub(crate) fn sample_value<T: Interpolate>(value: &Animatable<T>, animation: &Wc3Animation) -> T {
    value.sample(&animation.time()).unwrap_or_default()
}

#[cfg(test)]
#[path = "sampling_tests.rs"]
mod tests;
