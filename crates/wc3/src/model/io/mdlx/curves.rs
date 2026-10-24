//! Warcraft III animation records are projected into canonical logical curves.
use crate::model::conversion::Error;
use crate::model::error::Location;
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
    let failure = |error: SemanticError| {
        Error::from(error).with_context(Location::new().field("animation").field("curve"))
    };
    if let Some(keys) = source.step_keys().or_else(|| source.linear_keys()) {
        let mut indices: Vec<_> = (0..keys.len()).collect();
        indices.sort_by_key(|&index| keys.get(index).expect("source key").frame);
        let mut times = Vec::new();
        for &index in &indices {
            let key = keys.get(index).expect("source key");
            let time = Time::seconds((f64::from(key.frame) - (offset * 1000.).round()) / 1000.)
                .map_err(failure)?;
            times.push(time);
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
        let mut indices: Vec<_> = (0..keys.len()).collect();
        indices.sort_by_key(|&index| keys.get(index).expect("source key").frame);
        let mut times = Vec::new();
        for &index in &indices {
            let key = keys.get(index).expect("source key");
            let time = Time::seconds((f64::from(key.frame) - (offset * 1000.).round()) / 1000.)
                .map_err(failure)?;
            times.push(time);
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
    use crate::model::{Model, Quat, TimeSource, Track as SemanticTrack};

    #[test]
    fn import_stably_orders_native_keys_without_mutating_transport() {
        use crate::model::formats::mdlx::animation::ValueKeyframe;
        let keys = [
            ValueKeyframe {
                frame: 1000,
                value: 2.,
            },
            ValueKeyframe {
                frame: 0,
                value: 0.,
            },
            ValueKeyframe {
                frame: 1000,
                value: 3.,
            },
        ];
        for source in [
            Track::step(keys, None).unwrap(),
            Track::linear(keys, None).unwrap(),
        ] {
            let curve = read(&source, 0., |v| v).unwrap();
            assert_eq!(curve.sample(Time::seconds(1.).unwrap()), Some(3.));
            assert_eq!(curve.sample(Time::seconds(0.).unwrap()), Some(0.));
            assert_eq!(
                source
                    .step_keys()
                    .or_else(|| source.linear_keys())
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .frame,
                1000
            );
        }
        let keys = keys.map(|key| TangentKeyframe {
            frame: key.frame,
            value: key.value,
            in_tangent: key.value + 10.,
            out_tangent: key.value + 20.,
        });
        for source in [
            Track::hermite(keys, None).unwrap(),
            Track::bezier(keys, None).unwrap(),
        ] {
            let curve = read(&source, 0., |v| v).unwrap();
            assert_eq!(curve.sample(Time::seconds(1.).unwrap()), Some(3.));
            assert_eq!(curve.sample(Time::seconds(0.).unwrap()), Some(0.));
            assert_eq!(
                source
                    .hermite_keys()
                    .or_else(|| source.bezier_keys())
                    .unwrap()
                    .get(0)
                    .unwrap(),
                keys[0]
            );
            let middle = curve.sample(Time::seconds(0.5).unwrap()).unwrap();
            let expected = if source.hermite_keys().is_some() {
                2.
            } else {
                12.25
            };
            assert_eq!(middle, expected);
        }
    }

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
            let curve = read(&source, 0.0, Quat::from_array).unwrap();
            assert!(curve.uses_spherical_controls());
            for t in [0.1, 0.25, 0.5, 0.8] {
                let blend = |a: [f32; 4], b: [f32; 4], t: f32| {
                    Quat::from_array(a).slerp(Quat::from_array(b), t).to_array()
                };
                let t32 = t as f32;
                let expected = blend(
                    blend(a, b, t32),
                    blend(out, incoming, t32),
                    2. * t32 * (1. - t32),
                );
                let actual = curve.sample(Time::seconds(t).unwrap()).unwrap().to_array();
                for (actual, expected) in actual.into_iter().zip(expected) {
                    assert!((actual - expected).abs() < 1e-5);
                }
            }
            let mut builder = Model::<Mdlx>::builder("curve restriction");
            let _node = builder.reserve_node();
            let track: SemanticTrack<'_, Quat> = SemanticTrack {
                target: builder.add_value(Quat::IDENTITY),
                curve,
                clock: TimeSource::Animation,
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
