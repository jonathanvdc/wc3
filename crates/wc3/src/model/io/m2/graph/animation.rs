//! Borrowed native curves expressed as typed logical channels.
use super::*;
use crate::model::io::animation::crop;
use crate::model::io::materialize;
use crate::model::PropertyValue;
use glam::{Vec2, Vec3, Vec4};
use semantic::{Curve, CurveValue, Time, TimeSource};

pub(super) fn curve<'a, 'data: 'a, D: Dialect, T, U: CurveValue + 'a>(
    source: &AnimationResolver<'a, 'data, D>,
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
    curve_in_stream(
        source,
        track,
        owner,
        if *owner == source.bundle().key {
            *b"AFM2"
        } else {
            *b"AFSB"
        },
        sequence,
        start,
        map,
    )
}

pub(super) fn curve_in_stream<'a, 'data: 'a, D: Dialect, T, U: CurveValue + 'a>(
    source: &AnimationResolver<'a, 'data, D>,
    track: &'a Track<'data, T, D::Layout>,
    owner: &native::FileKey,
    stream: [u8; 4],
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
    let index = if D::VERSION < 264 || track.base.global_sequence != u16::MAX {
        0
    } else {
        sequence
    };
    match track.base.timeline.leaf(index) {
        Some(Array::Inline(values)) if !values.is_empty() => {}
        Some(Array::External(reference)) if reference.count != 0 => {}
        _ => return Ok(None),
    }
    let resolved = source
        .resolve_track(track, owner, stream, sequence)
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
pub(in super::super) fn bind<'a, T: CurveValue + PropertyValue + 'a>(
    curve: Curve<'a, T>,
    global: u16,
    target: Property<T>,
    clocks: &[semantic::ClockId],
) -> Result<semantic::Track<'a, T>, Error> {
    Ok(semantic::Track {
        target,
        curve,
        clock: if global == u16::MAX {
            TimeSource::Animation
        } else {
            TimeSource::Shared(
                *clocks
                    .get(global as usize)
                    .ok_or_else(|| Error::new("m2.animation.clock", "missing shared clock"))?,
            )
        },
    })
}

