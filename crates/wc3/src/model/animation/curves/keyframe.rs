//! Typed authoring keys and borrowed spline controls.
use super::CurveValue;
use crate::model::common::packed_identity;
use crate::model::common::{PackedArray, PackedValue};
use crate::model::Time;
use zerovec::ule::AsULE;

/// Per-key Kochanek–Bartels spline parameters.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TcbParameters {
    /// Controls the strength of the incoming and outgoing tangent response.
    pub tension: f32,
    /// Controls the difference between incoming and outgoing tangents.
    pub continuity: f32,
    /// Controls the balance between preceding and following segments.
    pub bias: f32,
}
/// A timed value for stepped, linear, or spherical curves.
#[derive(Clone, Debug)]
pub struct ValueKeyframe<T> {
    /// Timeline position in seconds.
    pub time: Time,
    /// Authored value.
    pub value: T,
}
/// A timed value with Hermite derivatives.
#[derive(Clone, Debug)]
pub struct HermiteKeyframe<T: CurveValue> {
    /// Timeline position in seconds.
    pub time: Time,
    /// Authored value.
    pub value: T,
    /// Incoming derivative, in the source curve's segment convention.
    pub incoming: T::Derivative,
    /// Outgoing derivative, in the source curve's segment convention.
    pub outgoing: T::Derivative,
}
/// A timed value with Bezier control points.
#[derive(Clone, Debug)]
pub struct BezierKeyframe<T: CurveValue> {
    /// Timeline position in seconds.
    pub time: Time,
    /// Authored value.
    pub value: T,
    /// Incoming control point.
    pub incoming: T::ControlPoint,
    /// Outgoing control point.
    pub outgoing: T::ControlPoint,
}
/// A timed value with authored Kochanek–Bartels parameters.
#[derive(Clone, Debug)]
pub struct TcbKeyframe<T> {
    /// Timeline position in seconds.
    pub time: Time,
    /// Authored value.
    pub value: T,
    /// Tension, continuity, and bias.
    pub parameters: TcbParameters,
}
/// Incoming and outgoing Hermite derivatives stored independently of key values.
#[derive(Clone, Debug)]
pub struct HermiteDerivatives<'a, T: PackedValue> {
    /// Derivatives approaching each key.
    pub incoming: PackedArray<'a, T>,
    /// Derivatives leaving each key.
    pub outgoing: PackedArray<'a, T>,
}
/// Incoming and outgoing Bezier control points stored independently of key values.
#[derive(Clone, Debug)]
pub struct BezierControls<'a, T: PackedValue> {
    /// Control points approaching each key.
    pub incoming: PackedArray<'a, T>,
    /// Control points leaving each key.
    pub outgoing: PackedArray<'a, T>,
}

impl AsULE for TcbParameters {
    type ULE = <[f32; 3] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        [self.tension, self.continuity, self.bias].to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let [tension, continuity, bias] = <[f32; 3]>::from_unaligned(value);
        Self {
            tension,
            continuity,
            bias,
        }
    }
}

packed_identity!(TcbParameters);
