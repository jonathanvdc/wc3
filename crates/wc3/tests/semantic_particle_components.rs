use std::borrow::Cow;
use wc3::model::profiles::mdlx;
use wc3::model::{
    AgeDrivenParticle, AgeDrivenParticleRendering, AgeDrivenParticleShape, AtlasParticle,
    AtlasParticleRendering, ModelParticle, ParticleEmission, ParticleLifetime, ParticleMotion,
    ProgrammedAge, ProgrammedDirection, ProgrammedForces, ProgrammedNoise, ProgrammedParticle,
    ProgrammedParticleGravity, StagedParticleAppearance,
};
use wc3::model::{
    Animated, ParticleAppearance, ParticleDefinition, ParticleDetails, ParticleScalar,
    ParticleTrigger, ZeroVec,
};

#[test]
fn model_particles_share_editable_rate_speed_and_lifetime() {
    let resource = String::from("particles/spark.mdx");
    let source = mdlx::Particle {
        details: ModelParticle {
            resource: Cow::Borrowed(&resource),
            longitude: Animated::constant(45.0),
            ..Default::default()
        },
        emission: ParticleEmission {
            rate: Animated::constant(13.0),
            variation: (),
            trigger: (),
            ..Default::default()
        },
        motion: ParticleMotion {
            speed: Animated::constant(7.0),
            variation: (),
            ..Default::default()
        },
        lifetime: ParticleLifetime {
            duration: Animated::constant(2.5),
            variation: (),
        },
        appearance: (),
    };
    let mut definition = ParticleDefinition::from(source.clone());
    let ParticleDetails::Model(details) = &definition.details else {
        panic!()
    };
    assert!(matches!(&details.resource, Cow::Borrowed(_)));
    definition.emission.rate = ParticleScalar::Animated(Animated::constant(21.0));
    let restored = mdlx::Particle::try_from(definition).unwrap();
    assert_eq!(restored.emission.rate.default, 21.0);
    assert_eq!(restored.details.resource.as_ptr(), resource.as_ptr());
    assert_eq!(restored.details.longitude, source.details.longitude);
    assert_eq!(restored.motion.speed, source.motion.speed);
    assert_eq!(restored.lifetime.duration, source.lifetime.duration);
}

#[test]
fn atlas_appearance_and_rendering_survive_composition() {
    let source = mdlx::Particle2 {
        motion: ParticleMotion {
            speed: Animated::constant(8.0),
            ..Default::default()
        },
        lifetime: ParticleLifetime {
            duration: 3.0,
            variation: (),
        },
        emission: ParticleEmission {
            trigger: true,
            variation: (),
            ..Default::default()
        },
        appearance: StagedParticleAppearance {
            colors: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            alpha: [17, 128, 255],
            uv_intervals: [[1, 3, 2]; 4],
            ..Default::default()
        },
        details: AtlasParticle {
            rendering: AtlasParticleRendering {
                atlas: [4, 8],
                ..Default::default()
            },
            ..Default::default()
        },
    };
    let definition = ParticleDefinition::from(source.clone());
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
    assert_eq!(mdlx::Particle2::try_from(definition).unwrap(), source);
}

#[test]
fn narrowing_retains_unsupported_authored_variation() {
    let mut definition = ParticleDefinition::from(mdlx::Particle::default());
    definition.motion.variation = ParticleScalar::Animated(Animated::constant(0.75));
    let original = mdlx::Particle::try_from(definition).unwrap_err();
    let ParticleScalar::Animated(value) = &original.motion.variation else {
        panic!()
    };
    assert_eq!(value.default, 0.75);
}

use wc3::model::profiles::{m2, m3};

