//! Borrowed native curves expressed as typed logical channels.
use super::*;
use crate::model::io::materialize;
use semantic::profiles::M2;
use semantic::{Clock, Curve, CurveValue, Property, Time};
use zerovec::ZeroVec;

pub(super) fn curve<'a, 'data: 'a, D: Dialect, T, U: CurveValue + 'a>(
    source: &'a Bundle<'data, D>,
    track: &'a Track<'data, T, D::Layout>,
    owner: &native::FileKey,
    sequence: usize,
    start: u32,
    map: impl Fn(T) -> U + Clone + Send + Sync + 'a,
) -> Result<Option<(Curve<'a, U>, u16)>, Error>
where
    [T; 3]: Value,
    T: Value + Send + Sync + 'a,
    T::ULE: Send + Sync,
    <[T; 3] as zerovec::ule::AsULE>::ULE: Send + Sync,
{
    if (0..track.base.timeline.len()).all(|i| match track.base.timeline.leaf(i) {
        Some(Array::Inline(v)) => v.is_empty(),
        Some(Array::External(r)) => r.count == 0,
        None => true,
    }) {
        return Ok(None);
    }
    let resolved = source
        .resolve_track(
            track,
            owner,
            if *owner == source.key {
                *b"AFM2"
            } else {
                *b"AFSB"
            },
            sequence,
        )
        .map_err(failure)?;
    if resolved.timestamps.is_empty() {
        return Ok(None);
    }
    let global = resolved.global_sequence;
    let range = if global == u16::MAX {
        track
            .base
            .timeline
            .ranges()
            .get(sequence)
            .map(|[a, b]| a as usize..b as usize + 1)
    } else {
        None
    };
    let first = range.as_ref().map_or(0, |r| r.start);
    let len = range.map_or(resolved.timestamps.len(), |r| r.end.saturating_sub(r.start));
    let timestamps = resolved.timestamps;
    if first + len > timestamps.len() {
        return Err(Error::new("m2.animation", "timestamp range exceeds array"));
    }
    let times = materialize(len, move |i| {
        let ms = timestamps.get(first + i).unwrap();
        Time::seconds(
            (f64::from(ms)
                - if global == u16::MAX {
                    f64::from(start)
                } else {
                    0.
                })
                / 1000.,
        )
        .unwrap()
    });
    let interpolation = match resolved.interpolation {
        NativeInterpolation::Step => semantic::Interpolation::Step,
        NativeInterpolation::Linear => semantic::Interpolation::Linear,
        NativeInterpolation::Hermite => semantic::Interpolation::Hermite,
        NativeInterpolation::Bezier => semantic::Interpolation::Bezier,
        _ => {
            return Err(Error::new(
                "m2.animation",
                "unrecognized interpolation operation",
            ));
        }
    };
    let curve = match resolved.values {
        ResolvedValues::Values(values) => {
            let data = materialize(len, move |i| map(values.get(first + i).unwrap()));
            Curve::from_samples(times, data, interpolation)?
        }
        ResolvedValues::Splines(values) => {
            let a = values.clone();
            let b = values.clone();
            let ma = map.clone();
            let mb = map.clone();
            Curve::from_spline_samples(
                times,
                materialize(len, move |i| map(values.get(first + i).unwrap()[0])),
                materialize(len, move |i| ma(a.get(first + i).unwrap()[1])),
                materialize(len, move |i| mb(b.get(first + i).unwrap()[2])),
                interpolation,
            )?
        }
    };
    Ok(Some((curve, global)))
}
pub(in super::super) fn bind<'a, T: CurveValue + 'a>(
    curve: Curve<'a, T>,
    global: u16,
    target: Property<T, M2>,
    clocks: &[semantic::ClockId],
) -> Result<semantic::Track<'a, T, M2>, Error> {
    Ok(semantic::Track {
        target,
        curve,
        clock: if global == u16::MAX {
            Clock::Clip
        } else {
            Clock::Shared(
                *clocks
                    .get(global as usize)
                    .ok_or_else(|| Error::new("m2.animation.clock", "missing shared clock"))?,
            )
        },
    })
}

