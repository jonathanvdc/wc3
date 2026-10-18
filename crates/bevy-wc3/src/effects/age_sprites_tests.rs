use super::*;
use bevy::ecs::world::CommandQueue;
use std::f32::consts::TAU;
use wc3::model::profiles::Universal;
use wc3::model::{AnimationProgram, AnimationSample};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
fn emitter(slot_values: &mut SlotValues) -> Emitter {
    let mut builder = Model::<Universal>::builder("particle identity");
    let node = builder.reserve_node();
    let feature = builder.reserve_feature(node);
    let flags = ParticleFlags::default();
    Emitter {
        feature,
        node,
        node_index: 0,
        rate: slot_values.add(10.),
        rate_variation: 0.,
        controls: Some(AgeDrivenControls {
            activation: ParticleActivation::Boolean(slot_values.add(true)),
            speed_variation: slot_values.add(0.1),
            gravity: ParticleGravity::Directional(slot_values.add(0.)),
            lifespan_variation: slot_values.add(0.),
            vertical: slot_values.add(0.5),
            horizontal: slot_values.add(TAU),
            width: slot_values.add(0.),
            length: slot_values.add(0.),
            z_source: slot_values.add(0.),
        }),
        speed: slot_values.add(10.),
        lifespan: slot_values.add(2.),
        generator: ParticleGenerator::Plane,
        flags,
        drag: 0.,
        wind: Vec3::ZERO,
        inherit_velocity: 0.,
        follow: [[0.; 2]; 2],
        spin: [0.; 4],
        appearance: ParticleAgeCurves::default(),
        atlas: [1, 1],
        age_appearance: None,
        multi_texture: false,
        head_tail: ParticleHeadTail::Head,
        tail_length: 0.,
        seed: 7,
        programmed: None,
        passes: vec![MaterialPass {
            material: particle_material(ParticleBlend::Alpha, None, false),
            animation: None,
        }],
    }
}
#[test]
fn birth_randomness_is_a_function_of_emitter_and_ordinal() {
    let mut slot_values = SlotValues::default();
    let emitter = emitter(&mut slot_values);
    let first = emitter
        .birth(&slot_values, 1., Mat4::IDENTITY, Vec3::ZERO, 17)
        .unwrap();
    let repeated = emitter
        .birth(&slot_values, 1., Mat4::IDENTITY, Vec3::ZERO, 17)
        .unwrap();
    let next = emitter
        .birth(&slot_values, 1., Mat4::IDENTITY, Vec3::ZERO, 18)
        .unwrap();
    assert_eq!(first.velocity, repeated.velocity);
    assert_eq!(first.spin, repeated.spin);
    assert_ne!(first.velocity, next.velocity);
}
#[test]
fn local_particles_keep_local_origins_while_world_births_detach() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    let node = Mat4::from_translation(Vec3::X * 10.);
    let world = emitter
        .birth(&slot_values, 0., node, Vec3::ZERO, 0)
        .unwrap();
    assert_eq!(world.origin, Vec3::X * 10.);
    emitter.flags.set_attached(true);
    let local = emitter
        .birth(&slot_values, 0., node, Vec3::ZERO, 0)
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
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    emitter.drag = 2.;
    let mut birth = emitter
        .birth(&slot_values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    birth.velocity = Vec3::X * 10.;
    birth.acceleration = Vec3::ZERO;
    let (position, velocity) = emitter.motion(&birth, 1.);
    assert!((position.x - 5. * (1. - (-2.0f32).exp())).abs() < 1e-5);
    assert!((velocity.x - 10. * (-2.0f32).exp()).abs() < 1e-5);
}
#[test]
fn appearance_uses_normalized_particle_age_and_reuses_mesh_buffers() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    emitter.speed = slot_values.add(0.);
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
        .birth(&slot_values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    let mut state = EmitterInstance {
        mesh: Handle::default(),
        passes: vec![],
        births: vec![birth],
        squirts: SquirtSchedule::default(),
        appearances: Vec::new(),
        order: Vec::new(),
        age_appearances: Vec::new(),
        fraction: 0.,
        ordinal: 0,
        previous_node: Mat4::IDENTITY,
        pose_scratch: Vec::new(),
        dropped: 0,
        skipped_seconds: 0.,
    };
    let mut builder = Model::<Universal>::builder("effect sample");
    builder.import_values(&slot_values).unwrap();
    let model = builder.finish().unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let frame = EffectFrame {
        traversal: SlotConstruct::construct(&mut slot_values),
        change: SlotConstruct::construct(&mut slot_values),
        root_visible: true,
        pose: None,
        selection: SlotConstruct::construct(&mut slot_values),
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
    emitter.rate = emitter.speed;
    let runtime = RuntimeAgeSprites {
        emitters: vec![emitter],
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut instances = AgeSpriteInstances {
        emitters: vec![state],
        previous_time: Some(0.),
        ..SlotConstruct::construct(&mut slot_values)
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
        .birth(&slot_values, -10., Mat4::IDENTITY, Vec3::ZERO, 0)
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
    let mut slot_values = SlotValues::default();
    let birth_size = std::mem::size_of::<Birth>();
    let appearance_size = std::mem::size_of::<ProgrammedAppearance>();
    let inline_size = std::mem::size_of::<(Birth, Option<ProgrammedAppearance>)>();
    println!(
        "Birth={birth_size}, ProgrammedAppearance={appearance_size}, previous inline stride={inline_size}"
    );
    assert!(birth_size <= 96, "M2 birth records must remain compact");
    assert!(inline_size > birth_size * 2);
    let emitter = emitter(&mut slot_values);
    let birth = emitter
        .birth(&slot_values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    let mut state = EmitterInstance {
        mesh: Handle::default(),
        passes: vec![],
        squirts: SquirtSchedule::default(),
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
        age_appearances: Vec::new(),
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
    state.births = (0..3)
        .map(|index| {
            let mut value = birth.clone();
            value.appearance_index = index;
            value.lifetime = if index == 0 { 1. } else { 10. };
            value
        })
        .collect();
    state.age_appearances = vec![AgeParticleAppearance::default(); 3];
    let pointer = state.age_appearances.as_ptr();
    state.retire(2.);
    assert_eq!(state.age_appearances.len(), 2);
    assert_eq!(
        state
            .births
            .iter()
            .map(|b| b.appearance_index)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(state.age_appearances.as_ptr(), pointer);
    state.retire(20.);
    assert!(state.age_appearances.is_empty());
    assert_eq!(state.age_appearances.as_ptr(), pointer);
}

#[test]
fn emitter_follow_interpolates_speed_knots_and_preserves_local_motion() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    emitter.follow = [[2., 0.25], [6., 0.75]];
    let translated = |distance| Mat4::from_translation(Vec3::X * distance);
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, translated(1.), 1.),
        Vec3::X * 0.25
    );
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, translated(4.), 1.),
        Vec3::X * 2.
    );
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, translated(8.), 1.),
        Vec3::X * 6.
    );
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, translated(4.), 0.),
        Vec3::ZERO
    );
    emitter.follow.swap(0, 1);
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, translated(4.), 1.),
        Vec3::X * 2.
    );
    emitter.flags.set_attached(true);
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, translated(4.), 1.),
        Vec3::ZERO
    );
    emitter.flags.set_attached(false);
    emitter.follow = [[2., 0.5], [2., 0.5]];
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, translated(4.), 1.),
        Vec3::X * 2.
    );
}

