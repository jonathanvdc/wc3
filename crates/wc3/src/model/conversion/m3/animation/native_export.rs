//! Exact integer and native-control keys rebuilt from active typed WEM curves.
use super::super::effects::metadata as native_metadata;
use super::super::hierarchy;
use super::*;
pub(super) fn export_native<E: Envelope>(
    model: &sem::Model<'_>,
    clip_index: usize,
    doc: &mut native::Document<'static, E>,
    target: NativeOutput<'_>,
    context: &mut Context,
    options: NativeExportOptions<'_>,
) -> Result<NativeBlocks, Error> {
    let NativeOutput { ids, selectors } = target;
    let NativeExportOptions {
        vector_base,
        quaternion_base,
        selected_container,
        mut bones,
    } = options;
    let mut output = NativeBlocks::default();
    let Some(native) = &model.native else {
        return Ok(output);
    };
    let control_maximum = properties(&native.document)?
        .last_key_value()
        .map_or(0, |(&id, _)| id);
    let raw = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let model_record = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let channels: Vec<_> = raw
        .resolve::<wr::AnimChannel>(model_record.anim_channels.channels.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let clips: Vec<_> = raw
        .resolve::<wr::Clip<wr::V7>>(root.clips.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let Some(clip) = clips
        .iter()
        .filter(|clip| clip.model == native.model)
        .nth(clip_index)
    else {
        return Ok(output);
    };
    for (id, block) in if selected_container.is_none_or(|index| index == 0) {
        events::export_events(model, clip, doc, context)?
    } else {
        Vec::new()
    } {
        ids.push(nr::U32 { value: id });
        selectors.push(nr::U32 {
            value: output.events.len() as u32,
        });
        output.events.push(block);
    }
    for (container_index, container) in raw
        .resolve::<wr::SubTrackContainer>(clip.containers.cast())?
        .iter()
        .enumerate()
    {
        if selected_container.is_some_and(|selected| selected != container_index) {
            continue;
        }
        let container = container?;
        let metadata = native_metadata::read(raw, &container.native)?;
        for track in raw
            .resolve::<wr::SubTrack<wr::V2>>(container.sub_tracks.cast())?
            .iter()
        {
            let track = track?;
            let channel = channels
                .iter()
                .find(|channel| channel.id == track.channel)
                .ok_or_else(|| Error::new("wem.animation.channel", "missing declaration"))?;
            let is_common = matches!(
                channel.target.channel,
                wr::Channel::TRANSLATION | wr::Channel::ROTATION | wr::Channel::SCALE
            );
            if channel.target.channel != wr::Channel(255)
                && !(is_common && selected_container.is_some())
            {
                continue;
            }
            let id = if is_common {
                let mapping = hierarchy::mapping(model)?;
                let index = *mapping
                    .get(channel.target.node as usize)
                    .ok_or_else(|| Error::new("animation.node", "missing mapping"))?;
                let existing = bones
                    .as_deref()
                    .and_then(|rows| rows.get(index as usize))
                    .map(|bone| match channel.target.channel {
                        wr::Channel::TRANSLATION => bone.position.animation_id,
                        wr::Channel::ROTATION => bone.rotation.animation_id,
                        _ => bone.scale.animation_id,
                    })
                    .unwrap_or(0);
                if existing != 0 {
                    existing
                } else {
                    let highest = channels
                        .iter()
                        .map(|channel| channel.id.max(channel.target.sub))
                        .chain(bones.as_deref().into_iter().flatten().flat_map(|bone| {
                            [
                                bone.position.animation_id,
                                bone.rotation.animation_id,
                                bone.scale.animation_id,
                                bone.batching.animation_id,
                            ]
                        }))
                        .max()
                        .unwrap_or(0);
                    highest.max(control_maximum).checked_add(1).ok_or_else(|| {
                        Error::new("animation.id", "native animation identity overflow")
                    })?
                }
            } else {
                channel.target.sub
            };
            if let Some(bones) = bones.as_deref_mut() {
                let mapping = hierarchy::mapping(model)?;
                if is_common {
                    let index = *mapping
                        .get(channel.target.node as usize)
                        .ok_or_else(|| Error::new("animation.node", "missing mapping"))?;
                    let bone = bones
                        .get_mut(index as usize)
                        .ok_or_else(|| Error::new("animation.node", "missing bone"))?;
                    let mode = if track.interp == wr::Interpolation::STEP {
                        0
                    } else {
                        1
                    };
                    match channel.target.channel {
                        wr::Channel::TRANSLATION => {
                            bone.position.animation_id = id;
                            bone.position.interpolation = mode;
                            bone.position.flags |= 1;
                        }
                        wr::Channel::ROTATION => {
                            bone.rotation.animation_id = id;
                            bone.rotation.interpolation = mode;
                            bone.rotation.flags |= 1;
                        }
                        _ => {
                            bone.scale.animation_id = id;
                            bone.scale.interpolation = mode;
                            bone.scale.flags |= 1;
                        }
                    }
                } else if channel.target.node < model_record.nodes.nodes.entries {
                    let node = raw
                        .resolve::<wr::Node<wr::V19>>(model_record.nodes.nodes.cast())?
                        .get(channel.target.node as usize)
                        .unwrap()?;
                    let values = native_metadata::read(raw, &node.native)?;
                    if values.get("m3.bone.batching.animation_id") == Some(&i64::from(id)) {
                        let index = mapping[channel.target.node as usize];
                        bones[index as usize].batching.animation_id = id;
                        bones[index as usize].batching.interpolation =
                            if track.interp == wr::Interpolation::STEP {
                                0
                            } else {
                                1
                            };
                        bones[index as usize].batching.flags |= 1;
                    }
                }
            }

            if ids.iter().any(|existing| existing.value == id) {
                return Err(Error::new(
                    "m3.animation.id",
                    "duplicate animation identity",
                ));
            }
            if channel.target.node < model_record.nodes.nodes.entries {
                let node = raw
                    .resolve::<wr::Node<wr::V19>>(model_record.nodes.nodes.cast())?
                    .get(channel.target.node as usize)
                    .unwrap()?;
                let fields = native_metadata::read(raw, &node.native)?;
                if let Some(&original) = fields.get("m3.bounds.source_id") {
                    if original as u32 != id {
                        continue;
                    }
                    let index = output.extents.len();
                    output.extents.push(bounds::export(
                        model, clip_index, clip, &node, doc, context,
                    )?);
                    ids.push(nr::U32 { value: id });
                    selectors.push(nr::U32 {
                        value: (12 << 16) | index as u32,
                    });
                    continue;
                }
            }
            let default_family = match channel.value_type {
                wr::AttrType::F32 => 5,
                wr::AttrType::F32X2 => 1,
                wr::AttrType::F32X3 => 2,
                wr::AttrType::F32X4 | wr::AttrType::QUAT => 3,
                wr::AttrType::U8X4 => 4,
                wr::AttrType::U16 => 8,
                wr::AttrType::I32 => 9,
                wr::AttrType::U32 => 10,
                _ => {
                    return Err(Error::new(
                        "wem.animation.type",
                        "unsupported native channel element type",
                    ));
                }
            };
            let family = metadata
                .get(&format!("m3.animation.{id}.family"))
                .copied()
                .unwrap_or(default_family) as u32;
            let origin = clock::start(raw, clip)?;
            let times = raw
                .resolve::<wr::Real>(track.times.cast())?
                .iter()
                .map(|time| clock::frame(time?.value, origin, context, "animation.time"))
                .collect::<Result<Vec<_>, Error>>()?;
            let data = raw
                .resolve::<wr::Byte>(track.values.cast())?
                .iter()
                .map(|row| row.map(|row| row.value))
                .collect::<Result<Vec<_>, _>>()?;
            let flags = metadata
                .get(&format!("m3.animation.{id}.flags"))
                .copied()
                .unwrap_or(0) as u32;
            let end_frame = metadata
                .get(&format!("m3.animation.{id}.end"))
                .copied()
                .unwrap_or((clip.duration * 1000.).round() as i64)
                as u32;
            if track.interp != wr::Interpolation::STEP && track.interp != wr::Interpolation::LINEAR
            {
                context.unsupported(
                    "animation.interpolation",
                    "M3 native control keys have no explicit tangent payload",
                )?;
            }
            let selector = output.add(
                doc,
                BlockInput {
                    family,
                    kind: channel.value_type,
                    times,
                    data,
                    flags,
                    end_frame,
                    vector_base,
                    quaternion_base,
                },
            )?;
            ids.push(nr::U32 { value: id });
            selectors.push(nr::U32 { value: selector });
        }
    }
    Ok(output)
}
