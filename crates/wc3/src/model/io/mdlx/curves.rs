//! Warcraft III animation records are projected into canonical logical curves.
use crate::model::conversion::Error;
use crate::model::formats::mdlx::animation::{
    Interpolation as NativeInterpolation, Track, TrackValue,
};
use crate::model::io::materialize;
use crate::model::{Curve, CurveDomain, CurveValue, Error as SemanticError, Interpolation, Time};

/// Materializes values once while retaining exact authored interpolation and tangents.
pub(super) fn read<'a, T, U>(
    source: &Track<'_, T>,
    offset: f64,
    map: impl Fn(T) -> U + Clone + Send + Sync + 'a,
) -> Result<Curve<'a, U>, Error>
where
    T: TrackValue + Send + Sync + 'a,
    T::ULE: Sync,
    U: CurveValue + 'a,
{
    read_maps(source, offset, map.clone(), map)
}

pub(super) fn read_maps<'a, T, U>(
    source: &Track<'_, T>,
    offset: f64,
    value_map: impl Fn(T) -> U + Clone + Send + Sync + 'a,
    tangent_map: impl Fn(T) -> U + Clone + Send + Sync + 'a,
) -> Result<Curve<'a, U>, Error>
where
    T: TrackValue + Send + Sync + 'a,
    T::ULE: Sync,
    U: CurveValue + 'a,
{
    let interpolation = match source.interpolation() {
        NativeInterpolation::Step => Interpolation::Step,
        NativeInterpolation::Linear => Interpolation::Linear,
        NativeInterpolation::Hermite => Interpolation::Hermite,
        NativeInterpolation::Bezier => Interpolation::Bezier,
    };
    let failure = |error: SemanticError| Error::new("animation.curve", error.to_string());
    if let Some(keys) = source.step_keys().or_else(|| source.linear_keys()) {
        let mut indices = Vec::new();
        let mut times = Vec::new();
        for (index, key) in keys.iter().enumerate() {
            let time = Time::seconds((f64::from(key.frame) - (offset * 1000.).round()) / 1000.)
                .map_err(failure)?;
            times.push(time);
            indices.push(index);
        }
        let values = materialize(indices.len(), move |index| {
            value_map(keys.get(indices[index]).expect("checked source key").value)
        });
        Curve::from_samples(times, values, interpolation).map_err(failure)
    } else {
        let keys = source
            .hermite_keys()
            .or_else(|| source.bezier_keys())
            .expect("typed native track has one key storage variant");
        let mut indices = Vec::new();
        let mut times = Vec::new();
        for (index, key) in keys.iter().enumerate() {
            let time = Time::seconds((f64::from(key.frame) - (offset * 1000.).round()) / 1000.)
                .map_err(failure)?;
            times.push(time);
            indices.push(index);
        }
        let incoming_indices = indices.clone();
        let outgoing_indices = indices.clone();
        let value_map = value_map.clone();
        let incoming_map = tangent_map.clone();
        let values = materialize(indices.len(), move |index| {
            value_map(keys.get(indices[index]).expect("checked source key").value)
        });
        let incoming = materialize(incoming_indices.len(), move |index| {
            incoming_map(
                keys.get(incoming_indices[index])
                    .expect("checked source key")
                    .in_tangent,
            )
        });
        let outgoing = materialize(outgoing_indices.len(), move |index| {
            tangent_map(
                keys.get(outgoing_indices[index])
                    .expect("checked source key")
                    .out_tangent,
            )
        });
        Curve::from_spline_samples(times, values, incoming, outgoing, interpolation)
            .and_then(|curve| curve.with_spherical_controls(U::Domain::SUPPORTS_ROTATION))
            .map_err(failure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::formats::mdlx::animation::TangentKeyframe;
    use crate::model::io::animation::crop;
    use crate::model::profiles::Mdlx;
    use crate::model::{Clock, Model, Quaternion, Track as SemanticTrack};

    #[test]
    fn imported_rotation_splines_preserve_native_angular_sampling() {
        let a = [0.0, 0.0, 0.0, 1.0];
        let b = [0.0, 0.0, 1.0, 0.0];
        let out = [0.0, 0.6, 0.0, 0.8];
        let incoming = [0.6, 0.0, 0.0, 0.8];
        let keys = [
            TangentKeyframe {
                frame: 0,
                value: a,
                in_tangent: a,
                out_tangent: out,
            },
            TangentKeyframe {
                frame: 1000,
                value: b,
                in_tangent: incoming,
                out_tangent: b,
            },
        ];
        for source in [
            Track::hermite(keys, None).unwrap(),
            Track::bezier(keys, None).unwrap(),
        ] {
            let curve = read(&source, 0.0, Quaternion).unwrap();
            assert!(curve.uses_spherical_controls());
            for t in [0.1, 0.25, 0.5, 0.8] {
                let expected = source.evaluate_in(t * 1000.0, 0..=1000).unwrap();
                let actual = curve.sample(Time::seconds(t).unwrap()).unwrap().0;
                for (actual, expected) in actual.into_iter().zip(expected) {
                    assert!((actual - expected).abs() < 1e-5);
                }
            }
            let mut builder = Model::<Mdlx>::builder("curve restriction");
            let node = builder.reserve_node();
            let track: SemanticTrack<'_, Quaternion, Mdlx> = SemanticTrack {
                target: node.rotation(),
                curve,
                clock: Clock::Clip,
            };
            let whole = crop(
                &track,
                Time::seconds(0.0).unwrap(),
                Time::seconds(1.0).unwrap(),
            )
            .unwrap();
            assert!(whole.curve.uses_spherical_controls());
            assert!(crop(
                &track,
                Time::seconds(0.25).unwrap(),
                Time::seconds(0.8).unwrap()
            )
            .is_err());
            assert!(whole.curve.into_owned().uses_spherical_controls());
        }
    }
}
