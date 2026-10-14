//! Complete logical Warcraft III scene payloads read directly from native records.
use super::animation::Decoder;
use super::state::ImportState;
use crate::model::conversion::Error;
use crate::model::mdlx::{self as native, ModelDialect};
use crate::model::semantic::profiles::mdlx as profile;
use crate::model::semantic::profiles::Mdlx;
use crate::model::semantic::{
    Clock, ClockId, Event, MaterialId, ModelBuilder, NodeId, TextureId, Time, Values,
};
use std::collections::BTreeMap;

pub(super) fn read<'a, D: ModelDialect>(
    source: &'a native::Model<'_, D>,
    builder: &mut ModelBuilder<'a, Mdlx>,
    ids: &BTreeMap<u32, NodeId>,
    textures: &[TextureId],
    materials: &[MaterialId],
    globals: &[ClockId],
    profile: &mut ImportState<'a>,
) -> Result<(), Error> {
    let node = |id| {
        ids.get(&id)
            .copied()
            .ok_or_else(|| Error::new("feature.node", "node identity does not exist"))
    };
    let texture = |index: u32| {
        if index == u32::MAX {
            Ok(None)
        } else {
            textures
                .get(index as usize)
                .copied()
                .map(Some)
                .ok_or_else(|| Error::new("emitter.texture", "texture identity does not exist"))
        }
    };
    let mut decoder = Decoder { builder, globals };
    for source in source.particle_emitters() {
        let id = node(source.node.object_id)?;
        let feature = decoder.builder.reserve_feature(id);
        let owner = feature;
        macro_rules! a {
            ($field:ident, $key:ident) => {
                decoder.bind(
                    &source.$field,
                    owner,
                    profile::properties::particle::$key,
                    |value| value,
                )?
            };
        }
        let flags = source.flags();
        {
            let payload = profile::Node::Particle(profile::Particle {
                resource: source.path.text(),
                emission_rate: a!(emission_rate, EMISSION_RATE),
                gravity: a!(gravity, GRAVITY),
                longitude: a!(longitude, LONGITUDE),
                latitude: a!(latitude, LATITUDE),
                life_span: a!(life_span, LIFE_SPAN),
                initial_velocity: a!(initial_velocity, INITIAL_VELOCITY),
                visibility: decoder.track(
                    source.visibility.as_ref(),
                    1.,
                    owner,
                    profile::properties::particle::VISIBILITY,
                    |value| value,
                )?,
                model_resource: flags.emitter_uses_mdl(),
                text_resource: flags.emitter_uses_tga(),
                unclassified: Default::default(),
            });
            decoder.builder.set_feature(feature, payload)?;
        }
    }
    for source in source.particle_emitters2() {
        let id = node(source.node.object_id)?;
        let feature = decoder.builder.reserve_feature(id);
        let owner = feature;
        macro_rules! a {
            ($field:ident, $key:ident) => {
                decoder.bind(
                    &source.$field,
                    owner,
                    profile::properties::particle2::$key,
                    |value| value,
                )?
            };
        }
        let flags = source.flags();
        let blend = match source.filter_mode {
            native::Particle2FilterMode::Blend => profile::Blend::Alpha,
            native::Particle2FilterMode::Additive => profile::Blend::Additive,
            native::Particle2FilterMode::Modulate => profile::Blend::Modulate,
            native::Particle2FilterMode::Modulate2x => profile::Blend::Modulate2x,
            native::Particle2FilterMode::AlphaKey => profile::Blend::AlphaKey,
            native::Particle2FilterMode::Unknown(value) => profile::Blend::Unknown(value),
        };
        let frames = match source.frames {
            native::Particle2Frames::Head => profile::Frames::Head,
            native::Particle2Frames::Tail => profile::Frames::Tail,
            native::Particle2Frames::Both => profile::Frames::Both,
            native::Particle2Frames::Unknown(value) => profile::Frames::Unknown(value),
        };
        {
            let payload = profile::Node::Particle2(profile::Particle2 {
                speed: a!(speed, SPEED),
                variation: a!(variation, VARIATION),
                latitude: a!(latitude, LATITUDE),
                gravity: a!(gravity, GRAVITY),
                life_span: source.life_span,
                emission_rate: a!(emission_rate, EMISSION_RATE),
                length: a!(length, LENGTH),
                width: a!(width, WIDTH),
                blend,
                atlas: [source.rows, source.columns],
                frames,
                tail_length: source.tail_length,
                middle_time: source.time,
                colors: source.segment_colors,
                alpha: source.alpha,
                scale: source.particle_scaling,
                uv_intervals: source.uv_animations,
                texture: texture(source.texture_id)?,
                squirt: source.squirt_enabled(),
                priority_plane: source.priority_plane,
                replaceable: source.replaceable_id,
                visibility: decoder.track(
                    source.visibility.as_ref(),
                    1.,
                    owner,
                    profile::properties::particle2::VISIBILITY,
                    |value| value,
                )?,
                model_space: flags.model_space(),
                xy_quad: flags.xy_quad(),
                line_emitter: flags.line_emitter(),
                sort_far_z: flags.sort_prims_far_z(),
                unlit: flags.unshaded(),
                unfogged: flags.unfogged(),
                unclassified: Default::default(),
            });
            decoder.builder.set_feature(feature, payload)?;
        }
    }
    for source in source.ribbon_emitters() {
        let id = node(source.node.object_id)?;
        let feature = decoder.builder.reserve_feature(id);
        let owner = feature;
        macro_rules! a {
            ($field:ident, $key:ident) => {
                decoder.bind(
                    &source.$field,
                    owner,
                    profile::properties::ribbon::$key,
                    |value| value,
                )?
            };
        }
        {
            let payload = profile::Node::Ribbon(profile::Ribbon {
                height_above: a!(height_above, HEIGHT_ABOVE),
                height_below: a!(height_below, HEIGHT_BELOW),
                opacity: a!(alpha, OPACITY),
                color: a!(color, COLOR),
                life_span: source.life_span,
                texture_slot: a!(texture_slot, TEXTURE_SLOT),
                emission_rate: source.emission_rate,
                atlas: [source.rows, source.columns],
                material: if source.material_id == u32::MAX {
                    None
                } else {
                    Some(*materials.get(source.material_id as usize).ok_or_else(|| {
                        Error::new("ribbon.material", "material identity does not exist")
                    })?)
                },
                gravity: source.gravity,
                visibility: decoder.track(
                    source.visibility.as_ref(),
                    1.,
                    owner,
                    profile::properties::ribbon::VISIBILITY,
                    |value| value,
                )?,
            });
            decoder.builder.set_feature(feature, payload)?;
        }
    }
    if let Ok(emitters) = source.try_popcorn_emitters() {
        for source in emitters {
            let id = node(source.node.object_id)?;
            let feature = decoder.builder.reserve_feature(id);
            let owner = feature;
            macro_rules! a {
                ($field:ident, $key:ident) => {
                    decoder.bind(
                        &source.$field,
                        owner,
                        profile::properties::popcorn::$key,
                        |value| value,
                    )?
                };
            }
            let flags = source.flags();
            {
                let payload = profile::Node::Popcorn(profile::Popcorn {
                    resource: source.path.text(),
                    visibility_guide: source.visibility_guide.text(),
                    life_span: a!(life_span, LIFE_SPAN),
                    emission_rate: a!(emission_rate, EMISSION_RATE),
                    speed: a!(speed, SPEED),
                    color: a!(color, COLOR),
                    opacity: a!(alpha, OPACITY),
                    replaceable: source.replaceable_id,
                    visibility: decoder.track(
                        source.visibility.as_ref(),
                        1.,
                        owner,
                        profile::properties::popcorn::VISIBILITY,
                        |value| value,
                    )?,
                    unlit: flags.unshaded(),
                    sort_far_z: flags.sort_prims_far_z(),
                    unfogged: flags.unfogged(),
                    scaling: flags.popcorn_scaling(),
                    unclassified: Default::default(),
                });
                decoder.builder.set_feature(feature, payload)?;
            }
        }
    }
    for source in source.attachments() {
        let id = node(source.node.object_id)?;
        let feature = decoder.builder.reserve_feature(id);
        {
            let payload = profile::Node::Attachment(profile::Attachment {
                resource: source.path.text(),
                id: source.id,
                visibility: decoder.track(
                    source.visibility.as_ref(),
                    1.,
                    feature,
                    profile::properties::attachment::VISIBILITY,
                    |value| value,
                )?,
            });
            decoder.builder.set_feature(feature, payload)?;
        }
    }
    for source in source.event_objects() {
        let id = node(source.node.object_id)?;
        let feature = decoder.builder.reserve_feature(id);
        let clock = if source.global_sequence_id == u32::MAX {
            None
        } else {
            Some(
                *globals
                    .get(source.global_sequence_id as usize)
                    .ok_or_else(|| Error::new("events.clock", "shared clock does not exist"))?,
            )
        };
        for frame in source.frames().iter() {
            let absolute = f64::from(frame) / 1000.;
            let event = |time, clock| Event {
                time,
                name: source.node.name.text(),
                node: Some(id),
                feature: Some(feature),
                clock,
                payload: Values::default(),
            };
            if let Some(clock) = clock {
                decoder
                    .builder
                    .add_shared_event(event(Time::seconds(absolute)?, Clock::Shared(clock)))?;
            } else {
                decoder
                    .builder
                    .add_timeline_event(event(Time::seconds(absolute)?, Clock::Timeline))?;
            }
        }
        {
            let payload = profile::Node::Events(profile::Events {
                name: source.node.name.text(),
            });
            decoder.builder.set_feature(feature, payload)?;
        }
    }
    for source in source.collision_shapes() {
        let id = node(source.node.object_id)?;
        let feature = decoder.builder.reserve_feature(id);
        let shape = match source.geometry {
            native::CollisionGeometry::Box(vertices) => profile::Collision::Box(vertices),
            native::CollisionGeometry::Plane(vertices) => profile::Collision::Plane(vertices),
            native::CollisionGeometry::Sphere(center, radius) => {
                profile::Collision::Sphere { center, radius }
            }
            native::CollisionGeometry::Cylinder(endpoints, radius) => {
                profile::Collision::Cylinder { endpoints, radius }
            }
        };
        {
            let payload = profile::Node::Collision(shape);
            decoder.builder.set_feature(feature, payload)?;
        }
    }
    super::scene_lights::read(source, &mut decoder, ids, profile)?;
    Ok(())
}
