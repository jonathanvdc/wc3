//! Typed key buffers retain exact integer domains and borrowed interpolation controls.
use super::super::attributes;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::animation::{AnyDeclaration, AnyTrack, Track};
use crate::model::semantic::{
    AttributeValues, Curve, CurveValue, Interpolation, Quaternion, TcbParameters, TextureId, Time,
    Values,
};
use crate::model::wem::{records, RawDocument, Record, Ref};

pub(in crate::model::conversion::wem) trait TcbField {
    fn reference(&self) -> Option<Ref<records::targets::Real>>;
}
impl TcbField for () {
    fn reference(&self) -> Option<Ref<records::targets::Real>> {
        None
    }
}
impl TcbField for Ref<records::targets::Real> {
    fn reference(&self) -> Option<Ref<records::targets::Real>> {
        Some(*self)
    }
}

fn reals<'view>(
    document: &'view RawDocument<'_>,
    reference: Ref<records::targets::Real>,
) -> Result<Values<'view, f32>, Error> {
    let Some(section) = document
        .referenced_section(reference)
        .map_err(|error| Error::new("wem.animation", error.to_string()))?
    else {
        return Ok(Values::owned(Vec::new()));
    };
    if section.tag() != records::Real::TAG || section.revision() != records::Real::VERSION {
        return Err(Error::new(
            "wem.animation",
            "time/control data requires the real-number layout",
        ));
    }
    let len = reference.entries as usize;
    let bytes = section
        .bytes()
        .get(
            ..len
                .checked_mul(4)
                .ok_or_else(|| Error::new("wem.animation", "real span overflow"))?,
        )
        .ok_or_else(|| Error::new("wem.animation", "real span exceeds section"))?;
    Ok(Values::view(len, move |index| {
        f32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
    }))
}
fn curve<'a, T: CurveValue + 'a>(
    times: Values<'a, Time>,
    packed: Values<'a, T>,
    mode: Interpolation,
) -> Result<Curve<'a, T>, Error> {
    if matches!(mode, Interpolation::Hermite | Interpolation::Bezier) {
        let count = times.len();
        let values = packed.clone();
        let incoming = packed.clone();
        Ok(Curve::from_spline_samples(
            times,
            Values::view(count, move |i| values.get(i * 3).unwrap()),
            Values::view(count, move |i| incoming.get(i * 3 + 1).unwrap()),
            Values::view(count, move |i| packed.get(i * 3 + 2).unwrap()),
            mode,
        )?)
    } else {
        Ok(Curve::from_samples(times, packed, mode)?)
    }
}

pub(in crate::model::conversion::wem) fn read<'view, D: records::SubTrackDialect>(
    document: &'view RawDocument<'_>,
    source: &records::SubTrack<D>,
    declaration: &AnyDeclaration,
    textures: &[TextureId],
) -> Result<AnyTrack<'view>, Error>
where
    D::Tcb: TcbField,
{
    let mode = match source.interp {
        records::Interpolation::Step => Interpolation::Step,
        records::Interpolation::Linear => Interpolation::Linear,
        records::Interpolation::Hermite => Interpolation::Hermite,
        records::Interpolation::Bezier => Interpolation::Bezier,
        records::Interpolation::Slerp => Interpolation::Slerp,
        _ => {
            return Err(Error::new(
                "wem.animation.interpolation",
                "unknown sampling convention",
            ));
        }
    };
    let seconds = reals(document, source.times)?;
    if seconds.iter().any(|value| !value.is_finite()) {
        return Err(Error::new("wem.animation.time", "key time is not finite"));
    }
    let times = seconds.map(|value| Time::seconds(f64::from(value)).unwrap());
    let multiplier = if matches!(mode, Interpolation::Hermite | Interpolation::Bezier) {
        3
    } else {
        1
    };
    let count = times
        .len()
        .checked_mul(multiplier)
        .ok_or_else(|| Error::new("wem.animation", "key cardinality overflow"))?;
    let tcb = match source.tcb.reference() {
        None => Values::owned(Vec::new()),
        Some(reference) => {
            let controls = reals(document, reference)?;
            if controls.len() % 3 != 0 {
                return Err(Error::new(
                    "wem.animation.tcb",
                    "control triples are incomplete",
                ));
            }
            Values::view(controls.len() / 3, move |i| TcbParameters {
                tension: controls.get(i * 3).unwrap(),
                continuity: controls.get(i * 3 + 1).unwrap(),
                bias: controls.get(i * 3 + 2).unwrap(),
            })
        }
    };
    macro_rules! typed {
        ($id:ident,$type:ident,$kind:ident,$variant:ident) => {{
            let layer = records::AttrLayer {
                type_: records::AttrType::$type,
                data: source.values,
                ..Default::default()
            };
            let AttributeValues::$kind(values) = attributes::read(document, &layer, count)? else {
                unreachable!()
            };
            AnyTrack::$variant(Track {
                channel: *$id,
                curve: curve(times, values, mode)?,
                auxiliary_tcb: tcb,
            })
        }};
    }
    Ok(match declaration {
        AnyDeclaration::Scalar(id, _) => typed!(id, F32, Scalar, Scalar),
        AnyDeclaration::Vector2(id, _) => typed!(id, F32x2, Vector2, Vector2),
        AnyDeclaration::Vector3(id, _) => typed!(id, F32x3, Vector3, Vector3),
        AnyDeclaration::Vector4(id, _) => typed!(id, F32x4, Vector4, Vector4),
        AnyDeclaration::Boolean(id, _) => typed!(id, Bool, Boolean, Boolean),
        AnyDeclaration::Unsigned16(id, _) => typed!(id, U16, U16, Unsigned16),
        AnyDeclaration::Unsigned(id, _) => typed!(id, U32, Unsigned, Unsigned),
        AnyDeclaration::Signed(id, _) => typed!(id, I32, Signed, Signed),
        AnyDeclaration::Bytes4(id, _) => typed!(id, U8x4, Byte4, Bytes4),
        AnyDeclaration::Unsigned4(id, _) => typed!(id, U32x4, Unsigned4, Unsigned4),
        AnyDeclaration::Rotation(id, _) => {
            let layer = records::AttrLayer {
                type_: records::AttrType::Quat,
                data: source.values,
                ..Default::default()
            };
            let AttributeValues::Quaternion(values) = attributes::read(document, &layer, count)?
            else {
                unreachable!()
            };
            AnyTrack::Rotation(Track {
                channel: *id,
                curve: curve(times, values.map(Quaternion), mode)?,
                auxiliary_tcb: tcb,
            })
        }
        AnyDeclaration::Texture(id, _) => {
            let layer = records::AttrLayer {
                type_: records::AttrType::U32,
                data: source.values,
                ..Default::default()
            };
            let AttributeValues::Unsigned(values) = attributes::read(document, &layer, count)?
            else {
                unreachable!()
            };
            if values
                .iter()
                .any(|value| value != u32::MAX && textures.get(value as usize).is_none())
            {
                return Err(Error::new(
                    "wem.animation.texture",
                    "key selects an absent shared texture",
                ));
            }
            let textures = textures.to_vec();
            AnyTrack::Texture(Track {
                channel: *id,
                curve: curve(
                    times,
                    values.map(move |index| {
                        if index == u32::MAX {
                            None
                        } else {
                            Some(textures[index as usize])
                        }
                    }),
                    mode,
                )?,
                auxiliary_tcb: tcb,
            })
        }
    })
}
