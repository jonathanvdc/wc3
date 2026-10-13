//! Native controllers installed as editable, exactly typed WEM channel arrays.
use super::*;
pub(in super::super) fn import_native_clips<E: Envelope>(
    source: &native::Model<'_, E>,
    target: wem::Document<'static>,
    context: &mut Context,
) -> Result<wem::Document<'static>, Error> {
    let document = source.document();
    let root = source.root_record().map_err(read_error)?;
    let mut properties = properties(&target)?;
    let mut batching_properties = Vec::new();
    macro_rules! bones {
        ($v:ty) => {
            for (index, row) in document
                .resolve::<nr::Bone<$v>>(root.bones().cast())
                .map_err(read_error)?
                .iter()
                .enumerate()
            {
                let bone = row.map_err(read_error)?;
                for (id, channel, mode) in [
                    (
                        bone.position.animation_id,
                        wr::Channel::Translation,
                        bone.position.interpolation,
                    ),
                    (
                        bone.rotation.animation_id,
                        wr::Channel::Rotation,
                        bone.rotation.interpolation,
                    ),
                    (
                        bone.scale.animation_id,
                        wr::Channel::Scale,
                        bone.scale.interpolation,
                    ),
                ] {
                    if id != 0 {
                        properties.insert(
                            id,
                            Property {
                                target: wr::TrackTarget {
                                    kind: wr::TrackTargetKind::Node,
                                    node: index as u32,
                                    channel,
                                    ..Default::default()
                                },
                                mode: mode.raw(),
                            },
                        );
                    }
                }
                if bone.batching.animation_id != 0 {
                    batching_properties.push((index, bone.batching.animation_id));
                    properties.insert(
                        bone.batching.animation_id,
                        Property {
                            target: wr::TrackTarget {
                                kind: wr::TrackTargetKind::Node,
                                node: index as u32,
                                channel: wr::Channel::from_raw(255),
                                sub: bone.batching.animation_id,
                                ..Default::default()
                            },
                            mode: (bone.batching.interpolation).raw(),
                        },
                    );
                }
            }
        };
    }
    if root.bones().entries != 0 {
        match revision(document, root.bones()).map_err(read_error)? {
            0 => bones!(nr::V0),
            1 => bones!(nr::V1),
            _ => return Err(Error::new("m3.bones", "unsupported bone revision")),
        }
    }
    let (container_ref, group_ref) =
        native::visit_m3_root!(root, |r| (r.sub_track_collections, r.animation_groups));
    let mut containers = Vec::new();
    macro_rules! containers {
        ($v:ty) => {
            for row in document
                .resolve::<nr::SubTrackContainer<$v>>(container_ref.cast())
                .map_err(read_error)?
                .iter()
            {
                let c = row.map_err(read_error)?;
                containers.push(nr::SubTrackContainer::<nr::V4> {
                    name: c.name,
                    runs_concurrent: c.runs_concurrent,
                    anim_priority: c.anim_priority,
                    animation_state_index: c.animation_state_index,
                    animation_state_index_copy: c.animation_state_index_copy,
                    anim_ids: c.anim_ids,
                    anim_refs: c.anim_refs,
                    unknown: c.unknown,
                    sdev: c.sdev,
                    sd2v: c.sd2v,
                    sd3v: c.sd3v,
                    sd4q: c.sd4q,
                    sdcc: c.sdcc,
                    sdr3: c.sdr3,
                    sdu8: c.sdu8,
                    sds6: c.sds6,
                    sdu6: c.sdu6,
                    sds3: c.sds3,
                    sdu3: c.sdu3,
                    sdfg: c.sdfg,
                    sdmb: c.sdmb,
                });
            }
        };
    }
    if container_ref.entries != 0 {
        match revision(document, container_ref).map_err(read_error)? {
            0 => containers!(nr::V0),
            1 => containers!(nr::V1),
            2 => containers!(nr::V2),
            3 => containers!(nr::V3),
            4 => containers!(nr::V4),
            _ => {
                return Err(Error::new(
                    "m3.controllers",
                    "unsupported controller revision",
                ));
            }
        }
    }
    let groups = document
        .resolve::<nr::AnimationGroup>(group_ref.cast())
        .map_err(read_error)?
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(read_error)?;
    let sequences = sequences::read(document, root.sequences())?;

    let mut wemroot = target.root::<wr::DocumentRecord<wr::V1>>()?;
    let mut model = target
        .raw()
        .resolve::<wr::ModelRecord<wr::V9>>(wemroot.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let mut channels: Vec<_> = target
        .raw()
        .resolve::<wr::AnimChannel>(model.anim_channels.channels.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let mut clips: Vec<_> = target
        .raw()
        .resolve::<wr::Clip<wr::V7>>(wemroot.clips.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let source_max = containers
        .iter()
        .map(|controller| {
            document
                .resolve::<nr::U32>(controller.anim_ids.cast())
                .map_err(read_error)?
                .iter()
                .map(|row| row.map(|row| row.value))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)
        })
        .collect::<Result<Vec<_>, Error>>()?
        .into_iter()
        .flatten()
        .max()
        .unwrap_or(0);
    let mut fresh = source_max
        .max(channels.iter().map(|channel| channel.id).max().unwrap_or(0))
        .checked_add(1)
        .ok_or_else(|| Error::new("animation.id", "identity overflow"))?;
    let initial_clip_count = clips.len();
    let root_id = target.root_id();
    let mut raw = target.into_raw();
    let state_ref = native::visit_m3_root!(root, |record| record.animation_states);
    let state_base = states::attach(document, state_ref, &mut raw, &mut model)?;

    if !batching_properties.is_empty() {
        let mut nodes: Vec<_> = raw
            .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
            .iter()
            .collect::<Result<_, _>>()?;
        for (index, id) in batching_properties {
            let mut entries: Vec<_> = raw
                .resolve::<wr::NativeEntry<wr::V2>>(nodes[index].native.entries.cast())?
                .iter()
                .collect::<Result<_, _>>()?;
            entries.push(wr::NativeEntry::<wr::V2> {
                name: text(&mut raw, "m3.bone.batching.animation_id")?,
                value: i64::from(id),
                ..Default::default()
            });
            nodes[index].native.entries = put(&mut raw, &entries)?.cast();
        }
        model.nodes.nodes = put(&mut raw, &nodes)?.cast();
    }

    for (index, source_sequence) in sequences.into_iter().enumerate() {
        let sequence = &source_sequence.value;
        let (name, start, end, flags) = (
            sequence.name,
            sequence.start_frame,
            sequence.end_frame,
            sequence.flags,
        );
        let duration = end
            .checked_sub(start)
            .ok_or_else(|| Error::new("m3.sequence.duration", "sequence ends before start"))?;
        let mut output_containers = Vec::new();
        let mut clip_events = Vec::new();
        let selected = if let Some(group) = groups.get(index) {
            document
                .resolve::<nr::U32>(group.subtrack_indices.cast())
                .map_err(read_error)?
                .iter()
                .map(|row| row.map(|row| row.value as usize))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?
        } else {
            Vec::new()
        };
        for selected in selected {
            let controller = containers
                .get(selected)
                .ok_or_else(|| Error::new("m3.controllers", "missing controller"))?;
            let ids = document
                .resolve::<nr::U32>(controller.anim_ids.cast())
                .map_err(read_error)?;
            let selectors = document
                .resolve::<nr::U32>(controller.anim_refs.cast())
                .map_err(read_error)?;
            if ids.len() != selectors.len() {
                return Err(Error::new("m3.controllers", "ID/selector count differs"));
            }
            let mut tracks = Vec::new();
            let mut metadata = Vec::new();
            for (id, selector) in ids.iter().zip(selectors.iter()) {
                let id = id.map_err(read_error)?.value;
                let selector = selector.map_err(read_error)?.value;
                if selector >> 16 == 0 {
                    events::import_events(
                        ImportBlock {
                            source: document,
                            controller,
                            selector,
                            id,
                            start,
                            clip: initial_clip_count + index,
                        },
                        &mut raw,
                        &mut model,
                        &mut clip_events,
                    )?;
                    continue;
                }
                if selector >> 16 == 12 {
                    bounds::import(
                        ImportBlock {
                            source: document,
                            controller,
                            selector,
                            id,
                            start,
                            clip: initial_clip_count + index,
                        },
                        BoundsTarget {
                            raw: &mut raw,
                            model: &mut model,
                            channels: &mut channels,
                            tracks: &mut tracks,
                            fresh: &mut fresh,
                        },
                    )?;
                    continue;
                }
                let Some(property) = properties.get(&id).cloned() else {
                    context.unsupported(
                        format!("m3.animations[{id}]"),
                        "channel has no native control identity",
                    )?;
                    continue;
                };
                let Some(data) = read_keys(document, controller, selector, start, context)? else {
                    continue;
                };
                let interpolation = match property.mode {
                    0 => wr::Interpolation::Step,
                    1 => wr::Interpolation::Linear,
                    _ => {
                        context.unsupported(
                            "m3.animation.interpolation",
                            "spline convention has no explicit tangent payload",
                        )?;
                        wr::Interpolation::Linear
                    }
                };
                let channel = wr::AnimChannel {
                    id,
                    target: property.target,
                    value_type: data.kind,
                    init_value: bytes(
                        &mut raw,
                        &data.values[..data
                            .values
                            .len()
                            .checked_div(data.times.len().max(1))
                            .unwrap_or(0)],
                    )?,
                };
                if let Some(existing) = channels.iter().find(|existing| existing.id == id) {
                    if existing.target != channel.target
                        || existing.value_type != channel.value_type
                    {
                        return Err(Error::new(
                            "m3.animation.channel",
                            "shared identity has incompatible target/type",
                        ));
                    }
                } else {
                    channels.push(channel);
                }
                tracks.push(wr::SubTrack::<wr::V2> {
                    channel: id,
                    interp: interpolation,
                    times: put(&mut raw, &data.times)?.cast(),
                    values: bytes(&mut raw, &data.values)?,
                    ..Default::default()
                });
                metadata.extend([
                    (format!("m3.animation.{id}.family"), i64::from(data.family)),
                    (format!("m3.animation.{id}.flags"), i64::from(data.flags)),
                    (format!("m3.animation.{id}.end"), i64::from(data.end)),
                ]);
            }
            metadata.push((
                "m3.controller.runs_concurrent".to_owned(),
                i64::from(controller.runs_concurrent),
            ));
            metadata.push((
                "m3.controller.state_index".to_owned(),
                i64::from(controller.animation_state_index) + state_base as i64,
            ));
            metadata.push((
                "m3.controller.state_index_copy".to_owned(),
                i64::from(controller.animation_state_index_copy) + state_base as i64,
            ));
            metadata.push((
                "m3.controller.unknown".to_owned(),
                i64::from(controller.unknown),
            ));
            let mut entries = Vec::new();
            for (name, value) in metadata {
                entries.push(wr::NativeEntry::<wr::V2> {
                    name: text(&mut raw, &name)?,
                    value,
                    ..Default::default()
                });
            }
            output_containers.push(wr::SubTrackContainer {
                name: text(
                    &mut raw,
                    &string(document, controller.name).map_err(read_error)?,
                )?,
                priority: i32::from(controller.anim_priority),
                concurrent: wem::Bool::from(controller.runs_concurrent != 0),
                sub_tracks: put(&mut raw, &tracks)?.cast(),
                native: wr::NativeBag {
                    entries: put(&mut raw, &entries)?.cast(),
                },
            });
        }
        clips.push(wr::Clip::<wr::V7> {
            events: put(&mut raw, &clip_events)?.cast(),
            native: sequences::metadata(&source_sequence, document, &mut raw)?,
            bounds: wr::Extent {
                minimum: sequence.bounds.minimum,
                maximum: sequence.bounds.maximum,
                sphere_radius: sequence.bounds.radius,
            },
            name: text(&mut raw, &string(document, name).map_err(read_error)?)?,
            duration: duration as f32 / 1000.,
            flags: wr::ClipFlags::default(),
            looping: (!flags.not_looping()).into(),
            containers: put(&mut raw, &output_containers)?.cast(),
            ..Default::default()
        });
    }
    model.anim_channels.channels = put(&mut raw, &channels)?.cast();
    wemroot.clips = put(&mut raw, &clips)?.cast();
    let id = raw
        .section_id(wemroot.models.index)
        .ok_or_else(|| Error::new("wem.model", "missing model table"))?;
    raw.replace(id, &[model])?;
    raw.replace(root_id, &[wemroot])?;
    Ok(wem::Document::from_raw(raw)?)
}
