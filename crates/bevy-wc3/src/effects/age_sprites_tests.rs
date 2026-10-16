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
        material_animation: None,
        programmed: None,
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
        .birth(
            &sample.at_times(Default::default(), Default::default(), Default::default()),
            1.,
            Mat4::IDENTITY,
            Vec3::ZERO,
            17,
        )
        .unwrap();
    let repeated = emitter
        .birth(
            &sample.at_times(Default::default(), Default::default(), Default::default()),
            1.,
            Mat4::IDENTITY,
            Vec3::ZERO,
            17,
        )
        .unwrap();
    let next = emitter
        .birth(
            &sample.at_times(Default::default(), Default::default(), Default::default()),
            1.,
            Mat4::IDENTITY,
            Vec3::ZERO,
            18,
        )
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
    let world = emitter
        .birth(
            &sample.at_times(Default::default(), Default::default(), Default::default()),
            0.,
            node,
            Vec3::ZERO,
            0,
        )
        .unwrap();
    assert_eq!(world.origin, Vec3::X * 10.);
    emitter.flags.set_model_space(true);
    let local = emitter
        .birth(
            &sample.at_times(Default::default(), Default::default(), Default::default()),
            0.,
            node,
            Vec3::ZERO,
            0,
        )
        .unwrap();
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
        .birth(
            &sample.at_times(Default::default(), Default::default(), Default::default()),
            0.,
            Mat4::IDENTITY,
            Vec3::ZERO,
            0,
        )
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
        .birth(
            &sample.at_times(Default::default(), Default::default(), Default::default()),
            0.,
            Mat4::IDENTITY,
            Vec3::ZERO,
            0,
        )
        .unwrap();
    let mut state = EmitterInstance {
        entity: Entity::PLACEHOLDER,
        mesh: Handle::default(),
        material: Handle::default(),
        births: vec![birth],
        appearances: Vec::new(),
        order: Vec::new(),
        fraction: 0.,
        ordinal: 0,
        previous_node: Mat4::IDENTITY,
        pose_scratch: Vec::new(),
        dropped: 0,
        skipped_seconds: 0.,
    };
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let frame = EffectFrame {
        traversal: Default::default(),
        change: Default::default(),
        root_visible: true,
        pose: None,
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
    state.births.clear();
    emitter.draw(&mut state, &frame, Mat4::IDENTITY, &mut mesh);
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions absent")
    };
    assert_eq!(positions.as_slice(), &[[0.; 3]; 3]);
    assert_eq!(positions.as_ptr(), pointer);
    let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
    else {
        panic!("colors absent")
    };
    assert_eq!(colors.as_slice(), &[[0.; 4]; 3]);

    // Mark the resident sentinel to detect any attribute rewrite by update.
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[123.; 3]; 3]);
    let mut meshes = Assets::<Mesh>::default();
    state.mesh = meshes.add(mesh);
    let handle = state.mesh.clone();
    let mut world = World::new();
    let mut query_state = world.query::<&mut SpriteEmitter>();
    let mut materials = Assets::<ModelMaterial>::default();
    emitter.rate = Animated::constant(0.);
    let runtime = RuntimeAgeSprites {
        emitters: vec![emitter],
        ..Default::default()
    };
    let mut instances = AgeSpriteInstances {
        emitters: vec![state],
        previous_time: Some(0.),
        ..Default::default()
    };
    runtime.update(
        &mut instances,
        frame,
        &mut meshes,
        &mut query_state.query_mut(&mut world),
        &mut materials,
    );
    let Some(VertexAttributeValues::Float32x3(positions)) = meshes
        .get(&handle)
        .unwrap()
        .attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions absent")
    };
    assert_eq!(
        positions.as_slice(),
        &[[123.; 3]; 3],
        "empty-to-empty must not rewrite resident attributes"
    );
    let mut expired = runtime.emitters[0]
        .birth(
            &sample.at_times(Default::default(), Default::default(), Default::default()),
            -10.,
            Mat4::IDENTITY,
            Vec3::ZERO,
            0,
        )
        .unwrap();
    expired.lifetime = 1.;
    instances.emitters[0].births.push(expired);
    instances.previous_time = Some(0.);
    runtime.update(
        &mut instances,
        frame,
        &mut meshes,
        &mut query_state.query_mut(&mut world),
        &mut materials,
    );
    let Some(VertexAttributeValues::Float32x3(positions)) = meshes
        .get(&handle)
        .unwrap()
        .attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions absent")
    };
    assert_eq!(
        positions.as_slice(),
        &[[0.; 3]; 3],
        "last expiry must upload the empty sentinel"
    );
}

#[test]
fn programmed_appearance_storage_preserves_compact_m2_stride_and_retires_in_lockstep() {
    let birth_size = std::mem::size_of::<Birth>();
    let appearance_size = std::mem::size_of::<ProgrammedAppearance>();
    let inline_size = std::mem::size_of::<(Birth, Option<ProgrammedAppearance>)>();
    println!("Birth={birth_size}, ProgrammedAppearance={appearance_size}, previous inline stride={inline_size}");
    assert!(birth_size <= 96, "M2 birth records must remain compact");
    assert!(inline_size > birth_size * 2);
    let model = Model::empty("appearance retirement").unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let sample = sample.at_times(Default::default(), Default::default(), Default::default());
    let birth = emitter()
        .birth(&sample, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    let mut state = EmitterInstance {
        entity: Entity::PLACEHOLDER,
        mesh: Handle::default(),
        material: Handle::default(),
        births: (0..3)
            .map(|index| {
                let mut birth = birth.clone();
                birth.lifetime = if index == 0 { 1. } else { 10. };
                birth.appearance_index = index;
                birth
            })
            .collect(),
        appearances: vec![
            ProgrammedAppearance::constant(1.),
            ProgrammedAppearance::constant(2.),
            ProgrammedAppearance::constant(3.),
        ],
        order: Vec::new(),
        fraction: 0.,
        ordinal: 0,
        previous_node: Mat4::IDENTITY,
        pose_scratch: Vec::new(),
        dropped: 0,
        skipped_seconds: 0.,
    };
    let pointer = state.appearances.as_ptr();
    state.retire(2.);
    assert_eq!(state.births.len(), 2);
    assert_eq!(state.appearances.len(), 2);
    assert_eq!(
        state
            .births
            .iter()
            .map(|birth| birth.appearance_index)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(state.appearances[0].sample(0.).size, [2.; 2]);
    assert_eq!(state.appearances[1].sample(0.).size, [3.; 2]);
    assert_eq!(pointer, state.appearances.as_ptr());
    state.retire(20.);
    assert!(state.births.is_empty());
    assert!(state.appearances.is_empty());
    assert_eq!(pointer, state.appearances.as_ptr());
}
