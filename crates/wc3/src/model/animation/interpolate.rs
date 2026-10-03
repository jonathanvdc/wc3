//! Value interpolation for track evaluation.
use super::TrackValue;
use crate::model::{Quaternion, Vec3};

/// Interpolation operations used by [`super::Track::evaluate`].
///
/// `t` is a fraction between zero and one. Tangents are stored MDX values;
/// scalar/vector Hermite tangents are not scaled by the segment duration.
/// `Quaternion` implements quaternion interpolation in `[x, y, z, w]` order:
/// linear uses shortest-path slerp and both spline modes use squad. Because
/// `Quaternion` is a type alias, these semantics also apply to `[f32; 4]`.
/// `u32` holds the left value for every interpolation mode.
pub trait Interpolate: TrackValue {
    /// Interpolates from `a` to `b` at normalized segment time `t`.
    fn linear(a: Self, b: Self, t: f32) -> Self;
    /// Evaluates a Hermite segment using the outgoing tangent of `a` and incoming tangent of `b`.
    fn hermite(a: Self, out_tangent: Self, in_tangent: Self, b: Self, t: f32) -> Self;
    /// Evaluates a Bezier segment using the outgoing control point of `a` and incoming control point of `b`.
    fn bezier(a: Self, out_tangent: Self, in_tangent: Self, b: Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn linear(a: Self, b: Self, t: f32) -> Self {
        a * (1.0 - t) + b * t
    }
    fn hermite(a: Self, out: Self, incoming: Self, b: Self, t: f32) -> Self {
        let t2 = t * t;
        let t3 = t2 * t;
        (2.0 * t3 - 3.0 * t2 + 1.0) * a
            + (t3 - 2.0 * t2 + t) * out
            + (-2.0 * t3 + 3.0 * t2) * b
            + (t3 - t2) * incoming
    }
    fn bezier(a: Self, out: Self, incoming: Self, b: Self, t: f32) -> Self {
        let s = 1.0 - t;
        s * s * s * a + 3.0 * s * s * t * out + 3.0 * s * t * t * incoming + t * t * t * b
    }
}
impl Interpolate for Vec3 {
    fn linear(a: Self, b: Self, t: f32) -> Self {
        [0, 1, 2].map(|i| f32::linear(a[i], b[i], t))
    }
    fn hermite(a: Self, out: Self, incoming: Self, b: Self, t: f32) -> Self {
        [0, 1, 2].map(|i| f32::hermite(a[i], out[i], incoming[i], b[i], t))
    }
    fn bezier(a: Self, out: Self, incoming: Self, b: Self, t: f32) -> Self {
        [0, 1, 2].map(|i| f32::bezier(a[i], out[i], incoming[i], b[i], t))
    }
}
impl Interpolate for u32 {
    fn linear(a: Self, _: Self, _: f32) -> Self {
        a
    }
    fn hermite(a: Self, _: Self, _: Self, _: Self, _: f32) -> Self {
        a
    }
    fn bezier(a: Self, _: Self, _: Self, _: Self, _: f32) -> Self {
        a
    }
}

fn normalize(q: Quaternion) -> Quaternion {
    let length = q.iter().map(|v| v * v).sum::<f32>().sqrt();
    if length > 0.0 {
        q.map(|v| v / length)
    } else {
        [0.0, 0.0, 0.0, 1.0]
    }
}
impl Interpolate for Quaternion {
    fn linear(a: Self, b: Self, t: f32) -> Self {
        let a = normalize(a);
        let mut b = normalize(b);
        let mut dot = a.iter().zip(b).map(|(a, b)| a * b).sum::<f32>();
        if dot < 0.0 {
            b = b.map(|v| -v);
            dot = -dot;
        }
        if dot > 0.9995 {
            return normalize([0, 1, 2, 3].map(|i| f32::linear(a[i], b[i], t)));
        }
        let angle = dot.clamp(-1.0, 1.0).acos();
        let left = ((1.0 - t) * angle).sin() / angle.sin();
        let right = (t * angle).sin() / angle.sin();
        normalize([0, 1, 2, 3].map(|i| a[i] * left + b[i] * right))
    }
    fn hermite(a: Self, out: Self, incoming: Self, b: Self, t: f32) -> Self {
        Self::linear(
            Self::linear(a, b, t),
            Self::linear(out, incoming, t),
            2.0 * t * (1.0 - t),
        )
    }
    fn bezier(a: Self, out: Self, incoming: Self, b: Self, t: f32) -> Self {
        Self::hermite(a, out, incoming, b, t)
    }
}
