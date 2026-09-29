//! Animation sequences, looping timelines, and typed keyframes.
//!
//! Sequence intervals and keyframe times are in milliseconds. Tracks can use a
//! model sequence or a global sequence that loops independently. Use
//! [`AnimationTrack`] constructors to choose stepped, linear, Hermite, or Bezier
//! interpolation; [`ValueKeyframe`] and [`TangentKeyframe`] hold the matching keys.
mod track_kind;
pub use track_kind::*;
mod keyframe;
pub use keyframe::{Interpolation, TangentKeyframe, ValueKeyframe};
mod track;
pub use track::AnimationTrack;
mod track_group;
pub(crate) use track_group::track_group;
mod sequence;
pub use sequence::{Sequence, SequenceFlags};
mod texture_animation;
pub use texture_animation::{TextureAnimation, TextureAnimationTrack};
mod geoset_animation;
pub use geoset_animation::{GeosetAnimation, GeosetAnimationFlags, GeosetTrack};
mod global_sequence;
pub use global_sequence::GlobalSequence;
