//! Direct Warcraft actor decoding and encoding without document-backed logical values.
use super::super::text;
use super::{read_asset, write_asset};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::actors::warcraft::properties;
use crate::model::semantic::profiles::wem::actors::warcraft::{
    Feature, FrameInterval, LifeStage, ParticleBlend, ParticleGeometry, Popcorn, ResourceParticles,
    Ribbon, SpriteParticles,
};
use crate::model::semantic::{
    Animated, FeatureId, MaterialSlotId, Property, PropertyTarget, TextureId,
};
use crate::model::wem::{records, RawDocument};
use std::collections::BTreeMap;

pub(in crate::model::conversion::wem) fn read<'a, D: records::NodePayloadDialect>(
    document: &'a RawDocument<'_>,
    payload: &records::NodePayload<D>,
    textures: &[TextureId],
    slots: &[MaterialSlotId],
) -> Result<Option<Feature<'a>>, Error> {
    Ok(Some(match payload {
        records::NodePayload::Wc3ParticleEmitter1(value) => {
            Feature::ResourceParticles(ResourceParticles {
                emission_rate: Animated::constant(value.emission_rate),
                gravity: Animated::constant(value.gravity),
                longitude: Animated::constant(value.longitude),
                latitude: Animated::constant(value.latitude),
                lifespan: Animated::constant(value.lifespan),
                speed: Animated::constant(value.speed),
                uses_mdl: value.uses_mdl.get(),
                uses_tga: value.uses_tga.get(),
                spawn_model: read_asset(document, &value.spawn_model)?,
            })
        }
        records::NodePayload::Wc3ParticleEmitter2(value) => {
            Feature::SpriteParticles(SpriteParticles {
                speed: Animated::constant(value.speed),
                variation: Animated::constant(value.variation),
                latitude: Animated::constant(value.latitude),
                gravity: Animated::constant(value.gravity),
                emission_rate: Animated::constant(value.emission_rate),
                width: Animated::constant(value.width),
                length: Animated::constant(value.length),
                squirt: value.squirt.get(),
                unshaded: value.unshaded.get(),
                sort_far_to_near: value.sort_prims_far_z.get(),
                line_emitter: value.line_emitter.get(),
                unfogged: value.unfogged.get(),
                xy_quad: value.xy_quad.get(),
                lifespan: value.lifespan,
                tail_length: value.tail_length,
                replaceable_id: value.replaceable_id,
                priority_plane: value.priority_plane,
                middle_age: value.time,
                blend: match value.filter {
                    records::Wc3ParticleFilter::Blend => ParticleBlend::AlphaBlend,
                    records::Wc3ParticleFilter::Additive => ParticleBlend::Additive,
                    records::Wc3ParticleFilter::Modulate => ParticleBlend::Multiply,
                    records::Wc3ParticleFilter::Modulate2x => ParticleBlend::Multiply2,
                    records::Wc3ParticleFilter::AlphaKey => ParticleBlend::AlphaTest,
                    records::Wc3ParticleFilter::Unknown(code) => ParticleBlend::Unknown(code),
                },
                geometry: match value.head_or_tail {
                    records::Wc3ParticleHeadOrTail::Head => ParticleGeometry::Head,
                    records::Wc3ParticleHeadOrTail::Tail => ParticleGeometry::Tail,
                    records::Wc3ParticleHeadOrTail::Both => ParticleGeometry::Both,
                    records::Wc3ParticleHeadOrTail::Unknown(code) => {
                        ParticleGeometry::Unknown(code)
                    }
                },
                atlas: [value.rows, value.columns],
                stages: [&value.start, &value.middle, &value.end].map(|stage| LifeStage {
                    color: stage.color,
                    opacity: stage.alpha,
                    scale: stage.scaling,
                }),
                frames: [
                    &value.head_life,
                    &value.head_decay,
                    &value.tail_life,
                    &value.tail_decay,
                ]
                .map(|frame| FrameInterval {
                    first: frame.start,
                    last: frame.end,
                    repetitions: frame.repeat,
                }),
                texture: lookup(textures, value.texture, "particle texture")?,
            })
        }
        records::NodePayload::Wc3RibbonEmitter(value) => Feature::Ribbon(Ribbon {
            height_above: Animated::constant(value.height_above),
            height_below: Animated::constant(value.height_below),
            color: value.color,
            lifespan: value.lifespan,
            texture_slot: value.texture_slot,
            emission_rate: value.emission_rate,
            gravity: value.gravity,
            opacity: value.alpha,
            atlas: [value.rows, value.columns],
            material_slot: lookup(slots, value.material_slot, "ribbon material slot")?,
        }),
        records::NodePayload::Wc3CornEmitter(value) => Feature::Popcorn(Popcorn {
            lifespan: Animated::constant(value.lifespan),
            emission_rate: Animated::constant(value.emission_rate),
            speed: Animated::constant(value.speed),
            unshaded: value.unshaded.get(),
            sort_far_to_near: value.sort_prims_far_z.get(),
            unfogged: value.unfogged.get(),
            scaling: value.popcorn_scaling.get(),
            color: value.color,
            opacity: value.alpha,
            replaceable_id: value.replaceable_id,
            effect: read_asset(document, &value.effect)?,
            animation_visibility: text::read(document, value.anim_visibility_guide)?,
        }),
        records::NodePayload::Wc3FaceFx(value) => {
            Feature::FaceAnimation(read_asset(document, &value.actor)?)
        }
        _ => return Ok(None),
    }))
}
fn lookup<T: Copy>(values: &[T], index: u32, field: &str) -> Result<Option<T>, Error> {
    if index == u32::MAX {
        return Ok(None);
    }
    values
        .get(index as usize)
        .copied()
        .map(Some)
        .ok_or_else(|| {
            Error::new(
                "wem.actor",
                format!("{field} index is outside its declared table"),
            )
        })
}
fn index<T: Ord>(values: &BTreeMap<T, u32>, id: Option<T>, field: &str) -> Result<u32, Error> {
    match id {
        None => Ok(u32::MAX),
        Some(id) => values
            .get(&id)
            .copied()
            .ok_or_else(|| Error::new("wem.actor", format!("{field} has no destination mapping"))),
    }
}
pub(in crate::model::conversion::wem) fn write<D: records::NodePayloadDialect>(
    document: &mut RawDocument<'_>,
    actor: &Feature<'_>,
    textures: &BTreeMap<TextureId, u32>,
    slots: &BTreeMap<MaterialSlotId, u32>,
) -> Result<records::NodePayload<D>, Error> {
    Ok(match actor {
        Feature::ResourceParticles(value) => records::NodePayload::Wc3ParticleEmitter1(Box::new(
            records::Wc3ParticleEmitter1Payload {
                emission_rate: value.emission_rate.default,
                gravity: value.gravity.default,
                longitude: value.longitude.default,
                latitude: value.latitude.default,
                lifespan: value.lifespan.default,
                speed: value.speed.default,
                uses_mdl: value.uses_mdl.into(),
                uses_tga: value.uses_tga.into(),
                spawn_model: write_asset(document, &value.spawn_model)?,
            },
        )),
        Feature::SpriteParticles(value) => records::NodePayload::Wc3ParticleEmitter2(Box::new(
            records::Wc3ParticleEmitter2Payload {
                speed: value.speed.default,
                variation: value.variation.default,
                latitude: value.latitude.default,
                gravity: value.gravity.default,
                emission_rate: value.emission_rate.default,
                width: value.width.default,
                length: value.length.default,
                squirt: value.squirt.into(),
                unshaded: value.unshaded.into(),
                sort_prims_far_z: value.sort_far_to_near.into(),
                line_emitter: value.line_emitter.into(),
                unfogged: value.unfogged.into(),
                xy_quad: value.xy_quad.into(),
                lifespan: value.lifespan,
                tail_length: value.tail_length,
                replaceable_id: value.replaceable_id,
                priority_plane: value.priority_plane,
                time: value.middle_age,
                filter: match value.blend {
                    ParticleBlend::AlphaBlend => records::Wc3ParticleFilter::Blend,
                    ParticleBlend::Additive => records::Wc3ParticleFilter::Additive,
                    ParticleBlend::Multiply => records::Wc3ParticleFilter::Modulate,
                    ParticleBlend::Multiply2 => records::Wc3ParticleFilter::Modulate2x,
                    ParticleBlend::AlphaTest => records::Wc3ParticleFilter::AlphaKey,
                    ParticleBlend::Unknown(code) => records::Wc3ParticleFilter::Unknown(code),
                },
                head_or_tail: match value.geometry {
                    ParticleGeometry::Head => records::Wc3ParticleHeadOrTail::Head,
                    ParticleGeometry::Tail => records::Wc3ParticleHeadOrTail::Tail,
                    ParticleGeometry::Both => records::Wc3ParticleHeadOrTail::Both,
                    ParticleGeometry::Unknown(code) => {
                        records::Wc3ParticleHeadOrTail::Unknown(code)
                    }
                },
                rows: value.atlas[0],
                columns: value.atlas[1],
                texture: index(textures, value.texture, "particle texture")?,
                start: records::Wc3ParticleSegment {
                    color: value.stages[0].color,
                    alpha: value.stages[0].opacity,
                    scaling: value.stages[0].scale,
                },
                middle: records::Wc3ParticleSegment {
                    color: value.stages[1].color,
                    alpha: value.stages[1].opacity,
                    scaling: value.stages[1].scale,
                },
                end: records::Wc3ParticleSegment {
                    color: value.stages[2].color,
                    alpha: value.stages[2].opacity,
                    scaling: value.stages[2].scale,
                },
                head_life: records::Wc3ParticleInterval {
                    start: value.frames[0].first,
                    end: value.frames[0].last,
                    repeat: value.frames[0].repetitions,
                },
                head_decay: records::Wc3ParticleInterval {
                    start: value.frames[1].first,
                    end: value.frames[1].last,
                    repeat: value.frames[1].repetitions,
                },
                tail_life: records::Wc3ParticleInterval {
                    start: value.frames[2].first,
                    end: value.frames[2].last,
                    repeat: value.frames[2].repetitions,
                },
                tail_decay: records::Wc3ParticleInterval {
                    start: value.frames[3].first,
                    end: value.frames[3].last,
                    repeat: value.frames[3].repetitions,
                },
            },
        )),
        Feature::Ribbon(value) => {
            records::NodePayload::Wc3RibbonEmitter(Box::new(records::Wc3RibbonEmitterPayload {
                height_above: value.height_above.default,
                height_below: value.height_below.default,
                color: value.color,
                lifespan: value.lifespan,
                texture_slot: value.texture_slot,
                emission_rate: value.emission_rate,
                gravity: value.gravity,
                alpha: value.opacity,
                rows: value.atlas[0],
                columns: value.atlas[1],
                material_slot: index(slots, value.material_slot, "ribbon material slot")?,
            }))
        }
        Feature::Popcorn(value) => {
            records::NodePayload::Wc3CornEmitter(Box::new(records::Wc3CornEmitterPayload {
                lifespan: value.lifespan.default,
                emission_rate: value.emission_rate.default,
                speed: value.speed.default,
                unshaded: value.unshaded.into(),
                sort_prims_far_z: value.sort_far_to_near.into(),
                unfogged: value.unfogged.into(),
                popcorn_scaling: value.scaling.into(),
                color: value.color,
                alpha: value.opacity,
                replaceable_id: value.replaceable_id,
                effect: write_asset(document, &value.effect)?,
                anim_visibility_guide: text::write_bytes(
                    document,
                    value.animation_visibility.as_bytes(),
                )?,
            }))
        }
        Feature::FaceAnimation(value) => {
            records::NodePayload::Wc3FaceFx(Box::new(records::Wc3FaceFxPayload {
                actor: write_asset(document, value)?,
            }))
        }
    })
}
pub(in crate::model::conversion::wem) fn bind_property(
    actor: &mut Feature<'_>,
    id: FeatureId,
    code: u32,
) -> Result<Option<Property<f32>>, Error> {
    let (field, descriptor) = match actor {
        Feature::ResourceParticles(value) => match records::Wc3Particle1Property::from_raw(code) {
            records::Wc3Particle1Property::EmissionRate => {
                (&mut value.emission_rate, properties::EMISSION_RATE)
            }
            records::Wc3Particle1Property::Gravity => (&mut value.gravity, properties::GRAVITY),
            records::Wc3Particle1Property::Longitude => {
                (&mut value.longitude, properties::LONGITUDE)
            }
            records::Wc3Particle1Property::Latitude => (&mut value.latitude, properties::LATITUDE),
            records::Wc3Particle1Property::Lifespan => (&mut value.lifespan, properties::LIFESPAN),
            records::Wc3Particle1Property::Speed => (&mut value.speed, properties::SPEED),
            _ => return Ok(None),
        },
        Feature::SpriteParticles(value) => match records::Wc3Particle2Property::from_raw(code) {
            records::Wc3Particle2Property::Speed => (&mut value.speed, properties::SPEED),
            records::Wc3Particle2Property::Variation => {
                (&mut value.variation, properties::VARIATION)
            }
            records::Wc3Particle2Property::Latitude => (&mut value.latitude, properties::LATITUDE),
            records::Wc3Particle2Property::Gravity => (&mut value.gravity, properties::GRAVITY),
            records::Wc3Particle2Property::EmissionRate => {
                (&mut value.emission_rate, properties::EMISSION_RATE)
            }
            records::Wc3Particle2Property::Width => (&mut value.width, properties::WIDTH),
            records::Wc3Particle2Property::Length => (&mut value.length, properties::LENGTH),
            _ => return Ok(None),
        },
        Feature::Ribbon(value) => match records::Wc3RibbonProperty::from_raw(code) {
            records::Wc3RibbonProperty::HeightAbove => {
                (&mut value.height_above, properties::HEIGHT_ABOVE)
            }
            records::Wc3RibbonProperty::HeightBelow => {
                (&mut value.height_below, properties::HEIGHT_BELOW)
            }
            _ => return Ok(None),
        },
        Feature::Popcorn(value) => match records::Wc3CornProperty::from_raw(code) {
            records::Wc3CornProperty::Lifespan => (&mut value.lifespan, properties::LIFESPAN),
            records::Wc3CornProperty::EmissionRate => {
                (&mut value.emission_rate, properties::EMISSION_RATE)
            }
            records::Wc3CornProperty::Speed => (&mut value.speed, properties::SPEED),
            _ => return Ok(None),
        },
        Feature::FaceAnimation(_) => return Ok(None),
    };
    let property = descriptor.bind(id).widen();
    field.property = Some(property.clone());
    Ok(Some(property))
}

