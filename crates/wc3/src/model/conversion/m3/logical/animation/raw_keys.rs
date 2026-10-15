//! Typed ordered keys are retained even when their property header is unavailable.
use crate::model::conversion::materialize;
use crate::model::conversion::Error;
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::profiles::m3::{BlockFlags, OrderedKeys, UnboundBlock, UnboundKeys};
use crate::model::semantic::{Bounds, Quaternion, Time};
use native::Envelope;

fn error(value: native::ReadError) -> Error {
    Error::new("m3.animation.unbound", value.to_string())
}
pub(super) fn read<'a, E: Envelope + Send + Sync>(
    document: &'a native::Document<'_, E>,
    controller: &nr::SubTrackContainer<nr::V4>,
    selector: u32,
    identity: Option<u32>,
) -> Result<UnboundBlock<'a>, Error> {
    read_index(
        document,
        controller,
        selector >> 16,
        (selector & 0xffff) as usize,
        identity,
    )
}
pub(super) fn read_index<'a, E: Envelope + Send + Sync>(
    document: &'a native::Document<'_, E>,
    controller: &nr::SubTrackContainer<nr::V4>,
    family: u32,
    index: usize,
    identity: Option<u32>,
) -> Result<UnboundBlock<'a>, Error> {
    macro_rules! keys {
        ($block:ty,$reference:expr,$record:ty,$convert:expr,$variant:ident) => {{
            let block = document
                .resolve::<$block>($reference.cast())
                .map_err(error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.animation.unbound", "missing typed key block"))?
                .map_err(error)?;
            let times = document
                .resolve::<nr::I32>(block.timestamps.cast())
                .map_err(error)?;
            let values = document
                .resolve::<$record>(block.keys.cast())
                .map_err(error)?;
            for value in times.iter() {
                value.map_err(error)?;
            }
            for value in values.iter() {
                value.map_err(error)?;
            }
            let keys = OrderedKeys {
                times: materialize(times.len(), move |index| {
                    Time::seconds(f64::from(times.get(index).unwrap().unwrap().value) / 1000.)
                        .unwrap()
                }),
                values: materialize(values.len(), move |index| {
                    ($convert)(values.get(index).unwrap().unwrap())
                }),
            };
            UnboundBlock {
                identity,
                flags: BlockFlags::from_bits_retain(block.flags.bits()),
                end: Time::seconds(f64::from(block.end_frame) / 1000.)?,
                keys: UnboundKeys::$variant(keys),
            }
        }};
    }
    let value = match family {
        1 => keys!(
            nr::AnimationBlockVector2,
            controller.sd2v,
            nr::Vector2,
            |value: nr::Vector2| value.value,
            Vector2
        ),
        2 => keys!(
            nr::AnimationBlockVector3,
            controller.sd3v,
            nr::Vector3,
            |value: nr::Vector3| value.value,
            Vector3
        ),
        3 => keys!(
            nr::AnimationBlockQuaternion,
            controller.sd4q,
            nr::Quaternion,
            |value: nr::Quaternion| Quaternion(value.value),
            Rotation
        ),
        4 => keys!(
            nr::AnimationBlockColor,
            controller.sdcc,
            nr::Color,
            |value: nr::Color| [
                value.value[2],
                value.value[1],
                value.value[0],
                value.value[3]
            ],
            Color
        ),
        5 => keys!(
            nr::AnimationBlockReal,
            controller.sdr3,
            nr::Real,
            |value: nr::Real| value.value,
            Scalar
        ),
        6 => keys!(
            nr::AnimationBlockByte,
            controller.sdu8,
            nr::Byte,
            |value: nr::Byte| value.value,
            Byte
        ),
        7 => keys!(
            nr::AnimationBlockI16,
            controller.sds6,
            nr::I16,
            |value: nr::I16| value.value,
            Signed16
        ),
        8 => keys!(
            nr::AnimationBlockU16,
            controller.sdu6,
            nr::U16,
            |value: nr::U16| value.value,
            Unsigned16
        ),
        9 => keys!(
            nr::AnimationBlockI32,
            controller.sds3,
            nr::I32,
            |value: nr::I32| value.value,
            Signed32
        ),
        10 => keys!(
            nr::AnimationBlockU32,
            controller.sdu3,
            nr::U32,
            |value: nr::U32| value.value,
            Unsigned32
        ),
        11 => keys!(
            nr::AnimationBlockFlag,
            controller.sdfg,
            nr::Flag,
            |value: nr::Flag| value.value.bits(),
            Flags
        ),
        12 => keys!(
            nr::AnimationBlockExtent,
            controller.sdmb,
            nr::ExtentRecord,
            |value: nr::ExtentRecord| Bounds {
                minimum: value.value.minimum,
                maximum: value.value.maximum,
                sphere_radius: value.value.radius
            },
            Extent
        ),
        _ => {
            return Err(Error::new(
                "m3.animation.unbound",
                "unknown numeric key family",
            ));
        }
    };
    value.keys.validate()?;
    Ok(value)
}
