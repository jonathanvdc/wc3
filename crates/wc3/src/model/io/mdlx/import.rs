//! Direct Warcraft III decoding into checked independent model graphs.
use super::storage::Storage;
use super::{curves, materials, state::ImportState};
use crate::model::conversion::{Context, Conversion, Error, ImportOptions};
use crate::model::formats::mdlx::{self as native, Chunk, Dialect, ModelChunk};
use crate::model::profiles::mdlx as profile;
use crate::model::profiles::Mdlx;
use crate::model::Model;
use crate::model::{
    AddressMode, Billboard, Bounds, Channel, Clip, Clock, Inheritance, Material, ModelBuilder,
    Node, Quaternion, Resource, Sampler, SharedClock, Texture, TextureTransform, Time, Track,
};
use std::borrow::Cow;
use std::collections::BTreeMap;

pub(in crate::model::io::mdlx) fn read<'source, 'a, D: Dialect, S: Storage<'source, 'a>>(
    source: &'source native::Document<'_, D>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, Mdlx>>, Error>
where
    D::Extension: Chunk,
{
    let mut context = Context::import(options);
    super::text::validate(source, &mut context)?;
    for chunk in &source.chunks {
        if !matches!(chunk, ModelChunk::Unknown(_) | ModelChunk::Extension(_)) {
            continue;
        }
        context.unknown_data(
            "chunks",
            "unknown source chunk has no semantic representation",
        )?;
    }
    let mut builder =
        ModelBuilder::<Mdlx>::with_profile(source.model_info().map_or(Cow::Borrowed(""), |info| {
            Cow::Owned(info.name.text().into_owned())
        }));
    if let Some(info) = source.model_info() {
        builder.set_bounds(Some(Bounds {
            minimum: info.minimum_extent,
            maximum: info.maximum_extent,
            sphere_radius: info.bounds_radius,
        }));
    } else {
        builder.set_bounds(None);
    }
    let source_nodes = source.nodes();
    let ids: Vec<_> = source_nodes
        .iter()
        .map(|_| builder.reserve_node())
        .collect();
    let mut identity = BTreeMap::new();
    for (node, id) in source_nodes.iter().zip(&ids) {
        if identity.insert(node.object_id, *id).is_some() {
            return Err(Error::new("nodes", "duplicate source node identity"));
        }
    }
    let pivots = source.pivot_points();
    for (node, id) in source_nodes.iter().zip(&ids) {
        let mut value = Node::named(S::text(node.name.text()));
        value.parent = if node.parent_id == u32::MAX {
            None
        } else {
            Some(
                *identity
                    .get(&node.parent_id)
                    .ok_or_else(|| Error::new("node.parent", "parent identity does not exist"))?,
            )
        };
        value.transform.pivot = pivots
            .get(node.object_id as usize)
            .copied()
            .unwrap_or_default();
        value.inheritance = Inheritance {
            translation: !node.flags.dont_inherit_translation(),
            rotation: !node.flags.dont_inherit_rotation(),
            scale: !node.flags.dont_inherit_scaling(),
        };
        value.billboard = if node.flags.billboard_lock_x()
            || node.flags.billboard_lock_y()
            || node.flags.billboard_lock_z()
        {
            Billboard::Locked {
                enabled: node.flags.billboarded(),
                x: node.flags.billboard_lock_x(),
                y: node.flags.billboard_lock_y(),
                z: node.flags.billboard_lock_z(),
            }
        } else if node.flags.billboarded() {
            Billboard::Spherical
        } else {
            Billboard::None
        };
        value.camera_anchored = node.flags.camera_anchored();
        builder.set_node(*id, value)?;
    }
    let mut texture_ids = Vec::new();
    for texture in source.textures() {
        let resource = if texture.replaceable_id != 0 {
            Resource::Replaceable(texture.replaceable_id)
        } else {
            {
                let bytes = texture.path.as_bytes();
                let end = bytes
                    .iter()
                    .position(|byte| *byte == 0)
                    .unwrap_or(bytes.len());
                Resource::Path(S::bytes(&bytes[..end]))
            }
        };
        texture_ids.push(builder.add_texture(Texture {
            controls: Default::default(),
            resource,
            color_space: Default::default(),
            sampler: Sampler {
                u: if texture.flags.wrap_width() {
                    AddressMode::Repeat
                } else {
                    AddressMode::Clamp
                },
                v: if texture.flags.wrap_height() {
                    AddressMode::Repeat
                } else {
                    AddressMode::Clamp
                },
            },
        })?);
    }
    let transform_ids = source
        .texture_animations()
        .map(|_| builder.add_texture_transform(TextureTransform::default()))
        .collect::<Result<Vec<_>, _>>()?;
    let mut material_ids = Vec::new();
    for material in source.materials() {
        let value = Material::<Mdlx> {
            name: Cow::Borrowed(""),
            surface: materials::read::<_, S>(material, &texture_ids, &transform_ids)?,
        };
        material_ids.push(builder.add_material(value)?);
    }
    let globals: Vec<_> = source
        .global_sequences()
        .iter()
        .map(|duration| {
            builder.add_clock(SharedClock {
                period: Time::seconds(f64::from(*duration) / 1000.)?,
            })
        })
        .collect::<Result<_, _>>()?;
    let mut native_profile = ImportState::default();
    for (source, id) in source.textures().zip(&texture_ids) {
        let mut flags = source.flags;
        flags.set_wrap_width(false);
        flags.set_wrap_height(false);
        native_profile.textures.push((
            *id,
            profile::TextureControls {
                fallback_path: if source.replaceable_id == 0 {
                    Cow::Borrowed(&[][..])
                } else {
                    let bytes = source.path.as_bytes();
                    S::bytes(
                        &bytes[..bytes
                            .iter()
                            .position(|value| *value == 0)
                            .unwrap_or(bytes.len())],
                    )
                },
                unclassified: flags.bits().into(),
            },
        ));
    }
    for (source, id) in source_nodes.iter().zip(&ids) {
        native_profile
            .node_controls
            .push((*id, source.flags.bits() & !0x7fff));
    }
    for emitter in source.particle_emitters() {
        if let Some((_, bits)) = native_profile
            .node_controls
            .iter_mut()
            .find(|(id, _)| *id == identity[&emitter.node.object_id])
        {
            *bits &= !emitter.flags().bits();
        }
    }
    for emitter in source.particle_emitters2() {
        if let Some((_, bits)) = native_profile
            .node_controls
            .iter_mut()
            .find(|(id, _)| *id == identity[&emitter.node.object_id])
        {
            *bits &= !emitter.flags().bits();
        }
    }
    if let Ok(emitters) = source.try_popcorn_emitters() {
        for emitter in emitters {
            if let Some((_, bits)) = native_profile
                .node_controls
                .iter_mut()
                .find(|(id, _)| *id == identity[&emitter.node.object_id])
            {
                *bits &= !emitter.flags().bits();
            }
        }
    }
    if let Some(info) = source.model_info() {
        native_profile.metadata.animation_file =
            Cow::Owned(info.animation_file_name.text().into_owned());
        native_profile.metadata.blend_time = f64::from(info.blend_time) / 1000.;
    }
    let mut clips = Vec::new();
    for sequence in source.sequences() {
        let duration =
            Time::seconds(f64::from(sequence.interval[1] - sequence.interval[0]) / 1000.)?;
        let mut clip = Clip::named(S::text(sequence.name.text())).with_duration(duration);
        if sequence.flags.non_looping() {
            clip = clip.once();
        }
        let id = builder.add_clip(clip)?;
        let start = f64::from(sequence.interval[0]) / 1000.;
        clips.push((id, start));
        let mut flags = sequence.flags;
        flags.set_non_looping(false);
        native_profile.sequences.push((
            id,
            profile::Sequence {
                start: Time::seconds(start)?,
                move_speed: sequence.move_speed,
                rarity: sequence.rarity,
                sync_point: sequence.sync_point,
                unclassified: flags.bits().into(),
                radius: sequence.extent.bounds_radius,
                bounds: [sequence.extent.minimum, sequence.extent.maximum],
            },
        ));
    }
    for (node, id) in source_nodes.iter().zip(&ids) {
        macro_rules! channel {
            ($source:expr,$property:expr,$variant:ident,$map:expr) => {
                if let Some(track) = $source {
                    let clock = track
                        .global_sequence_id()
                        .map(|index| {
                            globals.get(index as usize).copied().ok_or_else(|| {
                                Error::new("animation.clock", "shared clock does not exist")
                            })
                        })
                        .transpose()?
                        .map_or(Clock::Timeline, Clock::Shared);
                    let channel = Channel::$variant(Track {
                        target: $property,
                        curve: curves::read(track, 0., $map)?,
                        clock,
                    });
                    if matches!(clock, Clock::Shared(_)) {
                        builder.add_shared_channel(channel)?;
                    } else {
                        builder.add_timeline_channel(channel)?;
                    }
                }
            };
        }
        channel!(node.translation, id.translation(), Vector3, |value| value);
        channel!(node.rotation, id.rotation(), Rotation, Quaternion);
        channel!(node.scaling, id.scale(), Vector3, |value| value);
    }
    {
        let mut decoder = super::animation::Decoder {
            builder: &mut builder,
            globals: &globals,
        };
        for (source, id) in source.materials().zip(&material_ids) {
            let surface =
                materials::bind::<_, S>(source, &texture_ids, &transform_ids, *id, &mut decoder)?;
            decoder.builder.set_material_surface(*id, surface)?;
        }
        for (source, id) in source.texture_animations().zip(&transform_ids) {
            let mut transform = TextureTransform::default();
            if let Some(track) = source.translation.as_ref() {
                transform.translation = decoder.track_property(
                    track,
                    [0.; 3],
                    id.translation(),
                    "texture_transform.translation",
                    |value| value,
                    |value| value,
                )?;
            }
            if let Some(track) = source.rotation.as_ref() {
                transform.rotation = decoder.track_property(
                    track,
                    Quaternion::default(),
                    id.rotation(),
                    "texture_transform.rotation",
                    Quaternion,
                    Quaternion,
                )?;
            }
            if let Some(track) = source.scaling.as_ref() {
                transform.scale = decoder.track_property(
                    track,
                    [1.; 3],
                    id.scale(),
                    "texture_transform.scale",
                    |value| value,
                    |value| value,
                )?;
            }
            decoder.builder.set_texture_transform(*id, transform)?;
        }
    }
    // Geometry and complete feature payloads are decoded by focused domain adapters.
    super::geometry::read::<_, S>(
        source,
        &mut builder,
        (&identity, &ids),
        &material_ids,
        &clips,
        &globals,
        &mut native_profile,
    )?;
    super::scene::read::<_, S>(
        source,
        &mut builder,
        &identity,
        &texture_ids,
        &material_ids,
        &globals,
        &mut native_profile,
    )?;
    for (id, controls) in native_profile.textures {
        let mut texture = builder.texture(id).expect("imported texture").clone();
        texture.controls = controls;
        builder.set_texture(id, texture)?;
    }
    for (id, controls) in native_profile.sequences {
        builder.set_clip_controls(id, controls)?;
    }
    for (id, mut controls) in native_profile.geometry {
        controls.glider = native_profile.gliders.contains(&id);
        let mut mesh = builder.model().mesh(id).expect("imported mesh").clone();
        mesh.controls = controls;
        builder.set_mesh(id, mesh)?;
    }
    for id in ids {
        let controls = profile::NodeControls {
            unmapped_flags: native_profile
                .node_controls
                .iter()
                .find(|(key, _)| *key == id)
                .map_or(0, |(_, value)| *value)
                .into(),
            bone: native_profile
                .bones
                .iter()
                .find(|(key, _)| *key == id)
                .map(|(_, value)| value.clone()),
            bind_pose: native_profile
                .bind_poses
                .iter()
                .find(|(key, _)| *key == id)
                .map(|(_, value)| *value),
        };
        let mut node = builder.model().node(id).expect("imported node").clone();
        node.controls = controls;
        builder.set_node(id, node)?;
    }
    builder.set_metadata(native_profile.metadata);
    let asset = builder.finish()?;
    Ok(Conversion {
        value: asset,
        report: context.report,
    })
}
