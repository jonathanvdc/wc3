//! Complete logical Warcraft III scene payloads read directly from native records.
use super::animation::Decoder;
use super::state::ImportState;
use super::storage::Storage;
use crate::model::conversion::Error;
use crate::model::formats::mdlx::{self as native, Dialect};
use crate::model::materials::mdlx::AlphaAdditiveEncoding;
use crate::model::profiles::mdlx as profile;
use crate::model::profiles::Mdlx;
use crate::model::{Axis, SocketDefinition};
use crate::model::{
    ClockId, EventOccurrence, MaterialId, ModelBuilder, NodeId, Texture, Time, TimeSource,
};
use crate::model::{MdlxImageResource, MdlxParticle1Resource, PathResource};
use crate::model::{
    MdlxParticle1Details, MdlxParticle2Appearance, MdlxParticle2Details, MdlxParticle2Rendering,
    MdlxParticle2Shape, ParticleEmission, ParticleLifetime, ParticleMotion,
};
use crate::model::{MdlxRibbonControls, MdlxRibbonRendering, TrailAppearance, TrailGeometry};
use glam::Vec3;
use std::collections::BTreeMap;

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

        macro_rules! a {
            ($field:ident) => {
                decoder.bind(
                    &source.$field,
                    concat!("particle.", stringify!($field)),
                    |value| value.into(),
                )?
            };
        }
        let flags = source.flags();
        {
            let payload = profile::Node::Particle(profile::ParticleEmitter1 {
                details: MdlxParticle1Details {
                    resource: if flags.emitter_uses_mdl() {
                        MdlxParticle1Resource::Model((!source.path.text().is_empty()).then(|| {
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
                        MdlxParticle1Resource::Image((!source.path.text().is_empty()).then(|| {
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
                    longitude: a!(longitude),
                    latitude: a!(latitude),
                    text_resource: flags.emitter_uses_tga(),
                },
                emission: ParticleEmission {
                    rate: a!(emission_rate),
                    activation: decoder.track(
                        source.visibility.as_ref(),
                        1.,
                        "particle::VISIBILITY",
                        |value| value,
                    )?,
                    variation: (),
                    trigger: (),
                },
                motion: ParticleMotion {
                    gravity: a!(gravity),
                    speed: a!(initial_velocity),
                    variation: (),
                },
                lifetime: ParticleLifetime {
                    duration: a!(life_span),
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

        macro_rules! a {
            ($field:ident) => {
                decoder.bind(
                    &source.$field,
                    concat!("particle2.", stringify!($field)),
                    |value| value.into(),
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
            let payload = profile::Node::Particle2(profile::ParticleEmitter2 {
                motion: ParticleMotion {
                    speed: a!(speed),
                    variation: a!(variation),
                    gravity: a!(gravity),
                },
                details: MdlxParticle2Details {
                    shape: MdlxParticle2Shape {
                        latitude: a!(latitude),
                        length: a!(length),
                        width: a!(width),
                        line_emitter: flags.line_emitter(),
                    },
                    rendering: MdlxParticle2Rendering {
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
                },
                lifetime: ParticleLifetime {
                    duration: source.life_span,
                    variation: (),
                },
                emission: ParticleEmission {
                    rate: a!(emission_rate),
                    trigger: source.squirt_enabled(),
                    activation: decoder.track(
                        source.visibility.as_ref(),
                        1.,
                        "particle2::VISIBILITY",
                        |value| value,
                    )?,
                    variation: (),
                },
                appearance: MdlxParticle2Appearance {
                    middle_time: source.time,
                    colors: (source.segment_colors).map(Vec3::from_array),
                    alpha: source.alpha,
                    scale: Vec3::from_array(source.particle_scaling),
                    uv_intervals: source.uv_animations,
                },
            });
            decoder.builder.set_feature(feature, payload)?;
        }
    }
    for source in source.ribbon_emitters() {
        let id = node(source.node.object_id)?;
        let feature = decoder.builder.reserve_feature(id);

        macro_rules! a {
            ($field:ident) => {
                decoder.bind(
                    &source.$field,
                    concat!("ribbon.", stringify!($field)),
                    |value| value.into(),
                )?
            };
        }
        {
            let payload = profile::Node::Ribbon(profile::RibbonEmitter {
                geometry: TrailGeometry {
                    axis: Axis::PositiveY,
                    height_above: a!(height_above),
                    height_below: a!(height_below),
                },
                appearance: TrailAppearance {
                    opacity: a!(alpha),
                    color: a!(color),
                    texture_slot: a!(texture_slot),
                    atlas: [source.rows, source.columns],
                },
                lifetime: source.life_span,
                emission_rate: source.emission_rate,
                rendering: MdlxRibbonRendering {
                    material: if source.material_id == u32::MAX {
                        None
                    } else {
                        Some(*materials.get(source.material_id as usize).ok_or_else(|| {
                            Error::new("ribbon.material", "material identity does not exist")
                        })?)
                    },
                },
                controls: MdlxRibbonControls {
                    gravity: source.gravity,
                    visibility: decoder.track(
                        source.visibility.as_ref(),
                        1.,
                        "ribbon::VISIBILITY",
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

            macro_rules! a {
                ($field:ident) => {
                    decoder.bind(
                        &source.$field,
                        concat!("popcorn.", stringify!($field)),
                        |value| value.into(),
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
                    life_span: a!(life_span),
                    emission_rate: a!(emission_rate),
                    speed: a!(speed),
                    color: a!(color),
                    opacity: a!(alpha),
                    replaceable: source.replaceable_id,
                    visibility: decoder.track(
                        source.visibility.as_ref(),
                        1.,
                        "popcorn::VISIBILITY",
                        |value| value,
                    )?,
                    unlit: flags.unshaded(),
                    sort_far_z: flags.sort_prims_far_z(),
                    unfogged: flags.unfogged(),
                    scaling: flags.popcorn_scaling(),
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
                        "attachment::VISIBILITY",
                        |value| value,
                    )?,
                },
            });
            decoder.builder.set_feature(feature, payload)?;
        }
    }
    for source in source.event_objects() {
        let id = node(source.node.object_id)?;
        let clock = if source.global_sequence_id == u32::MAX {
            TimeSource::Animation
        } else {
            TimeSource::Shared(
                *globals
                    .get(source.global_sequence_id as usize)
                    .ok_or_else(|| Error::new("events.clock", "shared clock does not exist"))?,
            )
        };
        let event_source = decoder.builder.add_event_source(profile::EventSource {
            node: id,
            clock,
            name: S::text(source.node.name.text()),
        });
        for frame in source.frames().iter() {
            let absolute = f64::from(frame) / 1000.;
            decoder.builder.add_global_event(EventOccurrence {
                source: event_source,
                time: Time::seconds(absolute)?,
            })?;
        }
    }
    for source in source.collision_shapes() {
        let id = node(source.node.object_id)?;
        let feature = decoder.builder.reserve_feature(id);
        let shape = match source.geometry {
            native::CollisionGeometry::Box(vertices) => {
                profile::Collision::Box((vertices).map(Vec3::from_array))
            }
            native::CollisionGeometry::Plane(vertices) => {
                profile::Collision::Plane((vertices).map(Vec3::from_array))
            }
            native::CollisionGeometry::Sphere(center, radius) => profile::Collision::Sphere {
                center: Vec3::from_array(center),
                radius,
            },
            native::CollisionGeometry::Cylinder(endpoints, radius) => {
                profile::Collision::Cylinder {
                    endpoints: endpoints.map(Vec3::from_array),
                    radius,
                }
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
