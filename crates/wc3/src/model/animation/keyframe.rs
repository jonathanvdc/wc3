//! Keyframe values and their binary representations.

use crate::model::mdx;
/// An interpolation mode shared by all keys in a track.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Interpolation {
    Step,
    Linear,
    Hermite,
    Bezier,
}

/// A keyframe without interpolation tangents.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
pub struct ValueKeyframe<T> {
    /// Frame time in milliseconds.
    pub frame: u32,
    /// The value at this frame.
    pub value: T,
}

/// A keyframe with both interpolation tangents.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
pub struct TangentKeyframe<T> {
    /// Frame time in milliseconds.
    pub frame: u32,
    /// The value at this frame.
    pub value: T,
    /// Incoming tangent.
    pub in_tangent: T,
    /// Outgoing tangent.
    pub out_tangent: T,
}
