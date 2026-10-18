use std::borrow::Cow;
use wc3::model::profiles::mdlx;
use wc3::model::{
    AgeDrivenParticle, AgeDrivenParticleRendering, AgeDrivenParticleShape, AtlasParticle,
    AtlasParticleRendering, ModelParticle, ParticleEmission, ParticleLifetime, ParticleMotion,
    StagedParticleAppearance,
};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{FromFeature as SlotFromFeature, TryFromFeature as SlotTryFromFeature};
use wc3::model::{
    ParticleAppearance, ParticleDefinition, ParticleDetails, ParticleScalar, ParticleTrigger,
    ZeroVec,
};

#[test]
fn model_particles_share_editable_rate_speed_and_lifetime() {
    let mut slot_values = SlotValues::default();
    let resource = String::from("particles/spark.mdx");
    let mut draft = wc3::model::Model::<wc3::model::profiles::Mdlx>::builder("resources");
    let resource_id = draft.add_model_resource(wc3::model::PathResource::Path(Cow::Borrowed(
        resource.as_bytes(),
    )));
    let source = mdlx::Particle {
        details: ModelParticle {
            resource: wc3::model::ParticleResource::Model(Some(resource_id)),
            longitude: slot_values.add(45.0),
            ..SlotConstruct::construct(&mut slot_values)
        },
        emission: ParticleEmission {
            rate: slot_values.add(13.0),
            variation: (),
            trigger: (),
            ..SlotConstruct::construct(&mut slot_values)
        },
        motion: ParticleMotion {
            speed: slot_values.add(7.0),
            variation: (),
            ..SlotConstruct::construct(&mut slot_values)
        },
        lifetime: ParticleLifetime {
            duration: slot_values.add(2.5),
            variation: (),
        },
        appearance: (),
    };
    let mut definition = ParticleDefinition::from_feature(source.clone(), &mut slot_values);
    let ParticleDetails::Model(details) = &definition.details else {
        panic!()
    };
    assert_eq!(details.resource, source.details.resource);
    definition.emission.rate = ParticleScalar::Property(slot_values.add(21.0));
    let restored = mdlx::Particle::try_from_feature(definition, &slot_values).unwrap();
    assert_eq!(slot_values[restored.emission.rate], 21.0);
    assert_eq!(restored.details.resource, source.details.resource);
    assert_eq!(restored.details.longitude, source.details.longitude);
    assert_eq!(restored.motion.speed, source.motion.speed);
    assert_eq!(restored.lifetime.duration, source.lifetime.duration);
}

#[test]
fn atlas_appearance_and_rendering_survive_composition() {
    let mut slot_values = SlotValues::default();
    let source = mdlx::Particle2 {
        motion: ParticleMotion {
            speed: slot_values.add(8.0),
            ..SlotConstruct::construct(&mut slot_values)
        },
        lifetime: ParticleLifetime {
            duration: 3.0,
            variation: (),
        },
        emission: ParticleEmission {
            trigger: true,
            variation: (),
            ..SlotConstruct::construct(&mut slot_values)
        },
        appearance: StagedParticleAppearance {
            colors: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            alpha: [17, 128, 255],
            uv_intervals: [[1, 3, 2]; 4],
            ..SlotConstruct::construct(&mut slot_values)
        },
        details: AtlasParticle {
            rendering: AtlasParticleRendering {
                atlas: [4, 8],
                ..SlotConstruct::construct(&mut slot_values)
            },
            ..SlotConstruct::construct(&mut slot_values)
        },
    };
    let definition = ParticleDefinition::from_feature(source.clone(), &mut slot_values);
    assert!(matches!(
        definition.emission.trigger,
        ParticleTrigger::KeyedBurst
    ));
    let ParticleDetails::Atlas(details) = &definition.details else {
        panic!()
    };
    let ParticleAppearance::Staged(appearance) = &definition.appearance else {
        panic!()
    };
    assert_eq!(appearance.alpha, source.appearance.alpha);
    assert_eq!(details.rendering.atlas, source.details.rendering.atlas);
    assert_eq!(
        mdlx::Particle2::try_from_feature(definition, &slot_values).unwrap(),
        source
    );
}

#[test]
fn narrowing_retains_unsupported_authored_variation() {
    let mut slot_values = SlotValues::default();
    let mut definition = ParticleDefinition::from_feature(
        mdlx::Particle::construct(&mut slot_values),
        &mut slot_values,
    );
    definition.motion.variation = ParticleScalar::Property(slot_values.add(0.75));
    let original = mdlx::Particle::try_from_feature(definition, &slot_values).unwrap_err();
    let ParticleScalar::Property(value) = &original.motion.variation;
    assert_eq!(slot_values[value], 0.75);
}

