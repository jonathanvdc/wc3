//! Animation sequences, looping timelines, and typed keyframes.
//!
//! Sequence intervals and keyframe times are in milliseconds. Tracks can use a
//! model sequence or a global sequence that loops independently. Use
//! [`Track`] constructors to choose stepped, linear, Hermite, or Bezier
//! interpolation; [`ValueKeyframe`] and [`TangentKeyframe`] hold the matching keys.
//!
//! # Sample a property
//!
//! Construct a track, select a sequence, and supply elapsed playback time. Keys
//! use authored model-frame milliseconds; elapsed time starts at zero relative
//! to the selected sequence's start.
//!
//! ```
//! use wc3::model::mdlx::animation::{AnimationTime, Sequence, Track, ValueKeyframe};
//!
//! let sequence = Sequence::new("Stand", [100, 1100])?;
//! let track = Track::linear([
//!     ValueKeyframe { frame: 100, value: 0.0_f32 },
//!     ValueKeyframe { frame: 1100, value: 1.0_f32 },
//! ], None)?;
//! let time = AnimationTime {
//!     sequence: Some(&sequence),
//!     elapsed_ms: 500.0,
//!     global_sequences: &[],
//! };
//! assert_eq!(track.sample(&time), Some(0.5));
//! # Ok::<(), wc3::model::mdlx::ValueError>(())
//! ```
//!
//! [`AnimationTime`] resolves looping or clamped sequence time and a global
//! track's indexed duration. Global tracks use the same elapsed time but loop
//! according to their own durations; they can be sampled without a model sequence.
//! Missing clocks or applicable keys produce `None`. [`Animatable`] uses its base
//! value when its track cannot supply a sample.
//!
//! Constructors sort keys stably, retaining duplicate timestamps. Sampling uses
//! the last key at a timestamp. Key accessors borrow encoded ZeroVec arrays;
//! `get()` and `iter()` return ordinary keyframe values. Model and event codecs
//! preserve the resulting sorted order when writing.
//!
//! [`AnimationTime::event_occurrences`] traverses crossed event keys for tools
//! implementing playback; it does not interpret event names or perform effects.
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
mod events;
pub use events::EventOccurrence;