pub(in super::super) trait ChannelValue: PropertyValue + CurveValue {
    fn restrict<'a>(
        track: &semantic::Track<'a, Self>,
        start: Time,
        end: Time,
    ) -> Result<semantic::Track<'a, Self>, Error>
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
        value: &'v semantic::DynamicTrack<'data>,
    ) -> Option<&'v semantic::Track<'data, Self>>
    where
        Self: Sized;
}
macro_rules! channel_value {($($ty:ty=>$variant:ident),*)=>{$(impl ChannelValue for $ty{fn channel<'v,'data>(value:&'v semantic::DynamicTrack<'data>)->Option<&'v semantic::Track<'data,Self>>{if let semantic::DynamicTrack::$variant(value)=value{Some(value)}else{None}}})*};}
channel_value!(u32=>Unsigned,i32=>Signed,u16=>Unsigned16,u8=>Byte,bool=>Boolean);
macro_rules! float_channel {($($ty:ty=>$variant:ident),*)=>{$(impl ChannelValue for $ty{fn channel<'v,'data>(value:&'v semantic::DynamicTrack<'data>)->Option<&'v semantic::Track<'data,Self>>{if let semantic::DynamicTrack::$variant(value)=value{Some(value)}else{None}}fn restrict<'a>(track:&semantic::Track<'a,Self>,start:Time,end:Time)->Result<semantic::Track<'a,Self>,Error>{if track.curve.times().iter().any(|t|t<start||t>end){crop(track,start,end)}else{Ok(track.clone())}}})*};}
float_channel!(f32=>Scalar,Vec2 =>Vector2,Vec3 =>Vector3,Vec4 =>Vector4,semantic::Quat=>Rotation);
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
        .global_tracks()
        .filter(|v| matches!(v.clock(), TimeSource::Shared(_)))
        .filter_map(U::channel)
        .find(|t| t.target == *target);
    let tracks: Vec<_> = if let Some(shared) = shared {
        vec![Some(shared)]
    } else {
        model
            .clips()
            .iter()
            .map(|(id, _clip)| {
                model
                    .clip_tracks(id)
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
        TimeSource::Animation => u16::MAX,
        TimeSource::Shared(id) => {
            let duration = time_ms(
                model
                    .clock(id)
                    .ok_or_else(|| Error::new("m2.animation.clock", "missing clock"))?
                    .period(),
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
            let (source_start, duration) = match track.clock {
                TimeSource::Animation => {
                    let clip = model.clips().iter().nth(index).unwrap().1;
                    (clip.source_start, clip.duration())
                }
                TimeSource::Shared(id) => (Time::default(), model.clock(id).unwrap().period()),
            };
            let end = Time::seconds(source_start.as_seconds() + duration.as_seconds())?;
            let mut clipped = track.clone();
            clipped.curve = match track.clock {
                TimeSource::Animation => track.curve.select_interval(source_start, end),
                TimeSource::Shared(_) => U::restrict(track, source_start, end)?.curve,
            };
            let track = &clipped;
            for (key, time) in track.curve.times().iter().enumerate() {
                let time = time_ms(
                    Time::seconds(time.as_seconds() - source_start.as_seconds())?,
                    context,
                )?;
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

pub(in super::super) fn spline_curve<'a, 'data: 'a, D: Dialect, T: Value + 'a, U: CurveValue + 'a>(
    source: &AnimationResolver<'a, 'data, D>,
    track: &'a native::records::SplineTrack<'data, T, D::Layout>,
    sequence: usize,
    start: u32,
    map: impl Fn(T) -> U + Clone,
) -> Result<Option<(Curve<'a, U>, u16)>, Error>
where
    [T; 3]: Value,
    T::ULE: Send + Sync,
    <[T; 3] as zerovec::ule::AsULE>::ULE: Send + Sync,
{
    let resolved = source
        .resolve_spline_track(track, &source.bundle().key, *b"AFM2", sequence)
        .map_err(failure)?;
    let global = resolved.global_sequence;
    let times = resolved.timestamps;
    if times.is_empty() {
        return Ok(None);
    }
    let ResolvedValues::Splines(values) = resolved.values else {
        unreachable!("camera spline resolver returns tangent triplets")
    };
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
    if first.checked_add(len).is_none_or(|end| end > times.len()) {
        return Err(Error::new(
            "m2.camera.spline",
            "timestamp range exceeds array",
        ));
    }
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
    let key_map = map.clone();
    let keys = materialize(len, move |i| key_map(values.get(first + i).unwrap()[0]));
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
            materialize(len, |i| map(a.get(first + i).unwrap()[1])),
            materialize(len, |i| map(b.get(first + i).unwrap()[2])),
            interpolation,
        )?
    } else {
        Curve::from_samples(timestamps, keys, interpolation)?
    };
    Ok(Some((curve, global)))
}

pub(super) fn animated<V: ModelLayout, U: ChannelValue, T: Value>(
    model: &Graph<'_>,
    value: &Property<U>,
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
    let mut track = encode(model, value, starts, globals, context, &convert)?;
    let default = convert(
        *model.values().get(*value).expect("validated slot"),
        context,
    )?;
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
    value: &Property<U>,
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
    channel: semantic::DynamicTrack<'a>,
) -> Result<(), Error> {
    let (target, clock) = (channel.target(), channel.clock());
    match clock {
        TimeSource::Animation => {
            if let Some(clip) = clip {
                // Alias clips select another sequence's programme. Native track tables
                // may still have a leaf for their index, but it is not an independent
                // programme (the same rule applies to bone channels and events).
                if builder
                    .model()
                    .clip(clip)
                    .is_some_and(|clip| clip.properties.alias.is_some())
                {
                    return Ok(());
                }
                builder.add_track(clip, channel)?
            } else {
                return Err(Error::new(
                    "m2.animation.clock",
                    "sequence curve has no playback clip",
                ));
            }
        }
        TimeSource::Shared(_) => {
            if !builder
                .model()
                .global_tracks()
                .filter(|v| matches!(v.clock(), TimeSource::Shared(_)))
                .any(|v| (v.target(), v.clock()) == (target, clock))
            {
                builder.add_global_track(channel)?
            }
        }
    }
    Ok(())
}
