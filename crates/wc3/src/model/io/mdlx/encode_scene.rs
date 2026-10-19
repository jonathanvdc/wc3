//! Attached logical features mapped directly to native Warcraft III record families.
use super::encode_animation::Encoder;
use crate::model::conversion::Error;
use crate::model::formats::mdlx::{self as native, Dialect};
use crate::model::materials::mdlx::AlphaAdditiveEncoding;
use crate::model::profiles::mdlx as profile;
use crate::model::profiles::Mdlx;
use crate::model::{
    Axis, Billboard, FeatureId, Inheritance, MaterialId, MeshId, Model, NodeId, Quat, Texture,
};
use crate::model::{MdlxImageResource, MdlxParticle1Resource, PathResource};
use glam::Vec3;
use native::scene::Node;
use std::collections::{BTreeMap, BTreeSet};
use std::str::from_utf8;

pub(super) fn write<'a, T: Dialect>(
    target: &mut native::Document<'a, T>,
    encoder: &mut Encoder<'_, '_, '_>,
    nodes: BTreeMap<NodeId, Node<'static>>,
    textures: &BTreeMap<Texture, u32>,
    materials: &BTreeMap<MaterialId, u32>,
    meshes: &BTreeMap<MeshId, Vec<u32>>,
) -> Result<(), Error> {
    let model = encoder.model;
    let mut helpers = Vec::new();
    let mut bones = Vec::new();
    let mut claimed = BTreeSet::new();
    let mut particles = Vec::new();
    let mut particles2 = Vec::new();
    let mut ribbons = Vec::new();
    let mut popcorn = Vec::new();
    let mut attachments = Vec::new();
    let mut collisions = Vec::new();
    let mut animations = Vec::new();
    let mut animation_ids = BTreeMap::<FeatureId, Vec<u32>>::new();
    let mut lights = Vec::new();
    let mut cameras = Vec::new();
    let mut events = Vec::new();
    let mut next = nodes.len() as u32;
    let native_bones: BTreeSet<_> = model
        .nodes()
        .iter()
        .filter_map(|(id, node)| node.controls.bone.is_some().then_some(id))
        .collect();
    for (feature_id, feature) in model.features() {
        let source = &feature.payload;
        if let profile::Node::GeometryAnimation(source) = source {
            let mapped = source
                .mesh
                .map(|mesh| {
                    meshes.get(&mesh).ok_or_else(|| {
                        Error::new("geometry_animation.mesh", "mesh has no native draw")
                    })
                })
                .transpose()?;
            let targets = mapped.cloned().unwrap_or_else(|| vec![u32::MAX]);
            let mut indices = Vec::new();
            for mesh in targets {
                let mut record = native::GeosetAnimation::new(mesh);
                record.alpha = encoder.value(&source.opacity, Ok, "geometry_animation.opacity")?;
                record.color = encoder.value(
                    &source.color,
                    |v| Ok(v.to_array()),
                    "geometry_animation.color",
                )?;
                record.flags = native::GeosetAnimationFlags(0);
                record.flags.set_color(source.use_color);
                record.flags.set_drop_shadow(source.drop_shadow);
                indices.push(animations.len() as u32);
                animations.push(record);
            }
            animation_ids.insert(feature_id, indices);
            continue;
        }
        if let profile::Node::Camera(source) = source {
            cameras.push(super::encode_lights::camera::<T::Version>(
                source,
                feature.node,
                &nodes,
                encoder,
            )?);
            if camera_anchor_only(model, feature.node) {
                claimed.insert(feature.node);
            }
            continue;
        }
        let mut node = nodes[&feature.node].clone();
        let actors = encoder
            .model
            .node_features(feature.node)
            .filter(|(_, feature)| {
                !matches!(
                    feature,
                    profile::Node::GeometryAnimation(_) | profile::Node::Camera(_)
                )
            })
            .count();
        if actors > 1
            || native_bones.contains(&feature.node)
            || model
                .event_sources()
                .values()
                .any(|source| source.node == feature.node)
        {
            node = Node::new(&node.name.text(), next)
                .map_err(|error| Error::new("feature.name", error.to_string()))?;
            node.parent_id = nodes[&feature.node].object_id;
            next += 1;
        } else {
            claimed.insert(feature.node);
        }
        // Object-kind bits belong to the encoded record family, not semantic controls.
        match source {
            profile::Node::Particle(_)
            | profile::Node::Particle2(_)
            | profile::Node::ParticleProgram(_) => node.flags.set_particle_emitter(true),
            profile::Node::Ribbon(_) => node.flags.set_ribbon_emitter(true),
            profile::Node::Attachment(_) => node.flags.set_attachment(true),
            profile::Node::CollisionShape(_) => node.flags.set_collision_shape(true),
            profile::Node::Light(_) => node.flags.set_light(true),
            _ => {}
        }
        match source {
            profile::Node::Particle(source) => {
                let mut record = native::ParticleEmitter::new(
                    node,
                    particle_path(model, source.details.resource)?,
                )
                .map_err(|error| Error::new("particle.resource", error.to_string()))?;
                macro_rules! a {
                    ($($field:ident).+) => {
                        encoder.value(
                            &source.$($field).+,
                            Ok,
                            concat!("particle.", stringify!($($field).+)),
                        )?
                    };
                }
                record.emission_rate = a!(emission.rate);
                record.gravity = a!(motion.gravity);
                record.longitude = a!(details.longitude);
                record.latitude = a!(details.latitude);
                record.life_span = a!(lifetime.duration);
                record.initial_velocity = a!(motion.speed);
                record.visibility = super::encode_helpers::visibility(
                    &source.emission.activation,
                    encoder,
                    "particle.visibility",
                )?;
                let mut flags = record.flags();
                flags.set_emitter_uses_mdl(matches!(
                    source.details.resource,
                    MdlxParticle1Resource::Model(_)
                ));
                flags.set_emitter_uses_tga(source.details.text_resource);
                record.set_flags(flags);
                particles.push(record);
            }
            profile::Node::Particle2(source) => {
                let mut record = native::ParticleEmitter2::new(node);
                macro_rules! a {
                    ($($field:ident).+) => {
                        encoder.value(
                            &source.$($field).+,
                            Ok,
                            concat!("particle2.", stringify!($($field).+)),
                        )?
                    };
                }
                record.speed = a!(motion.speed);
                record.variation = a!(motion.variation);
                record.latitude = a!(details.shape.latitude);
                record.gravity = a!(motion.gravity);
                record.emission_rate = a!(emission.rate);
                record.length = a!(details.shape.length);
                record.width = a!(details.shape.width);
                record.life_span = source.lifetime.duration;
                record.rows = source.details.rendering.atlas[0];
                record.columns = source.details.rendering.atlas[1];
                record.tail_length = source.details.rendering.tail_length;
                record.time = source.appearance.middle_time;
                record.segment_colors = (source.appearance.colors).map(|v| v.to_array());
                record.alpha = source.appearance.alpha;
                record.particle_scaling = (source.appearance.scale).to_array();
                record.uv_animations = source.appearance.uv_intervals;
                record.texture_id = source
                    .details
                    .rendering
                    .texture
                    .map(|id| textures[&id])
                    .unwrap_or(u32::MAX);
                record.set_squirt_enabled(source.emission.trigger);
                record.priority_plane = source.details.rendering.priority_plane;
                record.replaceable_id = source.details.rendering.replaceable;
                record.filter_mode = match source.details.rendering.blend {
                    profile::Blend::Alpha => native::Particle2FilterMode::Blend,
                    profile::Blend::AdditiveAlpha(AlphaAdditiveEncoding::Additive) => {
                        native::Particle2FilterMode::Additive
                    }
                    profile::Blend::Modulate => native::Particle2FilterMode::Modulate,
                    profile::Blend::Modulate2x => native::Particle2FilterMode::Modulate2x,
                    profile::Blend::AlphaKey(()) => native::Particle2FilterMode::AlphaKey,
                    profile::Blend::Unknown(value) => native::Particle2FilterMode::Unknown(value),
                    _ => {
                        encoder.context.unsupported(
                            "particle2.blend",
                            "blend operation has no Warcraft III particle operation",
                        )?;
                        native::Particle2FilterMode::Blend
                    }
                };
                record.frames = match source.details.rendering.frames {
                    profile::Frames::Head => native::Particle2Frames::Head,
                    profile::Frames::Tail => native::Particle2Frames::Tail,
                    profile::Frames::Both => native::Particle2Frames::Both,
                    profile::Frames::Unknown(value) => native::Particle2Frames::Unknown(value),
                };
                let mut flags = record.flags();
                flags.set_unshaded(source.details.rendering.unlit);
                flags.set_sort_prims_far_z(source.details.rendering.sort_far_z);
                flags.set_line_emitter(source.details.shape.line_emitter);
                flags.set_unfogged(source.details.rendering.unfogged);
                flags.set_model_space(source.details.model_space);
                flags.set_xy_quad(source.details.rendering.xy_quad);
                record.set_flags(flags);
                record.visibility = super::encode_helpers::visibility(
                    &source.emission.activation,
                    encoder,
                    "particle2.visibility",
                )?;
                particles2.push(record);
            }
            profile::Node::Ribbon(source) => {
                if source.geometry.axis != Axis::PositiveY {
                    encoder.context.unsupported(
                        "ribbon.geometry.axis",
                        "Warcraft III ribbon cross-sections extend along local positive Y",
                    )?;
                }
                let mut record = native::RibbonEmitter::new(node);
                record.height_above =
                    encoder.value(&source.geometry.height_above, Ok, "ribbon.height_above")?;
                record.height_below =
                    encoder.value(&source.geometry.height_below, Ok, "ribbon.height_below")?;
                record.alpha = encoder.value(&source.appearance.opacity, Ok, "ribbon.opacity")?;
                record.color = encoder.value(
                    &source.appearance.color,
                    |v| Ok(v.to_array()),
                    "ribbon.color",
                )?;
                record.texture_slot =
                    encoder.value(&source.appearance.texture_slot, Ok, "ribbon.texture_slot")?;
                record.life_span = source.lifetime;
                record.emission_rate = source.emission_rate;
                record.rows = source.appearance.atlas[0];
                record.columns = source.appearance.atlas[1];
                record.material_id = source
                    .rendering
                    .material
                    .map(|id| materials[&id])
                    .unwrap_or(u32::MAX);
                record.gravity = source.controls.gravity;
                record.visibility = super::encode_helpers::visibility(
                    &source.controls.visibility,
                    encoder,
                    "ribbon.visibility",
                )?;
                ribbons.push(record);
            }
            profile::Node::ParticleProgram(source) => {
                let mut record = native::PopcornEmitter::new(
                    node,
                    source
                        .resource
                        .map(|id| model.effect_resource(id).expect("validated effect"))
                        .map(path_text)
                        .transpose()?
                        .unwrap_or(""),
                    &source.visibility_guide,
                )
                .map_err(|error| Error::new("popcorn.resource", error.to_string()))?;
                record.life_span = encoder.value(&source.life_span, Ok, "popcorn.life_span")?;
                record.emission_rate =
                    encoder.value(&source.emission_rate, Ok, "popcorn.emission_rate")?;
                record.speed = encoder.value(&source.speed, Ok, "popcorn.speed")?;
                record.color =
                    encoder.value(&source.color, |v| Ok(v.to_array()), "popcorn.color")?;
                record.alpha = encoder.value(&source.opacity, Ok, "popcorn.opacity")?;
                record.replaceable_id = source.replaceable;
                record.visibility = super::encode_helpers::visibility(
                    &source.visibility,
                    encoder,
                    "popcorn.visibility",
                )?;
                let mut flags = record.flags();
                flags.set_unshaded(source.unlit);
                flags.set_sort_prims_far_z(source.sort_far_z);
                flags.set_unfogged(source.unfogged);
                flags.set_popcorn_scaling(source.scaling);
                record.set_flags(flags);
                popcorn.push(record);
            }
            profile::Node::Attachment(source) => {
                let mut record = native::Attachment::new(
                    node,
                    from_utf8(
                        match model
                            .model_resource(source.resource)
                            .expect("validated model")
                        {
                            PathResource::Path(path) => path,
                        },
                    )
                    .map_err(|_| {
                        Error::new("attachment.resource", "attachment path must be UTF-8")
                    })?,
                    source.socket.identity,
                )
                .map_err(|error| Error::new("attachment.resource", error.to_string()))?;
                record.visibility = super::encode_helpers::visibility(
                    &source.socket.visibility,
                    encoder,
                    "attachment.visibility",
                )?;
                attachments.push(record);
            }
            profile::Node::CollisionShape(source) => {
                let record = match source {
                    profile::Collision::Box(points) => {
                        native::CollisionShape::new_box(node, (*points).map(|v| v.to_array()))
                    }
                    profile::Collision::Plane(points) => {
                        native::CollisionShape::new_plane(node, (*points).map(|v| v.to_array()))
                    }
                    profile::Collision::Sphere { center, radius } => {
                        native::CollisionShape::new_sphere(node, (*center).to_array(), *radius)
                    }
                    profile::Collision::Cylinder { endpoints, radius } => {
                        native::CollisionShape::new_cylinder(
                            node,
                            (*endpoints).map(|v| v.to_array()),
                            *radius,
                        )
                    }
                    profile::Collision::Unknown { .. } => {
                        encoder.context.unsupported(
                            "collision.kind",
                            "unknown collision shape has no native encoding",
                        )?;
                        continue;
                    }
                };
                collisions.push(record);
            }
            profile::Node::Light(source) => lights.push(super::encode_lights::light::<T::Version>(
                source, node, encoder,
            )?),
            profile::Node::GeometryAnimation(_) | profile::Node::Camera(_) => {
                unreachable!("handled non-node record family")
            }

            profile::Node::FaceActor(unavailable)
            | profile::Node::LightProgram(unavailable)
            | profile::Node::PointLight(unavailable) => match *unavailable {},
        }
    }
    for (source_id, source) in model.event_sources() {
        let mut node = nodes[&source.node].clone();
        if claimed.contains(&source.node)
            || native_bones.contains(&source.node)
            || model.features().values().any(|feature| {
                feature.node == source.node
                    && !matches!(
                        feature.payload,
                        profile::Node::GeometryAnimation(_) | profile::Node::Camera(_)
                    )
            })
            || model
                .event_sources()
                .values()
                .filter(|other| other.node == source.node)
                .count()
                > 1
        {
            node = Node::new(&node.name.text(), next)
                .map_err(|error| Error::new("event.name", error.to_string()))?;
            node.parent_id = nodes[&source.node].object_id;
            next += 1;
        } else {
            claimed.insert(source.node);
        }
        node.flags.set_event_object(true);
        events.push(super::encode_helpers::events(
            source, source_id, node, encoder,
        )?);
    }
    for (id, mut node) in nodes {
        if claimed.contains(&id) {
            continue;
        }
        if let Some(metadata) = model.node(id).and_then(|node| node.controls.bone.as_ref()) {
            let mesh = metadata
                .mesh
                .map(|id| {
                    meshes
                        .get(&id)
                        .ok_or_else(|| Error::new("bone.mesh", "mesh has no native draw"))
                })
                .transpose()?;
            if mesh.is_some_and(|indices| indices.len() != 1) {
                encoder.context.unsupported(
                    "bone.mesh",
                    "a native bone mesh gate cannot span multiple projected geosets",
                )?;
            }
            let animation = metadata
                .animation
                .map(|id| {
                    animation_ids.get(&id).ok_or_else(|| {
                        Error::new("bone.animation", "geometry animation has no native record")
                    })
                })
                .transpose()?;
            if animation.is_some_and(|indices| indices.len() != 1) {
                encoder.context.unsupported(
                    "bone.animation",
                    "a native bone controller reference cannot span multiple projected geosets",
                )?;
            }
            node.flags.set_bone(true);
            bones.push(native::Bone::new(
                node,
                mesh.and_then(|ids| ids.first())
                    .copied()
                    .unwrap_or(u32::MAX),
                animation
                    .and_then(|ids| ids.first())
                    .copied()
                    .unwrap_or(u32::MAX),
            ));
        } else {
            helpers.push(node);
        }
    }
    target.set_helpers(helpers);
    target.set_bones(bones);
    target.set_particle_emitters(particles);
    target.set_particle_emitters2(particles2);
    target.set_ribbon_emitters(ribbons);
    target.set_attachments(attachments);
    target.set_collision_shapes(collisions);
    target.set_geoset_animations(animations);
    target.set_lights(lights);
    target.set_cameras(cameras);
    target.set_event_objects(events);
    if !popcorn.is_empty() {
        if let Err(error) = target.try_set_popcorn_emitters(popcorn) {
            encoder.context.unsupported("popcorn", error.to_string())?;
        }
    }
    super::encode_helpers::controls(target, encoder, meshes)?;
    Ok(())
}

