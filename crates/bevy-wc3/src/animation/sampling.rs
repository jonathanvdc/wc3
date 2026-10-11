use super::Wc3Animation;
use std::ops::RangeInclusive;
use wc3::model::animation::{Animatable, Interpolate, Track, TrackValue};

pub(crate) fn track_time<'data, T: TrackValue>(
    track: &Track<'data, T>,
    animation: &Wc3Animation,
) -> Option<(f64, RangeInclusive<i32>)> {
    animation.time().track_time(track)
}

pub(crate) fn sample<'data, T: Interpolate>(
    track: &Track<'data, T>,
    animation: &Wc3Animation,
) -> Option<T> {
    track.sample(&animation.time())
}

pub(crate) fn sample_value<'data, T: Interpolate>(
    value: &Animatable<'data, T>,
    animation: &Wc3Animation,
) -> T {
    value.sample(&animation.time()).unwrap_or_default()
}

#[cfg(test)]
#[path = "sampling_tests.rs"]
mod tests;
