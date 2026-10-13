//! Typed M2 output planning and native graph construction.
use super::*;
use crate::model::conversion::animation::crop;

/// Plans a typed M2 bundle, checking index limits, curves and resource identities.
pub fn plan<D: ExportDialect>(
    asset: &Asset,
    options: &ExportOptions,
) -> Result<Plan<Bundle<'static, D>>, Error>
where
    D::Layout: ExportLayout + ParticleConversion + CameraConversion,
    <D::Layout as native::HeaderLayout>::Flipbooks<'static>: FlipbookField<'static, D::Layout>,
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
    <D::Layout as native::records::BoneLayout>::RotationValue: RotationComponents,
    <D::Layout as native::records::BoneLayout>::NameCrc: NameCrcField,
    <D::Layout as native::records::SequenceLayout>::BlendTime: BlendField,
    <D::Layout as native::records::TrackLayout>::Timeline<'static>: TimelineFactory,
    <D::Layout as native::records::TrackLayout>::TrackValues<'static, [f32; 3]>:
        ValuesFactory<[f32; 3]>,
    <D::Layout as native::records::TrackLayout>::TrackValues<'static, [f32; 4]>:
        ValuesFactory<[f32; 4]>,
    <D::Layout as native::records::TrackLayout>::TrackValues<'static, f32>: ValuesFactory<f32>,
    <D::Layout as native::records::TrackLayout>::TrackValues<'static, u8>: ValuesFactory<u8>,
    <D::Layout as native::records::TrackLayout>::TrackValues<'static, u32>: ValuesFactory<u32>,
    <D::Layout as native::records::TrackLayout>::TrackValues<'static, i16>: ValuesFactory<i16>,
    <D::Layout as native::records::TrackLayout>::TrackValues<'static, u16>: ValuesFactory<u16>,
    <D::Layout as native::records::TrackLayout>::SplineValues<'static, f32>: SplineFactory<f32>,
    <D::Layout as native::records::TrackLayout>::SplineValues<'static, [f32; 3]>:
        SplineFactory<[f32; 3]>,
    <D::Layout as native::records::TrackLayout>::TrackValues<
        'static,
        <D::Layout as native::records::BoneLayout>::RotationValue,
    >: ValuesFactory<<D::Layout as native::records::BoneLayout>::RotationValue>,
{
    let mut model = asset.model(options)?;
    let mut context = Context::export(options);
    if model.unit_scale != 1. {
        context.unsupported("model.unit_scale","M2 has implicit physical units; a non-unit WEM scale requires transforming geometry, animation, and effect distances")?;
    }
    if model.space != wr::CoordSpace::Blizzard {
        context.issue("model.space",IssueKind::Normalized,"M2 uses Blizzard coordinates; numeric coordinates retained without a basis transformation");
    }

    numeric_tracks(&mut model, &mut context)?;
    rest_tracks(&mut model);
    for (clip_index, clip) in model.clips.iter_mut().enumerate() {
        let mut seen = Vec::new();
        let mut kept = Vec::new();
        for (track_index, track) in clip.tracks.drain(..).enumerate() {
            if seen.contains(&track.target) {
                context.unsupported(
                    format!("clips[{clip_index}].tracks[{track_index}]"),
                    "M2 has one curve per property and cannot represent layered competing controls",
                )?;
            } else {
                seen.push(track.target);
                kept.push(track);
            }
        }
        clip.tracks = kept;
        for (track_index, track) in clip.tracks.iter_mut().enumerate() {
            if matches!(track.target, sm::Target::NativeProperty { .. }) {
                context.unsupported(
                    format!("clips[{clip_index}].tracks[{track_index}]"),
                    "the native animation identity belongs to a different profile",
                )?;
                continue;
            }
            let end = track.global_duration.unwrap_or(clip.duration);
            if track.keys.iter().any(|key| key.time < 0. || key.time > end) {
                *track = crop(track, 0., end)?;
                context.issue(
                    format!("clips[{clip_index}].tracks[{track_index}]"),
                    IssueKind::Normalized,
                    "restricted the curve to the native clock interval, retaining spline controls",
                );
            }
        }
    }
    let scene_nodes = native_nodes(&model)?;
    for (clip_index, clip) in model.clips.iter().enumerate() {
        for (track_index, track) in clip.tracks.iter().enumerate() {
            let sm::Target::Property { owner, channel } = track.target else {
                continue;
            };
            let known = if matches!(channel, 2000 | 2001 | 2100 | 2200 | 2201 | 2202 | 2300) {
                true
            } else {
                scene_nodes
                    .get(owner as usize)
                    .is_some_and(|node| match &node.payload {
                        wr::NodePayload::Attachment(_) => channel == 3,
                        wr::NodePayload::Light(_) => {
                            matches!(channel, 3 | 4 | 6 | 7 | 8 | 200 | 201)
                        }
                        wr::NodePayload::M2ParticleEmitter(_) => (1000..=1010).contains(&channel),
                        wr::NodePayload::Wc3RibbonEmitter(_) => (1100..=1105).contains(&channel),
                        wr::NodePayload::Camera(_) => (1200..=1203).contains(&channel),
                        wr::NodePayload::Event(_) => channel == 1400,
                        _ => false,
                    })
            };
            if !known {
                context.unsupported(
                    format!("clips[{clip_index}].tracks[{track_index}]"),
                    "animated property has no corresponding M2 control field",
                )?;
            }
        }
    }
    for payload in asset
        .opaque
        .iter()
        .filter(|payload| payload.format == Format::M2)
    {
        match options.unknown {
            UnknownPolicy::Reject => {
                return Err(Error::new(
                    format!("m2.chunks.{:?}", payload.tag),
                    "inactive opaque payload cannot be safely inserted into rebuilt native output",
                ));
            }
            UnknownPolicy::Archive => context.issue(
                format!("m2.chunks.{:?}", payload.tag),
                IssueKind::Archived,
                "opaque payload remains attached to the WEM conversion asset",
            ),
            UnknownPolicy::Drop => context.issue(
                format!("m2.chunks.{:?}", payload.tag),
                IssueKind::Dropped,
                "opaque payload omitted from rebuilt native output",
            ),
        }
    }

    let mut bundle = Bundle::from_model(native::Model::<D>::new());
    let core = bundle.model.core_mut();
    let output_key = restore_native_metadata(&model, core)?;
    core.name = ZeroVec::alloc_from_slice(&model.name.bytes().chain([0]).collect::<Vec<_>>());
    core.bounding = native_extent(model.bounds.clone());
    core.collision = core.bounding;
    for texture in &model.textures {
        core.textures.push(native::records::Texture {
            kind: TextureSource::from_raw(texture.replaceable),
            flags: TextureFlags::from_bits_retain(texture.flags.bits()),
            filename: ZeroVec::alloc_from_slice(
                &texture.path.bytes().chain([0]).collect::<Vec<_>>(),
            ),
        });
    }
    let mut start = 0u32;
    let mut starts = Vec::new();
    for (index, clip) in model.clips.iter().enumerate() {
        let duration = milliseconds(
            clip.duration,
            &mut context,
            &format!("clips[{index}].duration"),
        )?;
        starts.push(start);
        let data = clip_metadata(&model, index)?;
        if !clip.flags.is_empty() {
            context.unsupported(
                format!("clips[{index}].flags"),
                "M2 has no native equivalent to WEM clip playback flags",
            )?;
        }
        if !clip.looping {
            context.unsupported(
                format!("clips[{index}].looping"),
                "M2 has no verified native control for generic nonlooping playback",
            )?;
        }
        let mut flags = SequenceFlags::from_bits_retain(
            data.get("m2.sequence.flags").copied().unwrap_or(0) as u32,
        );
        flags.set_alias(false);
        flags.set_load_low_priority(false);
        flags.set_primary_bone_sequence(true);
        flags.set_stored_in_model(false);
        core.sequences.push(native::records::Sequence {
            id: u16::try_from(clip.id)
                .map_err(|_| Error::new("clips.id", "animation ID exceeds u16"))?,
            timing: SequenceClock::from_window(start, duration),
            flags,
            variation_index: data.get("m2.variation").copied().unwrap_or(0) as u16,
            move_speed: f32::from_bits(data.get("m2.move_speed").copied().unwrap_or(0) as u32),
            frequency: data.get("m2.frequency").copied().unwrap_or(1) as i16,
            padding: data.get("m2.padding").copied().unwrap_or(0) as u16,
            replay_min: data.get("m2.replay_min").copied().unwrap_or(0) as u32,
            replay_max: data.get("m2.replay_max").copied().unwrap_or(0) as u32,
            blend_time: BlendField::make(
                [
                    data.get("m2.blend_in").copied().unwrap_or(0) as u32,
                    data.get("m2.blend_out").copied().unwrap_or(0) as u32,
                ],
                &mut context,
            )?,
            variation_next: data.get("m2.variation_next").copied().unwrap_or(-1) as i16,
            bounding: core.bounding,
            ..Default::default()
        });
        start = start
            .checked_add(duration)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| Error::new("clips", "legacy clock overflow"))?;
    }
    let mut globals = Vec::new();
    for (index, node) in model.nodes.iter().enumerate() {
        if let Some(native) = scene_nodes.get(index) {
            if metadata(
                model.native.as_ref().unwrap().document.raw(),
                &native.native,
            )?
            .get("m2.physics.host")
                == Some(&1)
            {
                continue;
            }
        }
        if !node.bone
            && !scene_nodes.get(index).is_some_and(|node| {
                matches!(
                    node.payload,
                    wr::NodePayload::Attachment(_)
                        | wr::NodePayload::Light(_)
                        | wr::NodePayload::M2ParticleEmitter(_)
                        | wr::NodePayload::Wc3RibbonEmitter(_)
                        | wr::NodePayload::Camera(_)
                        | wr::NodePayload::Event(_)
                )
            })
        {
            context.unsupported(
                format!("nodes[{index}].kind"),
                "non-bone node requires a native attachment/effect mapping",
            )?;
        }
        if node.inverse_bind.is_some() {
            context.unsupported(format!("nodes[{index}].inverse_bind"),"M2 computes bind transforms from pivots; explicit matrices require geometry rebasing")?;
        }
        let mut bone = Bone::<D::Layout>::new();
        bone.pivot = node.pivot;
        bone.flags = native_flags(
            node.flags,
            0,
            &mut context,
            &format!("nodes[{index}].flags"),
        )?;
        if let Some(native) = scene_nodes.get(index) {
            let data = metadata(
                model.native.as_ref().unwrap().document.raw(),
                &native.native,
            )?;
            bone.flags = native_flags(
                node.flags,
                data.get("m2.bone.flags").copied().unwrap_or(0) as u32,
                &mut context,
                &format!("nodes[{index}].flags"),
            )?;
            bone.key_bone_id = data.get("m2.bone.key").copied().unwrap_or(-1) as i32;
            bone.submesh_id = data.get("m2.bone.mesh").copied().unwrap_or(0) as u16;
            bone.name_crc = NameCrcField::make(
                data.get("m2.bone.name_crc").copied().unwrap_or(0) as u32,
                &mut context,
            )?;
        }
        bone.parent_bone_id = node
            .parent
            .map(|id| {
                i16::try_from(id).map_err(|_| Error::new("nodes.parent", "bone parent exceeds i16"))
            })
            .transpose()?
            .unwrap_or(-1);
        bone.translation = export_track::<D::Layout, [f32; 3]>(
            &model,
            sm::Target::Translation(index as u32),
            &starts,
            &mut globals,
            &mut context,
            |v, _, path| {
                v.try_into()
                    .map_err(|_| Error::new(path, "translation must have three components"))
            },
        )?;
        bone.scale = export_track::<D::Layout, [f32; 3]>(
            &model,
            sm::Target::Scale(index as u32),
            &starts,
            &mut globals,
            &mut context,
            |v, _, path| {
                v.try_into()
                    .map_err(|_| Error::new(path, "scale must have three components"))
            },
        )?;
        bone.rotation = export_track::<D::Layout, _>(
            &model,
            sm::Target::Rotation(index as u32),
            &starts,
            &mut globals,
            &mut context,
            |v, context, path| {
                RotationComponents::encode(
                    v.try_into()
                        .map_err(|_| Error::new(path, "rotation must have four components"))?,
                    context,
                    path,
                )
            },
        )?;
        if model.clips.is_empty() {
            if node.translation != [0.; 3] {
                ensure_constant(&mut bone.translation, node.translation, &mut globals);
            }
            if node.scale != [1.; 3] {
                ensure_constant(&mut bone.scale, node.scale, &mut globals);
            }
            if node.rotation != [0., 0., 0., 1.] {
                ensure_constant(
                    &mut bone.rotation,
                    RotationComponents::encode(
                        node.rotation,
                        &mut context,
                        &format!("nodes[{index}].rotation"),
                    )?,
                    &mut globals,
                );
            }
        }
        core.bones.push(bone);
    }
    export_scene(&model, core, &starts, &mut globals, &mut context)?;
    export_animation(&model, core, &starts, &mut globals, &mut context)?;
    export_flipbooks(&model, core, &starts, &mut globals, &mut context)?;
    core.global_sequences = ZeroVec::alloc_from_slice(&globals);
    for (clip_index, clip) in model.clips.iter().enumerate() {
        for track in &clip.tracks {
            if let sm::Target::NativeProperty { .. } = track.target {
                context.unsupported(
                    format!("clips[{clip_index}].track"),
                    "foreign native animation identifier requires a target property mapping",
                )?;
            }
            if let sm::Target::Property { owner, channel } = track.target {
                if [2000, 2001, 2100, 2200, 2201, 2202, 2300].contains(&channel) {
                    continue;
                }
                if scene_nodes.get(owner as usize).is_some_and(|node| {
                    matches!(
                        &node.payload,
                        wr::NodePayload::Attachment(_)
                            | wr::NodePayload::Light(_)
                            | wr::NodePayload::M2ParticleEmitter(_)
                            | wr::NodePayload::Wc3RibbonEmitter(_)
                            | wr::NodePayload::Camera(_)
                            | wr::NodePayload::Event(_)
                    )
                }) && ([3, 4, 6, 7, 8, 200, 201].contains(&channel)
                    || (1000..=1010).contains(&channel)
                    || (1100..=1105).contains(&channel)
                    || (1200..=1203).contains(&channel)
                    || channel == 1400)
                {
                    continue;
                }
                context.unsupported(
                    format!("clips[{clip_index}].track"),
                    "property channel requires a native material/effect mapping",
                )?;
            }
        }
    }
    bundle.key = output_key;
    export_geometry(&model, &mut bundle, &mut context)?;
    D::texture_ids(
        &mut bundle.model,
        model.textures.iter().map(|t| t.id.unwrap_or(0)).collect(),
    )?;
    export_overrides(&model, &mut bundle, &mut context)?;
    export_physics(&model, &mut bundle, &mut context)?;
    bundle.validate_complete().map_err(failure)?;
    context.issue(
        "m2.output",
        IssueKind::Normalized,
        "native array addresses and supporting skin documents rebuilt deterministically",
    );
    Ok(Plan::new(bundle, context.report))
}
