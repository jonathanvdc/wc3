//! Typed program values lower to their declared packed key interpretation.
use super::super::attributes;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::animation::AnyTrack;
use crate::model::semantic::{
    AttributeValues, Curve, CurveValue, Interpolation, TextureId, Values,
};
use crate::model::wem::{records, RawDocument, Ref, Target};
use std::collections::BTreeMap;
fn put<T: crate::model::wem::Record + crate::model::wem::Write, U: Target>(
    document: &mut RawDocument<'_>,
    values: &[T],
) -> Result<Ref<U>, Error> {
    if values.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(values.len())
        .map_err(|_| Error::new("wem.animation", "record count exceeds u32"))?;
    let id = document
        .append(values)
        .map_err(|error| Error::new("wem.animation", error.to_string()))?;
    Ok(Ref::from_section(count, id.index()))
}
fn packed<T: CurveValue + 'static>(curve: &Curve<'_, T>) -> Values<'static, T> {
    let mut values = Vec::new();
    for (index, value) in curve.values().iter().enumerate() {
        values.push(value);
        if let (Some(incoming), Some(outgoing)) = (curve.incoming(), curve.outgoing()) {
            values.push(incoming.get(index).unwrap());
            values.push(outgoing.get(index).unwrap());
        }
    }
    Values::owned(values)
}
fn times<T: CurveValue>(
    document: &mut RawDocument<'_>,
    curve: &Curve<'_, T>,
) -> Result<Ref<records::targets::Real>, Error> {
    let values = curve
        .times()
        .iter()
        .map(|time| {
            let value = time.as_seconds() as f32;
            if !value.is_finite() || f64::from(value) != time.as_seconds() {
                return Err(Error::new(
                    "wem.animation.time",
                    "time is not exactly representable by native single precision",
                ));
            }
            Ok(records::Real { value })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    put(document, &values)
}
fn interpolation(mode: Interpolation) -> Result<records::Interpolation, Error> {
    Ok(match mode {
        Interpolation::Step => records::Interpolation::Step,
        Interpolation::Linear => records::Interpolation::Linear,
        Interpolation::Hermite => records::Interpolation::Hermite,
        Interpolation::Bezier => records::Interpolation::Bezier,
        Interpolation::Slerp => records::Interpolation::Slerp,
        Interpolation::Tcb => {
            return Err(Error::new(
                "wem.animation.interpolation",
                "no native sampling discriminator declares TCB interpolation",
            ));
        }
    })
}
pub(in crate::model::conversion::wem) fn write(
    document: &mut RawDocument<'_>,
    track: &AnyTrack<'_>,
    channel: u32,
    textures: &BTreeMap<TextureId, u32>,
) -> Result<records::SubTrack<records::V2>, Error> {
    macro_rules! typed {
        ($track:ident,$kind:ident) => {{
            let (_, values) =
                attributes::write(document, &AttributeValues::$kind(packed(&$track.curve)))?;
            (
                times(document, &$track.curve)?,
                values,
                interpolation($track.curve.interpolation())?,
                &$track.auxiliary_tcb,
            )
        }};
    }
    let (times, values, interp, tcb) = match track {
        AnyTrack::Scalar(track) => typed!(track, Scalar),
        AnyTrack::Vector2(track) => typed!(track, Vector2),
        AnyTrack::Vector3(track) => typed!(track, Vector3),
        AnyTrack::Vector4(track) => typed!(track, Vector4),
        AnyTrack::Boolean(track) => typed!(track, Boolean),
        AnyTrack::Unsigned16(track) => typed!(track, U16),
        AnyTrack::Unsigned(track) => typed!(track, Unsigned),
        AnyTrack::Signed(track) => typed!(track, Signed),
        AnyTrack::Bytes4(track) => typed!(track, Byte4),
        AnyTrack::Unsigned4(track) => typed!(track, Unsigned4),
        AnyTrack::Rotation(track) => {
            let (_, values) = attributes::write(
                document,
                &AttributeValues::Quaternion(packed(&track.curve).map(|value| value.components())),
            )?;
            (
                times(document, &track.curve)?,
                values,
                interpolation(track.curve.interpolation())?,
                &track.auxiliary_tcb,
            )
        }
        AnyTrack::Texture(track) => {
            let values = packed(&track.curve)
                .iter()
                .map(|value| match value {
                    None => Ok(u32::MAX),
                    Some(id) => textures.get(&id).copied().ok_or_else(|| {
                        Error::new(
                            "wem.animation.texture",
                            "key texture is absent from the actual resource arena",
                        )
                    }),
                })
                .collect::<Result<Vec<_>, _>>()?;
            let (_, values) =
                attributes::write(document, &AttributeValues::Unsigned(Values::owned(values)))?;
            (
                times(document, &track.curve)?,
                values,
                interpolation(track.curve.interpolation())?,
                &track.auxiliary_tcb,
            )
        }
    };
    let controls = tcb
        .iter()
        .flat_map(|value| [value.tension, value.continuity, value.bias])
        .map(|value| records::Real { value })
        .collect::<Vec<_>>();
    Ok(records::SubTrack {
        channel,
        interp,
        times,
        values,
        tcb: put(document, &controls)?,
    })
}
