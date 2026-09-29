//! Timed animation values and spline tangents.

use crate::model::{mdl, mdx};
/// An interpolation mode shared by all keys in a track.
#[derive(Clone, Copy, Debug, Eq, PartialEq, mdl::Read, mdl::Write)]
#[mdl(value)]
pub enum Interpolation {
    #[mdl(name = "DontInterp")]
    Step,
    Linear,
    Hermite,
    Bezier,
}

/// A keyframe without interpolation tangents.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
pub struct ValueKeyframe<T> {
    /// Signed frame time in milliseconds; negative times represent animation lead-in.
    pub frame: i32,
    /// The value at this frame.
    pub value: T,
}

/// A keyframe with both interpolation tangents.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
pub struct TangentKeyframe<T> {
    /// Signed frame time in milliseconds; negative times represent animation lead-in.
    pub frame: i32,
    /// The value at this frame.
    pub value: T,
    /// Incoming tangent.
    pub in_tangent: T,
    /// Outgoing tangent.
    pub out_tangent: T,
}