fn path_text<'a>(resource: &'a PathResource<'_>) -> Result<&'a str, Error> {
    let PathResource::Path(path) = resource;
    from_utf8(path).map_err(|_| {
        Error::new(
            "resources.path",
            "Warcraft III resource paths must be UTF-8",
        )
    })
}
fn particle_path<'a>(
    model: &'a Model<'_, Mdlx>,
    resource: MdlxParticle1Resource,
) -> Result<&'a str, Error> {
    match resource {
        MdlxParticle1Resource::Model(None) | MdlxParticle1Resource::Image(None) => Ok(""),
        MdlxParticle1Resource::Model(Some(id)) => {
            path_text(model.model_resource(id).expect("validated model"))
        }
        MdlxParticle1Resource::Image(Some(id)) => {
            match model.image_resource(id).expect("validated image") {
                MdlxImageResource::Path(path) => from_utf8(path)
                    .map_err(|_| Error::new("particle.resource", "particle path must be UTF-8")),
                _ => Err(Error::new(
                    "particle.resource",
                    "resource particle requires an image path",
                )),
            }
        }
    }
}

/// Camera positions live in the camera record. An otherwise unreferenced camera
/// anchor needs no helper record; retaining one would create an extra hierarchy
/// node on every semantic round trip. Preserve anchors with any other purpose.
fn camera_anchor_only(model: &Model<'_, Mdlx>, id: NodeId) -> bool {
    let node = model.node(id).expect("validated camera anchor");
    let rest = node.transform.defaults(model.values());
    node.parent.is_none()
        && model.values()[node.visible]
        && !model.is_animated(node.visible)
        && node.inheritance == Inheritance::default()
        && node.billboard == Billboard::None
        && !node.camera_anchored
        && node.controls.bone.is_none()
        && node.controls.billboard_flags == Default::default()
        && rest.rotation == Quat::IDENTITY
        && rest.scale == Vec3::from_array([1.; 3])
        && rest.pivot == Vec3::from_array([0.; 3])
        && !model.is_animated(node.transform.translation)
        && !model.is_animated(node.transform.rotation)
        && !model.is_animated(node.transform.scale)
        && !model.nodes().values().any(|other| other.parent == Some(id))
        && !model.skins().values().any(|skin| skin.joints.contains(&id))
        && !model
            .instances()
            .values()
            .any(|instance| instance.node == id)
        && !model.event_sources().values().any(|event| event.node == id)
        && model
            .node_features(id)
            .all(|(_, feature)| matches!(feature, profile::Node::Camera(_)))
}
