//! Tracks components for native animation conversion.
use super::*;


pub(in super::super) fn interpolation(
    value: NativeInterpolation,
) -> Result<sm::Interpolation, Error> {
    match value {
        NativeInterpolation::Step => Ok(sm::Interpolation::Step),
        NativeInterpolation::Linear => Ok(sm::Interpolation::Linear),
        NativeInterpolation::Bezier => Ok(sm::Interpolation::Bezier),
        NativeInterpolation::Hermite => Ok(sm::Interpolation::Hermite),
        _ => Err(Error::new(
            "m2.track.interpolation",
            "unknown interpolation code",
        )),
    }
}

pub(in super::super) fn interpolation_code(value: sm::Interpolation) -> NativeInterpolation {
    match value {
        sm::Interpolation::Step => NativeInterpolation::Step,
        sm::Interpolation::Linear => NativeInterpolation::Linear,
        sm::Interpolation::Bezier => NativeInterpolation::Bezier,
        sm::Interpolation::Hermite => NativeInterpolation::Hermite,
    }
}

pub(in super::super) fn import_track<'data, D: Dialect, T: Value>(
    source: &Bundle<'data, D>,
    track: &Track<'data, T>,
    owner: &native::FileKey,
    sequence: usize,
    start: u32,
    target: sm::Target,
    values: impl Fn(T) -> Vec<f32>,
) -> Result<Option<sm::Track>, Error>
where
    [T; 3]: Value,
{
    import_track_try(source, track, owner, sequence, start, target, |value| {
        Ok(values(value))
    })
}

