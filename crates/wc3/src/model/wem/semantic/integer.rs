//! Typed integer keys share the floating-point curve structure without narrowing.
use super::*;
use std::collections::BTreeSet;
trait Scalar: Copy + Default {
    const BYTES: usize;
    fn decode(bytes: &[u8]) -> Self;
    fn encode(self, bytes: &mut Vec<u8>);
}
macro_rules! scalar {
    ($t:ty,$size:expr) => {
        impl Scalar for $t {
            const BYTES: usize = $size;
            fn decode(bytes: &[u8]) -> Self {
                Self::from_le_bytes(bytes.try_into().unwrap())
            }
            fn encode(self, bytes: &mut Vec<u8>) {
                bytes.extend_from_slice(&self.to_le_bytes());
            }
        }
    };
}
scalar!(u32, 4);
scalar!(i32, 4);
scalar!(u16, 2);
scalar!(u8, 1);
fn factor(interpolation: Interpolation) -> usize {
    if matches!(
        interpolation,
        Interpolation::Hermite | Interpolation::Bezier
    ) {
        3
    } else {
        1
    }
}
pub(super) fn decode(
    kind: r::AttrType,
    target: Target,
    interpolation: Interpolation,
    layer: TrackLayer,
    duration: Option<f32>,
    times: &[r::Real],
    data: &[u8],
) -> Result<IntegerTrack, ReadError> {
    fn read<T: Scalar>(
        target: Target,
        interpolation: Interpolation,
        layer: TrackLayer,
        duration: Option<f32>,
        times: &[r::Real],
        data: &[u8],
        width: usize,
    ) -> Result<Track<T>, ReadError> {
        let stride = width * T::BYTES * factor(interpolation);
        if data.len() != times.len() * stride {
            return Err(ReadError::invalid(0, "integer key width"));
        }
        let keys = times
            .iter()
            .zip(data.chunks_exact(stride))
            .map(|(time, data)| {
                let values: Vec<_> = data.chunks_exact(T::BYTES).map(T::decode).collect();
                Key {
                    time: time.value,
                    value: values[..width].to_vec(),
                    incoming: (factor(interpolation) == 3)
                        .then(|| values[width..2 * width].to_vec()),
                    outgoing: (factor(interpolation) == 3).then(|| values[2 * width..].to_vec()),
                }
            })
            .collect();
        Ok(Track {
            layer,
            target,
            interpolation,
            global_duration: duration,
            keys,
        })
    }
    Ok(match kind {
        r::AttrType::U32 => IntegerTrack::Unsigned(read(
            target,
            interpolation,
            layer,
            duration,
            times,
            data,
            1,
        )?),
        r::AttrType::I32 => IntegerTrack::Signed(read(
            target,
            interpolation,
            layer,
            duration,
            times,
            data,
            1,
        )?),
        r::AttrType::U16 => IntegerTrack::Unsigned16(read(
            target,
            interpolation,
            layer,
            duration,
            times,
            data,
            1,
        )?),
        r::AttrType::U8X4 => IntegerTrack::Bytes4(read(
            target,
            interpolation,
            layer,
            duration,
            times,
            data,
            4,
        )?),
        r::AttrType::BOOL => IntegerTrack::Boolean(read(
            target,
            interpolation,
            layer,
            duration,
            times,
            data,
            1,
        )?),
        r::AttrType::U32X4 => IntegerTrack::Unsigned4(read(
            target,
            interpolation,
            layer,
            duration,
            times,
            data,
            4,
        )?),
        _ => return Err(ReadError::unsupported(0, "integer curve value type")),
    })
}
pub(super) fn validate(track: &IntegerTrack) -> Result<(), ReadError> {
    fn check<T>(track: &Track<T>, width: usize) -> Result<(), ReadError> {
        if !matches!(
            track.target,
            Target::Property { .. } | Target::NativeProperty { .. }
        ) {
            return Err(ReadError::invalid(0, "integer transform channel"));
        }
        if track
            .global_duration
            .is_some_and(|v| !v.is_finite() || v <= 0.)
        {
            return Err(ReadError::invalid(0, "integer global clock"));
        }
        let mut last = f32::NEG_INFINITY;
        for key in &track.keys {
            if !key.time.is_finite() || key.time < last || key.value.len() != width {
                return Err(ReadError::invalid(0, "integer key layout"));
            }
            if factor(track.interpolation) == 3
                && [&key.incoming, &key.outgoing]
                    .iter()
                    .any(|v| v.as_ref().is_none_or(|v| v.len() != width))
            {
                return Err(ReadError::invalid(0, "integer spline controls"));
            }
            last = key.time;
        }
        Ok(())
    }
    match track {
        IntegerTrack::Unsigned(t) => check(t, 1),
        IntegerTrack::Signed(t) => check(t, 1),
        IntegerTrack::Unsigned16(t) => check(t, 1),
        IntegerTrack::Bytes4(t) => check(t, 4),
        IntegerTrack::Boolean(t) => check(t, 1),
        IntegerTrack::Unsigned4(t) => check(t, 4),
    }
}
pub(super) fn encode(
    track: &IntegerTrack,
    doc: &mut RawDocument<'static>,
    channels: &mut Vec<r::AnimChannel>,
    next: &mut u32,
    reserved: &BTreeSet<u32>,
    original: Option<&editing::Original>,
) -> Result<r::SubTrackContainer, WriteError> {
    fn write<T: Scalar>(
        track: &Track<T>,
        layout: (r::AttrType, usize),
        doc: &mut RawDocument<'static>,
        channels: &mut Vec<r::AnimChannel>,
        next: &mut u32,
        reserved: &BTreeSet<u32>,
        original: Option<&editing::Original>,
    ) -> Result<r::SubTrackContainer, WriteError> {
        let (kind, width) = layout;
        let (owner, channel, sub) = match track.target {
            Target::NativeProperty { owner, id } => (owner, r::Channel(255), id),
            Target::Property { owner, channel } => (owner, r::Channel::EMITTER_PROPERTY, channel),
            _ => return Err(WriteError::invalid("integer transform channel")),
        };
        let id = if let Target::NativeProperty { id, .. } = track.target {
            id
        } else {
            while reserved.contains(next) {
                *next = next
                    .checked_add(1)
                    .ok_or_else(|| WriteError::invalid("channel identity overflow"))?;
            }
            let id = *next;
            *next = next
                .checked_add(1)
                .ok_or_else(|| WriteError::invalid("channel identity overflow"))?;
            id
        };
        let old = original.and_then(|o| {
            o.channels
                .iter()
                .find(|c| c.id == id && c.target.channel == r::Channel(255))
        });
        let mut target = old.map(|c| c.target.clone()).unwrap_or_default();
        target.node = owner;
        target.channel = channel;
        target.sub = sub;
        let mut init = Vec::new();
        if let Some(first) = track.keys.first() {
            for &v in &first.value {
                v.encode(&mut init);
            }
        } else {
            for _ in 0..width {
                T::default().encode(&mut init);
            }
        }
        let init_value = if let Some(old) = old.filter(|c| c.value_type == kind) {
            old.init_value
        } else {
            bytes(doc, &init)?
        };
        if let Some(existing) = channels.iter().find(|c| c.id == id) {
            if existing.target != target || existing.value_type != kind {
                return Err(WriteError::invalid("conflicting integer channel identity"));
            }
        } else {
            channels.push(r::AnimChannel {
                id,
                target,
                value_type: kind,
                init_value,
            });
        }
        let mut data = Vec::new();
        for key in &track.keys {
            for &v in &key.value {
                v.encode(&mut data);
            }
            if factor(track.interpolation) == 3 {
                for tangent in [&key.incoming, &key.outgoing] {
                    for &v in tangent
                        .as_ref()
                        .ok_or_else(|| WriteError::invalid("integer spline controls"))?
                    {
                        v.encode(&mut data);
                    }
                }
            }
        }
        let interp = match track.interpolation {
            Interpolation::Step => r::Interpolation::STEP,
            Interpolation::Linear => r::Interpolation::LINEAR,
            Interpolation::Hermite => r::Interpolation::HERMITE,
            Interpolation::Bezier => r::Interpolation::BEZIER,
        };
        let row = r::SubTrack::<r::V2> {
            channel: id,
            interp,
            times: floats(doc, &track.keys.iter().map(|k| k.time).collect::<Vec<_>>())?,
            values: bytes(doc, &data)?,
            tcb: Ref::default(),
        };
        let extra = track
            .global_duration
            .map(|v| {
                vec![(
                    "conversion.global_duration",
                    i64::from(v.to_bits()),
                    String::new(),
                )]
            })
            .unwrap_or_default();
        Ok(r::SubTrackContainer {
            name: text(doc, &track.layer.name)?,
            priority: track.layer.priority,
            concurrent: track.layer.concurrent.into(),
            sub_tracks: put(doc, &[row])?.cast(),
            native: metadata(doc, &extra)?,
        })
    }
    match track {
        IntegerTrack::Unsigned(t) => write(
            t,
            (r::AttrType::U32, 1),
            doc,
            channels,
            next,
            reserved,
            original,
        ),
        IntegerTrack::Signed(t) => write(
            t,
            (r::AttrType::I32, 1),
            doc,
            channels,
            next,
            reserved,
            original,
        ),
        IntegerTrack::Unsigned16(t) => write(
            t,
            (r::AttrType::U16, 1),
            doc,
            channels,
            next,
            reserved,
            original,
        ),
        IntegerTrack::Bytes4(t) => write(
            t,
            (r::AttrType::U8X4, 4),
            doc,
            channels,
            next,
            reserved,
            original,
        ),
        IntegerTrack::Boolean(t) => write(
            t,
            (r::AttrType::BOOL, 1),
            doc,
            channels,
            next,
            reserved,
            original,
        ),
        IntegerTrack::Unsigned4(t) => write(
            t,
            (r::AttrType::U32X4, 4),
            doc,
            channels,
            next,
            reserved,
            original,
        ),
    }
}
