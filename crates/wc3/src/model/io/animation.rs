//! Curve restrictions shared by native clocks with bounded unsigned timestamps.
use super::Error;
use crate::model::PropertyValue;
use crate::model::{Curve, CurveValue, Interpolation, Quat, Time, Track};
use glam::{Vec2, Vec3, Vec4};

#[derive(Clone)]
struct Key {
    time: f64,
    value: Vec<f32>,
    incoming: Option<Vec<f32>>,
    outgoing: Option<Vec<f32>>,
}
#[derive(Clone)]
struct RawTrack {
    keys: Vec<Key>,
    interpolation: Interpolation,
    rotation: bool,
}

/// Floating-point channel components supported by exact curve restriction.
pub trait FloatValue: CurveValue + PropertyValue {
    /// Whether component interpolation must honor quaternion rotation semantics.
    const IS_ROTATION: bool = false;
    /// Reads the scalar components in their authored order.
    fn components(&self) -> Vec<f32>;
    /// Reconstructs a value, checking component count and rotation constraints.
    fn from_components(components: Vec<f32>) -> Result<Self, Error>;
}
impl FloatValue for f32 {
    /// Reads the scalar components in their authored order.
    fn components(&self) -> Vec<f32> {
        vec![*self]
    }
    fn from_components(v: Vec<f32>) -> Result<Self, Error> {
        if v.len() == 1 {
            Ok(v[0])
        } else {
            Err(Error::new("curve", "scalar cardinality differs"))
        }
    }
}
macro_rules! float_vectors {
    ($($ty:ty),*) => {$ (
        impl FloatValue for $ty {
            fn components(&self) -> Vec<f32> { self.to_array().to_vec() }
            fn from_components(v: Vec<f32>) -> Result<Self, Error> {
                Ok(Self::from_array(v.try_into().map_err(|_| Error::new("curve", "vector cardinality differs"))?))
            }
        }
    )*};
}
float_vectors!(Vec2, Vec3, Vec4);
impl FloatValue for Quat {
    /// Whether component interpolation must honor quaternion rotation semantics.
    const IS_ROTATION: bool = true;
    /// Reads the scalar components in their authored order.
    fn components(&self) -> Vec<f32> {
        self.to_array().to_vec()
    }
    fn from_components(values: Vec<f32>) -> Result<Self, Error> {
        Ok(Self::from_array(values.try_into().map_err(|_| {
            Error::new("quaternion", "four components required")
        })?))
    }
}
/// Restricts a typed floating-point track without resampling its spline segments.
pub fn crop<'a, T: FloatValue + 'a>(
    track: &Track<'a, T>,
    start: Time,
    end: Time,
) -> Result<Track<'a, T>, Error> {
    if track.curve.uses_spherical_controls() {
        let times = track.curve.times();
        if start > end {
            return Err(Error::new("curve.window", "invalid crop interval"));
        }
        for boundary in [start, end] {
            if times.first().is_some_and(|first| boundary > first)
                && times.last().is_some_and(|last| boundary < last)
                && !times.iter().any(|key| key == boundary)
            {
                return Err(Error::new(
                    "curve.spherical",
                    "exact restriction inside a spherical spline segment is not implemented",
                ));
            }
        }
        let indices: Vec<_> = times
            .iter()
            .enumerate()
            .filter_map(|(i, t)| (t >= start && t <= end).then_some(i))
            .collect();
        let curve = Curve::from_spline_samples(
            indices
                .iter()
                .map(|i| times.get(*i).unwrap())
                .collect::<Vec<_>>(),
            indices
                .iter()
                .map(|i| track.curve.values().get(*i).unwrap())
                .collect::<Vec<_>>(),
            indices
                .iter()
                .map(|i| track.curve.incoming_value(*i).unwrap())
                .collect::<Vec<_>>(),
            indices
                .iter()
                .map(|i| track.curve.outgoing_value(*i).unwrap())
                .collect::<Vec<_>>(),
            track.curve.interpolation(),
        )?
        .with_spherical_controls(true)?;
        return Ok(Track {
            target: track.target,
            curve,
            clock: track.clock,
        });
    }
    if track.curve.interpolation() == Interpolation::Nlerp {
        return Err(Error::new(
            "curve.nlerp",
            "exact normalized-linear quaternion curve restriction is not implemented",
        ));
    }
    if track.curve.interpolation() == Interpolation::Tcb {
        return Err(Error::new(
            "curve.tcb",
            "exact restriction requires an established TCB tangent convention",
        ));
    }
    let raw = RawTrack {
        interpolation: track.curve.interpolation(),
        rotation: T::IS_ROTATION,
        keys: track
            .curve
            .times()
            .iter()
            .enumerate()
            .map(|(i, t)| Key {
                time: t.as_seconds(),
                value: track
                    .curve
                    .values()
                    .get(i)
                    .expect("curve cardinality")
                    .components(),
                incoming: track.curve.incoming_value(i).map(|v| v.components()),
                outgoing: track.curve.outgoing_value(i).map(|v| v.components()),
            })
            .collect(),
    };
    let restricted = crop_raw(&raw, start.as_seconds(), end.as_seconds())?;
    let times = restricted
        .keys
        .iter()
        .map(|k| Time::seconds(k.time))
        .collect::<Result<Vec<_>, _>>()?;
    let values = restricted
        .keys
        .iter()
        .map(|k| T::from_components(k.value.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    let curve = if matches!(
        restricted.interpolation,
        Interpolation::Hermite | Interpolation::Bezier
    ) {
        let incoming = restricted
            .keys
            .iter()
            .map(|k| T::from_components(k.incoming.clone().expect("spline tangents")))
            .collect::<Result<Vec<_>, _>>()?;
        let outgoing = restricted
            .keys
            .iter()
            .map(|k| T::from_components(k.outgoing.clone().expect("spline tangents")))
            .collect::<Result<Vec<_>, _>>()?;
        Curve::from_spline_samples(times, values, incoming, outgoing, restricted.interpolation)?
    } else {
        Curve::from_samples(times, values, restricted.interpolation)?
    };
    Ok(Track {
        target: track.target,
        curve,
        clock: track.clock,
    })
}

/// Restricts a floating-point track to a time interval without resampling splines.
/// Hermite derivatives and Bezier control points are adjusted for each shortened
/// segment. Linear rotation tracks retain spherical interpolation.
fn crop_raw(track: &RawTrack, start: f64, end: f64) -> Result<RawTrack, Error> {
    if !start.is_finite() || !end.is_finite() || start > end {
        return Err(Error::new("animation.window", "invalid crop interval"));
    }
    if track.keys.is_empty() {
        return Ok(track.clone());
    }
    let width = track.keys[0].value.len();
    if width == 0
        || track.keys.iter().any(|k| {
            k.value.len() != width || !k.time.is_finite() || k.value.iter().any(|v| !v.is_finite())
        })
        || track.keys.windows(2).any(|k| k[0].time > k[1].time)
    {
        return Err(Error::new(
            "animation.keys",
            "invalid floating-point key layout",
        ));
    }
    if matches!(
        track.interpolation,
        Interpolation::Hermite | Interpolation::Bezier
    ) && track.keys.iter().any(|k| {
        [&k.incoming, &k.outgoing].iter().any(|v| {
            v.as_ref()
                .is_none_or(|v| v.len() != width || v.iter().any(|v| !v.is_finite()))
        })
    }) {
        return Err(Error::new("animation.tangents", "invalid spline controls"));
    }
    let mut output = track.clone();
    let boundary = |time| -> Result<Key, Error> {
        if let Some(key) = track.keys.iter().rev().find(|k| k.time == time) {
            return Ok(key.clone());
        }
        let value = sample(track, time)?;
        Ok(Key {
            time,
            incoming: Some(vec![0.; width]),
            outgoing: Some(vec![0.; width]),
            value,
        })
    };
    output.keys = vec![boundary(start)?];
    output.keys.extend(
        track
            .keys
            .iter()
            .filter(|k| k.time > start && k.time < end)
            .cloned(),
    );
    if end > start {
        output.keys.push(boundary(end)?);
    }
    if matches!(
        track.interpolation,
        Interpolation::Hermite | Interpolation::Bezier
    ) {
        for i in 0..output.keys.len().saturating_sub(1) {
            let a = output.keys[i].time;
            let b = output.keys[i + 1].time;
            if a == b {
                continue;
            }
            let original = segment(track, a + (b - a) * 0.5);
            let (incoming, outgoing) = if let Some((left, right)) = original {
                let duration = right.time - left.time;
                let u0 = (a - left.time) / duration;
                let u1 = (b - left.time) / duration;
                let scale = u1 - u0;
                let mut out = Vec::with_capacity(width);
                let mut inc = Vec::with_capacity(width);
                for component in 0..width {
                    let c = coefficients(track.interpolation, left, right, component);
                    let da = derivative(c, u0) * scale;
                    let db = derivative(c, u1) * scale;
                    if track.interpolation == Interpolation::Hermite {
                        out.push(da as f32);
                        inc.push(db as f32);
                    } else {
                        out.push((polynomial(c, u0) + da / 3.) as f32);
                        inc.push((polynomial(c, u1) - db / 3.) as f32);
                    }
                }
                (inc, out)
            } else if track.interpolation == Interpolation::Hermite {
                (vec![0.; width], vec![0.; width])
            } else {
                (
                    output.keys[i + 1].value.clone(),
                    output.keys[i].value.clone(),
                )
            };
            output.keys[i].outgoing = Some(outgoing);
            output.keys[i + 1].incoming = Some(incoming);
        }
    }
    Ok(output)
}
fn segment(track: &RawTrack, time: f64) -> Option<(&Key, &Key)> {
    let end = track.keys.partition_point(|k| k.time <= time);
    if end == 0 || end == track.keys.len() {
        return None;
    }
    Some((&track.keys[end - 1], &track.keys[end]))
}
fn sample(track: &RawTrack, time: f64) -> Result<Vec<f32>, Error> {
    let Some((a, b)) = segment(track, time) else {
        return Ok(if time < track.keys[0].time {
            track.keys[0].value.clone()
        } else {
            track.keys.last().unwrap().value.clone()
        });
    };
    let u = (time - a.time) / (b.time - a.time);
    if track.interpolation == Interpolation::Step {
        return Ok(a.value.clone());
    }
    if matches!(
        track.interpolation,
        Interpolation::Linear | Interpolation::Slerp
    ) && track.rotation
    {
        return rotation(&a.value, &b.value, u);
    }
    Ok((0..a.value.len())
        .map(|i| {
            if track.interpolation == Interpolation::Linear {
                (f64::from(a.value[i]) * (1. - u) + f64::from(b.value[i]) * u) as f32
            } else {
                polynomial(coefficients(track.interpolation, a, b, i), u) as f32
            }
        })
        .collect())
}
fn coefficients(interpolation: Interpolation, a: &Key, b: &Key, i: usize) -> [f64; 4] {
    let p = f64::from(a.value[i]);
    let q = f64::from(b.value[i]);
    let out = f64::from(a.outgoing.as_ref().unwrap()[i]);
    let inc = f64::from(b.incoming.as_ref().unwrap()[i]);
    if interpolation == Interpolation::Hermite {
        [
            p,
            out,
            -3. * p - 2. * out + 3. * q - inc,
            2. * p + out - 2. * q + inc,
        ]
    } else {
        [
            p,
            3. * (out - p),
            3. * (p - 2. * out + inc),
            -p + 3. * out - 3. * inc + q,
        ]
    }
}
fn polynomial(c: [f64; 4], u: f64) -> f64 {
    ((c[3] * u + c[2]) * u + c[1]) * u + c[0]
}
fn derivative(c: [f64; 4], u: f64) -> f64 {
    (3. * c[3] * u + 2. * c[2]) * u + c[1]
}
fn rotation(a: &[f32], b: &[f32], u: f64) -> Result<Vec<f32>, Error> {
    if a.len() != 4 || b.len() != 4 {
        return Err(Error::new(
            "animation.rotation",
            "quaternion needs four components",
        ));
    }
    let normalize = |v: &[f32]| -> Result<[f64; 4], Error> {
        let length = v.iter().map(|&v| f64::from(v).powi(2)).sum::<f64>().sqrt();
        if length == 0. {
            return Err(Error::new("animation.rotation", "zero quaternion"));
        }
        Ok([0, 1, 2, 3].map(|i| f64::from(v[i]) / length))
    };
    let a = normalize(a)?;
    let mut b = normalize(b)?;
    let mut dot = a.iter().zip(b).map(|(a, b)| a * b).sum::<f64>();
    if dot < 0. {
        dot = -dot;
        b = b.map(|v| -v);
    }
    let (wa, wb) = if dot > 0.999999 {
        (1. - u, u)
    } else {
        let angle = dot.clamp(-1., 1.).acos();
        (
            ((1. - u) * angle).sin() / angle.sin(),
            (u * angle).sin() / angle.sin(),
        )
    };
    let mut value = [0, 1, 2, 3].map(|i| wa * a[i] + wb * b[i]);
    let length = value.iter().map(|v| v * v).sum::<f64>().sqrt();
    value = value.map(|v| v / length);
    Ok(value.into_iter().map(|v| v as f32).collect())
}

#[cfg(test)]
#[path = "animation_tests.rs"]
mod tests;
