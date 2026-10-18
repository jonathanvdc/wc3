use std::borrow::Cow;
use wc3::model::profiles::mdlx;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{
    M2ParticleDetails, M2ParticleRendering, M2ParticleShape, MdlxParticle1Details,
    MdlxParticle2Appearance, MdlxParticle2Details, MdlxParticle2Rendering, ParticleEmission,
    ParticleLifetime, ParticleMotion,
};
use wc3::model::{ParticleEmitter, ZeroVec};

#[test]
fn model_particles_share_editable_rate_speed_and_lifetime() {
    let mut slot_values = SlotValues::default();
    let resource = String::from("particles/spark.mdx");
    let mut draft = wc3::model::Model::<wc3::model::profiles::Mdlx>::builder("resources");
    let resource_id = draft.add_model_resource(wc3::model::PathResource::Path(Cow::Borrowed(
        resource.as_bytes(),
    )));
    let source = mdlx::ParticleEmitter1 {
        details: MdlxParticle1Details {
            resource: wc3::model::MdlxParticle1Resource::Model(Some(resource_id)),
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
    let mut definition = ParticleEmitter::from(source.clone());
    let ParticleEmitter::MdlxEmitter1(emitter) = &mut definition else {
        panic!()
    };
    assert_eq!(emitter.details.resource, source.details.resource);
    emitter.emission.rate = slot_values.add(21.0);
    let restored = mdlx::ParticleEmitter1::try_from(definition).unwrap();
    assert_eq!(slot_values[restored.emission.rate], 21.0);
    assert_eq!(restored.details.resource, source.details.resource);
    assert_eq!(restored.details.longitude, source.details.longitude);
    assert_eq!(restored.motion.speed, source.motion.speed);
    assert_eq!(restored.lifetime.duration, source.lifetime.duration);
}

#[test]
fn atlas_appearance_and_rendering_survive_composition() {
    let mut slot_values = SlotValues::default();
    let source = mdlx::ParticleEmitter2 {
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
        appearance: MdlxParticle2Appearance {
            colors: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            alpha: [17, 128, 255],
            uv_intervals: [[1, 3, 2]; 4],
            ..SlotConstruct::construct(&mut slot_values)
        },
        details: MdlxParticle2Details {
            rendering: MdlxParticle2Rendering {
                atlas: [4, 8],
                ..SlotConstruct::construct(&mut slot_values)
            },
            ..SlotConstruct::construct(&mut slot_values)
        },
    };
    let definition = ParticleEmitter::from(source.clone());
    let ParticleEmitter::MdlxEmitter2(emitter) = &definition else {
        panic!()
    };
    assert!(emitter.emission.trigger);
    let details = &emitter.details;
    let appearance = &emitter.appearance;
    assert_eq!(appearance.alpha, source.appearance.alpha);
    assert_eq!(details.rendering.atlas, source.details.rendering.atlas);
    assert_eq!(
        mdlx::ParticleEmitter2::try_from(definition).unwrap(),
        source
    );
}

#[test]
fn narrowing_rejects_a_different_particle_family_without_discarding_it() {
    let mut values = SlotValues::default();
    let source = mdlx::ParticleEmitter2::construct(&mut values);
    let rejected =
        mdlx::ParticleEmitter1::try_from(ParticleEmitter::from(source.clone())).unwrap_err();
    assert_eq!(mdlx::ParticleEmitter2::try_from(*rejected).unwrap(), source);
}

use wc3::model::profiles::m2;

#[test]
fn age_driven_roundtrip_preserves_borrowed_spline_and_common_randomness() {
    let mut slot_values = SlotValues::default();
    let spline = ZeroVec::alloc_from_slice(&[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
    let source = m2::ParticleEmitter {
        details: M2ParticleDetails {
            id: SlotConstruct::construct(&mut slot_values),
            flags: SlotConstruct::construct(&mut slot_values),
            rendering: M2ParticleRendering {
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
            shape: M2ParticleShape {
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
    let definition = ParticleEmitter::from(source);
    let ParticleEmitter::M2(emitter) = &definition else {
        panic!()
    };
    assert_eq!(
        emitter.details.shape.spline.as_bytes().as_ptr(),
        spline.as_bytes().as_ptr()
    );
    let restored = m2::ParticleEmitter::try_from(definition).unwrap();
    assert_eq!(
        restored.details.shape.spline.as_bytes().as_ptr(),
        spline.as_bytes().as_ptr()
    );
    assert_eq!(format!("{restored:?}"), before);
}