use wc3::model::profiles::m2;

#[test]
fn age_driven_roundtrip_preserves_borrowed_spline_and_common_randomness() {
    let mut slot_values = SlotValues::default();
    let spline = ZeroVec::alloc_from_slice(&[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
    let source = m2::Particle {
        details: AgeDrivenParticle {
            id: SlotConstruct::construct(&mut slot_values),
            flags: SlotConstruct::construct(&mut slot_values),
            rendering: AgeDrivenParticleRendering {
                blend: SlotConstruct::construct(&mut slot_values),
                color_index: SlotConstruct::construct(&mut slot_values),
                textures: vec![],
                texture_program: Some([5, 9]),
                model: SlotConstruct::construct(&mut slot_values),
                child_model: SlotConstruct::construct(&mut slot_values),
                head_tail: SlotConstruct::construct(&mut slot_values),
                texture_scale: SlotConstruct::construct(&mut slot_values),
                texture_tile_rotation: SlotConstruct::construct(&mut slot_values),
                atlas: [4, 8],
                tail_length: SlotConstruct::construct(&mut slot_values),
                scroll: Some([0.125; 8]),
            },
            shape: AgeDrivenParticleShape {
                generator: SlotConstruct::construct(&mut slot_values),
                legacy_shape: SlotConstruct::construct(&mut slot_values),
                vertical_range: SlotConstruct::construct(&mut slot_values),
                horizontal_range: SlotConstruct::construct(&mut slot_values),
                area_width: SlotConstruct::construct(&mut slot_values),
                area_length: SlotConstruct::construct(&mut slot_values),
                z_source: SlotConstruct::construct(&mut slot_values),
                spline: spline.as_slice().as_zerovec(),
            },
            twinkle_speed: SlotConstruct::construct(&mut slot_values),
            twinkle_percent: SlotConstruct::construct(&mut slot_values),
            twinkle_scale: SlotConstruct::construct(&mut slot_values),
            inherit_velocity: SlotConstruct::construct(&mut slot_values),
            drag: SlotConstruct::construct(&mut slot_values),
            spin: SlotConstruct::construct(&mut slot_values),
            tumble_minimum: SlotConstruct::construct(&mut slot_values),
            tumble_maximum: SlotConstruct::construct(&mut slot_values),
            wind: SlotConstruct::construct(&mut slot_values),
            wind_time: SlotConstruct::construct(&mut slot_values),
            follow: SlotConstruct::construct(&mut slot_values),
        },
        motion: ParticleMotion {
            speed: slot_values.add(6.0),
            variation: slot_values.add(0.4),
            gravity: SlotConstruct::construct(&mut slot_values),
        },
        lifetime: ParticleLifetime {
            duration: slot_values.add(5.0),
            variation: 1.5,
        },
        emission: ParticleEmission {
            rate: slot_values.add(17.0),
            variation: 2.0,
            activation: SlotConstruct::construct(&mut slot_values),
            trigger: (),
        },
        appearance: SlotConstruct::construct(&mut slot_values),
    };
    let before = format!("{source:?}");
    let definition = ParticleDefinition::from_feature(source, &mut slot_values);
    let ParticleDetails::AgeDriven(details) = &definition.details else {
        panic!()
    };
    assert_eq!(
        details.shape.spline.as_bytes().as_ptr(),
        spline.as_bytes().as_ptr()
    );
    let restored = m2::Particle::try_from_feature(definition, &slot_values).unwrap();
    assert_eq!(
        restored.details.shape.spline.as_bytes().as_ptr(),
        spline.as_bytes().as_ptr()
    );
    assert_eq!(format!("{restored:?}"), before);
}

#[test]
fn independently_authored_age_program_cannot_be_silently_discarded() {
    let mut slot_values = SlotValues::default();
    let atlas = ParticleDefinition::from_feature(
        mdlx::Particle2::construct(&mut slot_values),
        &mut slot_values,
    );
    let mut model = ParticleDefinition::from_feature(
        mdlx::Particle::construct(&mut slot_values),
        &mut slot_values,
    );
    model.appearance = atlas.appearance;
    let original = mdlx::Particle::try_from_feature(model, &slot_values).unwrap_err();
    assert!(matches!(original.appearance, ParticleAppearance::Staged(_)));
    assert!(matches!(original.details, ParticleDetails::Model(_)));
}