#[test]
fn age_driven_roundtrip_preserves_borrowed_spline_and_common_randomness() {
    let spline = ZeroVec::alloc_from_slice(&[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
    let source = m2::Particle {
        details: AgeDrivenParticle {
            id: Default::default(),
            flags: Default::default(),
            rendering: AgeDrivenParticleRendering {
                blend: Default::default(),
                color_index: Default::default(),
                textures: vec![],
                model: Default::default(),
                child_model: Default::default(),
                head_tail: Default::default(),
                texture_scale: Default::default(),
                texture_tile_rotation: Default::default(),
                atlas: [4, 8],
                tail_length: Default::default(),
                scroll: Some([0.125; 8]),
            },
            shape: AgeDrivenParticleShape {
                generator: Default::default(),
                legacy_shape: Default::default(),
                vertical_range: Default::default(),
                horizontal_range: Default::default(),
                area_width: Default::default(),
                area_length: Default::default(),
                z_source: Default::default(),
                spline: spline.as_slice().as_zerovec(),
            },
            twinkle_speed: Default::default(),
            twinkle_percent: Default::default(),
            twinkle_scale: Default::default(),
            inherit_velocity: Default::default(),
            drag: Default::default(),
            spin: Default::default(),
            tumble_minimum: Default::default(),
            tumble_maximum: Default::default(),
            wind: Default::default(),
            wind_time: Default::default(),
            follow: Default::default(),
        },
        motion: ParticleMotion {
            speed: Animated::constant(6.0),
            variation: Animated::constant(0.4),
            gravity: Default::default(),
        },
        lifetime: ParticleLifetime {
            duration: Animated::constant(5.0),
            variation: 1.5,
        },
        emission: ParticleEmission {
            rate: Animated::constant(17.0),
            variation: 2.0,
            activation: Default::default(),
            trigger: (),
        },
        appearance: Default::default(),
    };
    let before = format!("{source:?}");
    let definition = ParticleDefinition::from(source);
    let ParticleDetails::AgeDriven(details) = &definition.details else {
        panic!()
    };
    assert_eq!(
        details.shape.spline.as_bytes().as_ptr(),
        spline.as_bytes().as_ptr()
    );
    let restored = m2::Particle::try_from(definition).unwrap();
    assert_eq!(
        restored.details.shape.spline.as_bytes().as_ptr(),
        spline.as_bytes().as_ptr()
    );
    assert_eq!(format!("{restored:?}"), before);
}
#[test]
fn programmed_roundtrip_preserves_reset_controls_and_every_detail() {
    let spline = [m3::Property {
        initial: [1.0, 2.0, 3.0],
        reset: [9.0; 3],
        ..Default::default()
    }];
    let source = m3::Particle {
        details: ProgrammedParticle {
            material: Default::default(),
            additional_flags: Default::default(),
            direction: ProgrammedDirection {
                initial_yaw: Default::default(),
                initial_pitch: Default::default(),
                initial_horizontal: Default::default(),
                initial_vertical: Default::default(),
                drag: Default::default(),
            },
            kill_radius: 4.0,
            forces: ProgrammedForces {
                mass: Default::default(),
                mass_random: Default::default(),
                mass_size_multiplier: Default::default(),
                local_forces: Default::default(),
                world_forces: Default::default(),
                local_forces_fallback: Default::default(),
                world_forces_fallback: Default::default(),
                world_forces_mass_multiplier: Default::default(),
            },
            noise: ProgrammedNoise {
                noise_amplitude: 3.25,
                noise_frequency: Default::default(),
                noise_coherence: Default::default(),
                noise_edge: Default::default(),
            },
            index_plus_length: Default::default(),
            max_particles: Default::default(),
            emitter_shape: Default::default(),
            shape_outer: Default::default(),
            shape_inner: Default::default(),
            outer_radius: Default::default(),
            inner_radius: Default::default(),
            shape_sections: Default::default(),
            velocity_type: Default::default(),
            size_random_enable: Default::default(),
            size_random_animation: Default::default(),
            rotation_random_enable: Default::default(),
            rotation_random_animation: Default::default(),
            color_random_enable: Default::default(),
            color_start_random: Default::default(),
            color_mid_random: Default::default(),
            color_end_random: Default::default(),
            alpha_random_enable: Default::default(),
            flipbook_start_init_index: Default::default(),
            flipbook_start_stop_index: Default::default(),
            flipbook_end_init_index: Default::default(),
            flipbook_end_stop_index: Default::default(),
            flipbook_mid_time: Default::default(),
            flipbook_columns: Default::default(),
            flipbook_rows: Default::default(),
            flipbook_column_fraction: Default::default(),
            flipbook_row_fraction: Default::default(),
            bounce: Default::default(),
            friction: Default::default(),
            collision_spawn: Default::default(),
            collision_spawn_min: Default::default(),
            collision_spawn_max: Default::default(),
            collision_spawn_chance: Default::default(),
            collision_spawn_energy: Default::default(),
            collision_die_bounce: Default::default(),
            instance_type: Default::default(),
            tail_length: Default::default(),
            instance_angle: Default::default(),
            instance_distance: Default::default(),
            pitch: m3::VariationControl {
                kind: Default::default(),
                amplitude: Default::default(),
                frequency: Default::default(),
            },
            yaw: m3::VariationControl {
                kind: Default::default(),
                amplitude: Default::default(),
                frequency: Default::default(),
            },
            speed: m3::VariationControl {
                kind: Default::default(),
                amplitude: Default::default(),
                frequency: Default::default(),
            },
            size: m3::VariationControl {
                kind: Default::default(),
                amplitude: Default::default(),
                frequency: Default::default(),
            },
            alpha: m3::VariationControl {
                kind: Default::default(),
                amplitude: Default::default(),
                frequency: Default::default(),
            },
            color: m3::VariationControl {
                kind: Default::default(),
                amplitude: Default::default(),
                frequency: Default::default(),
            },
            rotation: m3::VariationControl {
                kind: Default::default(),
                amplitude: Default::default(),
                frequency: Default::default(),
            },
            horizontal: m3::VariationControl {
                kind: Default::default(),
                amplitude: Default::default(),
                frequency: Default::default(),
            },
            vertical: m3::VariationControl {
                kind: Default::default(),
                amplitude: Default::default(),
                frequency: Default::default(),
            },
            particle_velocity: Default::default(),
            phase_shift: Default::default(),
            flags: Default::default(),
            rotation_flags: Default::default(),
            color_smoothing: Default::default(),
            size_smoothing: Default::default(),
            rotation_smoothing: Default::default(),
            alpha_threshold: Default::default(),
            uv_offset: Default::default(),
            uv_angle: Default::default(),
            uv_tiling: Default::default(),
            spline_points: Cow::Borrowed(&spline),
            wind_multiplier: Default::default(),
            lod_reduce: Default::default(),
            lod_cut: Default::default(),
            lower_bound: Default::default(),
            upper_bound: Default::default(),
            trail: Default::default(),
            trail_chance: Default::default(),
            trail_emission_rate: Default::default(),
            projection: Default::default(),
            splat_chance: Default::default(),
            models: vec![m3::ExternalModel {
                path: Cow::Borrowed("spark.m3"),
                hash: Some(12),
            }],
            spawn_ribbon_on_bounce_chance: Default::default(),
            ribbon: Default::default(),
            noise_smoothness: Default::default(),
        },
        motion: ParticleMotion {
            speed: m3::Property {
                initial: 7.0,
                reset: 11.0,
                ..Default::default()
            },
            variation: Default::default(),
            gravity: ProgrammedParticleGravity {
                x: 0xdeadbeef,
                y: 0x12345678,
                magnitude: Default::default(),
            },
        },
        lifetime: ParticleLifetime {
            duration: Default::default(),
            variation: Default::default(),
        },
        appearance: ProgrammedAge {
            size_mid_time: 0.375,
            color_mid_time: Default::default(),
            alpha_mid_time: Default::default(),
            rotation_mid_time: Default::default(),
            size_mid_hold_time: Default::default(),
            color_mid_hold_time: Default::default(),
            alpha_mid_hold_time: Default::default(),
            rotation_mid_hold_time: Default::default(),
            size_animation: Default::default(),
            rotation_animation: Default::default(),
            color_start: Default::default(),
            color_mid: Default::default(),
            color_end: Default::default(),
        },
        emission: ParticleEmission {
            rate: Default::default(),
            trigger: Default::default(),
            variation: (),
            activation: (),
        },
    };
    let definition = ParticleDefinition::from(source.clone());
    let ParticleScalar::Programmed(speed) = &definition.motion.speed else {
        panic!()
    };
    assert_eq!(speed.initial, 7.0);
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
    let atlas = ParticleDefinition::from(mdlx::Particle2::default());
    let mut model = ParticleDefinition::from(mdlx::Particle::default());
    model.appearance = atlas.appearance;
    let original = mdlx::Particle::try_from(model).unwrap_err();
    assert!(matches!(original.appearance, ParticleAppearance::Staged(_)));
    assert!(matches!(original.details, ParticleDetails::Model(_)));
}
