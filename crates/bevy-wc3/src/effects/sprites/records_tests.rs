use super::*;

fn emitter() -> StagedSpriteParameters {
    StagedSpriteParameters {
        colors: [[1.; 4]; 3],
        scales: [1.; 3],
        middle: 0.5,
        intervals: [[0, 0, 1]; 4],
        atlas: [1, 1],
        model_space: false,
        xy_quad: false,
        tail_length: 0.,
        unlit: false,
        fog_enabled: true,
        fog_blend: 0,
        plane_axes: [Vec3::X, Vec3::Y],
    }
}

#[test]
fn xy_facing_survives_gravity_and_tail_conversion() {
    let record = ParticleInstance::new(
        Vec3::ZERO,
        Vec3::new(0.0, 2.0, 3.0),
        20.0,
        0.0,
        4.0,
        Vec3::new(2.0, 0.5, 3.0),
        false,
    );
    let facing = record.scale_facing[3];
    assert!((facing - (-PI * 0.5 + FRAC_PI_8)).abs() < 0.00001);
    let uniform = ParticleEmitterUniform::from_staged(&emitter(), Mat4::IDENTITY, 1.0);
    assert!(record.center(&uniform).z < 0.0);
    assert_eq!(record.as_tail().scale_facing, record.scale_facing);
    for velocity in [Vec3::ZERO, Vec3::Z, -Vec3::Z] {
        let stationary =
            ParticleInstance::new(Vec3::ZERO, velocity, 0.0, 0.0, 4.0, Vec3::ONE, false);
        assert!(stationary.scale_facing[3].is_finite());
    }
}

#[test]
fn unshaded_is_a_render_flag_independent_of_clock_and_tail_length() {
    let mut definition = emitter();
    let shaded = ParticleEmitterUniform::from_staged(&definition, Mat4::IDENTITY, 12.0);
    definition.unlit = true;
    let unshaded = ParticleEmitterUniform::from_staged(&definition, Mat4::IDENTITY, 12.0);
    assert_eq!(shaded.render_flags[0], 0);
    assert_eq!(unshaded.render_flags[0], 1);
    assert_eq!(shaded.clock_tail, unshaded.clock_tail);
}

#[test]
fn analytic_motion_and_model_space_follow_current_transform() {
    let record = ParticleInstance::new(
        Vec3::new(1.0, 2.0, 3.0),
        Vec3::new(4.0, 5.0, 6.0),
        2.0,
        10.0,
        5.0,
        Vec3::ONE,
        false,
    );
    let mut definition = emitter();
    let transform = GlobalTransform::from_translation(Vec3::new(100.0, 200.0, 300.0));
    let uniform = ParticleEmitterUniform::from_staged(&definition, transform.to_matrix(), 12.0);
    assert_eq!(record.center(&uniform), Vec3::new(9.0, 12.0, 11.0));
    definition.model_space = true;
    let uniform = ParticleEmitterUniform::from_staged(&definition, transform.to_matrix(), 12.0);
    assert_eq!(record.center(&uniform), Vec3::new(109.0, 212.0, 311.0));
}

#[test]
fn split_clock_preserves_short_ages_after_long_runtime() {
    let birth = 1_000_000_000.0;
    let record = ParticleInstance::new(Vec3::ZERO, Vec3::X, 0.0, birth, 1.0, Vec3::ONE, false);
    let uniform = ParticleEmitterUniform::from_staged(&emitter(), Mat4::IDENTITY, birth + 0.125);
    assert_eq!(record.center(&uniform), Vec3::new(0.125, 0.0, 0.0));
}

#[test]
fn vector_acceleration_and_drag_match_transport_sorting_motion() {
    let record = ParticleInstance::new(Vec3::ZERO, Vec3::X * 10., 0., 0., 4., Vec3::ONE, false)
        .with_acceleration(Vec3::Y * 2., 2.);
    let uniform = ParticleEmitterUniform::from_staged(&emitter(), Mat4::IDENTITY, 1.);
    let response = (1. - (-2.0f32).exp()) / 2.;
    let expected = Vec3::X * (10. * response) + Vec3::Y * (1. - response);
    assert!((record.center(&uniform) - expected).length() < 1e-5);
}

#[test]
fn fog_and_lighting_are_independent_of_motion_and_each_other() {
    let mut definition = emitter();
    let fogged = ParticleEmitterUniform::from_staged(&definition, Mat4::IDENTITY, 1.);
    definition.fog_enabled = false;
    definition.unlit = true;
    let unfogged = ParticleEmitterUniform::from_staged(&definition, Mat4::IDENTITY, 1.);
    assert_eq!(fogged.render_flags, [0, 1, 0, 0]);
    assert_eq!(unfogged.render_flags, [1, 0, 0, 0]);
    assert_eq!(fogged.clock_tail, unfogged.clock_tail);
    assert_eq!(fogged.world_from_local, unfogged.world_from_local);
}

