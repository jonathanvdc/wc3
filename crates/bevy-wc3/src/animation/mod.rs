//! Playback, track sampling, and node pose evaluation.
mod playback;
pub(crate) mod pose;
mod sampling;

pub(crate) use playback::advance_animation;
pub use playback::Wc3Animation;
pub(crate) use sampling::{sample, sample_value, track_time};
