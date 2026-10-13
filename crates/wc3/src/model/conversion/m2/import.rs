//! Native M2 bundle import orchestration.
use super::*;

/// Imports complete editable geometry, skeleton channels and native material passes.
pub fn import<D: Dialect>(
    source: &Bundle<'_, D>,
    options: &ImportOptions,
) -> Result<Conversion<Asset>, Error>
where
    D::Layout: ParticleConversion + CameraConversion,
    for<'data> <D::Layout as native::HeaderLayout>::Flipbooks<'data>:
        FlipbookField<'data, D::Layout>,
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
    <D::Layout as native::records::BoneLayout>::RotationValue: RotationComponents,
    <D::Layout as native::records::BoneLayout>::NameCrc: NameCrcField,
    <D::Layout as native::records::SequenceLayout>::BlendTime: BlendField,
{
    source.validate_complete().map_err(failure)?;
    let mut context = Context::import(options);
    let core = source.model.core();
    let mut model = sm::Model {
        name: Cow::Owned(text(core.name.iter(), "m2.name")?),
        bounds: extent(core.bounding),
        profile: wr::ProfileId::Wow,
        ..Default::default()
    };
    let texture_ids = source.model.extensions().find_map(|chunk| {
        if let Chunk::TextureIds(ids) = chunk {
            Some(ids)
        } else {
            None
        }
    });
    for (index, texture) in core.textures.iter().enumerate() {
        model.textures.push(sm::Texture {
            path: Cow::Owned(text(
                texture.filename.iter(),
                &format!("textures[{index}].path"),
            )?),
            id: texture_ids
                .and_then(|ids| ids.get(index))
                .filter(|id| *id != 0),
            replaceable: texture.kind.raw(),
            flags: wr::TextureFlags::from_bits_retain(texture.flags.bits()),
            ..Default::default()
        });
    }
    let (owner, bones) = source.bones().map_err(failure)?;
    for (index, bone) in bones.iter().enumerate() {
        model.nodes.push(sm::Node {
            name: Cow::Owned(format!("bone-{index}")),
            parent: (bone.parent_bone_id >= 0).then_some(bone.parent_bone_id as u32),
            pivot: bone.pivot,
            flags: common_flags(bone.flags),
            bone: true,
            ..Default::default()
        });
    }
    let mut native_materials = Vec::new();
    let mut profiles: Vec<(usize, bool)> = core
        .embedded_skins()
        .iter()
        .enumerate()
        .map(|(i, _)| (i, false))
        .collect();
    for dependency in source.model.dependencies(&source.key) {
        if let native::DependencyKind::Skin { index, lod } = dependency.kind {
            if !profiles.contains(&(index, lod)) {
                profiles.push((index, lod));
            }
        }
    }
    for (index, lod) in profiles {
        let profile = source.profile(index, lod).map_err(failure)?;
        match profile {
            ProfileRef::Embedded(profile) => import_profile(
                core,
                profile,
                index,
                lod,
                &mut model,
                &mut native_materials,
                &mut context,
            )?,
            ProfileRef::Skin(skin) => import_profile(
                core,
                &skin.profile,
                index,
                lod,
                &mut model,
                &mut native_materials,
                &mut context,
            )?,
            ProfileRef::Geometry(_) => context.unsupported(
                format!("profiles[{index}]"),
                "modern geometry structural and batch records have no verified semantic schema",
            )?,
        }
    }
    let sequences = sequence_tables(source, owner)?.sequences;
    for (index, sequence) in sequences.iter().enumerate() {
        let (start, duration) = sequence.timing.window();
        let mut clip = sm::Clip {
            name: Cow::Owned(format!(
                "animation-{}-{}",
                sequence.id, sequence.variation_index
            )),
            duration: duration as f32 / 1000.,
            id: u32::from(sequence.id),
            flags: Default::default(),
            ..Default::default()
        };
        for (bone_index, bone) in bones.iter().enumerate() {
            for (target, track) in [
                (
                    sm::Target::Translation(bone_index as u32),
                    &bone.translation,
                ),
                (sm::Target::Scale(bone_index as u32), &bone.scale),
            ] {
                if let Some(track) =
                    import_track(source, track, owner, index, start, target, |v| v.to_vec())?
                {
                    clip.tracks.push(track);
                }
            }
            if let Some(track) = import_track(
                source,
                &bone.rotation,
                owner,
                index,
                start,
                sm::Target::Rotation(bone_index as u32),
                |v| v.floats().to_vec(),
            )? {
                clip.tracks.push(track);
            }
        }
        model.clips.push(clip);
    }
    let mut opaque = Vec::new();
    for chunk in source.model.extensions() {
        match chunk {
            Chunk::SkinIds(_)
            | Chunk::TextureIds(_)
            | Chunk::AnimationIds(_)
            | Chunk::SkeletonId(_)
            | Chunk::PhysicsId(_)
            | Chunk::Physics(_)
            | Chunk::BoneIds(_) => {}
            Chunk::Unknown { tag, data } => match options.unknown {
                UnknownPolicy::Reject => {
                    return Err(Error::new(
                        format!("m2.chunks.{tag:?}"),
                        "unknown extension requires an explicit policy",
                    ));
                }
                UnknownPolicy::Drop => context.issue(
                    format!("m2.chunks.{tag:?}"),
                    IssueKind::Dropped,
                    "unknown extension excluded from active WEM records",
                ),
                UnknownPolicy::Archive => {
                    opaque.push(OpaquePayload {
                        format: Format::M2,
                        file: source.key.clone(),
                        tag: *tag,
                        revision: None,
                        count: None,
                        bytes: Cow::Owned(data.to_vec()),
                    });
                    context.issue(
                        format!("m2.chunks.{tag:?}"),
                        IssueKind::Archived,
                        "opaque extension retained separately without encoding the source file",
                    );
                }
            },
            _ => context.unsupported(
                format!("m2.chunks.{:?}", chunk.tag()),
                "extension data has no active editable mapping",
            )?,
        }
    }
    if model.clips.is_empty()
        && (!core.attachments.is_empty()
            || !core.lights.is_empty()
            || !core.particles.is_empty()
            || !core.events.is_empty()
            || !core.ribbons.is_empty()
            || !core.cameras.is_empty()
            || !core.colors.is_empty()
            || !core.texture_weights.is_empty()
            || !core.texture_transforms.is_empty()
            || !core.texture_flipbooks.records().is_empty())
    {
        model.clips.push(sm::Clip {
            name: Cow::Borrowed("static"),
            ..Default::default()
        });
    }
    import_animation(source, &mut model)?;
    import_flipbooks(source, &mut model)?;
    let patches = import_scene(source, &mut model)?;
    let document = sm::build(&model)?;
    let document = patch_scene(document, patches, core)?;
    let document = patch_materials(document, &native_materials)?;
    let document = patch_native_metadata(document, source)?;
    let document = patch_clips(document, source)?;
    let document = import_overrides(document, source)?;
    let document = import_physics(document, source, &mut context, options, &mut opaque)?;
    let asset = Asset { document, opaque };
    Ok(Conversion {
        value: asset,
        report: context.report,
    })
}
