//! Clock mappings for animation.
use super::*;
type KeyData<T> = (i32, T, Option<(T, T)>);
#[derive(Clone, Debug)]
pub(in crate::model::conversion::mdlx) struct TrackData<T> {
    pub(in crate::model::conversion::mdlx) interpolation: Interpolation,
    pub(in crate::model::conversion::mdlx) global: Option<u32>,
    pub(in crate::model::conversion::mdlx) keys: Vec<KeyData<T>>,
}
pub(in crate::model::conversion::mdlx) fn read_track<T: TrackValue>(
    track: &Track<'_, T>,
) -> TrackData<T> {
    let keys = if let Some(keys) = track.step_keys().or_else(|| track.linear_keys()) {
        keys.iter()
            .map(|key| (key.frame, key.value, None))
            .collect()
    } else {
        track
            .hermite_keys()
            .or_else(|| track.bezier_keys())
            .into_iter()
            .flat_map(|keys| keys.iter())
            .map(|key| {
                (
                    key.frame,
                    key.value,
                    Some((key.in_tangent, key.out_tangent)),
                )
            })
            .collect()
    };
    TrackData {
        interpolation: track.interpolation(),
        global: track.global_sequence_id(),
        keys,
    }
}

pub(in crate::model::conversion::mdlx) fn write_track<T: TrackValue>(
    data: &TrackData<T>,
) -> Result<Track<'static, T>, native::ValueError> {
    use native::animation::{TangentKeyframe, ValueKeyframe};
    let values = || {
        data.keys
            .iter()
            .map(|(frame, value, _)| ValueKeyframe {
                frame: *frame,
                value: *value,
            })
            .collect::<Vec<_>>()
    };
    let tangents = || {
        data.keys
            .iter()
            .map(|(frame, value, tangents)| {
                let (incoming, outgoing) = tangents.unwrap_or((*value, *value));
                TangentKeyframe {
                    frame: *frame,
                    value: *value,
                    in_tangent: incoming,
                    out_tangent: outgoing,
                }
            })
            .collect::<Vec<_>>()
    };
    match data.interpolation {
        Interpolation::Step => Track::step(values(), data.global),
        Interpolation::Linear => Track::linear(values(), data.global),
        Interpolation::Hermite => Track::hermite(tangents(), data.global),
        Interpolation::Bezier => Track::bezier(tangents(), data.global),
    }
}

pub(in crate::model::conversion::mdlx) fn semantic_track<const N: usize>(
    data: TrackData<[f32; N]>,
    target: s::Target,
    start: f32,
    globals: &[u32],
) -> s::Track {
    s::Track {
        layer: Default::default(),
        target,
        interpolation: match data.interpolation {
            Interpolation::Step => s::Interpolation::Step,
            Interpolation::Linear => s::Interpolation::Linear,
            Interpolation::Hermite => s::Interpolation::Hermite,
            Interpolation::Bezier => s::Interpolation::Bezier,
        },
        global_duration: data
            .global
            .and_then(|id| globals.get(id as usize))
            .map(|duration| *duration as f32 / 1000.),
        keys: data
            .keys
            .into_iter()
            .map(|(time, value, tangents)| s::Key {
                time: time as f32 / 1000. - if data.global.is_some() { 0. } else { start },
                value: value.to_vec(),
                incoming: tangents.map(|(incoming, _)| incoming.to_vec()),
                outgoing: tangents.map(|(_, outgoing)| outgoing.to_vec()),
            })
            .collect(),
    }
}

pub(in crate::model::conversion::mdlx) fn native_track_data<const N: usize>(
    track: &s::Track,
    start: f32,
    globals: &mut Vec<u32>,
    context: &mut Context,
    path: &str,
) -> Result<TrackData<[f32; N]>, Error> {
    let mut global = None;
    if let Some(duration) = track.global_duration {
        let milliseconds = duration * 1000.;
        if !milliseconds.is_finite() || milliseconds < 0. || milliseconds > u32::MAX as f32 {
            return Err(Error::new(path, "invalid global duration"));
        }
        let value = milliseconds.round() as u32;
        context.approximate(
            path,
            (value as f32 / 1000. - duration).abs(),
            "Global duration rounded to whole milliseconds",
        )?;
        global = Some(
            globals
                .iter()
                .position(|existing| *existing == value)
                .unwrap_or_else(|| {
                    globals.push(value);
                    globals.len() - 1
                }) as u32,
        );
    }
    let mut keys = Vec::new();
    for (index, key) in track.keys.iter().enumerate() {
        let milliseconds = (key.time + if global.is_some() { 0. } else { start }) * 1000.;
        if !milliseconds.is_finite()
            || milliseconds < i32::MIN as f32
            || milliseconds > i32::MAX as f32
        {
            return Err(Error::new(
                format!("{path}.keys[{index}].time"),
                "key time exceeds signed millisecond range",
            ));
        }
        let frame = milliseconds.round() as i32;
        context.approximate(
            format!("{path}.keys[{index}].time"),
            (milliseconds - frame as f32).abs() / 1000.,
            "Key time rounded to whole milliseconds",
        )?;
        let value: [f32; N] = key
            .value
            .as_slice()
            .try_into()
            .map_err(|_| Error::new(path, "track value has the wrong component width"))?;
        let tangents = match (&key.incoming, &key.outgoing) {
            (None, None) => None,
            (Some(incoming), Some(outgoing)) => Some((
                incoming
                    .as_slice()
                    .try_into()
                    .map_err(|_| Error::new(path, "incoming tangent width mismatch"))?,
                outgoing
                    .as_slice()
                    .try_into()
                    .map_err(|_| Error::new(path, "outgoing tangent width mismatch"))?,
            )),
            _ => return Err(Error::new(path, "spline key needs both tangent vectors")),
        };
        if matches!(
            track.interpolation,
            s::Interpolation::Hermite | s::Interpolation::Bezier
        ) && tangents.is_none()
        {
            return Err(Error::new(path, "spline key has no tangent vectors"));
        }
        keys.push((frame, value, tangents));
    }
    Ok(TrackData {
        interpolation: match track.interpolation {
            s::Interpolation::Step => Interpolation::Step,
            s::Interpolation::Linear => Interpolation::Linear,
            s::Interpolation::Hermite => Interpolation::Hermite,
            s::Interpolation::Bezier => Interpolation::Bezier,
        },
        global,
        keys,
    })
}

