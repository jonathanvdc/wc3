//! Curve restrictions shared by native clocks with bounded unsigned timestamps.
use super::Error;
use crate::model::wem::semantic::{Interpolation, Key, Target, Track};

/// Restricts a floating-point track to a time interval without resampling splines.
/// Hermite derivatives and Bezier control points are adjusted for each shortened
/// segment. Linear rotation tracks retain spherical interpolation.
pub fn crop(track: &Track, start: f32, end: f32) -> Result<Track, Error> {
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
            let original = segment(track, f64::from(a) + (f64::from(b) - f64::from(a)) * 0.5);
            let (incoming, outgoing) = if let Some((left, right)) = original {
                let duration = f64::from(right.time) - f64::from(left.time);
                let u0 = (f64::from(a) - f64::from(left.time)) / duration;
                let u1 = (f64::from(b) - f64::from(left.time)) / duration;
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
fn segment(track: &Track, time: f64) -> Option<(&Key, &Key)> {
    let end = track.keys.partition_point(|k| f64::from(k.time) <= time);
    if end == 0 || end == track.keys.len() {
        return None;
    }
    Some((&track.keys[end - 1], &track.keys[end]))
}
fn sample(track: &Track, time: f32) -> Result<Vec<f32>, Error> {
    let Some((a, b)) = segment(track, f64::from(time)) else {
        return Ok(if time < track.keys[0].time {
            track.keys[0].value.clone()
        } else {
            track.keys.last().unwrap().value.clone()
        });
    };
    let u = (f64::from(time) - f64::from(a.time)) / (f64::from(b.time) - f64::from(a.time));
    if track.interpolation == Interpolation::Step {
        return Ok(a.value.clone());
    }
    if track.interpolation == Interpolation::Linear && matches!(track.target, Target::Rotation(_)) {
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
