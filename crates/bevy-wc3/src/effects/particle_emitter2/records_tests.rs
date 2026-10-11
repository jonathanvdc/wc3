use super::*;
use wc3::model::scene::Node;

fn emitter() -> ParticleEmitter2 {
    ParticleEmitter2::new(Node::new("Particles", 0).unwrap())
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
    let uniform = ParticleEmitterUniform::new(&emitter(), &GlobalTransform::default(), 1.0);
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
    let shaded = ParticleEmitterUniform::new(&definition, &GlobalTransform::default(), 12.0);
    definition.node.flags.set_unshaded(true);
    let unshaded = ParticleEmitterUniform::new(&definition, &GlobalTransform::default(), 12.0);
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
    let uniform = ParticleEmitterUniform::new(&definition, &transform, 12.0);
    assert_eq!(record.center(&uniform), Vec3::new(9.0, 12.0, 11.0));
    definition.node.flags.set_model_space(true);
    let uniform = ParticleEmitterUniform::new(&definition, &transform, 12.0);
    assert_eq!(record.center(&uniform), Vec3::new(109.0, 212.0, 311.0));
}

#[test]
fn split_clock_preserves_short_ages_after_long_runtime() {
    let birth = 1_000_000_000.0;
    let record = ParticleInstance::new(Vec3::ZERO, Vec3::X, 0.0, birth, 1.0, Vec3::ONE, false);
    let uniform =
        ParticleEmitterUniform::new(&emitter(), &GlobalTransform::default(), birth + 0.125);
    assert_eq!(record.center(&uniform), Vec3::new(0.125, 0.0, 0.0));
}