pub(in super::super) fn import_track_try<'data, D: Dialect, T: Value>(
    source: &Bundle<'data, D>,
    track: &Track<'data, T>,
    owner: &native::FileKey,
    sequence: usize,
    start: u32,
    target: sm::Target,
    values: impl Fn(T) -> Result<Vec<f32>, Error>,
) -> Result<Option<sm::Track>, Error>
where
    [T; 3]: Value,
{
    if (0..track.base.timeline.len()).all(|index| match track.base.timeline.leaf(index) {
        Some(Array::Inline(values)) => values.is_empty(),
        Some(Array::External(reference)) => reference.count == 0,
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
    let global = resolved.global_sequence != u16::MAX;
    let duration = if global {
        Some(
            sequence_tables(source, owner)?
                .globals
                .get(usize::from(resolved.global_sequence))
                .ok_or_else(|| Error::new("m2.track.global_sequence", "missing global clock"))?
                as f32
                / 1000.,
        )
    } else {
        None
    };
    let range = if !global {
        track
            .base
            .timeline
            .ranges()
            .get(sequence)
            .map(|[first, last]| first as usize..=last as usize)
    } else {
        None
    };
    let mut keys = Vec::new();
    for (index, time) in resolved.timestamps.iter().enumerate() {
        if range.as_ref().is_some_and(|range| !range.contains(&index)) {
            continue;
        }
        let (value, incoming, outgoing) = match &resolved.values {
            ResolvedValues::Values(v) => (values(v.get(index).unwrap())?, None, None),
            ResolvedValues::Splines(v) => {
                let [value, incoming, outgoing] = v.get(index).unwrap();
                (
                    values(value)?,
                    Some(values(incoming)?),
                    Some(values(outgoing)?),
                )
            }
        };
        keys.push(sm::Key {
            time: if global {
                time
            } else {
                time.checked_sub(start)
                    .ok_or_else(|| Error::new("m2.track.time", "key precedes the sequence start"))?
            } as f32
                / 1000.,
            value,
            incoming,
            outgoing,
        });
    }
    Ok(Some(sm::Track {
        layer: Default::default(),
        target,
        interpolation: interpolation(resolved.interpolation)?,
        global_duration: duration,
        keys,
    }))
}

pub(in super::super) fn export_track<V: ModelLayout, T: Value>(
    model: &sm::Model<'_>,
    target: sm::Target,
    starts: &[u32],
    globals: &mut Vec<u32>,
    context: &mut Context,
    convert: impl Fn(&[f32], &mut Context, &str) -> Result<T, Error>,
) -> Result<Track<'static, T>, Error>
where
    [T; 3]: Value,
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, T>: ValuesFactory<T>,
{
    let tracks: Vec<_> = model
        .clips
        .iter()
        .map(|clip| clip.tracks.iter().find(|track| track.target == target))
        .collect();
    let Some(first) = tracks.iter().flatten().next() else {
        return Ok(Track::default());
    };
    if tracks.iter().flatten().any(|t| {
        t.interpolation != first.interpolation || t.global_duration != first.global_duration
    }) {
        return Err(Error::new(
            "animation.track",
            "M2 interpolation and global clock must be identical across sequences",
        ));
    }
    let global = if let Some(duration) = first.global_duration {
        let duration = milliseconds(duration, context, "animation.global_duration")?;
        let index = globals
            .iter()
            .position(|d| *d == duration)
            .unwrap_or_else(|| {
                globals.push(duration);
                globals.len() - 1
            });
        u16::try_from(index)
            .map_err(|_| Error::new("animation.global_duration", "too many global clocks"))?
    } else {
        u16::MAX
    };
    let mut times = Vec::new();
    let mut values = Vec::new();
    let mut splines = Vec::new();
    let is_spline = matches!(
        first.interpolation,
        sm::Interpolation::Bezier | sm::Interpolation::Hermite
    );
    if global != u16::MAX
        && tracks
            .iter()
            .flatten()
            .any(|track| track.keys != first.keys)
    {
        return Err(Error::new(
            "animation.global_track",
            "shared-clock values differ between clips",
        ));
    }
    for (index, track) in tracks.iter().copied().enumerate() {
        if global != u16::MAX && index > 0 {
            break;
        }
        let mut leaf_times = Vec::new();
        let mut leaf_values = Vec::new();
        let mut leaf_splines = Vec::new();
        if let Some(track) = track {
            for (key_index, key) in track.keys.iter().enumerate() {
                let path = format!("clips[{index}].keys[{key_index}]");
                let time = milliseconds(key.time, context, &path)?;
                let time = if V::VERSION < 264 && global == u16::MAX {
                    time.checked_add(starts[index])
                        .ok_or_else(|| Error::new(&path, "legacy animation clock overflow"))?
                } else {
                    time
                };
                leaf_times.push(time);
                let value = convert(&key.value, context, &path)?;
                if is_spline {
                    let incoming = convert(
                        key.incoming
                            .as_deref()
                            .ok_or_else(|| Error::new(&path, "missing spline incoming tangent"))?,
                        context,
                        &path,
                    )?;
                    let outgoing = convert(
                        key.outgoing
                            .as_deref()
                            .ok_or_else(|| Error::new(&path, "missing spline outgoing tangent"))?,
                        context,
                        &path,
                    )?;
                    leaf_splines.push([value, incoming, outgoing]);
                } else {
                    leaf_values.push(value);
                }
            }
        }
        times.push(leaf_times);
        values.push(leaf_values);
        splines.push(leaf_splines);
    }
    Ok(Track {
        base: native::records::TrackBase {
            interpolation: interpolation_code(first.interpolation),
            global_sequence: global,
            timeline: TimelineFactory::build(times, global != u16::MAX),
        },
        values: ValuesFactory::build(values, splines, is_spline),
    })
}

pub(in super::super) fn initial<T: Value, V: ModelLayout>(track: &Track<'_, T>) -> Option<T>
where
    [T; 3]: Value,
{
    match track.values.inline(0)? {
        native::records::SequenceValues::Values(values) => values.get(0),
        native::records::SequenceValues::Splines(values) => values.get(0).map(|v| v[0]),
    }
}

pub(in super::super) fn ensure_constant<V: ModelLayout, T: Value>(
    track: &mut Track<'static, T>,
    value: T,
    globals: &mut Vec<u32>,
) where
    [T; 3]: Value,
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, T>: ValuesFactory<T>,
{
    if (0..track.base.timeline.len()).any(|index| {
        track
            .base
            .timeline
            .inline(index)
            .is_some_and(|times| !times.is_empty())
    }) {
        return;
    }
    let index = globals
        .iter()
        .position(|value| *value == 1000)
        .unwrap_or_else(|| {
            globals.push(1000);
            globals.len() - 1
        });
    track.base.interpolation = NativeInterpolation::Step;
    track.base.global_sequence = index as u16;
    track.base.timeline = TimelineFactory::build(vec![vec![0]], true);
    track.values = ValuesFactory::build(vec![vec![value]], Vec::new(), false);
}
