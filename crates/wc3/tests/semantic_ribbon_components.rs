use wc3::model::semantic::profiles::{m2, mdlx};
use wc3::model::semantic::{AngleTrailControls, SurfaceTrailRendering};
use wc3::model::semantic::{Animated, TrailAtlasSlot, TrailControls, TrailEmitter, TrailRendering};
use wc3::model::semantic::{
    GravityTrailControls, ParticleLifetime, ParticleMotion, ProgrammedAge, ProgrammedDirection,
    ProgrammedForces, ProgrammedNoise, ProgrammedTrailGravity, TrailAppearance,
};

#[test]
fn trail_editing_preserves_wide_integer_rates_and_rejects_fractional_export() {
    let mut trail = TrailEmitter::from(mdlx::Ribbon {
        emission_rate: u32::MAX,
        appearance: TrailAppearance {
            texture_slot: Animated::constant(70_000),
            ..Default::default()
        },
        controls: GravityTrailControls {
            gravity: 9.8,
            ..Default::default()
        },
        ..Default::default()
    });
    assert_eq!(trail.emission_rate, f64::from(u32::MAX));
    trail.geometry.height_above.default = 2.5;
    let native = mdlx::Ribbon::try_from(trail.clone()).unwrap();
    assert_eq!(native.geometry.height_above.default, 2.5);
    assert_eq!(native.emission_rate, u32::MAX);
    trail.emission_rate = 0.5;
    let rejected = mdlx::Ribbon::try_from(trail).unwrap_err();
    assert_eq!(rejected.emission_rate, 0.5);
}

#[test]
fn edge_trail_export_checks_numeric_width_without_conflating_direction_and_gravity() {
    let mut trail = TrailEmitter::from(mdlx::Ribbon::default());
    trail.emission_rate = 0.5;
    trail.appearance.texture_slot = TrailAtlasSlot::Compact(Animated::constant(7));
    trail.rendering = TrailRendering::Surfaces(SurfaceTrailRendering {
        textures: vec![],
        materials: vec![],
    });
    trail.controls = TrailControls::EmissionAngle(AngleTrailControls {
        angle: 1.25,
        id: 9,
        enabled: Animated::constant(true),
        priority_plane: -2,
        color_index: 3,
        texture_transform: None,
    });
    let native = m2::Ribbon::try_from(trail.clone()).unwrap();
    assert_eq!(native.emission_rate, 0.5);
    assert_eq!(native.controls.angle, 1.25);
    trail.appearance.atlas[0] = 70_000;
    assert_eq!(
        m2::Ribbon::try_from(trail).unwrap_err().appearance.atlas[0],
        70_000
    );
}

