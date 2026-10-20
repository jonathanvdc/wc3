//! Typed M2 output planning and native graph construction.
use super::animation::{encode, time_ms};
use super::*;
use crate::model::M2ImageResource;
use crate::model::TimeSource;
use crate::model::{Billboard, TransformSemantics};
use glam::Vec3;
use zerovec::ZeroVec;

/// Plans a typed M2 bundle, checking index limits, curves and resource identities.
pub fn plan<D: ExportDialect>(
    asset: &Model<'_, M2Vocabulary>,
    options: &ExportOptions,
) -> Result<Plan<Bundle<'static, D>>, Error>
where
    D::Layout: ExportLayout + ParticleConversion + CameraConversion,
    <D::Layout as native::HeaderLayout>::Flipbooks<'static>: FlipbookField<'static, D::Layout>,
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
    <D::Layout as native::records::BoneLayout>::RotationValue: RotationComponents,
    <D::Layout as native::records::BoneLayout>::NameCrc: NameCrcField,
    <D::Layout as native::records::SequenceLayout>::BlendTime: BlendField,
    <D::Layout as native::records::SkinSectionLayout>::Tail: SectionSorting,
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
    asset.validate()?;
    let model = asset;
    let mut context = Context::export(options);
    for (_, clip) in model.clips() {
        if !clip.is_looping() {
            context.unsupported(
                "m2.clips.looping",
                "M2 playback looping has no verified native mapping",
            )?;
        }
    }
    let family = Some(model.metadata());

    let mut bundle = Bundle::from_model(native::Document::<D>::new());
    let core = bundle.model.core_mut();
    core.bounding = native_optional_bounds(model.bounds());
    if let Some(family) = family {
        core.flags = ModelFlags::from_bits_retain(family.flags.bits());
    }
    for (_, system) in model.systems() {
        if let profile::System::Collision(collision) = system {
            core.collision = native_bounds(collision.bounds);
            core.collision_vertices = ZeroVec::alloc_from_slice(
                &collision
                    .positions
                    .iter()
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|v| v.to_array())
                    .collect::<Vec<_>>(),
            );
            let normals = if collision.normals.is_empty() {
                collision
                    .triangles
                    .iter()
                    .map(|triangle| {
                        let [a, b, c] =
                            triangle.map(|index| collision.positions.get(index as usize).unwrap());
                        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                        let n = [
                            u[1] * v[2] - u[2] * v[1],
                            u[2] * v[0] - u[0] * v[2],
                            u[0] * v[1] - u[1] * v[0],
                        ];
                        let length = n.iter().map(|v| v * v).sum::<f32>().sqrt();
                        if length == 0. {
                            [0.; 3]
                        } else {
                            n.map(|v| v / length)
                        }
                    })
                    .collect::<Vec<_>>()
            } else {
                collision.normals.iter().map(|v| v.to_array()).collect()
            };
            core.collision_normals = ZeroVec::alloc_from_slice(&normals);
            core.collision_indices = ZeroVec::alloc_from_slice(
                &collision
                    .triangles
                    .iter()
                    .flatten()
                    .map(|v| {
                        u16::try_from(v)
                            .map_err(|_| Error::new("m2.collision", "index exceeds u16"))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
    }
    core.name = ZeroVec::alloc_from_slice(&model.name().bytes().chain([0]).collect::<Vec<_>>());
    let bone_controls: BTreeMap<_, _> = model
        .nodes()
        .iter()
        .filter_map(|(id, node)| node.properties.as_ref().map(|value| (id, value)))
        .collect();
    let mut nodes: Vec<_> = bone_controls.keys().copied().collect();
    for (_, attached) in model.features() {
        if !matches!(attached.payload, profile::ActorKind::Camera(_)) {
            let node = model.node(attached.node).unwrap();
            let animated = model
                .clips()
                .iter()
                .flat_map(|(id, _)| model.clip_tracks(id))
                .chain(
                    model
                        .global_tracks()
                        .filter(|v| matches!(v.clock(), TimeSource::Shared(_))),
                )
                .any(|channel| match channel {
                    semantic::DynamicTrack::Vector3(t) => {
                        t.target == node.transform.translation || t.target == node.transform.scale
                    }
                    semantic::DynamicTrack::Rotation(t) => t.target == node.transform.rotation,
                    _ => false,
                });
            if (animated
                || *model
                    .values()
                    .get(node.transform.rotation)
                    .expect("validated slot")
                    != semantic::Quat::default()
                || *model
                    .values()
                    .get(node.transform.scale)
                    .expect("validated slot")
                    != Vec3::from_array([1.; 3]))
                && !nodes.contains(&attached.node)
            {
                nodes.push(attached.node);
            }
        }
    }
    let mut textures = Vec::new();
    for image in asset.sampled_images() {
        if texture_index(asset, &textures, image).is_none() {
            textures.push(image);
        }
    }
    let mut texture_ids = Vec::new();
    for texture in &textures {
        let (kind, path, id) = match asset
            .image_resource(texture.resource)
            .expect("validated image")
        {
            M2ImageResource::Path(path) => (TextureSource::File, path.as_ref(), 0),
            M2ImageResource::FileDataId { id, filename } => {
                (TextureSource::File, filename.as_ref(), *id)
            }
            M2ImageResource::Replaceable { id, filename } => {
                (TextureSource::from_raw(*id), filename.as_ref(), 0)
            }
        };
        let mut flags = TextureFlags::default();
        flags.set_wrap_u(texture.sampler.u == semantic::AddressMode::Repeat);
        flags.set_wrap_v(texture.sampler.v == semantic::AddressMode::Repeat);
        core.textures.push(native::records::Texture {
            kind,
            flags,
            filename: ZeroVec::alloc_from_slice(
                &path.iter().copied().chain([0]).collect::<Vec<_>>(),
            ),
        });
        texture_ids.push(id)
    }
    let clips: Vec<_> = model.clips().iter().map(|(id, _)| id).collect();
    let mut starts = Vec::new();
    let mut start = 0u32;
    for (_, clip) in model.clips() {
        let duration = time_ms(clip.duration(), &mut context)?;
        starts.push(start);
        let data = &clip.properties;
        let mut flags = SequenceFlags::from_bits_retain(data.flags.bits());
        flags.set_alias(clip.properties.alias.is_some());
        flags.set_primary_bone_sequence(true);
        flags.set_load_low_priority(false);
        flags.set_stored_in_model(false);
        core.sequences.push(native::records::Sequence {
            timing: SequenceClock::from_window(start, duration),
            id: data.animation_id.get(),
            variation_index: data.variation,
            move_speed: data.movement_speed,
            flags,
            frequency: data.frequency,
            replay_min: data.replay[0],
            replay_max: data.replay[1],
            blend_time: BlendField::make(data.blend_time, &mut context)?,
            bounding: native_optional_bounds(data.bounds),
            alias_next: clip
                .properties
                .alias
                .map(|id| clips.iter().position(|v| *v == id).unwrap() as u16)
                .unwrap_or(0),
            variation_next: data
                .next_variation
                .map(|id| clips.iter().position(|v| *v == id).unwrap() as i16)
                .unwrap_or(-1),
            ..Default::default()
        });
        start = start
            .checked_add(duration)
            .and_then(|v| v.checked_add(1))
            .ok_or_else(|| Error::new("m2.animation", "legacy timeline overflow"))?
    }
    if let Some(family) = family {
        core.sequence_lookup = ZeroVec::alloc_from_slice(
            &family
                .animation_lookup
                .iter()
                .map(|id| {
                    id.map(|id| clips.iter().position(|v| *v == id).unwrap() as u16)
                        .unwrap_or(u16::MAX)
                })
                .collect::<Vec<_>>(),
        );
        core.key_bone_lookup = ZeroVec::alloc_from_slice(
            &family
                .key_bone_lookup
                .iter()
                .map(|id| {
                    id.map(|id| {
                        if model.node(id).is_none_or(|node| node.properties.is_none()) {
                            return Err(Error::new(
                                "m2.key_bone_lookup",
                                "key-bone binding requires a declared bone",
                            ));
                        }
                        let index =
                            nodes.iter().position(|value| *value == id).ok_or_else(|| {
                                Error::new("m2.key_bone_lookup", "key-bone binding is not exported")
                            })?;
                        let index = u16::try_from(index).map_err(|_| {
                            Error::new("m2.key_bone_lookup", "bone index exceeds lookup width")
                        })?;
                        if index == u16::MAX {
                            return Err(Error::new(
                                "m2.key_bone_lookup",
                                "bone index collides with the unbound sentinel",
                            ));
                        }
                        Ok(index)
                    })
                    .transpose()
                    .map(|value| value.unwrap_or(u16::MAX))
                })
                .collect::<Result<Vec<_>, Error>>()?,
        );
    }
    let mut globals = Vec::new();
    for id in &nodes {
        let node = model
            .node(*id)
            .ok_or_else(|| Error::new("m2.bone", "missing node"))?;
        let data = bone_controls.get(id).copied();
        let mut bone = Bone::<D::Layout>::new();
        bone.pivot = (node.transform.pivot).to_array();
        bone.parent_bone_id = node
            .parent
            .map(|p| {
                nodes
                    .iter()
                    .position(|v| *v == p)
                    .ok_or_else(|| Error::new("m2.bone.parent", "parent is not an exported bone"))
                    .and_then(|i| {
                        i16::try_from(i)
                            .map_err(|_| Error::new("m2.bone.parent", "index exceeds i16"))
                    })
            })
            .transpose()?
            .unwrap_or(-1);
        bone.flags = BoneFlags::from_bits_retain(data.map_or(0, |b| b.flags.bits()));
        bone.flags
            .set_dont_inherit_translation(!node.inheritance.translation);
        bone.flags
            .set_dont_inherit_rotation(!node.inheritance.rotation);
        bone.flags.set_dont_inherit_scale(!node.inheritance.scale);
        if node.transform_semantics != TransformSemantics::Affine
            && (!node.inheritance.translation
                || !node.inheritance.rotation
                || !node.inheritance.scale
                || node.billboard != Billboard::None
                || node.camera_anchored)
        {
            context.unsupported(
                "node.transform_semantics",
                "M2 requires affine composition for selective inheritance and camera controls",
            )?;
        }
        let flags = billboard::encode(
            node.billboard,
            data.map_or(BillboardFlags::default(), |b| b.billboard_flags),
            &mut context,
        )?;
        bone.flags.set_billboard(flags.enabled);
        bone.flags.set_billboard_lock_x(flags.x);
        bone.flags.set_billboard_lock_y(flags.y);
        bone.flags.set_billboard_lock_z(flags.z);
        bone.key_bone_id = data.map_or(-1, |b| b.key_bone);
        bone.submesh_id = data.map_or(0, |b| b.submesh);
        bone.name_crc = NameCrcField::make(data.map_or(0, |b| b.name_crc), &mut context)?;
        bone.translation = encode(
            model,
            &node.transform.translation,
            &starts,
            &mut globals,
            &mut context,
            |v, _| Ok(v.to_array()),
        )?;
        bone.scale = encode(
            model,
            &node.transform.scale,
            &starts,
            &mut globals,
            &mut context,
            |v, _| Ok(v.to_array()),
        )?;
        bone.rotation = encode(
            model,
            &node.transform.rotation,
            &starts,
            &mut globals,
            &mut context,
            |v, c| RotationComponents::encode(v.to_array(), c, "m2.bone.rotation"),
        )?;
        if *model
            .values()
            .get(node.transform.translation)
            .expect("validated slot")
            != Vec3::from_array([0.; 3])
        {
            ensure_constant(
                &mut bone.translation,
                (*model
                    .values()
                    .get(node.transform.translation)
                    .expect("validated slot"))
                .to_array(),
                &mut globals,
            )
        }
        if *model
            .values()
            .get(node.transform.scale)
            .expect("validated slot")
            != Vec3::from_array([1.; 3])
        {
            ensure_constant(
                &mut bone.scale,
                (*model
                    .values()
                    .get(node.transform.scale)
                    .expect("validated slot"))
                .to_array(),
                &mut globals,
            )
        }
        if *model
            .values()
            .get(node.transform.rotation)
            .expect("validated slot")
            != semantic::Quat::default()
        {
            ensure_constant(
                &mut bone.rotation,
                RotationComponents::encode(
                    model
                        .values()
                        .get(node.transform.rotation)
                        .expect("validated slot")
                        .to_array(),
                    &mut context,
                    "m2.bone.rotation",
                )?,
                &mut globals,
            )
        }
        core.bones.push(bone)
    }
    let controls = materials::export(model, core, &starts, &mut globals, &mut context)?;
    core.global_sequences = ZeroVec::alloc_from_slice(&globals);
    geometry::export(
        model,
        &nodes,
        &textures,
        &controls,
        options.look,
        &mut bundle,
        &mut context,
    )?;
    let particle_programs: Vec<_> = model
        .features()
        .iter()
        .filter_map(|(_, feature)| {
            if let profile::ActorKind::Particle(value) = &feature.payload {
                Some(value.details.rendering.texture_program)
            } else {
                None
            }
        })
        .collect();
    actors_export::export(
        model,
        &nodes,
        &textures,
        bundle.model.core_mut(),
        &starts,
        &mut globals,
        &mut context,
    )?;
    bundle.model.core_mut().global_sequences = ZeroVec::alloc_from_slice(&globals);
    if particle_programs.iter().any(Option::is_some) {
        let mut programs = vec![[0; 2]; bundle.model.core().materials.len()];
        programs.extend(particle_programs.into_iter().map(Option::unwrap_or_default));
        D::texture_combiner_hints(&mut bundle.model, programs, &mut context)?;
    }
    D::texture_ids(&mut bundle.model, texture_ids)?;
    for (_, system) in model.systems() {
        if let profile::System::BoneOverrides(values) = system {
            overrides::export(model, values, &nodes, &mut bundle, &mut context)?;
        }
    }
    physics::export(model, &nodes, &mut bundle, &mut context)?;
    bundle.validate_complete().map_err(failure)?;
    Ok(Plan::new(bundle, context.report))
}
fn native_optional_bounds(value: Option<semantic::Bounds>) -> native::records::Extent {
    value.map(native_bounds).unwrap_or(native::records::Extent {
        minimum: [f32::MAX; 3],
        maximum: [-f32::MAX; 3],
        radius: 0.0,
    })
}
fn native_bounds(value: semantic::Bounds) -> native::records::Extent {
    native::records::Extent {
        minimum: (value.minimum).to_array(),
        maximum: (value.maximum).to_array(),
        radius: value.sphere_radius,
    }
}
