//! Borrowed logical curves read directly from Warcraft III animation records.
use crate::model::conversion::Error;
use crate::model::mdlx::animation::{Interpolation as NativeInterpolation, Track, TrackValue};
use crate::model::semantic::{
    Curve, CurveValue, Error as SemanticError, Interpolation, Time, Values,
};

/// Decodes values lazily while retaining exact authored interpolation and tangents.
pub(super) fn read<'a, T, U>(
    source: &'a Track<'_, T>,
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
    source: &'a Track<'_, T>,
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
        let values = Values::view(indices.len(), move |index| {
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
        let values = Values::view(indices.len(), move |index| {
            value_map(keys.get(indices[index]).expect("checked source key").value)
        });
        let incoming = Values::view(incoming_indices.len(), move |index| {
            incoming_map(
                keys.get(incoming_indices[index])
                    .expect("checked source key")
                    .in_tangent,
            )
        });
        let outgoing = Values::view(outgoing_indices.len(), move |index| {
            tangent_map(
                keys.get(outgoing_indices[index])
                    .expect("checked source key")
                    .out_tangent,
            )
        });
        Curve::from_spline_samples(times, values, incoming, outgoing, interpolation)
            .map_err(failure)
    }
}
