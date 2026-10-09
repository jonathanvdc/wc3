//! Playback, track sampling, and node pose evaluation.
pub(crate) mod clocks;
mod playback;
pub(crate) mod pose;
mod sampling;

pub use playback::Wc3Animation;
pub(crate) use playback::{advance_animation, PosePlayback};
pub(crate) use sampling::{sample, sample_value, track_time};