#[test]
fn stationary_certificate_is_sufficient_for_bit_identical_centers_in_its_clock_domain() {
    let mut record = ParticleInstance::new(
        Vec3::new(-0.0, 0.0, -7.),
        Vec3::new(-0.0, 0.0, -0.0),
        0.,
        0.,
        10.,
        Vec3::ONE,
        false,
    )
    .with_acceleration(Vec3::new(-0.0, -0.0, 0.0), 0.);
    assert!(record.stationary());
    for birth in [0., 0.125, -0.125, 1e12, -1e12, 2f64.powi(60)] {
        let [high, low] = split_time(birth);
        record.position_birth[3] = high;
        record.lifetime_tail_birth[2] = low;
        assert!(record.stationary());
        let initial = record
            .center(&ParticleEmitterUniform::from_staged(
                &emitter(),
                Mat4::IDENTITY,
                birth,
            ))
            .to_array()
            .map(f32::to_bits);
        for now in [
            -2f64.powi(60),
            -1e12,
            -1.,
            0.,
            0.125,
            1.,
            1e12,
            2f64.powi(60),
        ] {
            let uniform = ParticleEmitterUniform::from_staged(&emitter(), Mat4::IDENTITY, now);
            assert!(bounded_clock([
                uniform.clock_tail[0],
                uniform.clock_tail[1]
            ]));
            assert_eq!(
                record.center(&uniform).to_array().map(f32::to_bits),
                initial
            );
        }
    }
    record.velocity_gravity[0] = f32::MIN_POSITIVE;
    assert!(!record.stationary());
    record.velocity_gravity[0] = 0.;
    record.acceleration_drag[0] = f32::NAN;
    assert!(!record.stationary());
    record.acceleration_drag[0] = 0.;
    record.acceleration_drag[3] = 0.000001; // Even zero-drag branch damped inputs remain conservative.
    assert!(!record.stationary());
    record.acceleration_drag[3] = 0.;
    record.position_birth[3] = f32::INFINITY;
    assert!(!record.stationary());
}

#[test]
fn center_dependencies_exclude_appearance_but_keep_actual_frame_and_unsafe_clock_inputs() {
    let mut parameters = emitter();
    let a = ParticleEmitterUniform::from_staged(&parameters, Mat4::IDENTITY, 1.);
    parameters.colors = [[0.25; 4]; 3];
    parameters.scales = [3., 4., 5.];
    parameters.tail_length = 100.;
    parameters.xy_quad = true;
    parameters.atlas = [4, 8];
    parameters.fog_enabled = false;
    parameters.unlit = true;
    let b = ParticleEmitterUniform::from_staged(
        &parameters,
        Mat4::from_translation(Vec3::splat(40.)),
        2.,
    );
    assert!(a.center_dependency(true).same(&b.center_dependency(true)));
    assert!(!a.center_dependency(false).same(&b.center_dependency(false)));
    parameters.model_space = true;
    let attached = ParticleEmitterUniform::from_staged(&parameters, Mat4::IDENTITY, 1.);
    assert!(!attached
        .center_dependency(true)
        .same(&a.center_dependency(true)));
    for matrix in [
        Mat4::from_translation(Vec3::X),
        Mat4::from_scale(Vec3::new(-2., 3., 4.)),
        Mat4::from_rotation_y(0.5),
    ] {
        let moved = ParticleEmitterUniform::from_staged(&parameters, matrix, 2.);
        assert!(!attached
            .center_dependency(true)
            .same(&moved.center_dependency(true)));
    }
    for time in [f64::INFINITY, 1e30, 2e30] {
        let extreme = ParticleEmitterUniform::from_staged(&parameters, Mat4::IDENTITY, time);
        assert!(!attached
            .center_dependency(true)
            .same(&extreme.center_dependency(true)));
    }
    let boundary = 2f64.powi(60);
    let at_boundary = ParticleEmitterUniform::from_staged(&emitter(), Mat4::IDENTITY, boundary);
    let outside = ParticleEmitterUniform::from_staged(&emitter(), Mat4::IDENTITY, boundary * 2.);
    assert!(a
        .center_dependency(true)
        .same(&at_boundary.center_dependency(true)));
    assert!(!a
        .center_dependency(true)
        .same(&outside.center_dependency(true)));
    assert!(!bounded_clock([-0.0, 0.]));
    assert!(!bounded_clock([0., f32::NAN]));
}
