//! Typed native key block construction from authored logical curves.
use crate::model::conversion::{Context, Error};
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::{
    profiles::m3 as surface, Channel, Clock, Interpolation, Quaternion, Time,
};
use native::{Envelope, Ref};

fn append<E: Envelope, T: native::Record + native::Write<E> + native::RecordLayout<E>>(
    document: &mut native::Document<'_, E>,
    rows: &[T],
) -> Result<Ref<T>, Error> {
    if rows.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(rows.len())
        .map_err(|_| Error::new("m3.animation.count", "native array count exceeds u32"))?;
    let section = document
        .append(rows)
        .map_err(|e| Error::new("m3.animation.write", e.to_string()))?;
    Ok(Ref::from_section(count, section.index()))
}
fn milliseconds(value: Time, origin: Time, context: &mut Context) -> Result<i32, Error> {
    let exact = (value.as_seconds() + origin.as_seconds()) * 1000.;
    if !exact.is_finite()
        || exact.round() < f64::from(i32::MIN)
        || exact.round() > f64::from(i32::MAX)
    {
        return Err(Error::new(
            "m3.animation.time",
            "timestamp exceeds native signed milliseconds",
        ));
    }
    let raw = exact.round() as i32;
    context.approximate(
        "m3.animation.time",
        ((f64::from(raw) - exact) / 1000.).abs() as f32,
        "timestamp quantized to native milliseconds",
    )?;
    Ok(raw)
}
fn color(value: [f32; 4], context: &mut Context) -> Result<[u8; 4], Error> {
    let mut rgba = [0; 4];
    for (index, value) in value.into_iter().enumerate() {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(Error::new(
                "m3.animation.color",
                "color exceeds normalized byte range",
            ));
        }
        let raw = (value * 255.).round() as u8;
        context.approximate(
            "m3.animation.color",
            (f32::from(raw) / 255. - value).abs(),
            "color quantized to native byte components",
        )?;
        rgba[index] = raw;
    }
    Ok([rgba[2], rgba[1], rgba[0], rgba[3]])
}
#[derive(Default)]
pub(super) struct Blocks {
    pub vectors2: Vec<nr::AnimationBlockVector2>,
    pub vectors3: Vec<nr::AnimationBlockVector3>,
    pub quaternions: Vec<nr::AnimationBlockQuaternion>,
    pub colors: Vec<nr::AnimationBlockColor>,
    pub floats: Vec<nr::AnimationBlockReal>,
    pub bytes: Vec<nr::AnimationBlockByte>,
    pub signed16: Vec<nr::AnimationBlockI16>,
    pub unsigned16: Vec<nr::AnimationBlockU16>,
    pub signed32: Vec<nr::AnimationBlockI32>,
    pub unsigned32: Vec<nr::AnimationBlockU32>,
    pub flags: Vec<nr::AnimationBlockFlag>,
    pub events: Vec<nr::AnimationBlockEvent>,
    pub extents: Vec<nr::AnimationBlockExtent>,
}
impl Blocks {
    pub(super) fn install<E: Envelope>(
        self,
        document: &mut native::Document<'_, E>,
        controller: &mut nr::SubTrackContainer<nr::V4>,
    ) -> Result<(), Error> {
        controller.sd2v = append(document, &self.vectors2)?.cast();
        controller.sd3v = append(document, &self.vectors3)?.cast();
        controller.sd4q = append(document, &self.quaternions)?.cast();
        controller.sdcc = append(document, &self.colors)?.cast();
        controller.sdr3 = append(document, &self.floats)?.cast();
        controller.sdu8 = append(document, &self.bytes)?.cast();
        controller.sds6 = append(document, &self.signed16)?.cast();
        controller.sdu6 = append(document, &self.unsigned16)?.cast();
        controller.sds3 = append(document, &self.signed32)?.cast();
        controller.sdu3 = append(document, &self.unsigned32)?.cast();
        controller.sdfg = append(document, &self.flags)?.cast();
        controller.sdev = append(document, &self.events)?.cast();
        controller.sdmb = append(document, &self.extents)?.cast();
        Ok(())
    }
    pub(super) fn add<E: Envelope>(
        &mut self,
        document: &mut native::Document<'_, E>,
        channel: &Channel<'_>,
        origin: Time,
        end: Time,
        controls: Option<&surface::BlockControls>,
        context: &mut Context,
    ) -> Result<Option<u32>, Error> {
        if channel.clock() != Clock::Timeline {
            context.unsupported(
                "m3.animation.clock",
                "M3 sequence controllers require absolute timeline clocks",
            )?;
            return Ok(None);
        }
        let flags = nr::AnimationBlockFlags::from(controls.map_or(0, |c| c.flags.bits()));
        let end_frame = if let Some(controls) = controls {
            u32::try_from(milliseconds(controls.end, Time::default(), context)?)
                .map_err(|_| Error::new("m3.animation.end", "block end cannot be negative"))?
        } else {
            u32::try_from(milliseconds(end, origin, context)?)
                .map_err(|_| Error::new("m3.animation.end", "block end cannot be negative"))?
        };
        macro_rules! block {
            ($track:ident,$record:ident,$convert:expr,$field:ident,$block:ident,$family:expr) => {{
                if matches!(
                    $track.curve.interpolation(),
                    Interpolation::Hermite
                        | Interpolation::Bezier
                        | Interpolation::Tcb
                        | Interpolation::Slerp
                        | Interpolation::Nlerp
                ) {
                    context.unsupported(
                        "m3.animation.interpolation",
                        "native M3 controllers do not encode the requested interpolation directive",
                    )?;
                    return Ok(None);
                }
                let times = $track
                    .curve
                    .times()
                    .iter()
                    .map(|t| milliseconds(t, origin, context).map(|value| nr::I32 { value }))
                    .collect::<Result<Vec<_>, _>>()?;
                let values = $track
                    .curve
                    .values()
                    .iter()
                    .map(|v| ($convert)(v).map(|value| nr::$record { value }))
                    .collect::<Result<Vec<_>, Error>>()?;
                let index = u16::try_from(self.$field.len()).map_err(|_| {
                    Error::new(
                        "m3.animation.selector",
                        "more than 65536 blocks in one key family",
                    )
                })?;
                self.$field.push(nr::$block {
                    timestamps: append(document, &times)?.cast(),
                    keys: append(document, &values)?.cast(),
                    flags,
                    end_frame: if controls.is_none() {
                        times
                            .last()
                            .map_or(end_frame, |time| time.value.max(0) as u32)
                    } else {
                        end_frame
                    },
                });
                Some(($family << 16) | u32::from(index))
            }};
        }
        Ok(match channel {
            Channel::Scalar(t) => block!(t, Real, Ok, floats, AnimationBlockReal, 5),
            Channel::Vector2(t) => {
                block!(t, Vector2, Ok, vectors2, AnimationBlockVector2, 1)
            }
            Channel::Vector3(t) => {
                block!(t, Vector3, Ok, vectors3, AnimationBlockVector3, 2)
            }
            Channel::Rotation(t) => block!(
                t,
                Quaternion,
                |v: Quaternion| Ok(v.components()),
                quaternions,
                AnimationBlockQuaternion,
                3
            ),
            Channel::Bytes4(t) => block!(t, Color, Ok, colors, AnimationBlockColor, 4),
            Channel::Vector4(t) => {
                block!(
                    t,
                    Color,
                    |v| color(v, context),
                    colors,
                    AnimationBlockColor,
                    4
                )
            }
            Channel::Byte(t) => block!(t, Byte, Ok, bytes, AnimationBlockByte, 6),
            Channel::Signed(t)
                if controls.is_some_and(|c| c.encoding == surface::KeyEncoding::Signed16) =>
            {
                block!(
                    t,
                    I16,
                    |v| i16::try_from(v).map_err(|_| Error::new(
                        "m3.animation.integer",
                        "value exceeds authored signed16 domain"
                    )),
                    signed16,
                    AnimationBlockI16,
                    7
                )
            }
            Channel::Signed(t) => block!(t, I32, Ok, signed32, AnimationBlockI32, 9),
            Channel::Unsigned16(t) => block!(t, U16, Ok, unsigned16, AnimationBlockU16, 8),
            Channel::Unsigned(t)
                if controls.is_some_and(|c| c.encoding == surface::KeyEncoding::Flags) =>
            {
                block!(
                    t,
                    Flag,
                    |v| Ok(nr::FlagBits::from(v)),
                    flags,
                    AnimationBlockFlag,
                    11
                )
            }
            Channel::Unsigned(t) => block!(t, U32, Ok, unsigned32, AnimationBlockU32, 10),
            _ => {
                context.unsupported(
                    "m3.animation.type",
                    "channel requires a structured or projected native key encoding",
                )?;
                None
            }
        })
    }
}
