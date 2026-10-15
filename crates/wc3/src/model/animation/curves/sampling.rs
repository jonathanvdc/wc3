//! Evaluation of checked semantic curves without a rendering dependency.
use super::{Curve, CurveShape, CurveValue, TcbParameters};
use crate::model::{Quaternion, TextureId, Time};
use std::array::from_fn;
use std::ops::RangeInclusive;

/// Mathematical operations used when evaluating a semantic curve.
/// Implementations must preserve the value's domain when producing a result.
pub trait SampleValue: CurveValue {
    /// Returns a sampled key in the value's mathematical domain.
    fn sampled_key(value: Self) -> Self {
        value
    }
    /// Selects an equivalent representation adjacent to a reference value.
    fn adjacent(_reference: &Self, value: Self) -> Self {
        value
    }
    /// Evaluates a component-weighted sum, normalizing rotation results.
    fn weighted(values: &[(Self, f64)]) -> Self;
    /// Interpolates adjacent values; rotation values use the shortest arc.
    fn interpolate(a: Self, b: Self, amount: f64, normalized_linear: bool) -> Self {
        let _ = normalized_linear;
        Self::weighted(&[(a, 1.0 - amount), (b, amount)])
    }
}
macro_rules! numeric {
    ($($ty:ty),*) => {$ (
        impl SampleValue for $ty {
            fn weighted(values: &[(Self, f64)]) -> Self {
                values.iter().map(|(v, w)| *v as f64 * w).sum::<f64>() as Self
            }
        }
    )*};
}
numeric!(f32, f64, u32, i32, u16, u8);
macro_rules! arrays {
    ($($ty:ty),*) => {$ (
        impl<const N: usize> SampleValue for [$ty; N] {
            fn weighted(values: &[(Self, f64)]) -> Self {
                from_fn(|i| values.iter().map(|(v, w)| v[i] as f64 * w).sum::<f64>() as $ty)
            }
        }
    )*};
}
arrays!(f32, u32, u8);
macro_rules! discrete {
    ($($ty:ty),*) => {$ (
        impl SampleValue for $ty {
            fn weighted(values: &[(Self, f64)]) -> Self { values[0].0 }
        }
    )*};
}
discrete!(bool, TextureId, Option<TextureId>);
fn normalized(value: [f32; 4]) -> Quaternion {
    let length = value
        .iter()
        .map(|v| f64::from(*v).powi(2))
        .sum::<f64>()
        .sqrt();
    if length > 0.0 {
        Quaternion(value.map(|v| (f64::from(v) / length) as f32))
    } else {
        Quaternion([0.0, 0.0, 0.0, 1.0])
    }
}
impl SampleValue for Quaternion {
    fn sampled_key(value: Self) -> Self {
        normalized(value.0)
    }
    fn adjacent(reference: &Self, mut value: Self) -> Self {
        if reference
            .0
            .iter()
            .zip(value.0)
            .map(|(a, b)| f64::from(*a) * f64::from(b))
            .sum::<f64>()
            < 0.0
        {
            value.0 = value.0.map(|v| -v);
        }
        value
    }
    fn weighted(values: &[(Self, f64)]) -> Self {
        normalized(from_fn(|i| {
            values
                .iter()
                .map(|(v, w)| f64::from(v.0[i]) * w)
                .sum::<f64>() as f32
        }))
    }
    fn interpolate(a: Self, b: Self, amount: f64, normalized_linear: bool) -> Self {
        let a = normalized(a.0);
        let mut b = normalized(b.0);
        let mut dot =
            a.0.iter()
                .zip(b.0)
                .map(|(a, b)| f64::from(*a) * f64::from(b))
                .sum::<f64>();
        if dot < 0.0 {
            b.0 = b.0.map(|v| -v);
            dot = -dot;
        }
        if normalized_linear || dot > 0.9995 {
            return Self::weighted(&[(a, 1.0 - amount), (b, amount)]);
        }
        let angle = dot.clamp(-1.0, 1.0).acos();
        Self::weighted(&[
            (a, ((1.0 - amount) * angle).sin() / angle.sin()),
            (b, (amount * angle).sin() / angle.sin()),
        ])
    }
}
impl<T: SampleValue> Curve<'_, T> {
    /// Samples a curve in seconds, clamping outside its keys. Empty curves return
    /// `None`. At coincident keys the last key wins; the first key supplies the
    /// incoming segment's endpoint. This method does not wrap playback clocks.
    pub fn sample(&self, time: Time) -> Option<T> {
        self.sample_keys(time, 0, self.times.len())
    }
    /// Samples only keys inside an inclusive playback interval. Empty intervals
    /// or intervals containing no keys return `None`; neighboring clips cannot
    /// contribute interpolation endpoints.
    pub fn sample_in(&self, time: Time, interval: RangeInclusive<Time>) -> Option<T> {
        if interval.start() > interval.end() {
            return None;
        }
        let partition = |boundary: Time, inclusive: bool| {
            let mut low = 0;
            let mut high = self.times.len();
            while low < high {
                let middle = low + (high - low) / 2;
                let key = self.times.get(middle).unwrap();
                if key < boundary || (inclusive && key == boundary) {
                    low = middle + 1;
                } else {
                    high = middle;
                }
            }
            low
        };
        self.sample_keys(
            time,
            partition(*interval.start(), false),
            partition(*interval.end(), true),
        )
    }
    fn sample_keys(&self, time: Time, first: usize, count: usize) -> Option<T> {
        if first == count {
            return None;
        }
        let mut low = first;
        let mut high = count;
        while low < high {
            let middle = low + (high - low) / 2;
            if self.times.get(middle)? <= time {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        if low == first {
            return self.values.get(first).map(T::sampled_key);
        }
        let left = low - 1;
        let a = self.values.get(left)?;
        if low == count || self.times.get(left)? == time || matches!(self.shape, CurveShape::Step) {
            return Some(T::sampled_key(a));
        }
        let b = self.values.get(low)?;
        let start = self.times.get(left)?.as_seconds();
        let end = self.times.get(low)?.as_seconds();
        let t = (time.as_seconds() - start) / (end - start);
        let t2 = t * t;
        let t3 = t2 * t;
        let h = [
            2.0 * t3 - 3.0 * t2 + 1.0,
            -2.0 * t3 + 3.0 * t2,
            t3 - 2.0 * t2 + t,
            t3 - t2,
        ];
        if self.spherical_controls {
            let out = self.outgoing_value(left)?;
            let incoming = self.incoming_value(low)?;
            return Some(T::interpolate(
                T::interpolate(a, b, t, false),
                T::interpolate(out, incoming, t, false),
                2.0 * t * (1.0 - t),
                false,
            ));
        }
        Some(match &self.shape {
            CurveShape::Step => a,
            CurveShape::Linear | CurveShape::Slerp => T::interpolate(a, b, t, false),
            CurveShape::Nlerp => T::interpolate(a, b, t, true),
            CurveShape::Hermite(controls) => T::weighted(&[
                (a, h[0]),
                (b, h[1]),
                (T::derivative_value(controls.outgoing.get(left)?), h[2]),
                (T::derivative_value(controls.incoming.get(low)?), h[3]),
            ]),
            CurveShape::Bezier(controls) => T::weighted(&[
                (a, (1.0 - t).powi(3)),
                (b, t3),
                (
                    T::control_point_value(controls.outgoing.get(left)?),
                    3.0 * (1.0 - t).powi(2) * t,
                ),
                (
                    T::control_point_value(controls.incoming.get(low)?),
                    3.0 * (1.0 - t) * t2,
                ),
            ]),
            CurveShape::Tcb(parameters) => {
                // Coincident keys bound a discontinuity and never contribute a
                // zero-duration neighboring slope.
                let previous = left
                    .checked_sub(1)
                    .filter(|i| *i >= first)
                    .filter(|i| self.times.get(*i).unwrap().as_seconds() < start);
                let next = (low + 1 < count)
                    .then_some(low + 1)
                    .filter(|i| self.times.get(*i).unwrap().as_seconds() > end);
                let b = T::adjacent(&a, b);
                let p = previous
                    .and_then(|i| self.values.get(i))
                    .unwrap_or_else(|| a.clone());
                let n = next
                    .and_then(|i| self.values.get(i))
                    .unwrap_or_else(|| b.clone());
                let p = T::adjacent(&a, p);
                let n = T::adjacent(&b, n);
                let weights = |v: TcbParameters, incoming: bool| {
                    let tension = 0.5 * (1.0 - f64::from(v.tension));
                    let c = f64::from(v.continuity) * if incoming { -1.0 } else { 1.0 };
                    (
                        tension * (1.0 + c) * (1.0 + f64::from(v.bias)),
                        tension * (1.0 - c) * (1.0 - f64::from(v.bias)),
                    )
                };
                let (o1, o2) = weights(parameters.get(left)?, false);
                let (i1, i2) = weights(parameters.get(low)?, true);
                let out_scale = previous.map_or(1.0, |i| {
                    (end - start) / (start - self.times.get(i).unwrap().as_seconds())
                });
                let in_scale = next.map_or(1.0, |i| {
                    (end - start) / (self.times.get(i).unwrap().as_seconds() - end)
                });
                T::weighted(&[
                    (p, -h[2] * o1 * out_scale),
                    (a, h[0] + h[2] * (o1 * out_scale - o2) - h[3] * i1),
                    (b, h[1] + h[2] * o2 + h[3] * (i1 - i2 * in_scale)),
                    (n, h[3] * i2 * in_scale),
                ])
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn time(value: f64) -> Time {
        Time::seconds(value).unwrap()
    }
    fn close(value: f32, expected: f32) {
        assert!((value - expected).abs() < 1e-5, "{value} != {expected}");
    }
    #[test]
    fn empty_clamping_and_duplicate_boundaries() {
        let empty = Curve::<f32>::step([], []).unwrap();
        assert_eq!(empty.sample(time(0.0)), None);
        let curve = Curve::linear(
            [time(0.0), time(1.0), time(1.0), time(2.0)],
            [0.0f32, 2.0, 8.0, 10.0],
        )
        .unwrap();
        assert_eq!(curve.sample(time(-1.0)), Some(0.0));
        assert_eq!(curve.sample(time(0.5)), Some(1.0));
        assert_eq!(curve.sample(time(1.0)), Some(8.0));
        assert_eq!(curve.sample(time(1.5)), Some(9.0));
        assert_eq!(curve.sample(time(3.0)), Some(10.0));
    }
    #[test]
    fn interval_excludes_neighboring_clip_keys() {
        let curve = Curve::linear(
            [time(0.0), time(1.0), time(2.0), time(3.0)],
            [100.0f32, 1.0, 2.0, 100.0],
        )
        .unwrap();
        assert_eq!(curve.sample_in(time(0.5), time(1.0)..=time(2.0)), Some(1.0));
        assert_eq!(curve.sample_in(time(2.5), time(1.0)..=time(2.0)), Some(2.0));
        assert_eq!(curve.sample_in(time(1.5), time(1.0)..=time(2.0)), Some(1.5));
        assert_eq!(curve.sample_in(time(1.5), time(1.1)..=time(1.9)), None);
    }
    #[test]
    fn owned_curves_retain_spline_controls() {
        let curve = Curve::bezier(
            [time(0.0), time(1.0)],
            [0.0f32, 2.0],
            [0.0, 2.0],
            [0.0, 2.0],
        )
        .unwrap()
        .into_owned();
        close(curve.sample(time(0.25)).unwrap(), 0.3125);
    }
    #[test]
    fn discrete_and_numeric_values() {
        assert_eq!(
            Curve::step([time(0.0), time(1.0)], [false, true])
                .unwrap()
                .sample(time(0.5)),
            Some(false)
        );
        assert_eq!(
            Curve::linear([time(0.0), time(1.0)], [[0u8, 10], [10, 20]])
                .unwrap()
                .sample(time(0.5)),
            Some([5, 15])
        );
        assert_eq!(
            Curve::linear([time(0.0), time(1.0)], [-10i32, 10])
                .unwrap()
                .sample(time(0.5)),
            Some(0)
        );
    }
    #[test]
    fn hermite_and_bezier_controls() {
        let times = [time(0.0), time(1.0)];
        let h = Curve::hermite(times, [0.0f32, 2.0], [0.0, 0.0], [0.0, 0.0]).unwrap();
        close(h.sample(time(0.25)).unwrap(), 0.3125);
        let b = Curve::bezier(times, [0.0f32, 2.0], [0.0, 2.0], [0.0, 2.0]).unwrap();
        close(b.sample(time(0.25)).unwrap(), 0.3125);
    }
    #[test]
    fn tcb_straight_line_and_nonuniform_time() {
        let curve = Curve::tcb(
            [time(0.0), time(1.0), time(3.0), time(4.0)],
            [0.0f32, 1.0, 3.0, 4.0],
            [TcbParameters::default(); 4],
        )
        .unwrap();
        close(curve.sample(time(2.0)).unwrap(), 2.0);
        close(curve.sample(time(1.5)).unwrap(), 1.5);
        let held = Curve::tcb(
            [time(0.0), time(1.0)],
            [0.0f32, 1.0],
            [TcbParameters {
                tension: 1.0,
                ..Default::default()
            }; 2],
        )
        .unwrap();
        close(held.sample(time(0.25)).unwrap(), 0.15625);
    }
    #[test]
    fn rotations_follow_shortest_arc_and_normalize_splines() {
        let times = [time(0.0), time(1.0)];
        let identity = Quaternion::IDENTITY;
        let opposite = Quaternion([0.0, 0.0, 0.0, -1.0]);
        let curve = Curve::linear(times, [identity, opposite]).unwrap();
        assert_eq!(curve.sample(time(0.5)), Some(identity));
        let turn = Quaternion([0.0, 0.0, 1.0, 0.0]);
        let curve = Curve::slerp(times, [identity, turn]).unwrap();
        let halfway = curve.sample(time(0.5)).unwrap();
        close(halfway.0[2], 0.5f32.sqrt());
        close(halfway.0[3], 0.5f32.sqrt());
        let spline = Curve::hermite(times, [identity, turn], [[0.0; 4]; 2], [[0.0; 4]; 2]).unwrap();
        let q = spline.sample(time(0.5)).unwrap();
        close(q.0.iter().map(|v| v * v).sum(), 1.0);
    }
}