pub(in crate::model::conversion::mdlx) fn native_track<const N: usize>(
    track: &s::Track,
    start: f32,
    globals: &mut Vec<u32>,
    context: &mut Context,
    path: &str,
) -> Result<Track<'static, [f32; N]>, Error>
where
    [f32; N]: TrackValue,
{
    let data = native_track_data::<N>(track, start, globals, context, path)?;
    write_track(&data).map_err(|error| failure(path, error))
}

pub(in crate::model::conversion::mdlx) fn merge_track<T: TrackValue>(
    slot: &mut Option<Track<'static, T>>,
    incoming: Track<'static, T>,
    path: &str,
) -> Result<(), Error> {
    let Some(previous) = slot.take() else {
        *slot = Some(incoming);
        return Ok(());
    };
    let mut data = read_track(&previous);
    let other = read_track(&incoming);
    if data.interpolation != other.interpolation || data.global != other.global {
        return Err(Error::new(
            path,
            "clip channels disagree on interpolation or shared clock",
        ));
    }
    for key in other.keys {
        if let Some(existing) = data.keys.iter().find(|existing| existing.0 == key.0) {
            if existing != &key {
                return Err(Error::new(
                    path,
                    "clip channels assign different values at the same timeline time",
                ));
            }
        } else {
            data.keys.push(key);
        }
    }
    data.keys.sort_by_key(|key| key.0);
    *slot = Some(write_track(&data).map_err(|error| failure(path, error))?);
    Ok(())
}

pub(in crate::model::conversion::mdlx) fn clip_starts(
    model: &s::Model<'_>,
    context: &mut Context,
) -> Result<Vec<u32>, Error> {
    if [
        ProfileId::Wc3Classic,
        ProfileId::Wc3Reforged,
        ProfileId::Wc3Definitive,
    ]
    .contains(&model.profile)
    {
        return Ok(model.clips.iter().map(|clip| clip.id).collect());
    }
    let mut starts = Vec::new();
    let mut at = 0u32;
    for (index, clip) in model.clips.iter().enumerate() {
        starts.push(at);
        let duration = clip.duration * 1000.;
        if !duration.is_finite() || duration < 0. || duration > u32::MAX as f32 {
            return Err(Error::new(
                format!("clips[{index}].duration"),
                "invalid clip duration",
            ));
        }
        at = at
            .checked_add(duration.round() as u32)
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| Error::new("clips", "timeline exceeds 32-bit millisecond range"))?;
    }
    if !starts.is_empty() {
        context.issue(
            "clips",
            IssueKind::Normalized,
            "Independent clips placed in disjoint millisecond timeline intervals",
        );
    }
    Ok(starts)
}

pub(in crate::model::conversion::mdlx) fn native_globals(
    asset: &Asset,
    options: &ExportOptions,
) -> Result<Vec<u32>, Error> {
    let raw = asset.document.raw();
    let root = asset.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let graph = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(options.model as usize)
        .transpose()?
        .ok_or_else(|| Error::new("model", "missing model"))?;
    for profile in raw
        .resolve::<wr::ProfileMaterialSet>(graph.profile_sets.cast())?
        .iter()
    {
        let profile = profile?;
        if profile.profile == options.profile.unwrap_or(root.default_profile) {
            let meta = metadata(raw, profile.native)?;
            if let Some((_, text)) = meta.get("mdlx.global_sequences") {
                return if text.is_empty() {
                    Ok(Vec::new())
                } else {
                    text.split(',')
                        .map(|value| {
                            value
                                .parse()
                                .map_err(|error| failure("global_sequences", error))
                        })
                        .collect()
                };
            }
        }
    }
    Ok(Vec::new())
}