#[test]
fn world_particles_follow_only_translation_after_their_birth() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    emitter.follow = [[0., 0.5], [1., 0.5]];
    emitter.rate = slot_values.add(2.);
    emitter.speed = slot_values.add(0.);
    let prior_birth = emitter
        .birth(&slot_values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    let mut builder = Model::<Universal>::builder("effect sample");
    builder.import_values(&slot_values).unwrap();
    let model = builder.finish().unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let mesh = meshes.add(Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    ));
    let state = EmitterInstance {
        mesh,
        passes: vec![],
        births: vec![prior_birth],
        squirts: SquirtSchedule::default(),
        appearances: Vec::new(),
        order: Vec::new(),
        age_appearances: Vec::new(),
        fraction: 0.,
        ordinal: 1,
        previous_node: Mat4::IDENTITY,
        pose_scratch: Vec::new(),
        dropped: 0,
        skipped_seconds: 0.,
    };
    let runtime = RuntimeAgeSprites {
        emitters: vec![emitter],
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut instances = AgeSpriteInstances {
        emitters: vec![state],
        previous_time: Some(0.),
        previous_basis: Some(Mat4::IDENTITY),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut world = World::new();
    let mut query = world.query::<&mut SpriteEmitter>();
    let mut materials = Assets::<ModelMaterial>::default();
    let nodes = [Mat4::from_translation(Vec3::X * 4.)];
    let frame = EffectFrame {
        traversal: SlotConstruct::construct(&mut slot_values),
        change: SlotConstruct::construct(&mut slot_values),
        root_visible: true,
        pose: None,
        selection: SlotConstruct::construct(&mut slot_values),
        time: 1.,
        sample: &sample,
        node_poses: &nodes,
        node_visible: &[true],
        basis_world: Mat4::IDENTITY,
        camera_world: Some(Mat4::IDENTITY),
    };
    runtime.update(
        &mut instances,
        frame,
        &mut meshes,
        &mut query.query_mut(&mut world),
        &mut materials,
    );
    let births = &instances.emitters[0].births;
    assert_eq!(births.len(), 3);
    assert_eq!(births[0].origin, Vec3::X * 2.);
    // The halfway birth starts at x=2, then follows half of the remaining
    // emitter displacement. The endpoint birth has no follow interval yet.
    assert!((births[1].origin.x - 3.).abs() < 1e-5);
    assert!((births[2].origin.x - 4.).abs() < 1e-5);
}

#[test]
fn programmed_passes_share_population_capacity_pause_and_seek() {
    let mut slot_values = SlotValues::default();
    let mut programme = ProgrammedSprite::fixture(&mut slot_values);
    programme.rate = slot_values.add(0.);
    programme.squirt = slot_values.add(7);
    programme.max_particles = 3;
    let mut source = emitter(&mut slot_values);
    let mut second = source.passes[0].clone();
    second.material = super::super::programmed::no_shadow_material();
    second.material.base.base_color = Color::BLACK;
    second.material.base.depth_bias = 3.;
    source.passes.push(second);
    let emitter = Emitter::programmed(source.feature, source.node, 0, programme, source.passes);
    let runtime = RuntimeAgeSprites {
        emitters: vec![emitter],
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut builder = Model::<Universal>::builder("effect sample");
    builder.import_values(&slot_values).unwrap();
    let model = builder.finish().unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<ModelMaterial>::default();
    let mut world = World::new();
    let root = world.spawn_empty().id();
    let mut queue = CommandQueue::default();
    let mut instances = runtime.spawn(
        &mut Commands::new(&mut queue, &world),
        &mut meshes,
        &mut materials,
        root,
    );
    queue.apply(&mut world);
    assert_eq!(instances.emitters.len(), 1);
    assert_eq!(meshes.len(), 1);
    assert_eq!(materials.len(), 2);
    let state = &instances.emitters[0];
    assert_eq!(state.passes.len(), 2);
    assert_eq!(world.query::<&SpriteEmitter>().iter(&world).count(), 1);
    for pass in &state.passes {
        assert_eq!(world.get::<Mesh3d>(pass.entity).unwrap().0, state.mesh);
    }
    assert_ne!(state.passes[0].material, state.passes[1].material);
    assert!(!world
        .entity(state.passes[0].entity)
        .contains::<NotShadowCaster>());
    assert!(!world
        .entity(state.passes[0].entity)
        .contains::<NotShadowReceiver>());
    assert!(world
        .entity(state.passes[1].entity)
        .contains::<NotShadowCaster>());
    assert!(world
        .entity(state.passes[1].entity)
        .contains::<NotShadowReceiver>());
    let second = materials.get(&state.passes[1].material).unwrap();
    assert_eq!(second.base.base_color, Color::BLACK);
    assert_eq!(second.base.depth_bias, 3.);
    let mut query = world.query::<&mut SpriteEmitter>();
    for (time, change) in [
        (0., EffectChange::Continuous),
        (0., EffectChange::Continuous),
        (10., EffectChange::Seek),
    ] {
        let selection = AnimationSample {
            clip_time: ModelTime::seconds(time).unwrap(),
            ..SlotConstruct::construct(&mut slot_values)
        };
        runtime.update(
            &mut instances,
            EffectFrame {
                traversal: SlotConstruct::construct(&mut slot_values),
                change,
                root_visible: true,
                pose: None,
                selection,
                time,
                sample: &sample,
                node_poses: &[Mat4::IDENTITY],
                node_visible: &[true],
                basis_world: Mat4::IDENTITY,
                camera_world: Some(Mat4::IDENTITY),
            },
            &mut meshes,
            &mut query.query_mut(&mut world),
            &mut materials,
        );
        assert_eq!(instances.emitters[0].dropped, 4);
        if let Some(pass) = instances.emitters[0].passes.first() {
            let statistics = world.get::<SpriteEmitter>(pass.entity).unwrap();
            assert_eq!(statistics.alive, instances.emitters[0].births.len());
            assert_eq!(statistics.dropped, 4);
        }
        if time == 0. {
            assert_eq!(instances.emitters[0].births.len(), 3);
            assert_eq!(instances.emitters[0].appearances.len(), 3);
        } else {
            assert!(instances.emitters[0].births.is_empty());
        }
    }
}

#[test]
fn semantic_gravity_is_independent_of_the_emitter_bone_and_scalar_packing_flags() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    emitter.flags.set_packed_gravity(true);
    emitter.controls.as_mut().unwrap().gravity =
        ParticleGravity::Vector(slot_values.add([1., -2., 3.]));
    let node =
        Mat4::from_scale_rotation_translation(Vec3::splat(4.), Quat::from_rotation_x(1.2), Vec3::X);
    let basis = Mat4::from_rotation_y(0.7);
    let (birth, _) = emitter
        .birth_in(&slot_values, 0., node, basis, Vec3::ZERO, 0)
        .unwrap();
    assert!(birth
        .acceleration
        .abs_diff_eq(basis.transform_vector3(Vec3::new(1., -2., 3.)), 1e-6));
    emitter.flags.set_attached(true);
    let (birth, _) = emitter
        .birth_in(&slot_values, 0., node, basis, Vec3::ZERO, 0)
        .unwrap();
    assert_eq!(birth.acceleration, Vec3::new(1., -2., 3.));
}

#[test]
fn sphere_and_plane_use_distinct_polar_axes_and_head_flags_do_not_aim_velocity() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    emitter.controls.as_mut().unwrap().vertical = slot_values.add(0.);
    emitter.controls.as_mut().unwrap().horizontal = slot_values.add(0.);
    emitter.speed = slot_values.add(2.);
    emitter.controls.as_mut().unwrap().speed_variation = slot_values.add(0.);
    emitter.controls.as_mut().unwrap().width = slot_values.add(1.);
    emitter.controls.as_mut().unwrap().length = slot_values.add(1.);
    emitter.flags.set_head(true);
    let plane = emitter
        .birth(&slot_values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    assert_eq!(plane.velocity, Vec3::Z * 2.);
    emitter.generator = ParticleGenerator::Sphere;
    let sphere = emitter
        .birth(&slot_values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    assert_eq!(sphere.origin, Vec3::X);
    assert_eq!(sphere.velocity, Vec3::X * 2.);
    emitter.flags.set_sphere_up(true);
    let up = emitter
        .birth(&slot_values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    assert_eq!(up.origin, Vec3::X);
    assert_eq!(up.velocity, Vec3::Z * 2.);
    emitter.controls.as_mut().unwrap().z_source = slot_values.add(1.);
    let aimed = emitter
        .birth(&slot_values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    assert!(aimed
        .velocity
        .abs_diff_eq(Vec3::new(1., 0., -1.).normalize() * 2., 1e-6));

    assert!(emitter
        .emitter_frame(Mat4::IDENTITY)
        .transform_vector3(Vec3::X)
        .abs_diff_eq(Vec3::Y, 1e-6));
}

#[test]
fn additional_uvs_preserve_primary_atlas_and_remain_independent() {
    let mut buffers = SpriteMeshBuffers {
        positions: Vec::new(),
        normals: Vec::new(),
        uv: Vec::new(),
        uv_secondary: Some(Vec::new()),
        uv_tertiary: Some(Vec::new()),
        colors: Vec::new(),
    };
    let corners = [Vec3::ZERO, Vec3::X, Vec3::ONE, Vec3::Y];
    buffers.quad(
        corners,
        3,
        [2, 2],
        Some(([Vec2::new(0.25, 0.5), Vec2::new(0.75, 0.0)], [2.0, 0.25])),
        [1.; 4],
        Mat4::IDENTITY,
    );
    assert_eq!(buffers.uv[0], [0.5, 0.5]);
    assert_eq!(buffers.uv[2], [1.0, 1.0]);
    assert_eq!(buffers.uv_secondary.as_ref().unwrap()[2], [2.25, 2.5]);
    assert_eq!(buffers.uv_tertiary.as_ref().unwrap()[2], [1.0, 0.25]);
    assert_eq!(
        buffers.uv.len(),
        buffers.uv_secondary.as_ref().unwrap().len()
    );
}

#[test]
fn bone_size_inheritance_is_opt_in_and_uniform() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    let basis = Mat4::from_scale(Vec3::splat(2.));
    let node = basis * Mat4::from_scale(Vec3::new(3., 7., 9.));
    assert_eq!(emitter.particle_scale(node, basis), [2.; 2]);
    emitter.flags.set_inherit_bone_scale(true);
    assert_eq!(emitter.particle_scale(node, basis), [6.; 2]);
}

#[test]
fn native_uniform_scale_variation_preserves_aspect_ratio() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    emitter.appearance.scale_variation = [0.4, 0.9];
    for seed in 0..16 {
        let birth = emitter
            .birth(&slot_values, 0., Mat4::IDENTITY, Vec3::ZERO, seed)
            .unwrap();
        assert_eq!(birth.scale_variation[0], birth.scale_variation[1]);
        assert!((0.6..=1.4).contains(&birth.scale_variation[0]));
    }
}
