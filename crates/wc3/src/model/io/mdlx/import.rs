//! Direct Warcraft III decoding into checked independent model graphs.
use super::storage::Storage;
use super::{curves, materials, state::ImportState};
use crate::model::conversion::{Context, Conversion, Error, ImportOptions};
use crate::model::formats::mdlx::{self as native, Chunk, Dialect, ModelChunk};
use crate::model::profiles::mdlx as profile;
use crate::model::profiles::Mdlx;
use crate::model::{
    AddressMode, Animation, Billboard, Bounds, Clip, DynamicTrack, ImageResource, Inheritance,
    Material, ModelBuilder, Quaternion, Sampler, SharedClock, Texture, Time, TimeSource, Track,
};
use crate::model::{MdlxImageResource, Model, TextureControls};
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
    let mut native_profile = ImportState::default();
    native_profile.metadata.name = source.model_info().map_or(Cow::Borrowed(""), |info| {
        Cow::Owned(info.name.text().into_owned())
    });
    native_profile.metadata.bounds = source.model_info().map(|info| Bounds {
        minimum: info.minimum_extent,
        maximum: info.maximum_extent,
        sphere_radius: info.bounds_radius,
    });
    let mut builder = ModelBuilder::<Mdlx>::with_profile(profile::ModelMetadata::default());
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
        let mut value = builder.new_node(S::text(node.name.text()));
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
        value.billboard = if node.flags.billboarded() {
            Billboard::Spherical
        } else if node.flags.billboard_lock_x()
            || node.flags.billboard_lock_y()
            || node.flags.billboard_lock_z()
        {
            Billboard::Locked {
                enabled: true,
                x: node.flags.billboard_lock_x(),
                y: node.flags.billboard_lock_y(),
                z: node.flags.billboard_lock_z(),
            }
        } else {
            Billboard::None
        };
        value.camera_anchored = node.flags.camera_anchored();
        builder.set_node(*id, value)?;
    }
    let mut texture_ids = Vec::new();
    for texture in source.textures() {
        let resource = if texture.replaceable_id != 0 {
            ImageResource::Replaceable(texture.replaceable_id)
        } else {
            {
                let bytes = texture.path.as_bytes();
                let end = bytes
                    .iter()
                    .position(|byte| *byte == 0)
                    .unwrap_or(bytes.len());
                ImageResource::Path(S::bytes(&bytes[..end]))
            }
        };
        let resource = builder.intern_image_resource(resource.try_into()?);
        texture_ids.push(Texture {
            controls: Some(TextureControls::Mdlx(profile::TextureControls {
                fallback_path: if texture.replaceable_id != 0 && !texture.path.text().is_empty() {
                    let bytes = texture.path.as_bytes();
                    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
                    Some(
                        builder.intern_image_resource(MdlxImageResource::Path(S::bytes(
                            &bytes[..end],
                        ))),
                    )
                } else {
                    None
                },
                unclassified: (texture.flags.bits() & !3).into(),
            })),
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
        });
    }
    let transforms: Vec<_> = source.texture_animations().collect();
    let mut material_ids = Vec::new();
    for material in source.materials() {
        let mut surface =
            materials::read::<_, S>(material, &texture_ids, &transforms, builder.values_mut())?;
        if surface.has_unexecuted_declarations() {
            context.unsupported(
                "material.empty_declarations",
                "nondefault declarations on an empty material require the native codec",
            )?;
            surface = Default::default();
        }
        let value = Material::<Mdlx> {
            name: Cow::Borrowed(""),
            surface,
        };
        material_ids.push(builder.add_material(value)?);
    }
    let globals: Vec<_> = source
        .global_sequences()
        .iter()
        .map(|duration| {
            builder.add_clock(if *duration == 0 {
                SharedClock::Frozen
            } else {
                SharedClock::Repeating {
                    period: Time::seconds(f64::from(*duration) / 1000.)?,
                }
            })
        })
        .collect::<Result<_, _>>()?;
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
        let mut clip = Clip::named(S::text(sequence.name.text()))
            .with_duration(duration)
            .with_source_start(Time::seconds(f64::from(sequence.interval[0]) / 1000.)?);
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
                        .map_or(TimeSource::Animation, TimeSource::Shared);
                    let channel = DynamicTrack::$variant(Track {
                        target: $property,
                        curve: curves::read(track, 0., $map)?,
                        clock,
                    });
                    builder.add_global_track(channel)?;
                }
            };
        }
        channel!(
            node.translation,
            builder.model().node(*id).unwrap().transform.translation,
            Vector3,
            |value| value
        );
        channel!(
            node.rotation,
            builder.model().node(*id).unwrap().transform.rotation,
            Rotation,
            Quaternion
        );
        channel!(
            node.scaling,
            builder.model().node(*id).unwrap().transform.scale,
            Vector3,
            |value| value
        );
    }
    {
        let mut decoder = super::animation::Decoder {
            builder: &mut builder,
            globals: &globals,
        };
        for (source, id) in source.materials().zip(&material_ids) {
            // Empty declarations were already checked by the import loss policy.
            // There are no shader inputs to bind or reread from the native header.
            if source.layers.is_empty() {
                continue;
            }
            let surface = materials::bind::<_, S>(source, &texture_ids, &transforms, &mut decoder)?;
            decoder.builder.set_material_surface(*id, surface)?;
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
    for (id, controls) in native_profile.sequences {
        builder.set_clip_controls(id, controls)?;
    }
    for (id, mut controls) in native_profile.geometry {
        controls.glider = native_profile.gliders.contains(&id);
        let mut mesh = builder.model().mesh(id).expect("imported mesh").clone();
        mesh.controls = controls;
        builder.set_mesh(id, mesh)?;
    }
    for (id, source_node) in ids.into_iter().zip(source_nodes.iter()) {
        let controls = profile::NodeControls {
            spherical_axis_locks: if source_node.flags.billboarded() {
                profile::BillboardAxisLocks {
                    x: source_node.flags.billboard_lock_x(),
                    y: source_node.flags.billboard_lock_y(),
                    z: source_node.flags.billboard_lock_z(),
                }
            } else {
                Default::default()
            },
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
    // Native ordinary tracks share one authored timeline. Only independent clocks
    // remain model-selected; clips explicitly select the ordinary animation.
    let globals = builder.model().global_animations().to_vec();
    builder.set_global_animations(Vec::new());
    for id in globals {
        let source = builder.remove_animation(id).expect("imported animation");
        let mut ordinary = Animation::default();
        let mut shared = Animation::default();
        for channel in source.tracks {
            match channel.clock() {
                TimeSource::Animation => ordinary.tracks.push(channel),
                TimeSource::Shared(_) => shared.tracks.push(channel),
            }
        }
        for (id, event) in source.events {
            match event.clock {
                TimeSource::Animation => {
                    ordinary.events.insert(id, event);
                }
                TimeSource::Shared(_) => {
                    shared.events.insert(id, event);
                }
            }
        }
        if !shared.tracks.is_empty() || !shared.events.is_empty() {
            let animation = builder.add_animation(shared);
            builder.add_global_animation(animation)?;
        }
        if !ordinary.tracks.is_empty() || !ordinary.events.is_empty() {
            let animation = builder.add_animation(ordinary);
            for &(clip, _) in &clips {
                let mut selection = builder.model().clip(clip).expect("imported clip").clone();
                selection.animations.push(animation);
                builder.set_clip(clip, selection)?;
            }
        }
    }
    builder.set_metadata(native_profile.metadata);
    let asset = builder.finish()?;
    Ok(Conversion {
        value: asset,
        report: context.report,
    })
}
