use wc3::model::profiles::{m2, mdlx};
use wc3::model::{AngleTrailControls, SurfaceTrailRendering};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{
    GravityTrailControls, ParticleLifetime, ParticleMotion, ProgrammedAge, ProgrammedDirection,
    ProgrammedForces, ProgrammedNoise, ProgrammedTrailGravity, TrailAppearance,
};
use wc3::model::{TrailAtlasSlot, TrailControls, TrailEmitter, TrailRendering};

#[test]
fn trail_editing_preserves_wide_integer_rates_and_rejects_fractional_export() {
    let mut slot_values = SlotValues::default();
    let mut trail = TrailEmitter::from(mdlx::Ribbon {
        emission_rate: u32::MAX,
        appearance: TrailAppearance {
            texture_slot: slot_values.add(70_000),
            ..SlotConstruct::construct(&mut slot_values)
        },
        controls: GravityTrailControls {
            gravity: 9.8,
            ..SlotConstruct::construct(&mut slot_values)
        },
        ..SlotConstruct::construct(&mut slot_values)
    });
    assert_eq!(trail.emission_rate, f64::from(u32::MAX));
    slot_values.set(trail.geometry.height_above, 2.5).unwrap();
    let native = mdlx::Ribbon::try_from(trail.clone()).unwrap();
    assert_eq!(slot_values[native.geometry.height_above], 2.5);
    assert_eq!(native.emission_rate, u32::MAX);
    trail.emission_rate = 0.5;
    let rejected = mdlx::Ribbon::try_from(trail).unwrap_err();
    assert_eq!(rejected.emission_rate, 0.5);
}

