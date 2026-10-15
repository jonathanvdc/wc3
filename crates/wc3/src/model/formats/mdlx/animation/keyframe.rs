//! Timed animation values and spline tangents.

use crate::model::formats::mdlx::{mdl, mdx};

/// A keyframe with a timestamp in milliseconds.
pub trait Keyframe {
    /// Signed frame time; negative times represent animation lead-in.
    fn frame(&self) -> i32;
}

/// An interpolation mode shared by all keys in a track.
#[derive(Clone, Copy, Debug, Eq, PartialEq, mdl::Read, mdl::Write)]
#[mdl(value)]
pub enum Interpolation {
    #[mdl(name = "DontInterp")]
    /// Holds the preceding key value until the next key.
    Step,
    /// Interpolates between adjacent values without tangents.
    Linear,
    /// Uses outgoing and incoming Hermite tangents.
    Hermite,
    /// Uses outgoing and incoming Bezier control points.
    Bezier,
}

/// A keyframe without interpolation tangents.
#[derive(Clone, Copy, Debug, PartialEq, mdx::Read, mdx::ReadOwned, mdx::Write)]
pub struct ValueKeyframe<T> {
    /// Signed frame time in milliseconds; negative times represent animation lead-in.
    pub frame: i32,
    /// The value at this frame.
    pub value: T,
}

/// A keyframe with both interpolation tangents.
#[derive(Clone, Copy, Debug, PartialEq, mdx::Read, mdx::ReadOwned, mdx::Write)]
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

impl<T> Keyframe for ValueKeyframe<T> {
    fn frame(&self) -> i32 {
        self.frame
    }
}

impl<T> Keyframe for TangentKeyframe<T> {
    fn frame(&self) -> i32 {
        self.frame
    }
}

use zerovec::ule::AsULE;
impl<T: AsULE> AsULE for ValueKeyframe<T> {
    type ULE = <(i32, T) as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        (self.frame, self.value).to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let (frame, value) = <(i32, T)>::from_unaligned(value);
        Self { frame, value }
    }
}
impl<T: AsULE> AsULE for TangentKeyframe<T> {
    type ULE = <(i32, T, T, T) as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        (self.frame, self.value, self.in_tangent, self.out_tangent).to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let (frame, value, in_tangent, out_tangent) = <(i32, T, T, T)>::from_unaligned(value);
        Self {
            frame,
            value,
            in_tangent,
            out_tangent,
        }
    }
}
