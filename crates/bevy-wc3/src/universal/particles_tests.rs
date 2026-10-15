use super::*;
use std::f32::consts::TAU;
use wc3::model::profiles::Universal;
use wc3::model::{AnimationProgram, AnimationSample};
fn emitter() -> Emitter {
    let mut builder = Model::<Universal>::builder("particle identity");
    let node = builder.reserve_node();
    let feature = builder.reserve_feature(node);
    let mut flags = ParticleFlags::default();
    flags.set_billboard(true);
    Emitter {
        feature,
        node,
        node_index: 0,
        rate: Animated::constant(10.),
        rate_variation: 0.,
        activation: ParticleActivation::Boolean(Animated::constant(true)),
        speed: Animated::constant(10.),
        speed_variation: Animated::constant(0.1),
        gravity: ParticleGravity::Directional(Animated::constant(0.)),
        lifespan: Animated::constant(2.),
        lifespan_variation: Animated::constant(0.),
        vertical: Animated::constant(0.5),
        horizontal: Animated::constant(TAU),
        width: Animated::constant(0.),
        length: Animated::constant(0.),
        z_source: Animated::constant(0.),
        generator: ParticleGenerator::Plane,
        flags,
        drag: 0.,
        wind: Vec3::ZERO,
        inherit_velocity: 0.,
        spin: [0.; 4],
        appearance: ParticleAgeCurves::default(),
        atlas: [1, 1],
        texture_scale: 1.,
        head_tail: ParticleHeadTail::Head,
        tail_length: 0.,
        seed: 7,
        material: particle_material(ParticleBlend::Alpha, None, false),
    }
}
#[test]
fn birth_randomness_is_a_function_of_emitter_and_ordinal() {
    let model = Model::empty("sample").unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let emitter = emitter();
    let first = emitter
        .birth(&sample, 1., Mat4::IDENTITY, Vec3::ZERO, 17)
        .unwrap();
    let repeated = emitter
        .birth(&sample, 1., Mat4::IDENTITY, Vec3::ZERO, 17)
        .unwrap();
    let next = emitter
        .birth(&sample, 1., Mat4::IDENTITY, Vec3::ZERO, 18)
        .unwrap();
    assert_eq!(first.velocity, repeated.velocity);
    assert_eq!(first.spin, repeated.spin);
    assert_ne!(first.velocity, next.velocity);
}
#[test]
fn local_particles_keep_local_origins_while_world_births_detach() {
    let model = Model::empty("sample").unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let mut emitter = emitter();
    let node = Mat4::from_translation(Vec3::X * 10.);
    let world = emitter.birth(&sample, 0., node, Vec3::ZERO, 0).unwrap();
    assert_eq!(world.origin, Vec3::X * 10.);
    emitter.flags.set_model_space(true);
    let local = emitter.birth(&sample, 0., node, Vec3::ZERO, 0).unwrap();
    assert_eq!(local.origin, Vec3::ZERO);
    assert_eq!(
        Mat4::from_translation(Vec3::X * 20.).transform_point3(local.origin),
        Vec3::X * 20.
    );
    assert_eq!(world.origin, Vec3::X * 10.);
}
#[test]
fn analytic_drag_matches_exponential_velocity_and_displacement() {
    let model = Model::empty("sample").unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let mut emitter = emitter();
    emitter.drag = 2.;
    let mut birth = emitter
        .birth(&sample, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    birth.velocity = Vec3::X * 10.;
    birth.acceleration = Vec3::ZERO;
    let (position, velocity) = emitter.motion(&birth, 1.);
    assert!((position.x - 5. * (1. - (-2.0f32).exp())).abs() < 1e-5);
    assert!((velocity.x - 10. * (-2.0f32).exp()).abs() < 1e-5);
}
#[test]
fn appearance_uses_normalized_particle_age_and_reuses_mesh_buffers() {
    let model = Model::empty("sample").unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let mut emitter = emitter();
    emitter.speed = Animated::constant(0.);
    emitter.appearance.alpha = Some(
        Curve::linear(
            [
                ModelTime::seconds(0.).unwrap(),
                ModelTime::seconds(1.).unwrap(),
            ],
            [1.0f32, 0.],
        )
        .unwrap(),
    );
    let birth = emitter
        .birth(&sample, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    let mut state = EmitterInstance {
        entity: Entity::PLACEHOLDER,
        mesh: Handle::default(),
        births: vec![birth],
        order: Vec::new(),
        fraction: 0.,
        ordinal: 0,
        previous_node: Mat4::IDENTITY,
        dropped: 0,
        skipped_seconds: 0.,
    };
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let frame = ParticleFrame {
        selection: Default::default(),
        time: 1.,
        sample: &sample,
        node_poses: &[Mat4::IDENTITY],
        node_visible: &[true],
        basis_world: Mat4::IDENTITY,
        camera_world: Some(Mat4::IDENTITY),
    };
    emitter.draw(&mut state, &frame, Mat4::IDENTITY, &mut mesh);
    let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
    else {
        panic!("colors absent")
    };
    assert_eq!(colors.len(), 6);
    assert!(colors.iter().all(|color| (color[3] - 0.5).abs() < 1e-5));
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions absent")
    };
    let pointer = positions.as_ptr();
    emitter.draw(&mut state, &frame, Mat4::IDENTITY, &mut mesh);
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions absent")
    };
    assert_eq!(pointer, positions.as_ptr());
}