pub(in crate::model::conversion::wem) fn property_code(
    actor: &Feature<'_>,
    target: &PropertyTarget,
) -> Option<u32> {
    match actor {
        Feature::ResourceParticles(value) => {
            if value
                .emission_rate
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3Particle1Property::EmissionRate.raw());
            }
            if value
                .gravity
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3Particle1Property::Gravity.raw());
            }
            if value
                .longitude
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3Particle1Property::Longitude.raw());
            }
            if value
                .latitude
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3Particle1Property::Latitude.raw());
            }
            if value
                .lifespan
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3Particle1Property::Lifespan.raw());
            }
            if value
                .speed
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3Particle1Property::Speed.raw());
            }
            None
        }
        Feature::SpriteParticles(value) => {
            if value
                .speed
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3Particle2Property::Speed.raw());
            }
            if value
                .variation
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3Particle2Property::Variation.raw());
            }
            if value
                .latitude
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3Particle2Property::Latitude.raw());
            }
            if value
                .gravity
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3Particle2Property::Gravity.raw());
            }
            if value
                .emission_rate
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3Particle2Property::EmissionRate.raw());
            }
            if value
                .width
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3Particle2Property::Width.raw());
            }
            if value
                .length
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3Particle2Property::Length.raw());
            }
            None
        }
        Feature::Ribbon(value) => {
            if value
                .height_above
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3RibbonProperty::HeightAbove.raw());
            }
            if value
                .height_below
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3RibbonProperty::HeightBelow.raw());
            }
            None
        }
        Feature::Popcorn(value) => {
            if value
                .lifespan
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3CornProperty::Lifespan.raw());
            }
            if value
                .emission_rate
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3CornProperty::EmissionRate.raw());
            }
            if value
                .speed
                .property
                .as_ref()
                .is_some_and(|property| property.target() == target)
            {
                return Some(records::Wc3CornProperty::Speed.raw());
            }
            None
        }
        Feature::FaceAnimation(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::wem::records::V19;

    #[test]
    fn sprite_edits_preserve_signed_priority_and_full_width_atlas_values() {
        let document = RawDocument::new();
        let payload = records::NodePayload::<V19>::Wc3ParticleEmitter2(Box::new(
            records::Wc3ParticleEmitter2Payload {
                texture: u32::MAX,
                priority_plane: -17,
                head_decay: records::Wc3ParticleInterval {
                    start: 16_777_217,
                    end: u32::MAX - 1,
                    repeat: 16_777_219,
                },
                ..Default::default()
            },
        ));
        let mut actor = read(&document, &payload, &[], &[]).unwrap().unwrap();
        let Feature::SpriteParticles(value) = &mut actor else {
            panic!()
        };
        value.speed.default = 42.0;
        value.priority_plane = -91;
        value.stages[1].opacity = 121;
        let mut output = RawDocument::new();
        let encoded: records::NodePayload<V19> =
            write(&mut output, &actor, &BTreeMap::new(), &BTreeMap::new()).unwrap();
        let records::NodePayload::Wc3ParticleEmitter2(value) = encoded else {
            panic!()
        };
        assert_eq!(value.speed, 42.0);
        assert_eq!(value.priority_plane, -91);
        assert_eq!(value.middle.alpha, 121);
        assert_eq!(value.head_decay.start, 16_777_217);
        assert_eq!(value.head_decay.end, u32::MAX - 1);
        assert_eq!(value.head_decay.repeat, 16_777_219);
        assert_eq!(value.texture, u32::MAX);
    }

    #[test]
    fn facial_resource_keeps_numeric_and_borrowed_non_utf8_path() {
        let mut document = RawDocument::new();
        let path = text::write_bytes(&mut document, &[b'x', 0xff, b'y']).unwrap();
        let payload = records::NodePayload::<V19>::Wc3FaceFx(Box::new(records::Wc3FaceFxPayload {
            actor: records::AssetKey {
                group: 27,
                id: 123,
                path,
            },
        }));
        let actor = read(&document, &payload, &[], &[]).unwrap().unwrap();
        let Feature::FaceAnimation(value) = &actor else {
            panic!()
        };
        assert_eq!(value.namespace, 27);
        assert_eq!(value.numeric, Some(123));
        assert!(matches!(value.path, std::borrow::Cow::Borrowed(_)));
        let mut output = RawDocument::new();
        let encoded: records::NodePayload<V19> =
            write(&mut output, &actor, &BTreeMap::new(), &BTreeMap::new()).unwrap();
        let records::NodePayload::Wc3FaceFx(value) = encoded else {
            panic!()
        };
        assert_eq!(value.actor.group, 27);
        assert_eq!(value.actor.id, 123);
        assert_eq!(
            text::read_bytes(&output, value.actor.path)
                .unwrap()
                .as_ref(),
            &[b'x', 0xff, b'y']
        );
    }

    #[test]
    fn binding_animation_retains_the_authored_payload_baseline() {
        use crate::model::semantic::profiles::Wem;
        use crate::model::semantic::{Model, Node};
        let mut builder = Model::<Wem>::builder("binding");
        let node = builder.add_node(Node::named("emitter")).unwrap();
        let id = builder.reserve_feature(node);
        let document = RawDocument::new();
        let payload = records::NodePayload::<V19>::Wc3ParticleEmitter1(Box::new(
            records::Wc3ParticleEmitter1Payload {
                speed: 31.0,
                ..Default::default()
            },
        ));
        let mut actor = read(&document, &payload, &[], &[]).unwrap().unwrap();
        let property = bind_property(&mut actor, id, records::Wc3Particle1Property::Speed.raw())
            .unwrap()
            .unwrap();
        assert_eq!(
            property_code(&actor, property.target()),
            Some(records::Wc3Particle1Property::Speed.raw())
        );
        let Feature::ResourceParticles(value) = actor else {
            panic!()
        };
        assert_eq!(value.speed.default, 31.0);
        assert_eq!(value.speed.property, Some(property));
    }

    #[test]
    fn actor_references_require_the_declared_document_tables() {
        let document = RawDocument::new();
        let payload = records::NodePayload::<V19>::Wc3RibbonEmitter(Box::default());
        assert!(read(&document, &payload, &[], &[]).is_err());
        let payload = records::NodePayload::<V19>::Wc3ParticleEmitter2(Box::default());
        assert!(read(&document, &payload, &[], &[]).is_err());
    }
}
