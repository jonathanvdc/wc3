use std::borrow::Cow;
use wc3::model::profiles::mdlx;
use wc3::model::{
    AgeDrivenParticle, AgeDrivenParticleRendering, AgeDrivenParticleShape, AtlasParticle,
    AtlasParticleRendering, ModelParticle, ParticleEmission, ParticleLifetime, ParticleMotion,
    ProgrammedAge, ProgrammedDirection, ProgrammedForces, ProgrammedNoise, ProgrammedParticle,
    ProgrammedParticleGravity, StagedParticleAppearance,
};
use wc3::model::{Construct as SlotConstruct, Values as SlotValues};
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
    definition.emission.rate = ParticleScalar::Animated(slot_values.add(21.0));
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
    definition.motion.variation = ParticleScalar::Animated(slot_values.add(0.75));
    let original = mdlx::Particle::try_from_feature(definition, &slot_values).unwrap_err();
    let ParticleScalar::Animated(value) = &original.motion.variation else {
        panic!()
    };
    assert_eq!(slot_values[value], 0.75);
}

use wc3::model::profiles::{m2, m3};

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
fn programmed_roundtrip_preserves_reset_controls_and_every_detail() {
    let mut slot_values = SlotValues::default();
    let spline = [m3::Property {
        value: slot_values.add([1.0, 2.0, 3.0]),
        reset: [9.0; 3],
        ..SlotConstruct::construct(&mut slot_values)
    }];
    let source = m3::Particle {
        details: ProgrammedParticle {
            material: SlotConstruct::construct(&mut slot_values),
            additional_flags: SlotConstruct::construct(&mut slot_values),
            direction: ProgrammedDirection {
                initial_yaw: SlotConstruct::construct(&mut slot_values),
                initial_pitch: SlotConstruct::construct(&mut slot_values),
                initial_horizontal: SlotConstruct::construct(&mut slot_values),
                initial_vertical: SlotConstruct::construct(&mut slot_values),
                drag: SlotConstruct::construct(&mut slot_values),
            },
            kill_radius: 4.0,
            forces: ProgrammedForces {
                mass: SlotConstruct::construct(&mut slot_values),
                mass_random: SlotConstruct::construct(&mut slot_values),
                mass_size_multiplier: SlotConstruct::construct(&mut slot_values),
                local_forces: SlotConstruct::construct(&mut slot_values),
                world_forces: SlotConstruct::construct(&mut slot_values),
                local_forces_fallback: SlotConstruct::construct(&mut slot_values),
                world_forces_fallback: SlotConstruct::construct(&mut slot_values),
                world_forces_mass_multiplier: SlotConstruct::construct(&mut slot_values),
            },
            noise: ProgrammedNoise {
                noise_amplitude: 3.25,
                noise_frequency: SlotConstruct::construct(&mut slot_values),
                noise_coherence: SlotConstruct::construct(&mut slot_values),
                noise_edge: SlotConstruct::construct(&mut slot_values),
            },
            index_plus_length: SlotConstruct::construct(&mut slot_values),
            max_particles: SlotConstruct::construct(&mut slot_values),
            emitter_shape: SlotConstruct::construct(&mut slot_values),
            shape_outer: SlotConstruct::construct(&mut slot_values),
            shape_inner: SlotConstruct::construct(&mut slot_values),
            outer_radius: SlotConstruct::construct(&mut slot_values),
            inner_radius: SlotConstruct::construct(&mut slot_values),
            shape_sections: SlotConstruct::construct(&mut slot_values),
            velocity_type: SlotConstruct::construct(&mut slot_values),
            size_random_enable: SlotConstruct::construct(&mut slot_values),
            size_random_animation: SlotConstruct::construct(&mut slot_values),
            rotation_random_enable: SlotConstruct::construct(&mut slot_values),
            rotation_random_animation: SlotConstruct::construct(&mut slot_values),
            color_random_enable: SlotConstruct::construct(&mut slot_values),
            color_start_random: SlotConstruct::construct(&mut slot_values),
            color_mid_random: SlotConstruct::construct(&mut slot_values),
            color_end_random: SlotConstruct::construct(&mut slot_values),
            alpha_random_enable: SlotConstruct::construct(&mut slot_values),
            flipbook_start_init_index: SlotConstruct::construct(&mut slot_values),
            flipbook_start_stop_index: SlotConstruct::construct(&mut slot_values),
            flipbook_end_init_index: SlotConstruct::construct(&mut slot_values),
            flipbook_end_stop_index: SlotConstruct::construct(&mut slot_values),
            flipbook_mid_time: SlotConstruct::construct(&mut slot_values),
            flipbook_columns: SlotConstruct::construct(&mut slot_values),
            flipbook_rows: SlotConstruct::construct(&mut slot_values),
            flipbook_column_fraction: SlotConstruct::construct(&mut slot_values),
            flipbook_row_fraction: SlotConstruct::construct(&mut slot_values),
            bounce: SlotConstruct::construct(&mut slot_values),
            friction: SlotConstruct::construct(&mut slot_values),
            collision_spawn: SlotConstruct::construct(&mut slot_values),
            collision_spawn_min: SlotConstruct::construct(&mut slot_values),
            collision_spawn_max: SlotConstruct::construct(&mut slot_values),
            collision_spawn_chance: SlotConstruct::construct(&mut slot_values),
            collision_spawn_energy: SlotConstruct::construct(&mut slot_values),
            collision_die_bounce: SlotConstruct::construct(&mut slot_values),
            instance_type: SlotConstruct::construct(&mut slot_values),
            tail_length: SlotConstruct::construct(&mut slot_values),
            instance_angle: SlotConstruct::construct(&mut slot_values),
            instance_distance: SlotConstruct::construct(&mut slot_values),
            pitch: m3::VariationControl {
                kind: SlotConstruct::construct(&mut slot_values),
                amplitude: SlotConstruct::construct(&mut slot_values),
                frequency: SlotConstruct::construct(&mut slot_values),
            },
            yaw: m3::VariationControl {
                kind: SlotConstruct::construct(&mut slot_values),
                amplitude: SlotConstruct::construct(&mut slot_values),
                frequency: SlotConstruct::construct(&mut slot_values),
            },
            speed: m3::VariationControl {
                kind: SlotConstruct::construct(&mut slot_values),
                amplitude: SlotConstruct::construct(&mut slot_values),
                frequency: SlotConstruct::construct(&mut slot_values),
            },
            size: m3::VariationControl {
                kind: SlotConstruct::construct(&mut slot_values),
                amplitude: SlotConstruct::construct(&mut slot_values),
                frequency: SlotConstruct::construct(&mut slot_values),
            },
            alpha: m3::VariationControl {
                kind: SlotConstruct::construct(&mut slot_values),
                amplitude: SlotConstruct::construct(&mut slot_values),
                frequency: SlotConstruct::construct(&mut slot_values),
            },
            color: m3::VariationControl {
                kind: SlotConstruct::construct(&mut slot_values),
                amplitude: SlotConstruct::construct(&mut slot_values),
                frequency: SlotConstruct::construct(&mut slot_values),
            },
            rotation: m3::VariationControl {
                kind: SlotConstruct::construct(&mut slot_values),
                amplitude: SlotConstruct::construct(&mut slot_values),
                frequency: SlotConstruct::construct(&mut slot_values),
            },
            horizontal: m3::VariationControl {
                kind: SlotConstruct::construct(&mut slot_values),
                amplitude: SlotConstruct::construct(&mut slot_values),
                frequency: SlotConstruct::construct(&mut slot_values),
            },
            vertical: m3::VariationControl {
                kind: SlotConstruct::construct(&mut slot_values),
                amplitude: SlotConstruct::construct(&mut slot_values),
                frequency: SlotConstruct::construct(&mut slot_values),
            },
            particle_velocity: SlotConstruct::construct(&mut slot_values),
            phase_shift: SlotConstruct::construct(&mut slot_values),
            flags: SlotConstruct::construct(&mut slot_values),
            rotation_flags: SlotConstruct::construct(&mut slot_values),
            color_smoothing: SlotConstruct::construct(&mut slot_values),
            size_smoothing: SlotConstruct::construct(&mut slot_values),
            rotation_smoothing: SlotConstruct::construct(&mut slot_values),
            alpha_threshold: SlotConstruct::construct(&mut slot_values),
            uv_offset: SlotConstruct::construct(&mut slot_values),
            uv_angle: SlotConstruct::construct(&mut slot_values),
            uv_tiling: SlotConstruct::construct(&mut slot_values),
            spline_points: Cow::Borrowed(&spline),
            wind_multiplier: SlotConstruct::construct(&mut slot_values),
            lod_reduce: SlotConstruct::construct(&mut slot_values),
            lod_cut: SlotConstruct::construct(&mut slot_values),
            lower_bound: SlotConstruct::construct(&mut slot_values),
            upper_bound: SlotConstruct::construct(&mut slot_values),
            trail: SlotConstruct::construct(&mut slot_values),
            trail_chance: SlotConstruct::construct(&mut slot_values),
            trail_emission_rate: SlotConstruct::construct(&mut slot_values),
            projection: SlotConstruct::construct(&mut slot_values),
            splat_chance: SlotConstruct::construct(&mut slot_values),
            models: vec![m3::ExternalModel {
                path: Cow::Borrowed("spark.m3"),
                hash: Some(12),
            }],
            spawn_ribbon_on_bounce_chance: SlotConstruct::construct(&mut slot_values),
            ribbon: SlotConstruct::construct(&mut slot_values),
            noise_smoothness: SlotConstruct::construct(&mut slot_values),
        },
        motion: ParticleMotion {
            speed: m3::Property {
                value: slot_values.add(7.0),
                reset: 11.0,
                ..SlotConstruct::construct(&mut slot_values)
            },
            variation: SlotConstruct::construct(&mut slot_values),
            gravity: ProgrammedParticleGravity {
                x: 0xdeadbeef,
                y: 0x12345678,
                magnitude: SlotConstruct::construct(&mut slot_values),
            },
        },
        lifetime: ParticleLifetime {
            duration: SlotConstruct::construct(&mut slot_values),
            variation: SlotConstruct::construct(&mut slot_values),
        },
        appearance: ProgrammedAge {
            size_mid_time: 0.375,
            color_mid_time: SlotConstruct::construct(&mut slot_values),
            alpha_mid_time: SlotConstruct::construct(&mut slot_values),
            rotation_mid_time: SlotConstruct::construct(&mut slot_values),
            size_mid_hold_time: SlotConstruct::construct(&mut slot_values),
            color_mid_hold_time: SlotConstruct::construct(&mut slot_values),
            alpha_mid_hold_time: SlotConstruct::construct(&mut slot_values),
            rotation_mid_hold_time: SlotConstruct::construct(&mut slot_values),
            size_animation: SlotConstruct::construct(&mut slot_values),
            rotation_animation: SlotConstruct::construct(&mut slot_values),
            color_start: SlotConstruct::construct(&mut slot_values),
            color_mid: SlotConstruct::construct(&mut slot_values),
            color_end: SlotConstruct::construct(&mut slot_values),
        },
        emission: ParticleEmission {
            rate: SlotConstruct::construct(&mut slot_values),
            trigger: SlotConstruct::construct(&mut slot_values),
            variation: (),
            activation: (),
        },
    };
    let definition = ParticleDefinition::from_feature(source.clone(), &mut slot_values);
    let ParticleScalar::Programmed(speed) = &definition.motion.speed else {
        panic!()
    };
    assert_eq!(slot_values[speed.value], 7.0);
    assert_eq!(speed.reset, 11.0);
    let ParticleDetails::Programmed(details) = &definition.details else {
        panic!()
    };
    assert!(matches!(&details.spline_points, Cow::Borrowed(_)));
    assert_eq!(details.noise.noise_amplitude, 3.25);
    let ParticleAppearance::Programmed(appearance) = &definition.appearance else {
        panic!()
    };
    assert_eq!(appearance.size_mid_time, 0.375);
    let restored = m3::Particle::try_from(definition).unwrap();
    assert_eq!(restored, source);
    assert_eq!(restored.details.spline_points.as_ptr(), spline.as_ptr());
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
