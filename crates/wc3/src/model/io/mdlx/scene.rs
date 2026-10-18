//! Complete logical Warcraft III scene payloads read directly from native records.
use super::animation::Decoder;
use super::state::ImportState;
use super::storage::Storage;
use crate::model::conversion::Error;
use crate::model::formats::mdlx::{self as native, Dialect};
use crate::model::materials::mdlx::AlphaAdditiveEncoding;
use crate::model::profiles::mdlx as profile;
use crate::model::profiles::Mdlx;
use crate::model::{
    AtlasParticle, AtlasParticleRendering, AtlasParticleShape, ModelParticle, ParticleEmission,
    ParticleLifetime, ParticleMotion, StagedParticleAppearance,
};
use crate::model::{Axis, SocketDefinition};
use crate::model::{ClockId, Event, MaterialId, ModelBuilder, NodeId, Texture, Time, TimeSource};
use crate::model::{GravityTrailControls, LayeredTrailRendering, TrailAppearance, TrailGeometry};
use crate::model::{MdlxImageResource, ParticleResource, PathResource};
use std::collections::BTreeMap;
use zerovec::ZeroVec;

pub(super) fn read<'source, 'a, D: Dialect, S: Storage<'source, 'a>>(
    source: &'source native::Document<'_, D>,
    builder: &mut ModelBuilder<'a, Mdlx>,
    ids: &BTreeMap<u32, NodeId>,
    textures: &[Texture],
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
                details: ModelParticle {
                    resource: if flags.emitter_uses_mdl() {
                        ParticleResource::Model((!source.path.text().is_empty()).then(|| {
                            decoder
                                .builder
                                .add_model_resource(PathResource::Path(S::bytes(
                                    source
                                        .path
                                        .as_bytes()
                                        .split(|byte| *byte == 0)
                                        .next()
                                        .unwrap_or_default(),
                                )))
                        }))
                    } else {
                        ParticleResource::Image((!source.path.text().is_empty()).then(|| {
                            decoder
                                .builder
                                .add_image_resource(MdlxImageResource::Path(S::bytes(
                                    source
                                        .path
                                        .as_bytes()
                                        .split(|byte| *byte == 0)
                                        .next()
                                        .unwrap_or_default(),
                                )))
                        }))
                    },
                    longitude: a!(longitude, LONGITUDE),
                    latitude: a!(latitude, LATITUDE),
                    text_resource: flags.emitter_uses_tga(),
                    unclassified: Default::default(),
                },
                emission: ParticleEmission {
                    rate: a!(emission_rate, EMISSION_RATE),
                    activation: decoder.track(
                        source.visibility.as_ref(),
                        1.,
                        owner,
                        profile::properties::particle::VISIBILITY,
                        |value| value,
                    )?,
                    variation: (),
                    trigger: (),
                },
                motion: ParticleMotion {
                    gravity: a!(gravity, GRAVITY),
                    speed: a!(initial_velocity, INITIAL_VELOCITY),
                    variation: (),
                },
                lifetime: ParticleLifetime {
                    duration: a!(life_span, LIFE_SPAN),
                    variation: (),
                },
                appearance: (),
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
            native::Particle2FilterMode::Additive => {
                profile::Blend::AdditiveAlpha(AlphaAdditiveEncoding::Additive)
            }
            native::Particle2FilterMode::Modulate => profile::Blend::Modulate,
            native::Particle2FilterMode::Modulate2x => profile::Blend::Modulate2x,
            native::Particle2FilterMode::AlphaKey => profile::Blend::AlphaKey(()),
            native::Particle2FilterMode::Unknown(value) => profile::Blend::Unknown(value),
        };
        let frames = match source.frames {
            native::Particle2Frames::Head => profile::Frames::Head,
            native::Particle2Frames::Tail => profile::Frames::Tail,
            native::Particle2Frames::Both => profile::Frames::Both,
            native::Particle2Frames::Unknown(value) => profile::Frames::Unknown(value),
        };
        {
            let payload = profile::Node::AtlasParticles(profile::Particle2 {
                motion: ParticleMotion {
                    speed: a!(speed, SPEED),
                    variation: a!(variation, VARIATION),
                    gravity: a!(gravity, GRAVITY),
                },
                details: AtlasParticle {
                    shape: AtlasParticleShape {
                        latitude: a!(latitude, LATITUDE),
                        length: a!(length, LENGTH),
                        width: a!(width, WIDTH),
                        line_emitter: flags.line_emitter(),
                    },
                    rendering: AtlasParticleRendering {
                        blend,
                        atlas: [source.rows, source.columns],
                        frames,
                        tail_length: source.tail_length,
                        texture: texture(source.texture_id)?,
                        priority_plane: source.priority_plane,
                        replaceable: source.replaceable_id,
                        xy_quad: flags.xy_quad(),
                        sort_far_z: flags.sort_prims_far_z(),
                        unlit: flags.unshaded(),
                        unfogged: flags.unfogged(),
                    },
                    model_space: flags.model_space(),
                    unclassified: Default::default(),
                },
                lifetime: ParticleLifetime {
                    duration: source.life_span,
                    variation: (),
                },
                emission: ParticleEmission {
                    rate: a!(emission_rate, EMISSION_RATE),
                    trigger: source.squirt_enabled(),
                    activation: decoder.track(
                        source.visibility.as_ref(),
                        1.,
                        owner,
                        profile::properties::particle2::VISIBILITY,
                        |value| value,
                    )?,
                    variation: (),
                },
                appearance: StagedParticleAppearance {
                    middle_time: source.time,
                    colors: source.segment_colors,
                    alpha: source.alpha,
                    scale: source.particle_scaling,
                    uv_intervals: source.uv_animations,
                },
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
                geometry: TrailGeometry {
                    axis: Axis::PositiveY,
                    height_above: a!(height_above, HEIGHT_ABOVE),
                    height_below: a!(height_below, HEIGHT_BELOW),
                },
                appearance: TrailAppearance {
                    opacity: a!(alpha, OPACITY),
                    color: a!(color, COLOR),
                    texture_slot: a!(texture_slot, TEXTURE_SLOT),
                    atlas: [source.rows, source.columns],
                },
                lifetime: source.life_span,
                emission_rate: source.emission_rate,
                rendering: LayeredTrailRendering {
                    material: if source.material_id == u32::MAX {
                        None
                    } else {
                        Some(*materials.get(source.material_id as usize).ok_or_else(|| {
                            Error::new("ribbon.material", "material identity does not exist")
                        })?)
                    },
                },
                controls: GravityTrailControls {
                    gravity: source.gravity,
                    visibility: decoder.track(
                        source.visibility.as_ref(),
                        1.,
                        owner,
                        profile::properties::ribbon::VISIBILITY,
                        |value| value,
                    )?,
                },
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
                let payload = profile::Node::ParticleProgram(profile::Popcorn {
                    resource: if source.path.text().is_empty() {
                        None
                    } else {
                        Some(
                            decoder
                                .builder
                                .add_effect_resource(PathResource::Path(S::bytes(
                                    source
                                        .path
                                        .as_bytes()
                                        .split(|byte| *byte == 0)
                                        .next()
                                        .unwrap_or_default(),
                                ))),
                        )
                    },
                    visibility_guide: S::text(source.visibility_guide.text()),
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
                resource: decoder
                    .builder
                    .add_model_resource(PathResource::Path(S::bytes(
                        source
                            .path
                            .as_bytes()
                            .split(|byte| *byte == 0)
                            .next()
                            .unwrap_or_default(),
                    ))),
                appearance: (),
                socket: SocketDefinition {
                    identity: source.id,
                    control: (),
                    visibility: decoder.track(
                        source.visibility.as_ref(),
                        1.,
                        feature,
                        profile::properties::attachment::VISIBILITY,
                        |value| value,
                    )?,
                },
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
                name: S::text(source.node.name.text()),
                node: Some(id),
                feature: Some(feature),
                clock,
                payload: ZeroVec::new(),
            };
            if let Some(clock) = clock {
                decoder
                    .builder
                    .add_global_event(event(Time::seconds(absolute)?, TimeSource::Shared(clock)))?;
            } else {
                decoder
                    .builder
                    .add_global_event(event(Time::seconds(absolute)?, TimeSource::Timeline))?;
            }
        }
        {
            let payload = profile::Node::Event(profile::Events {
                name: S::text(source.node.name.text()),
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
            let payload = profile::Node::CollisionShape(shape);
            decoder.builder.set_feature(feature, payload)?;
        }
    }
    super::scene_lights::read::<_, S>(source, &mut decoder, ids, profile)?;
    Ok(())
}
