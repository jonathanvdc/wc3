//! Sampling clocks shared by live animation and offline preparation.
use wc3::model::animation::{Animatable, AnimationTime, Interpolate, Track, TrackValue};

#[derive(Clone, Copy)]
pub(crate) struct SamplingTime<'a> {
    pub(crate) model: AnimationTime<'a>,
    pub(crate) global_elapsed_ms: f64,
}

impl<'a> SamplingTime<'a> {
    pub(crate) fn live(model: AnimationTime<'a>) -> Self {
        Self {
            global_elapsed_ms: model.elapsed_ms,
            model,
        }
    }

    fn clock(&self, global: bool) -> AnimationTime<'a> {
        AnimationTime {
            elapsed_ms: if global {
                self.global_elapsed_ms
            } else {
                self.model.elapsed_ms
            },
            ..self.model
        }
    }

    pub(crate) fn track<T: Interpolate>(&self, track: &Track<T>) -> Option<T> {
        track.sample(&self.clock(track.global_sequence_id().is_some()))
    }

    pub(crate) fn sample<T: TrackValue + Interpolate>(&self, value: &Animatable<T>) -> Option<T> {
        value
            .track()
            .and_then(|track| self.track(track))
            .or_else(|| value.value().copied())
    }
}