#[test]
fn edge_trail_export_checks_numeric_width_without_conflating_direction_and_gravity() {
    let mut slot_values = SlotValues::default();
    let mut trail = TrailEmitter::from(mdlx::Ribbon::construct(&mut slot_values));
    trail.emission_rate = 0.5;
    trail.appearance.texture_slot = TrailAtlasSlot::Compact(slot_values.add(7));
    trail.rendering = TrailRendering::Surfaces(SurfaceTrailRendering {
        textures: vec![],
        materials: vec![],
    });
    trail.controls = TrailControls::EmissionAngle(AngleTrailControls {
        angle: 1.25,
        id: 9,
        enabled: slot_values.add(true),
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
    let mut slot_values = SlotValues::default();
    use std::borrow::Cow;
    use std::ptr;
    use wc3::model::profiles::{m3, Universal};
    use wc3::model::{Model, ParticleScalar, ProgrammedRibbon};
    fn variation(slot_values: &mut SlotValues) -> m3::VariationControl {
        m3::VariationControl {
            kind: SlotConstruct::construct(slot_values),
            amplitude: property(slot_values),
            frequency: property(slot_values),
        }
    }
    fn property(slot_values: &mut SlotValues) -> m3::PropertyBinding<f32> {
        m3::PropertyBinding {
            value: slot_values.add(2.5),
            reset: -1.25,
            control: m3::AnimationControl {
                source_id: 71,
                reserved: -9,
                ..SlotConstruct::construct(slot_values)
            },
        }
    }
    let mut builder = Model::<Universal>::builder("ribbon components");
    let node = builder.add_named_node("ribbon spline").unwrap();
    let spline = [m3::SplinePoint {
        node,
        emission_offset: [1., 2., 3.],
        emission_vector: [0., 1., 0.],
        velocity: property(&mut slot_values),
        reserved: 0xf123_4567,
        velocity_base_factor: property(&mut slot_values),
        velocity_end_factor: property(&mut slot_values),
        yaw_variation: variation(&mut slot_values),
        pitch_variation: variation(&mut slot_values),
        velocity_variation: variation(&mut slot_values),
        yaw: property(&mut slot_values),
        pitch: property(&mut slot_values),
        emission_vector_norm_factor: 0.75,
        velocity_norm_factor: 1.5,
    }];
    let source = m3::Ribbon {
        fallback_node: node,
        material: SlotConstruct::construct(&mut slot_values),
        additional_flags: SlotConstruct::construct(&mut slot_values),
        motion: ParticleMotion {
            speed: property(&mut slot_values),
            variation: property(&mut slot_values),
            gravity: ProgrammedTrailGravity {
                x: 1.375,
                y: -0.625,
                magnitude: 14.125,
            },
        },
        direction: ProgrammedDirection {
            initial_yaw: property(&mut slot_values),
            initial_pitch: property(&mut slot_values),
            initial_horizontal: property(&mut slot_values),
            initial_vertical: property(&mut slot_values),
            drag: 28.125,
        },
        lifetime: ParticleLifetime {
            duration: property(&mut slot_values),
            variation: property(&mut slot_values),
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
            size_animation: SlotConstruct::construct(&mut slot_values),
            rotation_animation: SlotConstruct::construct(&mut slot_values),
            color_start: SlotConstruct::construct(&mut slot_values),
            color_mid: SlotConstruct::construct(&mut slot_values),
            color_end: SlotConstruct::construct(&mut slot_values),
        },
        forces: ProgrammedForces {
            mass: 29.125,
            mass_random: 30.125,
            mass_size_multiplier: 31.125,
            local_forces: SlotConstruct::construct(&mut slot_values),
            world_forces: SlotConstruct::construct(&mut slot_values),
            local_forces_fallback: SlotConstruct::construct(&mut slot_values),
            world_forces_fallback: SlotConstruct::construct(&mut slot_values),
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
        ribbon_type: SlotConstruct::construct(&mut slot_values),
        divisions: 44.125,
        edges: 55,
        inner_radius: 46.125,
        max_length: property(&mut slot_values),
        spline_points: Cow::Borrowed(&spline),
        active: SlotConstruct::construct(&mut slot_values),
        flags: SlotConstruct::construct(&mut slot_values),
        size_smoothing: SlotConstruct::construct(&mut slot_values),
        color_smoothing: SlotConstruct::construct(&mut slot_values),
        friction: 53.125,
        bounce: 54.125,
        lod_reduce: 65,
        lod_cut: 66,
        yaw: variation(&mut slot_values),
        pitch: variation(&mut slot_values),
        speed: variation(&mut slot_values),
        size: variation(&mut slot_values),
        alpha: variation(&mut slot_values),
        particle_velocity: property(&mut slot_values),
        overlay: property(&mut slot_values),
        deprecated_unknown: -17,
    };
    let mut ribbon = ProgrammedRibbon::from(source.clone());
    assert!(matches!(&ribbon.spline_points, Cow::Borrowed(_)));
    assert!(ptr::eq(ribbon.spline_points.as_ptr(), spline.as_ptr()));
    assert_eq!(m3::Ribbon::try_from(ribbon.clone()).unwrap(), source);
    let ParticleScalar::Programmed(duration) = &mut ribbon.lifetime.duration else {
        unreachable!()
    };
    slot_values.set(duration.value, 7.75).unwrap();
    let edited = m3::Ribbon::try_from(ribbon.clone()).unwrap();
    assert_eq!(slot_values[edited.lifetime.duration.value], 7.75);
    assert_eq!(
        edited.lifetime.duration.reset,
        source.lifetime.duration.reset
    );
    assert_eq!(edited.motion.gravity.x, 1.375);
    assert_eq!(edited.motion.gravity.y, -0.625);
    ribbon.motion.speed = ParticleScalar::Property(slot_values.add(33.));
    let rejected = m3::Ribbon::try_from(ribbon).unwrap_err();
    let ParticleScalar::Property(speed) = &rejected.motion.speed else {
        unreachable!()
    };
    assert_eq!(slot_values[speed], 33.);
    assert!(ptr::eq(rejected.spline_points.as_ptr(), spline.as_ptr()));
    assert_eq!(rejected.kill_radius, 0xf123_4567);
}
