//! Exact native texture selectors share the WEM key structures with floating-point tracks.
use super::*;
pub(in crate::model::conversion::mdlx) fn overlay_integer<D: ModelDialect>(
    model: &mut native::Model<'static, D>,
    semantic: &s::Model<'_>,
    ids: &[u32],
    starts: &[u32],
    globals: &mut Vec<u32>,
    context: &mut Context,
) -> Result<(), Error> {
    let mut tracks = BTreeMap::<(u32, u32), Option<Track<'static, u32>>>::new();
    for (clip_index, clip) in semantic.clips.iter().enumerate() {
        for (index, integer) in clip.integer_tracks.iter().enumerate() {
            let path = format!("clips[{clip_index}].integer_tracks[{index}]");
            let track = match integer {
                s::IntegerTrack::Unsigned(track) => track.clone(),
                s::IntegerTrack::Unsigned16(track) => widen(track),
                s::IntegerTrack::Boolean(track) => widen(track),
                s::IntegerTrack::Bytes4(_) | s::IntegerTrack::Unsigned4(_) => {
                    context.unsupported(
                        &path,
                        "MDLX texture selectors cannot represent four-component integer channels",
                    )?;
                    continue;
                }
                s::IntegerTrack::Signed(track) => s::Track {
                    layer: track.layer.clone(),
                    target: track.target,
                    interpolation: track.interpolation,
                    global_duration: track.global_duration,
                    keys: track
                        .keys
                        .iter()
                        .map(|key| {
                            Ok(s::Key {
                                time: key.time,
                                value: key
                                    .value
                                    .iter()
                                    .map(|v| u32::try_from(*v).map_err(|e| failure(&path, e)))
                                    .collect::<Result<_, _>>()?,
                                incoming: key
                                    .incoming
                                    .as_ref()
                                    .map(|v| {
                                        v.iter()
                                            .map(|v| {
                                                u32::try_from(*v).map_err(|e| failure(&path, e))
                                            })
                                            .collect::<Result<_, _>>()
                                    })
                                    .transpose()?,
                                outgoing: key
                                    .outgoing
                                    .as_ref()
                                    .map(|v| {
                                        v.iter()
                                            .map(|v| {
                                                u32::try_from(*v).map_err(|e| failure(&path, e))
                                            })
                                            .collect::<Result<_, _>>()
                                    })
                                    .transpose()?,
                            })
                        })
                        .collect::<Result<_, Error>>()?,
                },
            };
            let s::Target::Property { owner, channel } = track.target else {
                context
                    .unsupported(&path, "integer channel target has no MDLX texture selector")?;
                continue;
            };
            if channel != u32::from_le_bytes(*b"KMTF") && channel != u32::from_le_bytes(*b"KRTX") {
                context.unsupported(&path, "integer channel has no MDLX texture selector")?;
                continue;
            }
            let timing = s::Track {
                layer: track.layer.clone(),
                target: track.target,
                interpolation: track.interpolation,
                global_duration: track.global_duration,
                keys: track
                    .keys
                    .iter()
                    .map(|key| s::Key {
                        time: key.time,
                        value: vec![0.],
                        incoming: key.incoming.as_ref().map(|_| vec![0.]),
                        outgoing: key.outgoing.as_ref().map(|_| vec![0.]),
                    })
                    .collect(),
            };
            let clock = native_track_data::<1>(
                &timing,
                starts[clip_index] as f32 / 1000.,
                globals,
                context,
                &path,
            )?;
            let mut keys = Vec::new();
            for ((frame, _, _), key) in clock.keys.into_iter().zip(track.keys) {
                let value = *key
                    .value
                    .first()
                    .filter(|_| key.value.len() == 1)
                    .ok_or_else(|| Error::new(&path, "integer texture selector needs one value"))?;
                let tangents = match (key.incoming, key.outgoing) {
                    (None, None) => None,
                    (Some(a), Some(b)) if a.len() == 1 && b.len() == 1 => Some((a[0], b[0])),
                    _ => return Err(Error::new(&path, "integer tangents need one value each")),
                };
                keys.push((frame, value, tangents));
            }
            let native = write_track(&TrackData {
                interpolation: clock.interpolation,
                global: clock.global,
                keys,
            })
            .map_err(|e| failure(&path, e))?;
            merge_track(tracks.entry((owner, channel)).or_default(), native, &path)?;
        }
    }
    for ((owner, channel), track) in tracks {
        let Some(track) = track else {
            continue;
        };
        if channel == u32::from_le_bytes(*b"KMTF") {
            if owner & 0x80000000 == 0 {
                return Err(Error::new(
                    "integer.material",
                    "invalid material owner namespace",
                ));
            }
            let material = ((owner >> 16) & 0x7fff) as usize;
            let layer = (owner & 0xffff) as usize;
            let mut materials = model.materials_mut();
            let target = materials
                .nth(material)
                .and_then(|material| material.layers.get_mut(layer))
                .ok_or_else(|| Error::new("integer.material", "missing native layer"))?;
            target.texture_id.set_track(track);
        } else {
            let id = *ids
                .get(owner as usize)
                .ok_or_else(|| Error::new("integer.ribbon", "missing node"))?;
            let available = model
                .ribbon_emitters()
                .map(|record| record.node.object_id)
                .collect::<Vec<_>>();
            let target = model
                .ribbon_emitters_mut()
                .find(|record| record.node.object_id == id)
                .ok_or_else(|| Error::new("integer.ribbon", format!("node {owner} / native ID {id} is not a ribbon emitter; available IDs {available:?}")))?;
            target.texture_slot.set_track(track);
        }
    }
    Ok(())
}

fn widen<T: Copy + Into<u32>>(track: &s::Track<T>) -> s::Track<u32> {
    s::Track {
        layer: track.layer.clone(),
        target: track.target,
        interpolation: track.interpolation,
        global_duration: track.global_duration,
        keys: track
            .keys
            .iter()
            .map(|key| s::Key {
                time: key.time,
                value: key.value.iter().map(|v| (*v).into()).collect(),
                incoming: key
                    .incoming
                    .as_ref()
                    .map(|values| values.iter().map(|v| (*v).into()).collect()),
                outgoing: key
                    .outgoing
                    .as_ref()
                    .map(|values| values.iter().map(|v| (*v).into()).collect()),
            })
            .collect(),
    }
}
