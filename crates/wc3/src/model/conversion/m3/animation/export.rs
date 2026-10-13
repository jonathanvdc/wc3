//! Native animation graph construction.
use super::super::hierarchy;
use super::*;
type AnimationRefs = (
    Ref<nr::targets::Sequence>,
    Ref<nr::targets::AnimationGroup>,
    Ref<nr::targets::SubTrackContainer>,
    Ref<nr::targets::AnimationState>,
);
pub(in super::super) fn export_clips<E: Envelope>(
    model: &sem::Model<'_>,
    document: &mut native::Document<'static, E>,
    bones: &mut [nr::Bone<nr::V1>],
    context: &mut Context,
) -> Result<AnimationRefs, Error> {
    let node_map = hierarchy::mapping(model)?;
    let mut sequences = Vec::new();
    let mut groups = Vec::new();
    let mut controllers = Vec::new();
    let mut states = states::export(model, document)?;
    for (index, clip) in model.clips.iter().enumerate() {
        let path = format!("clips[{index}]");
        let duration = quantize(
            clip.duration,
            1000.,
            0.,
            u32::MAX as f32,
            &format!("{path}.duration"),
            context,
        )?;
        let end = (duration * 1000.).round() as u32;
        if let Some(mut native_controllers) = graph::graph(model, index, document, bones, context)?
        {
            let name = append_string(document, &clip.name)?;
            let start = controllers.len();
            let indices: Vec<_> = (start..start + native_controllers.len())
                .map(|value| nr::U32 {
                    value: value as u32,
                })
                .collect();
            groups.push(nr::AnimationGroup {
                name,
                subtrack_indices: append(document, &indices)?.cast(),
            });
            for controller in &mut native_controllers {
                if usize::from(controller.animation_state_index) < states.len()
                    && usize::from(controller.animation_state_index_copy) < states.len()
                {
                    continue;
                }
                let state_index = u16::try_from(states.len())
                    .map_err(|_| Error::new("animation.states", "too many states"))?;
                controller.animation_state_index = state_index;
                controller.animation_state_index_copy = state_index;
                states.push(nr::AnimationState {
                    anim_ids: controller.anim_ids,
                    ..Default::default()
                });
            }
            controllers.extend(native_controllers);
            sequences.push(nr::Sequence::<nr::V2> {
                id: clip.id as i32,
                index: index as i32,
                name,
                start_frame: 0,
                end_frame: end,
                flags: {
                    let mut flags = nr::SequenceFlags::default();
                    flags.set_not_looping(!clip.looping);
                    flags
                },
                bounds: nr::Extent {
                    minimum: model.bounds.minimum,
                    maximum: model.bounds.maximum,
                    radius: model.bounds.sphere_radius,
                },
                ..Default::default()
            });
            continue;
        }
        let mut ids = Vec::new();
        let mut selectors = Vec::new();
        let mut vectors = Vec::new();
        let mut quaternions = Vec::new();
        for (track_index, track) in clip.tracks.iter().enumerate() {
            let track_path = format!("{path}.tracks[{track_index}]");
            if track.global_duration.is_some() {
                context.unsupported(
                    &track_path,
                    "native M3 controller export does not support independent global clocks",
                )?;
                continue;
            }
            let (node, channel, width) = match track.target {
                sem::Target::Translation(n) => (n, 0, 3),
                sem::Target::Rotation(n) => (n, 1, 4),
                sem::Target::Scale(n) => (n, 2, 3),
                sem::Target::NativeProperty { .. } => continue,
                sem::Target::Property { .. } => {
                    context
                        .unsupported(&track_path, "property channel has no common M3 mapping")?;
                    continue;
                }
            };
            let native_node = *node_map
                .get(node as usize)
                .ok_or_else(|| Error::new(&track_path, "track references a missing node"))?;
            let bone = bones
                .get_mut(native_node as usize)
                .ok_or_else(|| Error::new(&track_path, "track references a missing node"))?;
            let animation_id = native_node
                .checked_mul(3)
                .and_then(|v| v.checked_add(channel + 1))
                .ok_or_else(|| Error::new(&track_path, "animation identity overflow"))?;
            let mode = match track.interpolation {
                sem::Interpolation::Step => 0,
                sem::Interpolation::Linear => 1,
                _ => {
                    context.unsupported(
                        &track_path,
                        "M3 spline keys do not expose explicit source tangents",
                    )?;
                    1
                }
            };
            match channel {
                0 => {
                    bone.position.animation_id = animation_id;
                    bone.position.interpolation = nr::Interpolation::from_raw(mode);
                    bone.position.flags = nr::AnimationFlags(
                        bone.position.flags.bits() | nr::AnimationFlags::VALID.bits(),
                    );
                }
                1 => {
                    bone.rotation.animation_id = animation_id;
                    bone.rotation.interpolation = nr::Interpolation::from_raw(mode);
                    bone.rotation.flags = nr::AnimationFlags(
                        bone.rotation.flags.bits() | nr::AnimationFlags::VALID.bits(),
                    );
                }
                _ => {
                    bone.scale.animation_id = animation_id;
                    bone.scale.interpolation = nr::Interpolation::from_raw(mode);
                    bone.scale.flags = nr::AnimationFlags(
                        bone.scale.flags.bits() | nr::AnimationFlags::VALID.bits(),
                    );
                }
            }
            let mut timestamps = Vec::new();
            let mut vec_keys = Vec::new();
            let mut quat_keys = Vec::new();
            let mut previous = None;
            for (key_index, key) in track.keys.iter().enumerate() {
                let p = format!("{track_path}.keys[{key_index}]");
                if key.value.len() != width || key.value.iter().any(|v| !v.is_finite()) {
                    return Err(Error::new(&p, "key value has invalid width or components"));
                }
                let time = quantize(
                    key.time,
                    1000.,
                    0.,
                    i32::MAX as f32,
                    &format!("{p}.time"),
                    context,
                )?;
                let time = (time * 1000.).round() as i32;
                if previous.is_some_and(|prev| time <= prev) {
                    return Err(Error::new(
                        &p,
                        "keys must remain strictly ordered after millisecond encoding",
                    ));
                }
                previous = Some(time);
                timestamps.push(nr::I32 { value: time });
                if width == 3 {
                    vec_keys.push(nr::Vector3 {
                        value: key.value.as_slice().try_into().unwrap(),
                    });
                } else {
                    quat_keys.push(nr::Quaternion {
                        value: key.value.as_slice().try_into().unwrap(),
                    });
                }
            }
            let time_ref = append(document, &timestamps)?.cast();
            ids.push(nr::U32 {
                value: animation_id,
            });
            if width == 3 {
                let key_ref = append(document, &vec_keys)?.cast();
                selectors.push(nr::U32 {
                    value: 0x20000 | vectors.len() as u32,
                });
                vectors.push(nr::AnimationBlockVector3 {
                    timestamps: time_ref,
                    flags: nr::AnimationBlockFlags(0),
                    end_frame: end,
                    keys: key_ref,
                });
            } else {
                let key_ref = append(document, &quat_keys)?.cast();
                selectors.push(nr::U32 {
                    value: 0x30000 | quaternions.len() as u32,
                });
                quaternions.push(nr::AnimationBlockQuaternion {
                    timestamps: time_ref,
                    flags: nr::AnimationBlockFlags(0),
                    end_frame: end,
                    keys: key_ref,
                });
            }
        }
        let native_blocks = export_native(
            model,
            index,
            document,
            NativeOutput {
                ids: &mut ids,
                selectors: &mut selectors,
            },
            context,
            NativeExportOptions {
                vector_base: vectors.len(),
                quaternion_base: quaternions.len(),
                ..Default::default()
            },
        )?;
        let mut pairs = ids.into_iter().zip(selectors).collect::<Vec<_>>();
        pairs.sort_by_key(|(id, _)| id.value);
        let (ids, selectors): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
        let ids_ref = append(document, &ids)?.cast();
        let selectors_ref = append(document, &selectors)?.cast();
        let vectors_ref = append(document, &vectors)?.cast();
        let quaternions_ref = append(document, &quaternions)?.cast();
        let name = append_string(document, &clip.name)?;
        let indices = append(
            document,
            &[nr::U32 {
                value: index as u32,
            }],
        )?
        .cast();
        groups.push(nr::AnimationGroup {
            name,
            subtrack_indices: indices,
        });
        states.push(nr::AnimationState {
            anim_ids: ids_ref,
            ..Default::default()
        });
        let mut controller = nr::SubTrackContainer::<nr::V4> {
            name,
            anim_ids: ids_ref,
            anim_refs: selectors_ref,
            animation_state_index: index as u16,
            animation_state_index_copy: index as u16,
            sd3v: vectors_ref,
            sd4q: quaternions_ref,
            ..Default::default()
        };
        native_blocks.install(document, &mut controller)?;
        controllers.push(controller);
        sequences.push(nr::Sequence::<nr::V2> {
            id: clip.id as i32,
            index: index as i32,
            name,
            start_frame: 0,
            end_frame: end,
            flags: {
                let mut flags = nr::SequenceFlags::default();
                flags.set_not_looping(!clip.looping);
                flags
            },
            bounds: nr::Extent {
                minimum: model.bounds.minimum,
                maximum: model.bounds.maximum,
                radius: model.bounds.sphere_radius,
            },
            ..Default::default()
        });
    }
    let versions = sequences::decorate(model, document, &mut sequences)?;
    Ok((
        sequences::write(document, &sequences, &versions)?,
        append(document, &groups)?.cast(),
        append(document, &controllers)?.cast(),
        append(document, &states)?.cast(),
    ))
}
