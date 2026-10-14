//! Borrowed typed keys retain native integer precision and millisecond clocks.
use super::super::bindings::Target;
use crate::model::conversion::{Context, Error};
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::{profiles::m3 as surface, EventId, FeatureId, PropertyTarget};
use crate::model::semantic::{
    Channel, Clock, Curve, Interpolation, Quaternion, Time, Track, Values,
};
use native::Envelope;

fn read_error(error: native::ReadError) -> Error {
    Error::new("m3.animation", error.to_string())
}
pub(super) fn controls<E: Envelope>(
    document: &native::Document<'_, E>,
    controller: &nr::SubTrackContainer<nr::V4>,
    selector: u32,
    source_id: u32,
    targets: Vec<PropertyTarget>,
    events: Vec<EventId>,
    extents: Vec<FeatureId>,
) -> Result<surface::BlockControls, Error> {
    let index = (selector & 0xffff) as usize;
    macro_rules! block {
        ($record:ty,$reference:expr,$encoding:ident) => {{
            let row = document
                .resolve::<$record>($reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.animation.block", "missing block"))?
                .map_err(read_error)?;
            (
                surface::KeyEncoding::$encoding,
                row.flags.bits(),
                row.end_frame,
            )
        }};
    }
    let (encoding, flags, end) = match selector >> 16 {
        0 => block!(nr::AnimationBlockEvent, controller.sdev, Event),
        1 => block!(nr::AnimationBlockVector2, controller.sd2v, Vector2),
        2 => block!(nr::AnimationBlockVector3, controller.sd3v, Vector3),
        3 => block!(nr::AnimationBlockQuaternion, controller.sd4q, Quaternion),
        4 => block!(nr::AnimationBlockColor, controller.sdcc, Color),
        5 => block!(nr::AnimationBlockReal, controller.sdr3, Float),
        6 => block!(nr::AnimationBlockByte, controller.sdu8, Byte),
        7 => block!(nr::AnimationBlockI16, controller.sds6, Signed16),
        8 => block!(nr::AnimationBlockU16, controller.sdu6, Unsigned16),
        9 => block!(nr::AnimationBlockI32, controller.sds3, Signed32),
        10 => block!(nr::AnimationBlockU32, controller.sdu3, Unsigned32),
        11 => block!(nr::AnimationBlockFlag, controller.sdfg, Flags),
        12 => block!(nr::AnimationBlockExtent, controller.sdmb, Extent),
        _ => {
            return Err(Error::new(
                "m3.animation.selector",
                "unknown native key family",
            ));
        }
    };
    Ok(surface::BlockControls {
        source_id,
        targets,
        events,
        extents,
        encoding,
        flags: surface::BlockFlags::from_bits_retain(flags),
        end: Time::seconds(f64::from(end) / 1000.)?,
    })
}

pub(super) fn read<'a, E: Envelope + Send + Sync>(
    document: &'a native::Document<'_, E>,
    controller: &nr::SubTrackContainer<nr::V4>,
    selector: u32,
    target: &Target,
    mode: Interpolation,
    context: &mut Context,
) -> Result<Option<Channel<'a>>, Error> {
    let index = (selector & 0xffff) as usize;
    macro_rules! keys {
        ($block:ty, $reference:expr, $record:ty, $convert:expr, $target:pat => $property:expr, $channel:ident) => {{
            let blocks = document
                .resolve::<$block>($reference.cast())
                .map_err(read_error)?;
            let block = blocks
                .get(index)
                .ok_or_else(|| Error::new("m3.animation.selector", "missing key block"))?
                .map_err(read_error)?;
            let times = document
                .resolve::<nr::I32>(block.timestamps.cast())
                .map_err(read_error)?;
            let values = document
                .resolve::<$record>(block.keys.cast())
                .map_err(read_error)?;
            if times.len() != values.len() {
                return Err(Error::new(
                    "m3.animation.keys",
                    "timestamp/value count differs",
                ));
            }
            for value in times.iter() {
                value.map_err(read_error)?;
            }
            for value in values.iter() {
                value.map_err(read_error)?;
            }
            if let $target = target {
                let curve = Curve::new(
                    Values::view(times.len(), move |i| {
                        Time::seconds(f64::from(times.get(i).unwrap().unwrap().value) / 1000.)
                            .unwrap()
                    }),
                    Values::view(values.len(), move |i| {
                        ($convert)(values.get(i).unwrap().unwrap())
                    }),
                    mode,
                )?;
                Some(Channel::$channel(Track {
                    target: $property.clone(),
                    curve,
                    clock: Clock::Timeline,
                }))
            } else {
                context.unsupported("m3.animation.type", "key family and property type differ")?;
                None
            }
        }};
    }
    Ok(match selector >> 16 {
        1 => {
            keys!(nr::AnimationBlockVector2,controller.sd2v,nr::Vector2,|v:nr::Vector2|v.value,Target::Vector2(p)=>p,Vector2)
        }
        2 => {
            keys!(nr::AnimationBlockVector3,controller.sd3v,nr::Vector3,|v:nr::Vector3|v.value,Target::Vector3(p)=>p,Vector3)
        }
        3 => {
            keys!(nr::AnimationBlockQuaternion,controller.sd4q,nr::Quaternion,|v:nr::Quaternion|Quaternion(v.value),Target::Rotation(p)=>p,Rotation)
        }
        4 => match target {
            Target::Vector4(_) => {
                keys!(nr::AnimationBlockColor,controller.sdcc,nr::Color,|v:nr::Color|[f32::from(v.value[2])/255.,f32::from(v.value[1])/255.,f32::from(v.value[0])/255.,f32::from(v.value[3])/255.],Target::Vector4(p)=>p,Vector4)
            }
            _ => {
                keys!(nr::AnimationBlockColor,controller.sdcc,nr::Color,|v:nr::Color|v.value,Target::Bytes4(p)=>p,Bytes4)
            }
        },
        5 => {
            keys!(nr::AnimationBlockReal,controller.sdr3,nr::Real,|v:nr::Real|v.value,Target::Scalar(p)=>p,Scalar)
        }
        6 => {
            keys!(nr::AnimationBlockByte,controller.sdu8,nr::Byte,|v:nr::Byte|v.value,Target::Byte(p)=>p,Byte)
        }
        7 => {
            keys!(nr::AnimationBlockI16,controller.sds6,nr::I16,|v:nr::I16|i32::from(v.value),Target::Signed(p)=>p,Signed)
        }
        8 => {
            keys!(nr::AnimationBlockU16,controller.sdu6,nr::U16,|v:nr::U16|v.value,Target::Unsigned16(p)=>p,Unsigned16)
        }
        9 => {
            keys!(nr::AnimationBlockI32,controller.sds3,nr::I32,|v:nr::I32|v.value,Target::Signed(p)=>p,Signed)
        }
        10 => {
            keys!(nr::AnimationBlockU32,controller.sdu3,nr::U32,|v:nr::U32|v.value,Target::Unsigned(p)=>p,Unsigned)
        }
        11 => {
            keys!(nr::AnimationBlockFlag,controller.sdfg,nr::Flag,|v:nr::Flag|v.value.bits(),Target::Unsigned(p)=>p,Unsigned)
        }
        _ => {
            context.unsupported(
                "m3.animation.family",
                "structured event or extent keys require a structured destination",
            )?;
            None
        }
    })
}