pub(in super::super) trait ChannelValue: CurveValue {
    fn restrict<'a>(
        track: &semantic::Track<'a, Self, M2>,
        start: Time,
        end: Time,
    ) -> Result<semantic::Track<'a, Self, M2>, Error>
    where
        Self: Sized,
    {
        if track.curve.times().iter().any(|t| t < start || t > end) {
            return Err(Error::new(
                "m2.animation.window",
                "nonfloating keys exceed the native clock interval",
            ));
        }
        Ok(track.clone())
    }

    fn channel<'v, 'data>(
        value: &'v semantic::Channel<'data, M2>,
    ) -> Option<&'v semantic::Track<'data, Self, M2>>
    where
        Self: Sized;
}
macro_rules! channel_value {($($ty:ty=>$variant:ident),*)=>{$(impl ChannelValue for $ty{fn channel<'v,'data>(value:&'v semantic::Channel<'data, M2>)->Option<&'v semantic::Track<'data,Self,M2>>{if let semantic::Channel::$variant(value)=value{Some(value)}else{None}}})*};}
channel_value!(u32=>Unsigned,i32=>Signed,u16=>Unsigned16,u8=>Byte,bool=>Boolean);
macro_rules! float_channel {($($ty:ty=>$variant:ident),*)=>{$(impl ChannelValue for $ty{fn channel<'v,'data>(value:&'v semantic::Channel<'data, M2>)->Option<&'v semantic::Track<'data,Self,M2>>{if let semantic::Channel::$variant(value)=value{Some(value)}else{None}}fn restrict<'a>(track:&semantic::Track<'a,Self,M2>,start:Time,end:Time)->Result<semantic::Track<'a,Self,M2>,Error>{if track.curve.times().iter().any(|t|t<start||t>end){crate::model::io::animation::crop(track,start,end)}else{Ok(track.clone())}}})*};}
float_channel!(f32=>Scalar,[f32;2]=>Vector2,[f32;3]=>Vector3,[f32;4]=>Vector4,semantic::Quaternion=>Rotation);
pub(super) fn encode<V: ModelLayout, U: ChannelValue, T: Value>(
    model: &Graph<'_>,
    target: &Property<U>,
    starts: &[u32],
    globals: &mut Vec<u32>,
    context: &mut Context,
    convert: impl Fn(U, &mut Context) -> Result<T, Error>,
) -> Result<Track<'static, T, V>, Error>
where
    [T; 3]: Value,
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, T>: ValuesFactory<T>,
{
    let shared = model
        .shared_channels()
        .iter()
        .filter_map(U::channel)
        .find(|t| t.target == *target);
    let tracks: Vec<_> = if let Some(shared) = shared {
        vec![Some(shared)]
    } else {
        model
            .clips()
            .map(|(_, clip)| {
                clip.channels()
                    .iter()
                    .filter_map(U::channel)
                    .find(|t| t.target == *target)
            })
            .collect()
    };
    let Some(first) = tracks.iter().flatten().next() else {
        return Ok(Track::default());
    };
    if tracks
        .iter()
        .flatten()
        .any(|track| track.curve.uses_spherical_controls())
    {
        return Err(Error::new(
            "m2.animation.interpolation",
            "M2 export cannot preserve spherical spline controls",
        ));
    }
    if tracks
        .iter()
        .flatten()
        .any(|t| t.clock != first.clock || t.curve.interpolation() != first.curve.interpolation())
    {
        return Err(Error::new(
            "m2.animation",
            "clock and interpolation differ across clips",
        ));
    }
    let global = match first.clock {
        Clock::Timeline => {
            return Err(Error::new(
                "m2.animation.clock",
                "M2 requires a clip or shared sequence clock",
            ));
        }
        Clock::Clip => u16::MAX,
        Clock::Shared(id) => {
            let duration = time_ms(
                model
                    .clock(id)
                    .ok_or_else(|| Error::new("m2.animation.clock", "missing clock"))?
                    .period,
                context,
            )?;
            let i = globals
                .iter()
                .position(|d| *d == duration)
                .unwrap_or_else(|| {
                    globals.push(duration);
                    globals.len() - 1
                });
            u16::try_from(i).map_err(|_| Error::new("m2.animation.clock", "too many clocks"))?
        }
    };
    let interpolation = match first.curve.interpolation() {
        semantic::Interpolation::Slerp | semantic::Interpolation::Nlerp => {
            return Err(Error::new(
                "m2.animation.interpolation",
                "M2 has no distinct spherical or normalized-linear interpolation directive",
            ));
        }
        semantic::Interpolation::Tcb => {
            return Err(Error::new(
                "m2.animation.interpolation",
                "M2 has no tension/continuity/bias key encoding",
            ));
        }
        semantic::Interpolation::Step => NativeInterpolation::Step,
        semantic::Interpolation::Linear => NativeInterpolation::Linear,
        semantic::Interpolation::Hermite => NativeInterpolation::Hermite,
        semantic::Interpolation::Bezier => NativeInterpolation::Bezier,
    };
    let spline = interpolation.spline();
    let mut times = Vec::new();
    let mut values = Vec::new();
    let mut splines = Vec::new();
    for (index, track) in tracks.into_iter().enumerate() {
        if global != u16::MAX && index > 0 {
            break;
        }
        let mut ts = Vec::new();
        let mut vs = Vec::new();
        let mut ss = Vec::new();
        if let Some(track) = track {
            let period = match track.clock {
                Clock::Timeline => {
                    return Err(Error::new(
                        "m2.animation.clock",
                        "M2 requires a clip or shared sequence clock",
                    ));
                }
                Clock::Clip => model.clips().nth(index).unwrap().1.duration(),
                Clock::Shared(id) => model.clock(id).unwrap().period,
            };
            let clipped = U::restrict(track, Time::default(), period)?;
            let track = &clipped;
            for (key, time) in track.curve.times().iter().enumerate() {
                let time = time_ms(time, context)?;
                ts.push(if V::VERSION < 264 && global == u16::MAX {
                    time.checked_add(starts[index])
                        .ok_or_else(|| Error::new("m2.animation", "legacy time overflow"))?
                } else {
                    time
                });
                let value = convert(track.curve.values().get(key).unwrap(), context)?;
                if spline {
                    ss.push([
                        value,
                        convert(track.curve.incoming_value(key).unwrap(), context)?,
                        convert(track.curve.outgoing_value(key).unwrap(), context)?,
                    ])
                } else {
                    vs.push(value)
                }
            }
        }
        times.push(ts);
        values.push(vs);
        splines.push(ss)
    }
    Ok(Track {
        base: native::records::TrackBase {
            interpolation,
            global_sequence: global,
            timeline: TimelineFactory::build(times, global != u16::MAX),
        },
        values: ValuesFactory::build(values, splines, spline),
    })
}
pub(super) fn time_ms(time: Time, context: &mut Context) -> Result<u32, Error> {
    let value = time.as_seconds() * 1000.;
    if !(0. ..=f64::from(u32::MAX)).contains(&value) {
        return Err(Error::new(
            "m2.animation.time",
            "time exceeds the unsigned millisecond clock",
        ));
    }
    let rounded = value.round();
    context.approximate(
        "m2.animation.time",
        ((rounded - value) / 1000.).abs() as f32,
        "time quantized to milliseconds",
    )?;
    Ok(rounded as u32)
}

pub(super) fn resolve<'a, 'data: 'a, D: Dialect, T: Value>(
    source: &'a Bundle<'data, D>,
    array: Array<'a, T>,
    sequence: usize,
) -> Result<ZeroVec<'a, T>, Error> {
    match array {
        Array::Inline(v) => Ok(v),
        Array::External(reference) if reference.count == 0 => Ok(ZeroVec::new()),
        Array::External(reference) => {
            let seq = source
                .model
                .core()
                .sequences
                .get(sequence)
                .ok_or_else(|| Error::new("m2.camera.sequence", "missing sequence"))?;
            let key=source.model.dependencies(&source.key).into_iter().find(|d|matches!(d.kind,native::DependencyKind::Animation{animation_id,variation_id}if animation_id==seq.id&&variation_id==seq.variation_index)).ok_or_else(||Error::new("m2.camera.animation","missing external animation identity"))?.key;
            let Some(File::Animation(animation)) = source.file(&key) else {
                return Err(Error::new(
                    "m2.camera.animation",
                    "missing external animation file",
                ));
            };
            let data = animation
                .stream(*b"AFM2")
                .ok_or_else(|| Error::new("m2.camera.animation", "missing model stream"))?;
            Ok(Decoder::new(data, D::VERSION, ModelFlags::default())
                .array::<T>(reference)
                .map_err(failure)?)
        }
    }
}
pub(in super::super) fn spline_curve<'a, 'data: 'a, D: Dialect, T: Value + CurveValue + 'a>(
    source: &'a Bundle<'data, D>,
    track: &'a native::records::SplineTrack<'data, T, D::Layout>,
    sequence: usize,
    start: u32,
) -> Result<Option<(Curve<'a, T>, u16)>, Error>
where
    [T; 3]: Value,
    T::ULE: Send + Sync,
    <[T; 3] as zerovec::ule::AsULE>::ULE: Send + Sync,
{
    use native::records::SplineStorage as _;
    let global = track.base.global_sequence;
    let leaf = if global != u16::MAX || D::VERSION < 264 {
        0
    } else {
        sequence
    };
    let Some(times) = track.base.timeline.leaf(leaf) else {
        return Ok(None);
    };
    let times = resolve(source, times, sequence)?;
    if times.is_empty() {
        return Ok(None);
    }
    let values = resolve(
        source,
        track
            .values
            .leaf(leaf)
            .ok_or_else(|| Error::new("m2.camera.spline", "missing value leaf"))?,
        sequence,
    )?;
    let range = if global == u16::MAX {
        track
            .base
            .timeline
            .ranges()
            .get(sequence)
            .map(|[a, b]| a as usize..b as usize + 1)
    } else {
        None
    };
    let first = range.as_ref().map_or(0, |r| r.start);
    let len = range.map_or(times.len(), |r| r.end - r.start);
    let timestamps = materialize(len, move |i| {
        Time::seconds(
            (f64::from(times.get(first + i).unwrap())
                - if global == u16::MAX {
                    f64::from(start)
                } else {
                    0.
                })
                / 1000.,
        )
        .unwrap()
    });
    let a = values.clone();
    let b = values.clone();
    let keys = materialize(len, move |i| values.get(first + i).unwrap()[0]);
    let interpolation = match track.base.interpolation {
        NativeInterpolation::Step => semantic::Interpolation::Step,
        NativeInterpolation::Linear => semantic::Interpolation::Linear,
        NativeInterpolation::Hermite => semantic::Interpolation::Hermite,
        NativeInterpolation::Bezier => semantic::Interpolation::Bezier,
        _ => return Err(Error::new("m2.camera.spline", "unknown interpolation")),
    };
    let curve = if matches!(
        interpolation,
        semantic::Interpolation::Hermite | semantic::Interpolation::Bezier
    ) {
        Curve::from_spline_samples(
            timestamps,
            keys,
            materialize(len, move |i| a.get(first + i).unwrap()[1]),
            materialize(len, move |i| b.get(first + i).unwrap()[2]),
            interpolation,
        )?
    } else {
        Curve::from_samples(timestamps, keys, interpolation)?
    };
    Ok(Some((curve, global)))
}

pub(super) fn animated<V: ModelLayout, U: ChannelValue, T: Value>(
    model: &Graph<'_>,
    value: &Animated<U>,
    starts: &[u32],
    globals: &mut Vec<u32>,
    context: &mut Context,
    convert: impl Fn(U, &mut Context) -> Result<T, Error>,
) -> Result<Track<'static, T, V>, Error>
where
    [T; 3]: Value,
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, T>: ValuesFactory<T>,
{
    let mut track = if let Some(property) = &value.property {
        encode(model, property, starts, globals, context, &convert)?
    } else {
        Track::default()
    };
    let default = convert(value.default, context)?;
    ensure_constant(&mut track, default, globals);
    Ok(track)
}
pub(super) fn fixed_alpha(value: f32, context: &mut Context) -> Result<i16, Error> {
    if !value.is_finite() || !(0. ..=1.).contains(&value) {
        return Err(Error::new(
            "m2.alpha",
            "opacity must be within zero and one",
        ));
    }
    let raw = (value * 32767.).round() as i16;
    context.approximate(
        "m2.alpha",
        (f32::from(raw) / 32767. - value).abs(),
        "opacity quantized to fixed16",
    )?;
    Ok(raw)
}
pub(in super::super) fn animated_spline<V: ModelLayout, U: ChannelValue, T: Value>(
    model: &Graph<'_>,
    value: &Animated<U>,
    starts: &[u32],
    globals: &mut Vec<u32>,
    context: &mut Context,
    convert: impl Fn(U, &mut Context) -> Result<T, Error>,
) -> Result<native::records::SplineTrack<'static, T, V>, Error>
where
    [T; 3]: Value,
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, T>: ValuesFactory<T>,
    V::SplineValues<'static, T>: SplineFactory<T>,
{
    let regular = animated::<V, U, T>(model, value, starts, globals, context, convert)?;
    let leaves = (0..regular.base.timeline.len())
        .map(|i| match regular.values.inline(i) {
            Some(native::records::SequenceValues::Values(v)) => v.iter().map(|v| [v; 3]).collect(),
            Some(native::records::SequenceValues::Splines(v)) => v.iter().collect(),
            None => Vec::new(),
        })
        .collect();
    Ok(native::records::SplineTrack {
        base: regular.base,
        values: SplineFactory::build(leaves),
    })
}
pub(in super::super) fn sequences(
    clips: &[semantic::ClipId],
) -> Vec<(usize, Option<semantic::ClipId>)> {
    if clips.is_empty() {
        vec![(0, None)]
    } else {
        clips
            .iter()
            .copied()
            .enumerate()
            .map(|(i, id)| (i, Some(id)))
            .collect()
    }
}
pub(in super::super) fn insert<'a>(
    builder: &mut GraphBuilder<'a>,
    clip: Option<semantic::ClipId>,
    channel: semantic::Channel<'a, M2>,
) -> Result<(), Error> {
    let (target, clock) = (channel.target(), channel.clock());
    match clock {
        Clock::Timeline => {
            return Err(Error::new(
                "m2.animation.clock",
                "M2 cannot attach absolute timeline channels",
            ));
        }
        Clock::Clip => {
            if let Some(clip) = clip {
                builder.add_channel(clip, channel)?
            } else {
                return Err(Error::new(
                    "m2.animation.clock",
                    "sequence curve has no playback clip",
                ));
            }
        }
        Clock::Shared(_) => {
            if !builder
                .model()
                .shared_channels()
                .iter()
                .any(|v| (v.target(), v.clock()) == (target, clock))
            {
                builder.add_shared_channel(channel)?
            }
        }
    }
    Ok(())
}
