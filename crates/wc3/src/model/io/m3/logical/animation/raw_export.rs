//! Rebuild typed key pools whose property declaration is absent.
use super::{blocks::Blocks, export::append};
use crate::model::conversion::{Context, Error};
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::profiles::m3::{UnboundBlock, UnboundKeys};
use native::Envelope;

pub(super) fn add<E: Envelope>(
    document: &mut native::Container<'_, E>,
    blocks: &mut Blocks,
    source: &UnboundBlock<'_>,
    context: &mut Context,
) -> Result<Option<u32>, Error> {
    let flags = nr::AnimationBlockFlags::from(source.flags.bits());
    let end_frame = super::export::unsigned_time(source.end, context)?;
    macro_rules! block {
        ($keys:ident, $field:ident, $record:ident, $block:ident, $family:expr, $convert:expr) => {{
            let times = $keys
                .times
                .iter()
                .map(|time| {
                    let exact = time.as_seconds() * 1000.;
                    if exact.round() < f64::from(i32::MIN) || exact.round() > f64::from(i32::MAX) {
                        return Err(Error::new(
                            "m3.animation.time",
                            "timestamp exceeds signed milliseconds",
                        ));
                    }
                    let value = exact.round() as i32;
                    context.approximate(
                        "m3.animation.time",
                        ((f64::from(value) - exact) / 1000.).abs() as f32,
                        "timestamp quantized to milliseconds",
                    )?;
                    Ok(nr::I32 { value })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            let values = $keys
                .values
                .iter()
                .map(|value| nr::$record {
                    value: ($convert)(value),
                })
                .collect::<Vec<_>>();
            let index = blocks.$field.len();
            blocks.$field.push(nr::$block {
                timestamps: append(document, &times)?.cast(),
                keys: append(document, &values)?.cast(),
                flags,
                end_frame,
            });
            if source.identity.is_some() {
                Some(
                    ($family << 16)
                        | u32::from(u16::try_from(index).map_err(|_| {
                            Error::new(
                                "m3.animation.selector",
                                "key pool index exceeds selector width",
                            )
                        })?),
                )
            } else {
                None
            }
        }};
    }
    Ok(match &source.keys {
        UnboundKeys::Scalar(keys) => block!(keys, floats, Real, AnimationBlockReal, 5, |v| v),
        UnboundKeys::Vector2(keys) => {
            block!(keys, vectors2, Vector2, AnimationBlockVector2, 1, |v| v)
        }
        UnboundKeys::Vector3(keys) => {
            block!(keys, vectors3, Vector3, AnimationBlockVector3, 2, |v| v)
        }
        UnboundKeys::Rotation(keys) => block!(
            keys,
            quaternions,
            Quaternion,
            AnimationBlockQuaternion,
            3,
            |v: crate::model::Quaternion| v.components()
        ),
        UnboundKeys::Color(keys) => {
            block!(keys, colors, Color, AnimationBlockColor, 4, |v: [u8; 4]| [
                v[2], v[1], v[0], v[3]
            ])
        }
        UnboundKeys::Byte(keys) => block!(keys, bytes, Byte, AnimationBlockByte, 6, |v| v),
        UnboundKeys::Signed16(keys) => block!(keys, signed16, I16, AnimationBlockI16, 7, |v| v),
        UnboundKeys::Unsigned16(keys) => block!(keys, unsigned16, U16, AnimationBlockU16, 8, |v| v),
        UnboundKeys::Signed32(keys) => block!(keys, signed32, I32, AnimationBlockI32, 9, |v| v),
        UnboundKeys::Unsigned32(keys) => {
            block!(keys, unsigned32, U32, AnimationBlockU32, 10, |v| v)
        }
        UnboundKeys::Flags(keys) => block!(
            keys,
            flags,
            Flag,
            AnimationBlockFlag,
            11,
            nr::FlagBits::from
        ),
        UnboundKeys::Extent(keys) => {
            let index = blocks.extents.len();
            let times = keys
                .times
                .iter()
                .map(|time| {
                    super::structured_export::signed_time(time, Default::default(), context)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let values = keys
                .values
                .iter()
                .map(|v| nr::ExtentRecord {
                    value: nr::Extent {
                        minimum: v.minimum,
                        maximum: v.maximum,
                        radius: v.sphere_radius,
                    },
                })
                .collect::<Vec<_>>();
            blocks.extents.push(nr::AnimationBlockExtent {
                timestamps: append(document, &times)?.cast(),
                keys: append(document, &values)?.cast(),
                flags,
                end_frame,
            });
            if source.identity.is_some() {
                Some(
                    (12 << 16)
                        | u32::from(u16::try_from(index).map_err(|_| {
                            Error::new(
                                "m3.animation.selector",
                                "extent pool exceeds selector width",
                            )
                        })?),
                )
            } else {
                None
            }
        }
        UnboundKeys::Events(_) => None,
    })
}
