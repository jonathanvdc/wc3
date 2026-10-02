//! Animation sequences, looping timelines, and typed keyframes.
//!
//! Sequence intervals and keyframe times are in milliseconds. Tracks can use a
//! model sequence or a global sequence that loops independently. Use
//! [`Track`] constructors to choose stepped, linear, Hermite, or Bezier
//! interpolation; [`ValueKeyframe`] and [`TangentKeyframe`] hold the matching keys.
mod track_value;
pub use track_value::TrackValue;
mod keyframe;
pub use keyframe::{Interpolation, Keyframe, TangentKeyframe, ValueKeyframe};
mod track;
pub use track::Track;
mod animatable;
pub use animatable::Animatable;
mod sequence;
pub use sequence::{Sequence, SequenceFlags};
mod texture_animation;
pub use texture_animation::TextureAnimation;
mod geoset_animation;
pub use geoset_animation::{GeosetAnimation, GeosetAnimationFlags};
mod global_sequence;
pub use global_sequence::GlobalSequence;

mod interpolate;
pub use interpolate::Interpolate;

mod time;
pub use time::AnimationTime;
