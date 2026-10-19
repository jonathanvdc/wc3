//! Transport records for animation sequences, clocks, and typed keyframes.
//!
//! Sequence intervals and keyframe times use milliseconds. Track constructors
//! select interpolation and its matching value or tangent key representation.
//! Readers and writers retain authored key and event order, including duplicates.
//! Import into a semantic [`Model`](crate::model::Model) to canonicalize curves,
//! resolve playback clocks, sample properties, and traverse event occurrences.
mod track_value;
pub use track_value::TrackValue;
mod keyframe;
pub use keyframe::{Interpolation, TangentKeyframe, ValueKeyframe};
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