#[test]
fn programmed_ribbon_composition_preserves_native_controls_and_borrowed_splines() {
    use std::borrow::Cow;
    use std::ptr;
    use wc3::model::semantic::profiles::{m3, Universal};
    use wc3::model::semantic::{Model, Node, ParticleScalar, ProgrammedRibbon};
    fn variation() -> m3::VariationControl {
        m3::VariationControl {
            kind: Default::default(),
            amplitude: property(),
            frequency: property(),
        }
    }
    fn property() -> m3::Property<f32> {
        m3::Property {
            initial: 2.5,
            reset: -1.25,
            control: m3::AnimationControl {
                source_id: 71,
                reserved: -9,
                ..Default::default()
            },
            binding: None,
        }
    }
    let mut builder = Model::<Universal>::builder("ribbon components");
    let node = builder.add_node(Node::named("ribbon spline")).unwrap();
    let spline = [m3::SplinePoint {
        node,
        emission_offset: [1., 2., 3.],
        emission_vector: [0., 1., 0.],
        velocity: property(),
        reserved: 0xf123_4567,
        velocity_base_factor: property(),
        velocity_end_factor: property(),
        yaw_variation: variation(),
        pitch_variation: variation(),
        velocity_variation: variation(),
        yaw: property(),
        pitch: property(),
        emission_vector_norm_factor: 0.75,
        velocity_norm_factor: 1.5,
    }];
    let source = m3::Ribbon {
        fallback_node: node,
        material: Default::default(),
        additional_flags: Default::default(),
        motion: ParticleMotion {
            speed: property(),
            variation: property(),
            gravity: ProgrammedTrailGravity {
                x: 1.375,
                y: -0.625,
                magnitude: 14.125,
            },
        },
        direction: ProgrammedDirection {
            initial_yaw: property(),
            initial_pitch: property(),
            initial_horizontal: property(),
            initial_vertical: property(),
            drag: 28.125,
        },
        lifetime: ParticleLifetime {
            duration: property(),
            variation: property(),
        },
        kill_radius: 0xf123_4567,
        appearance: ProgrammedAge {
            size_mid_time: 15.125,
            color_mid_time: 16.125,
            alpha_mid_time: 17.125,
            rotation_mid_time: 18.125,
            size_mid_hold_time: 19.125,
            color_mid_hold_time: 20.125,
            alpha_mid_hold_time: 21.125,
            rotation_mid_hold_time: 22.125,
            size_animation: Default::default(),
            rotation_animation: Default::default(),
            color_start: Default::default(),
            color_mid: Default::default(),
            color_end: Default::default(),
        },
        forces: ProgrammedForces {
            mass: 29.125,
            mass_random: 30.125,
            mass_size_multiplier: 31.125,
            local_forces: Default::default(),
            world_forces: Default::default(),
            local_forces_fallback: Default::default(),
            world_forces_fallback: Default::default(),
            world_forces_mass_multiplier: 36.125,
        },
        noise: ProgrammedNoise {
            noise_amplitude: 37.125,
            noise_frequency: 38.125,
            noise_coherence: 39.125,
            noise_edge: 40.125,
        },
        index_plus_length: 51,
        emitter_shape: 52,
        ribbon_type: Default::default(),
        divisions: 44.125,
        edges: 55,
        inner_radius: 46.125,
        max_length: property(),
        spline_points: Cow::Borrowed(&spline),
        active: Default::default(),
        flags: Default::default(),
        size_smoothing: Default::default(),
        color_smoothing: Default::default(),
        friction: 53.125,
        bounce: 54.125,
        lod_reduce: 65,
        lod_cut: 66,
        yaw: variation(),
        pitch: variation(),
        speed: variation(),
        size: variation(),
        alpha: variation(),
        particle_velocity: property(),
        overlay: property(),
        deprecated_unknown: -17,
    };
    let mut ribbon = ProgrammedRibbon::from(source.clone());
    assert!(matches!(&ribbon.spline_points, Cow::Borrowed(_)));
    assert!(ptr::eq(ribbon.spline_points.as_ptr(), spline.as_ptr()));
    assert_eq!(m3::Ribbon::try_from(ribbon.clone()).unwrap(), source);
    let ParticleScalar::Programmed(duration) = &mut ribbon.lifetime.duration else {
        unreachable!()
    };
    duration.initial = 7.75;
    let edited = m3::Ribbon::try_from(ribbon.clone()).unwrap();
    assert_eq!(edited.lifetime.duration.initial, 7.75);
    assert_eq!(
        edited.lifetime.duration.reset,
        source.lifetime.duration.reset
    );
    assert_eq!(edited.motion.gravity.x, 1.375);
    assert_eq!(edited.motion.gravity.y, -0.625);
    ribbon.motion.speed = ParticleScalar::Animated(Animated::constant(33.));
    let rejected = m3::Ribbon::try_from(ribbon).unwrap_err();
    let ParticleScalar::Animated(speed) = &rejected.motion.speed else {
        unreachable!()
    };
    assert_eq!(speed.default, 33.);
    assert!(ptr::eq(rejected.spline_points.as_ptr(), spline.as_ptr()));
    assert_eq!(rejected.kill_radius, 0xf123_4567);
}
