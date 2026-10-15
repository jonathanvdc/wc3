//! Direct native M2 adapters for the independent authoring graph.
use super::*;
use crate::model::io::materialize;
use crate::model::{self as semantic, profiles::m2 as profile};
mod actors;
mod actors_export;
pub(super) mod animation;
mod export;
mod geometry;
mod materials;
mod overrides;
mod physics;
use animation::{bind, curve};
use semantic::profiles::NoControls;
use semantic::profiles::M2 as M2Vocabulary;
pub(in super::super) type Graph<'a> = semantic::Model<'a, M2Vocabulary>;
pub(in super::super) type GraphBuilder<'a> = semantic::ModelBuilder<'a, M2Vocabulary>;
pub use export::plan;
use semantic::{Animated, Channel, Node, Resource, SharedClock, Time};

/// Imports an M2 bundle without a canonical intermediate encoding.
pub fn import<'a, 'data: 'a, D: Dialect>(
    source: &'a Bundle<'data, D>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, M2Vocabulary>>, Error>
where
    D::Layout: ParticleConversion + CameraConversion,
    for<'v> <D::Layout as native::HeaderLayout>::Flipbooks<'v>: FlipbookField<'v, D::Layout>,
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
    <D::Layout as native::records::BoneLayout>::RotationValue: RotationComponents + Send + Sync,
    <D::Layout as native::records::BoneLayout>::NameCrc: NameCrcField,
    <D::Layout as native::records::SequenceLayout>::BlendTime: BlendField,
    <D::Layout as native::records::SkinSectionLayout>::Tail: SectionSorting,
    <<D::Layout as native::records::BoneLayout>::RotationValue as zerovec::ule::AsULE>::ULE:
        Send + Sync,
    <[<D::Layout as native::records::BoneLayout>::RotationValue; 3] as zerovec::ule::AsULE>::ULE:
        Send + Sync,
{
    let mut cores = source.model.cores();
    let core = cores
        .next()
        .ok_or_else(|| Error::new("m2.core", "model does not contain a core payload"))?;
    if cores.next().is_some() {
        return Err(Error::new(
            "m2.core",
            "conversion requires one model core; multiple core payloads are ambiguous",
        ));
    }
    source.validate_complete().map_err(failure)?;

    let mut context = Context::import(options);
    let name = core.name.as_bytes();
    let name = std::str::from_utf8(name.strip_suffix(&[0]).unwrap_or(name))
        .map_err(|_| Error::new("m2.name", "invalid UTF-8 name"))?;
    let mut builder = GraphBuilder::with_profile(Cow::Borrowed(name));
    builder.set_bounds(Some(bounds(core.bounding)));
    let mut family = profile::ModelMetadata {
        flags: profile::values::ModelFlags::from_bits_retain(core.flags.bits()),
        ..Default::default()
    };
    let texture_ids = source.model.extensions().find_map(|chunk| {
        if let Chunk::TextureIds(ids) = chunk {
            Some(ids)
        } else {
            None
        }
    });
    let mut textures = Vec::new();
    for (index, texture) in core.textures.iter().enumerate() {
        let path = texture.filename.as_bytes();
        let path = path.strip_suffix(&[0]).unwrap_or(path);
        let resource = if let Some(id) = texture_ids
            .and_then(|ids| ids.get(index))
            .filter(|id| *id != 0)
        {
            Resource::FileDataId(id)
        } else if texture.kind == TextureSource::File {
            Resource::Path(Cow::Borrowed(path))
        } else {
            Resource::Replaceable(texture.kind.raw())
        };
        textures.push(builder.add_texture(semantic::Texture {
            controls: profile::TextureControls {
                path: if !path.is_empty() && !matches!(resource, Resource::Path(_)) {
                    Some(Cow::Borrowed(path))
                } else {
                    None
                },
                flags: profile::SupplementalTextureFlags::from_bits_retain(texture.flags.bits()),
            },
            resource,
            color_space: semantic::ColorSpace::Unspecified,
            sampler: semantic::Sampler {
                u: if texture.flags.wrap_u() {
                    semantic::AddressMode::Repeat
                } else {
                    semantic::AddressMode::Clamp
                },
                v: if texture.flags.wrap_v() {
                    semantic::AddressMode::Repeat
                } else {
                    semantic::AddressMode::Clamp
                },
            },
        })?);
    }
    let (owner, bones) = source.bones().map_err(failure)?;
    let nodes: Vec<_> = bones.iter().map(|_| builder.reserve_node()).collect();
    for (index, bone) in bones.iter().enumerate() {
        let mut node = Node::named(format!("bone-{index}"));
        if bone.parent_bone_id >= 0 {
            node.parent = Some(
                *nodes
                    .get(bone.parent_bone_id as usize)
                    .ok_or_else(|| Error::new("m2.bone.parent", "missing parent bone"))?,
            )
        }
        node.transform.pivot = bone.pivot;
        node.inheritance = semantic::Inheritance {
            translation: !bone.flags.dont_inherit_translation(),
            rotation: !bone.flags.dont_inherit_rotation(),
            scale: !bone.flags.dont_inherit_scale(),
        };
        node.billboard = if bone.flags.billboard_lock_x()
            || bone.flags.billboard_lock_y()
            || bone.flags.billboard_lock_z()
        {
            semantic::Billboard::Locked {
                enabled: bone.flags.billboard(),
                x: bone.flags.billboard_lock_x(),
                y: bone.flags.billboard_lock_y(),
                z: bone.flags.billboard_lock_z(),
            }
        } else if bone.flags.billboard() {
            semantic::Billboard::Spherical
        } else {
            semantic::Billboard::None
        };
        let mut behavior = bone.flags;
        behavior.set_dont_inherit_translation(false);
        behavior.set_dont_inherit_rotation(false);
        behavior.set_dont_inherit_scale(false);
        behavior.set_billboard(false);
        behavior.set_billboard_lock_x(false);
        behavior.set_billboard_lock_y(false);
        behavior.set_billboard_lock_z(false);
        node.controls = Some(profile::BoneControls {
            key_bone: bone.key_bone_id,
            flags: profile::values::BoneBehaviorFlags::from_bits_retain(behavior.bits()),
            submesh: bone.submesh_id,
            name_crc: bone.name_crc.get(),
        });
        builder.set_node(nodes[index], node)?;
    }
    let tables = sequence_tables(source, owner)?;
    let mut clocks = Vec::new();
    for duration in tables.globals.iter() {
        clocks.push(builder.add_clock(SharedClock {
            period: Time::seconds(f64::from(duration) / 1000.)?,
        })?)
    }
    let mut clips = Vec::new();
    for seq in tables.sequences {
        let (_, duration) = seq.timing.window();
        clips.push(
            builder.add_clip(
                semantic::Clip::named(format!("animation-{}-{}", seq.id, seq.variation_index))
                    .with_duration(Time::seconds(f64::from(duration) / 1000.)?),
            )?,
        )
    }
    family.animation_lookup = tables
        .lookup
        .iter()
        .map(|i| {
            if i == u16::MAX {
                Ok(None)
            } else {
                clips
                    .get(i as usize)
                    .copied()
                    .map(Some)
                    .ok_or_else(|| Error::new("m2.animation_lookup", "missing clip"))
            }
        })
        .collect::<Result<_, _>>()?;
    let lookup = if owner == &source.key {
        Some(&core.key_bone_lookup)
    } else {
        source
            .file(owner)
            .and_then(|file| {
                if let File::Skeleton(file) = file {
                    file.as_typed::<D::Layout>()
                } else {
                    None
                }
            })
            .and_then(|file| {
                file.chunks.iter().find_map(|chunk| {
                    if let native::companions::SkeletonChunk::Bones { lookup, .. } = chunk {
                        Some(lookup)
                    } else {
                        None
                    }
                })
            })
    };
    if let Some(lookup) = lookup {
        family.key_bone_lookup = lookup
            .iter()
            .map(|i| {
                if i == u16::MAX {
                    Ok(None)
                } else {
                    nodes
                        .get(i as usize)
                        .copied()
                        .map(Some)
                        .ok_or_else(|| Error::new("m2.key_bone_lookup", "missing bone"))
                }
            })
            .collect::<Result<_, _>>()?;
    }
    for (index, sequence) in tables.sequences.iter().enumerate() {
        let (start, _) = sequence.timing.window();
        let id = clips[index];
        let mut flags = profile::values::SequenceFlags::from_bits_retain(sequence.flags.bits());
        flags.set_alias(false);
        builder.set_clip_controls(
            id,
            profile::SequenceControls {
                animation_id: sequence.id,
                variation: sequence.variation_index,
                movement_speed: sequence.move_speed,
                flags,
                frequency: sequence.frequency,
                replay: [sequence.replay_min, sequence.replay_max],
                blend_time: sequence.blend_time.parts(),
                bounds: bounds(sequence.bounding),
                next_variation: if sequence.variation_next >= 0 {
                    Some(
                        *clips
                            .get(sequence.variation_next as usize)
                            .ok_or_else(|| Error::new("m2.sequence.next", "missing variation"))?,
                    )
                } else {
                    None
                },
            },
        )?;
        if sequence.flags.alias() {
            let target = *clips
                .get(sequence.alias_next as usize)
                .ok_or_else(|| Error::new("m2.sequence.alias", "missing clip"))?;
            let clip = builder.model().clip(id).unwrap().clone().with_alias(target);
            builder.set_clip(id, clip)?;
            continue;
        }
        for (bone_index, bone) in bones.iter().enumerate() {
            for (target, track) in [
                (nodes[bone_index].translation(), &bone.translation),
                (nodes[bone_index].scale(), &bone.scale),
            ] {
                if let Some((curve, global)) =
                    curve::<D, [f32; 3], [f32; 3]>(source, track, owner, index, start, |v| v)?
                {
                    builder
                        .add_channel(id, Channel::Vector3(bind(curve, global, target, &clocks)?))?
                }
            }
            if let Some((curve, global)) =
                curve(source, &bone.rotation, owner, index, start, |v| {
                    semantic::Quaternion(v.floats())
                })?
            {
                animation::insert(
                    &mut builder,
                    Some(id),
                    Channel::Rotation(bind(curve, global, nodes[bone_index].rotation(), &clocks)?),
                )?
            }
        }
    }
    if clips.is_empty() {
        for (index, bone) in bones.iter().enumerate() {
            for (target, track) in [
                (nodes[index].translation(), &bone.translation),
                (nodes[index].scale(), &bone.scale),
            ] {
                if track.base.global_sequence != u16::MAX {
                    if let Some((c, g)) =
                        curve::<D, [f32; 3], [f32; 3]>(source, track, owner, 0, 0, |v| v)?
                    {
                        animation::insert(
                            &mut builder,
                            None,
                            Channel::Vector3(bind(c, g, target, &clocks)?),
                        )?
                    }
                }
            }
            if bone.rotation.base.global_sequence != u16::MAX {
                if let Some((c, g)) = curve(source, &bone.rotation, owner, 0, 0, |v| {
                    semantic::Quaternion(v.floats())
                })? {
                    animation::insert(
                        &mut builder,
                        None,
                        Channel::Rotation(bind(c, g, nodes[index].rotation(), &clocks)?),
                    )?
                }
            }
        }
    }
    geometry::import(
        source,
        &nodes,
        &textures,
        &clips,
        &clocks,
        &mut builder,
        &mut context,
    )?;
    actors::import(source, &nodes, &textures, &clips, &clocks, &mut builder)?;
    let collision_positions = &core.collision_vertices;
    let collision_indices = &core.collision_indices;
    let collision_normals = &core.collision_normals;
    let collision = (!core.collision_vertices.is_empty()).then(|| profile::Collision {
        positions: materialize(core.collision_vertices.len(), move |i| {
            collision_positions.get(i).unwrap()
        }),
        triangles: materialize(core.collision_indices.len() / 3, move |i| {
            std::array::from_fn(|j| u32::from(collision_indices.get(i * 3 + j).unwrap()))
        }),
        normals: materialize(core.collision_normals.len(), move |i| {
            collision_normals.get(i).unwrap()
        }),
        bounds: bounds(core.collision),
    });
    if let Some(collision) = collision {
        builder.add_system(profile::System::Collision(collision))?;
    }
    let overrides = overrides::import(source, &nodes)?;
    if !overrides.is_empty() {
        builder.add_system(profile::System::BoneOverrides(overrides))?;
    }
    for physics in physics::import(source, &nodes, &mut builder, &mut context)? {
        builder.add_system(profile::System::Physics(physics))?;
    }
    builder.set_metadata(family);
    let asset = builder.finish()?;
    for chunk in source.model.extensions() {
        match chunk {
            Chunk::Core(_)
            | Chunk::PhysicsId(_)
            | Chunk::SkinIds(_)
            | Chunk::AnimationIds(_)
            | Chunk::BoneIds(_)
            | Chunk::SkeletonId(_)
            | Chunk::TextureIds(_)
            | Chunk::Physics(_)
            | Chunk::Unknown { .. } => {}
            _ => context.unsupported(
                "m2.extensions",
                "extension semantics do not yet have a logical mapping",
            )?,
        }
        if let Chunk::Unknown { .. } = chunk {
            context.unknown_data(
                "m2.chunk",
                "unknown extension has no semantic representation",
            )?;
        }
    }
    Ok(Conversion {
        value: asset,
        report: context.report,
    })
}
pub(super) fn bounds(value: native::records::Extent) -> semantic::Bounds {
    semantic::Bounds {
        minimum: value.minimum,
        maximum: value.maximum,
        sphere_radius: value.radius,
    }
}
