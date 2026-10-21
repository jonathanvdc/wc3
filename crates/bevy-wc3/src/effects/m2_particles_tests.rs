use super::*;
use bevy::math::{Quat, Vec2, Vec3};
use std::f32::consts::TAU;
use wc3::model::profiles::Universal;
use wc3::model::ModelMetadata;
use wc3::model::{AnimationProgram, AnimationSample};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
pub(super) fn emitter(slot_values: &mut SlotValues) -> Emitter {
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let node = builder.reserve_node();
    let feature = builder.reserve_feature(node);
    let flags = ParticleFlags::default();
    Emitter {
        feature,
        node,
        node_index: 0,
        emission: EmissionMode::Continuous,
        rate: slot_values.add(10.),
        rate_variation: 0.,
        controls: Some(AgeDrivenControls {
            activation: slot_values.add(true),
            speed_variation: slot_values.add(0.1),
            gravity: slot_values.add(Vec3::from_array([0.; 3])),
            lifespan_variation: 0.,
            vertical: slot_values.add(0.5),
            horizontal: slot_values.add(TAU),
            width: slot_values.add(0.),
            length: slot_values.add(0.),
            z_source: slot_values.add(0.),
        }),
        speed: slot_values.add(10.),
        lifespan: slot_values.add(2.),
        generator: ParticleGenerator::Plane,
        spline: Vec::new(),
        bones: None,
        flags,
        model: None,
        drag: 0.,
        wind: Vec3::ZERO,
        inherit_velocity: 0.,
        follow: [Vec2::ZERO; 2],
        follow_unit_scale: 1.,
        spin: [0.; 4],
        appearance: M2ParticleAppearance::default(),
        atlas: [1, 1],
        age_appearance: None,
        multi_texture: false,
        head_tail: ParticleHeadTail::Head,
        tail_length: 0.,
        seed: 7,

        gpu_style: AgeSpriteStyle {
            textures: [None, None, None],
            blend: ParticleBlend::Alpha,
            priority: 0.,
            sort: SortPolicy::Far,
            composition: Default::default(),
            uv_flip_v: false,
            unlit: false,
            head: true,
            tail: false,
        },
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
        samples: Default::default(),
        mesh: Handle::default(),
        passes: vec![],
        births: vec![birth],
        order: Vec::new(),
        age_appearances: Vec::new(),
        models: None,
        fraction: 0.,
        squirt: SquirtState::default(),
        ordinal: 0,
        previous_node: Mat4::IDENTITY,
        pose_scratch: Vec::new(),
        dropped: 0,
        skipped_seconds: 0.,
    };
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
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
    };
    emitter.draw(
        &mut state,
        &frame,
        Mat4::IDENTITY,
        &mut mesh,
        Mat4::IDENTITY,
    );
    let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
    else {
        panic!("colors absent")
    };
    assert_eq!(colors.len(), 4);
    assert!(colors.iter().all(|color| (color[3] - 0.5).abs() < 1e-5));
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions absent")
    };
    let pointer = positions.as_ptr();
    emitter.draw(
        &mut state,
        &frame,
        Mat4::IDENTITY,
        &mut mesh,
        Mat4::IDENTITY,
    );
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions absent")
    };
    assert_eq!(pointer, positions.as_ptr());
    // Keep each sorted particle's head immediately before its tail, with the
    // same two triangle windings as the former expanded vertex stream.
    emitter.head_tail = ParticleHeadTail::Both;
    emitter.tail_length = 0.25;
    state.births[0].origin = Vec3::Z * 2.;
    state.births[0].velocity = Vec3::X;
    let mut farther = state.births[0].clone();
    farther.origin = Vec3::NEG_Z * 2.;
    state.births.push(farther);
    emitter.draw(
        &mut state,
        &frame,
        Mat4::IDENTITY,
        &mut mesh,
        Mat4::IDENTITY,
    );
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions absent")
    };
    assert_eq!(positions.len(), 16);
    for (quad, expected_z) in positions.chunks_exact(4).zip([-2., -2., 2., 2.]) {
        assert!(quad.iter().all(|position| position[2] == expected_z));
    }
    assert_eq!(
        mesh.indices().unwrap().iter().collect::<Vec<_>>(),
        [0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15],
    );
    let pointer = positions.as_ptr();
    state.births.clear();
    emitter.draw(
        &mut state,
        &frame,
        Mat4::IDENTITY,
        &mut mesh,
        Mat4::IDENTITY,
    );
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions absent")
    };
    assert_eq!(positions.as_slice(), &[[0.; 3]; 3]);
    assert_eq!(positions.as_ptr(), pointer);
    assert_eq!(mesh.indices(), Some(&Indices::U16(vec![0, 1, 2])));
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
    let mut gpu_world = World::new();
    let mut gpu_query = gpu_world.query::<&mut AgeSpriteData>();
    let mut query_state = world.query::<&mut SpriteEmitter>();
    emitter.rate = emitter.speed;
    let runtime = RuntimeM2Particles {
        emitters: vec![Arc::new(emitter)],
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut instances = M2ParticleInstances {
        emitters: vec![state],
        previous_time: Some(0.),
        ..SlotConstruct::construct(&mut slot_values)
    };
    runtime.update(
        &mut instances,
        frame,
        &mut meshes,
        &mut gpu_query.query_mut(&mut gpu_world),
        &mut query_state.query_mut(&mut world),
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
        &mut gpu_query.query_mut(&mut gpu_world),
        &mut query_state.query_mut(&mut world),
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
fn emitter_follow_requires_its_flag_and_clamps_the_solved_response() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    emitter.follow = [Vec2::new(2., 0.25), Vec2::new(6., 0.75)];
    let translated = |distance| Mat4::from_translation(Vec3::X * distance);
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, translated(4.), 1.),
        Vec3::ZERO,
        "nonzero authored defaults do not enable follow"
    );
    emitter.flags.set_follow_position(true);
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, translated(1.), 1.),
        Vec3::X * 0.125
    );
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, translated(4.), 1.),
        Vec3::X * 2.
    );
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, translated(8.), 1.),
        Vec3::X * 8.
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
    emitter.follow = [Vec2::new(2., 0.5); 2];
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, translated(4.), 1.),
        Vec3::ZERO
    );
}

#[test]
fn world_particles_follow_only_translation_after_their_birth() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    emitter.flags.set_follow_position(true);
    emitter.follow = [Vec2::new(0., 0.5), Vec2::new(1., 0.5)];
    emitter.rate = slot_values.add(2.);
    emitter.speed = slot_values.add(0.);
    let prior_birth = emitter
        .birth(&slot_values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
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
        samples: Default::default(),
        mesh,
        passes: vec![],
        births: vec![prior_birth],
        order: Vec::new(),
        age_appearances: Vec::new(),
        models: None,
        fraction: 0.,
        squirt: SquirtState::default(),
        ordinal: 1,
        previous_node: Mat4::IDENTITY,
        pose_scratch: Vec::new(),
        dropped: 0,
        skipped_seconds: 0.,
    };
    let runtime = RuntimeM2Particles {
        emitters: vec![Arc::new(emitter)],
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut instances = M2ParticleInstances {
        emitters: vec![state],
        previous_time: Some(0.),
        previous_basis: Some(Mat4::IDENTITY),
        adopted: HashMap::new(),
    };
    let mut world = World::new();
    let mut gpu_world = World::new();
    let mut gpu_query = gpu_world.query::<&mut AgeSpriteData>();
    let mut query = world.query::<&mut SpriteEmitter>();
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
    };
    runtime.update(
        &mut instances,
        frame,
        &mut meshes,
        &mut gpu_query.query_mut(&mut gpu_world),
        &mut query.query_mut(&mut world),
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
fn semantic_gravity_is_independent_of_the_emitter_bone_and_scalar_packing_flags() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    emitter.flags.set_packed_gravity(true);
    emitter.controls.as_mut().unwrap().gravity = slot_values.add(Vec3::from_array([1., -2., 3.]));
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
    assert_eq!(buffers.positions.len(), 4);
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    update_sprite_indices(&mut mesh, buffers.positions.len());
    let indices: Vec<_> = mesh.indices().unwrap().iter().collect();
    assert_eq!(indices, [0, 1, 2, 0, 2, 3]);
    for (index, corner) in indices.into_iter().zip([0, 1, 2, 0, 2, 3]) {
        assert_eq!(buffers.positions[index], corners[corner].to_array());
        assert_eq!(buffers.normals[index], buffers.normals[corner]);
        assert_eq!(buffers.colors[index], [1.; 4]);
    }
}

#[test]
fn indexed_sprite_topology_reuses_storage_through_budget_and_empty_transitions() {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let vertices = MAX_PARTICLES * 8;
    update_sprite_indices(&mut mesh, vertices);
    let Some(Indices::U16(indices)) = mesh.indices() else {
        panic!("compact sprite indices absent")
    };
    let pointer = indices.as_ptr();
    assert_eq!(indices.len(), MAX_PARTICLES * 12);
    assert_eq!(indices.last(), Some(&u16::MAX));
    assert_eq!(indices.iter().copied().max(), Some(u16::MAX));
    for vertices in [8, 3, 4, 8, MAX_PARTICLES * 8] {
        update_sprite_indices(&mut mesh, vertices);
        let Some(Indices::U16(indices)) = mesh.indices() else {
            panic!("compact sprite indices absent")
        };
        assert_eq!(indices.as_ptr(), pointer);
        if vertices == 3 {
            assert_eq!(indices, &[0, 1, 2]);
        } else {
            assert_eq!(indices.len(), vertices / 4 * 6);
            for (quad, triangle_pair) in indices.chunks_exact(6).enumerate() {
                let base = quad as u16 * 4;
                assert_eq!(
                    triangle_pair,
                    &[base, base + 1, base + 2, base, base + 2, base + 3]
                );
            }
        }
    }
}

#[test]
fn bone_size_inheritance_is_opt_in_and_uniform() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    let basis = Mat4::from_scale(Vec3::splat(2.));
    let node = basis * Mat4::from_scale(Vec3::new(3., 7., 9.));
    assert_eq!(emitter.particle_scale(node, basis), Vec2::splat(2.));
    emitter.flags.set_inherit_bone_scale(true);
    assert_eq!(emitter.particle_scale(node, basis), Vec2::splat(6.));
}

#[test]
fn native_uniform_scale_variation_preserves_aspect_ratio() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    emitter.appearance.scale_variation = Vec2::from_array([0.4, 0.9]);
    for seed in 0..16 {
        let birth = emitter
            .birth(&slot_values, 0., Mat4::IDENTITY, Vec3::ZERO, seed)
            .unwrap();
        assert_eq!(birth.scale_variation[0], birth.scale_variation[1]);
        assert!((0.6..=1.4).contains(&birth.scale_variation[0]));
    }
}

// Mirrors the shader's corner expansion, independently of the old CPU oracle.
fn gpu_sample_corners(
    record: &AgeSpriteRecord,
    uniform: &AgeSpriteUniform,
    camera: Mat4,
    tail: bool,
) -> [Vec3; 4] {
    let back = camera.z_axis.truncate().normalize_or_zero();
    let mut right = camera.x_axis.truncate().normalize_or_zero();
    let mut up = camera.y_axis.truncate().normalize_or_zero();
    let mut position = Vec3::from_slice(&record.position[..3]);
    let velocity = Vec3::from_slice(&record.velocity_tail[..3]);
    if uniform.flags[0] != 0 {
        right = Vec3::from_slice(&uniform.axes[0][..3]);
        up = Vec3::from_slice(&uniform.axes[1][..3]);
    } else if uniform.flags[1] != 0 {
        up = (velocity - back * velocity.dot(back))
            .try_normalize()
            .unwrap_or(up);
        right = up.cross(back).normalize_or_zero();
    }
    let [width, height, cos, sin] = record.size_spin;
    let mut side = (right * cos + up * sin) * width;
    let mut vertical = (up * cos - right * sin) * height;
    if tail {
        let extent = velocity * record.velocity_tail[3];
        side = extent
            .cross(camera.w_axis.truncate() - position)
            .normalize_or_zero()
            * width;
        vertical = extent * 0.5;
        position -= vertical;
    }
    [
        position - side - vertical,
        position + side - vertical,
        position + side + vertical,
        position - side + vertical,
    ]
}

#[test]
fn gpu_live_samples_preserve_cpu_geometry_across_views_and_coordinate_frames() {
    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    emitter.head_tail = ParticleHeadTail::Both;
    emitter.gpu_style.head = true;
    emitter.gpu_style.tail = true;
    emitter.tail_length = 0.75;
    emitter.appearance.alpha = Some(
        Curve::linear(
            [
                ModelTime::seconds(0.).unwrap(),
                ModelTime::seconds(1.).unwrap(),
            ],
            [1., 0.25],
        )
        .unwrap(),
    );
    let mut birth = emitter
        .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    birth.origin = Vec3::new(1., 2., -3.);
    birth.velocity = Vec3::new(2., 0.5, 1.);
    birth.spin = 0.3;
    birth.spin_rate = 0.4;
    birth.scale_variation = Vec2::new(0.8, 1.1);
    let mut state = EmitterInstance {
        samples: Default::default(),
        mesh: Handle::default(),
        passes: Vec::new(),
        births: vec![birth],
        age_appearances: Vec::new(),
        models: None,
        order: Vec::new(),
        fraction: 0.,
        squirt: SquirtState::default(),
        ordinal: 0,
        previous_node: Mat4::IDENTITY,
        pose_scratch: Vec::new(),
        dropped: 0,
        skipped_seconds: 0.,
    };
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    builder.import_values(&values).unwrap();
    let model = builder.finish().unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let basis = Mat4::from_scale_rotation_translation(
        Vec3::new(2., 3., 4.),
        Quat::from_rotation_z(0.4),
        Vec3::X,
    );
    let node = basis * Mat4::from_rotation_x(0.7);
    let mut data = AgeSpriteData {
        records: Default::default(),
        live_indices: Default::default(),
        uniform: Default::default(),
        style: emitter.gpu_style.clone(),
    };
    for attached in [false, true] {
        emitter.flags.set_attached(attached);
        for mode in 0..3 {
            data.style.uv_flip_v = mode == 2;
            emitter.flags.set_unfogged(mode == 1);
            emitter.flags.set_refraction(mode == 2);
            emitter.flags.set_align_xy(mode == 1);
            emitter.flags.set_velocity_oriented(mode == 2);
            for camera in [
                Mat4::IDENTITY,
                Mat4::from_rotation_translation(Quat::from_rotation_y(0.9), Vec3::new(4., 3., 8.)),
            ] {
                let frame = EffectFrame {
                    traversal: Default::default(),
                    change: Default::default(),
                    root_visible: true,
                    pose: None,
                    selection: Default::default(),
                    time: 0.5,
                    sample: &sample,
                    node_poses: &[node],
                    node_visible: &[true],
                    basis_world: basis,
                };
                emitter.sample_gpu(&mut state, &frame, node, &mut data);
                assert_eq!(data.uniform.flags[3], u32::from(mode != 1));
                assert_eq!(data.uniform.atlas[2] & 8, u32::from(mode == 2) << 3);
                assert_eq!(data.uniform.atlas[2] & 7, data.style.composition.flags());
                assert_eq!(data.records.len(), 1);
                assert_eq!(data.live_indices.as_slice(), &[0, 0x8000_0000]);
                assert!((data.records[0].color[3] - 0.8125).abs() < 1e-6);
                let snapshot = data.records.clone();
                let mut mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                );
                emitter.draw(&mut state, &frame, node, &mut mesh, camera);
                let Some(VertexAttributeValues::Float32x3(positions)) =
                    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                else {
                    panic!("positions absent")
                };
                let Some(VertexAttributeValues::Float32x3(normals)) =
                    mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
                else {
                    panic!("normals absent")
                };
                for (quad_index, (quad, tail)) in
                    positions.chunks_exact(4).zip([false, true]).enumerate()
                {
                    let generated =
                        gpu_sample_corners(&data.records[0], &data.uniform, camera, tail);
                    let normal = (generated[1] - generated[0])
                        .cross(generated[2] - generated[0])
                        .normalize_or_zero();
                    let cpu_normal = basis
                        .inverse()
                        .transpose()
                        .transform_vector3(Vec3::from_array(normals[quad_index * 4]))
                        .normalize_or_zero();
                    assert!(cpu_normal.abs_diff_eq(normal, 1e-5));
                    for (cpu, gpu) in quad.iter().zip(generated) {
                        assert!(
                            Vec3::from_array(*cpu)
                                .abs_diff_eq(basis.inverse().transform_point3(gpu), 1e-5),
                            "mode {mode}, attached {attached}, tail {tail}"
                        );
                    }
                }
                assert!(
                    Arc::ptr_eq(&snapshot, &data.records),
                    "per-view expansion must not rewrite shared samples"
                );
            }
        }
    }
}

#[test]
fn spline_births_follow_authored_path_and_aim_from_the_source_height() {
    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    emitter.generator = ParticleGenerator::Spline;
    emitter.spline = vec![Vec3::ZERO, Vec3::X, Vec3::new(1., 3., 0.)];
    emitter.controls.as_mut().unwrap().length = values.add(0.5);
    emitter.controls.as_mut().unwrap().width = values.add(1.);
    emitter.controls.as_mut().unwrap().speed_variation = values.add(0.);
    emitter.speed = values.add(2.);
    for ordinal in 0..16 {
        let birth = emitter
            .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, ordinal)
            .unwrap();
        assert_eq!(birth.origin.x, 1.);
        assert!((0. ..=3.).contains(&birth.origin.y));
        assert_eq!(birth.velocity, Vec3::Y * 2.);
    }
    emitter.controls.as_mut().unwrap().z_source = values.add(1.);
    let birth = emitter
        .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    assert!(birth
        .velocity
        .abs_diff_eq((birth.origin - Vec3::Z).normalize() * 2., 1e-6));
}

#[test]
#[ignore = "opt-in CPU preparation comparison; prints timings without asserting machine-specific limits"]
fn benchmark_compact_samples_against_cpu_corner_transport() {
    use super::super::age_sprites_gpu::sort_for_benchmark;
    use super::super::render::SortScratch;
    use bevy::math::Affine3A;
    use std::hint::black_box;
    use std::time::Instant;

    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    emitter.head_tail = ParticleHeadTail::Both;
    emitter.gpu_style.tail = true;
    emitter.tail_length = 0.4;
    let births = (0..MAX_PARTICLES)
        .map(|ordinal| {
            let mut birth = emitter
                .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, ordinal as u64)
                .unwrap();
            birth.origin = Vec3::new(
                (ordinal % 23) as f32,
                (ordinal % 17) as f32,
                -(ordinal as f32) * 0.01,
            );
            birth
        })
        .collect();
    let mut state = EmitterInstance {
        samples: Default::default(),
        mesh: Handle::default(),
        passes: Vec::new(),
        births,
        age_appearances: Vec::new(),
        models: None,
        order: Vec::new(),
        fraction: 0.,
        squirt: SquirtState::default(),
        ordinal: 0,
        previous_node: Mat4::IDENTITY,
        pose_scratch: Vec::new(),
        dropped: 0,
        skipped_seconds: 0.,
    };
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    builder.import_values(&values).unwrap();
    let model = builder.finish().unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let frame = EffectFrame {
        traversal: Default::default(),
        change: Default::default(),
        root_visible: true,
        pose: None,
        selection: Default::default(),
        time: 0.5,
        sample: &sample,
        node_poses: &[Mat4::IDENTITY],
        node_visible: &[true],
        basis_world: Mat4::IDENTITY,
    };
    let mut data = AgeSpriteData {
        records: Default::default(),
        live_indices: Default::default(),
        uniform: Default::default(),
        style: emitter.gpu_style.clone(),
    };
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    emitter.draw(
        &mut state,
        &frame,
        Mat4::IDENTITY,
        &mut mesh,
        Mat4::IDENTITY,
    );
    emitter.sample_gpu(&mut state, &frame, Mat4::IDENTITY, &mut data);
    let rounds = 64;
    let start = Instant::now();
    for _ in 0..rounds {
        emitter.draw(
            black_box(&mut state),
            &frame,
            Mat4::IDENTITY,
            black_box(&mut mesh),
            Mat4::IDENTITY,
        );
    }
    let corners = start.elapsed().as_secs_f64() / f64::from(rounds);
    let start = Instant::now();
    for _ in 0..rounds {
        emitter.sample_gpu(
            black_box(&mut state),
            &frame,
            Mat4::IDENTITY,
            black_box(&mut data),
        );
    }
    let samples = start.elapsed().as_secs_f64() / f64::from(rounds);
    let mut scratch = SortScratch::default();
    let view = Affine3A::IDENTITY;
    let start = Instant::now();
    for _ in 0..rounds {
        emitter.sample_gpu(
            black_box(&mut state),
            &frame,
            Mat4::IDENTITY,
            black_box(&mut data),
        );
        scratch.clear();
        sort_for_benchmark(&data, &view, &mut scratch);
        black_box(&scratch.order);
    }
    let samples_and_sort = start.elapsed().as_secs_f64() / f64::from(rounds);
    println!(
        "Matched single-view CPU preparation: indexed corners + sorting {corners:.6}s; compact samples + actual renderer sorting {samples_and_sort:.6}s; ratio {:.2}x (GPU upload/submission excluded)",
        corners / samples_and_sort
    );
    println!(
        "8192 head+tail particles: CPU indexed corners {corners:.6}s; compact samples {samples:.6}s; preparation ratio {:.2}x (compact-only measurement excludes renderer sorting)",
        corners / samples
    );
    println!(
        "live payload: {} compact bytes vs {} indexed-corner bytes, excluding draw-order buffers",
        data.records.len() * size_of::<AgeSpriteRecord>(),
        MAX_PARTICLES * 2 * 204
    );
}

fn implosion_state(births: Vec<Birth>) -> EmitterInstance {
    EmitterInstance {
        samples: Default::default(),
        mesh: Handle::default(),
        passes: Vec::new(),
        births,
        age_appearances: Vec::new(),
        models: None,
        order: Vec::new(),
        fraction: 0.,
        squirt: SquirtState::default(),
        ordinal: 0,
        previous_node: Mat4::IDENTITY,
        pose_scratch: Vec::new(),
        dropped: 0,
        skipped_seconds: 0.,
    }
}
fn implosion_interval(start: f64, end: f64) -> ImplosionInterval {
    ImplosionInterval {
        start,
        end,
        checked_end: end,
        centers: [Vec3::ZERO; 2],
        follow: Vec3::ZERO,
    }
}

#[test]
fn implosion_is_sphere_only_and_starts_at_each_birth_occurrence() {
    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    let mut birth = emitter
        .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    birth.origin = Vec3::X;
    birth.velocity = Vec3::X;
    birth.acceleration = Vec3::ZERO;
    let mut state = implosion_state(vec![birth.clone()]);
    emitter.flags.set_implosion_filter(true);
    emitter.retire_implosion(&mut state, implosion_interval(0., 1.));
    assert_eq!(
        state.births.len(),
        1,
        "plane emitters ignore the sphere-only flag"
    );
    emitter.generator = ParticleGenerator::Sphere;
    emitter.flags.set_implosion_filter(false);
    emitter.retire_implosion(&mut state, implosion_interval(0., 1.));
    assert_eq!(
        state.births.len(),
        1,
        "unflagged spheres retain outward particles"
    );
    emitter.flags.set_implosion_filter(true);
    emitter.retire_implosion(&mut state, implosion_interval(0., 1.));
    assert!(state.births.is_empty());

    birth.time = 1.;
    state.births.push(birth);
    emitter.retire_implosion(&mut state, implosion_interval(0., 1.));
    assert_eq!(state.births.len(), 1, "an endpoint birth has not moved yet");
    emitter.retire_implosion(&mut state, implosion_interval(1., 1.01));
    assert!(
        state.births.is_empty(),
        "its first outward movement retires it"
    );
}

#[test]
fn implosion_uses_interpolated_live_centers_and_local_attached_coordinates() {
    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    emitter.generator = ParticleGenerator::Sphere;
    emitter.flags.set_implosion_filter(true);
    let mut birth = emitter
        .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    birth.origin = Vec3::X;
    birth.velocity = Vec3::NEG_X;
    birth.acceleration = Vec3::ZERO;
    let mut state = implosion_state(vec![birth.clone()]);
    let interval = ImplosionInterval {
        start: 0.,
        end: 1.,
        checked_end: 0.1,
        centers: [Vec3::ZERO, Vec3::X * 2.],
        follow: Vec3::ZERO,
    };
    emitter.retire_implosion(&mut state, interval);
    assert_eq!(
        state.births.len(),
        1,
        "the emitter has not overtaken the particle yet"
    );
    let mut interval = implosion_interval(0., 0.5);
    interval.centers = [Vec3::X * 2.; 2];
    emitter.retire_implosion(&mut state, interval);
    assert!(
        state.births.is_empty(),
        "detached motion uses the live center, not birth center"
    );
    emitter.flags.set_attached(true);
    state.births.push(birth);
    let mut interval = implosion_interval(0., 0.5);
    interval.centers = [Vec3::X * 2.; 2];
    emitter.retire_implosion(&mut state, interval);
    assert_eq!(
        state.births.len(),
        1,
        "attached filtering stays in local coordinates"
    );
    emitter.retire_implosion(&mut state, implosion_interval(0.5, 1.1));
    assert!(
        state.births.is_empty(),
        "an inward particle retires after passing the local center"
    );
}

#[test]
fn implosion_includes_follow_translation_in_attempted_movement() {
    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    emitter.generator = ParticleGenerator::Sphere;
    emitter.flags.set_implosion_filter(true);
    let mut birth = emitter
        .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    birth.origin = Vec3::X * 3.; // Original x=1 plus the full observed follow x=2.
    birth.velocity = Vec3::ZERO;
    birth.acceleration = Vec3::ZERO;
    let mut state = implosion_state(vec![birth]);
    let mut interval = implosion_interval(0., 1.);
    interval.centers = [Vec3::ZERO, Vec3::X * 2.];
    interval.follow = Vec3::X * 2.;
    emitter.retire_implosion(&mut state, interval);
    assert!(state.births.is_empty());
}

#[test]
fn implosion_long_update_reports_omitted_time_once_per_emitter() {
    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    emitter.generator = ParticleGenerator::Sphere;
    emitter.flags.set_implosion_filter(true);
    emitter.rate = values.add(0.);
    let mut birth = emitter
        .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    birth.origin = Vec3::ZERO;
    birth.velocity = Vec3::ZERO;
    birth.acceleration = Vec3::ZERO;
    birth.lifetime = 1000.;
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    builder.import_values(&values).unwrap();
    let model = builder.finish().unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let frame = EffectFrame {
        traversal: Default::default(),
        change: Default::default(),
        root_visible: true,
        pose: None,
        selection: Default::default(),
        time: 100.,
        sample: &sample,
        node_poses: &[Mat4::IDENTITY],
        node_visible: &[true],
        basis_world: Mat4::IDENTITY,
    };
    let runtime = RuntimeM2Particles {
        emitters: vec![Arc::new(emitter)],
        ..Default::default()
    };
    let mut meshes = Assets::<Mesh>::default();
    let mut gpu_world = World::new();
    let mut gpu_query = gpu_world.query::<&mut AgeSpriteData>();
    let mut stats_world = World::new();
    let mut stats_query = stats_world.query::<&mut SpriteEmitter>();
    let mut clock = EmissionClock::new(0., 100.);
    for _ in clock.by_ref() {}
    for population in [4, MAX_PARTICLES] {
        let state = implosion_state(vec![birth.clone(); population]);
        let mut instances = M2ParticleInstances {
            emitters: vec![state],
            previous_time: Some(0.),
            previous_basis: Some(Mat4::IDENTITY),
            adopted: HashMap::new(),
        };
        runtime.update(
            &mut instances,
            frame,
            &mut meshes,
            &mut gpu_query.query_mut(&mut gpu_world),
            &mut stats_query.query_mut(&mut stats_world),
        );
        let expected = if population == 4 {
            clock.omitted_seconds()
        } else {
            100. - 4. / 60.
        };
        assert_eq!(instances.emitters[0].births.len(), population);
        assert_eq!(instances.emitters[0].skipped_seconds, expected);
        assert!(instances.emitters[0].skipped_seconds > 31.);
        assert_eq!(
            instances.previous_time,
            Some(100.),
            "budgets must not defer the emitter clock"
        );
    }
}

#[test]
fn opaque_txac_selectors_do_not_disable_known_multi_texture_controls() {
    use super::super::fixture_tests::gravity_fixture;
    use wc3::model::conversion::ImportOptions;
    use wc3::model::io::m2 as semantic_m2;

    for three_colors in [false, true] {
        for alpha_modulate4x in [false, true] {
            let mut native = gravity_fixture(false, 0.0f32.to_bits());
            let core = native.model.core_mut();
            core.textures = vec![core.textures[0].clone(); 3];
            let particle = &mut core.particles[0];
            particle.flags.set_multi_texture(true);
            particle.flags.set_refraction(true);
            particle.flags.set_multi_texture_three_colors(three_colors);
            particle
                .flags
                .set_multi_texture_modulate4x(alpha_modulate4x);
            particle.texture_id = (1 << 5) | (2 << 10);
            let mut model: Model<'_, Universal> =
                semantic_m2::import(&native, &ImportOptions::default())
                    .unwrap()
                    .value
                    .widen();
            let (feature, payload) = model
                .features()
                .iter()
                .find_map(|(id, feature)| {
                    matches!(feature.payload, NodeFeature::Particle(_))
                        .then(|| (id, feature.payload.clone()))
                })
                .unwrap();
            let node_indices = model
                .nodes()
                .iter()
                .enumerate()
                .map(|(index, (id, _))| (id, index))
                .collect();
            for selectors in [None, Some([0, 0]), Some([17, 42])] {
                let mut changed = payload.clone();
                let NodeFeature::Particle(ParticleEmitter::M2(definition)) = &mut changed else {
                    unreachable!()
                };
                definition.details.rendering.texture_program = selectors;
                model
                    .edit(|builder| builder.set_feature(feature, changed))
                    .unwrap();
                let compiled = RuntimeM2Particles::compile(
                    &model,
                    &node_indices,
                    &TextureImages::default(),
                    &HashMap::new(),
                );
                assert_eq!(compiled.emitters.len(), 1);
                let style = &compiled.emitters[0].gpu_style;
                assert_eq!(
                    style.composition,
                    AgeSpriteComposition::Multi {
                        three_colors,
                        alpha_modulate4x
                    }
                );
                assert_eq!(
                    style.composition.flags(),
                    1 | (u32::from(three_colors) << 1) | (u32::from(alpha_modulate4x) << 2)
                );
                assert!(style.unlit);
                assert_eq!(
                    compiled.diagnostics.iter().any(|(_, issue)| {
                        issue.kind == RenderDiagnosticKind::Unsupported
                            && issue.message.contains("TXAC selectors")
                    }),
                    selectors == Some([17, 42])
                );
                assert!(!compiled.diagnostics.iter().any(|(_, issue)| {
                    issue.message.contains("extra particle textures")
                        || issue.message.contains("modulation-gain")
                        || issue.message.contains("refraction requires")
                }));
            }
        }
    }
}

#[test]
#[ignore = "exports original deterministic benchmark births under /tmp for matched independent CPU comparisons"]
fn write_matched_particle_benchmark_births() {
    use std::fmt::Write as _;
    use std::fs::{read_to_string, write};

    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    emitter.head_tail = ParticleHeadTail::Both;
    emitter.gpu_style.tail = true;
    emitter.tail_length = 0.4;
    let mut output = String::from(
        "# 8192 particles; sample_age=0.5; head=true; tail=true; tail_time=0.4; white_rgba=1; size_xy=1; spin=0; drag=0; identity_camera=true\norigin_x,origin_y,origin_z,velocity_x,velocity_y,velocity_z,acceleration_x,acceleration_y,acceleration_z,lifetime,scale_x,scale_y\n",
    );
    for ordinal in 0..MAX_PARTICLES {
        let mut birth = emitter
            .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, ordinal as u64)
            .unwrap();
        birth.origin = Vec3::new(
            (ordinal % 23) as f32,
            (ordinal % 17) as f32,
            -(ordinal as f32) * 0.01,
        );
        writeln!(
            output,
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            birth.origin.x,
            birth.origin.y,
            birth.origin.z,
            birth.velocity.x,
            birth.velocity.y,
            birth.velocity.z,
            birth.acceleration.x,
            birth.acceleration.y,
            birth.acceleration.z,
            birth.lifetime,
            birth.scale_variation.x,
            birth.scale_variation.y
        )
        .unwrap();
    }
    let path = "/tmp/wc3-matched-particle-births.csv";
    if let Ok(previous) = read_to_string(path) {
        let first_difference = previous
            .lines()
            .zip(output.lines())
            .enumerate()
            .find(|(_, (before, actual))| before != actual);
        if let Some((line, (before, actual))) = first_difference {
            println!(
                "Previous input first differs at line {}: previous={before}; actual={actual}",
                line + 1
            );
        }
        let mut changed_fields = 0;
        let mut max_absolute_difference = 0f64;
        for (before, actual) in previous.lines().skip(2).zip(output.lines().skip(2)) {
            for (before, actual) in before.split(',').zip(actual.split(',')) {
                if let (Ok(before), Ok(actual)) = (before.parse::<f32>(), actual.parse::<f32>()) {
                    changed_fields += usize::from(before.to_bits() != actual.to_bits());
                    max_absolute_difference =
                        max_absolute_difference.max((f64::from(before) - f64::from(actual)).abs());
                }
            }
        }
        println!(
            "Previous input comparison: {changed_fields} changed f32 fields, maximum absolute difference {max_absolute_difference:e}; row counts {} previous / {} actual",
            previous.lines().count().saturating_sub(2),
            output.lines().count().saturating_sub(2)
        );
    }
    write(path, output).unwrap();
    println!("Original deterministic benchmark births: {path}");
}

#[test]
fn disabled_geometry_keeps_m2_simulation_and_statistics_without_live_records() {
    use super::super::fixture_tests::gravity_fixture;
    use bevy::ecs::system::SystemState;
    use bevy::ecs::world::CommandQueue;
    use wc3::model::conversion::ImportOptions;
    use wc3::model::io::m2 as semantic_m2;

    let source = gravity_fixture(false, 0.0f32.to_bits());
    let mut model: Model<'_, Universal> = semantic_m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value
        .widen();
    let (feature, mut payload) = model
        .features()
        .iter()
        .find_map(|(id, value)| {
            matches!(value.payload, NodeFeature::Particle(_)).then(|| (id, value.payload.clone()))
        })
        .unwrap();
    let NodeFeature::Particle(ParticleEmitter::M2(definition)) = &mut payload else {
        unreachable!()
    };
    definition.details.rendering.head_tail = ParticleHeadTail::Disabled;
    model
        .edit(|builder| builder.set_feature(feature, payload))
        .unwrap();
    let node_indices = model
        .nodes()
        .iter()
        .enumerate()
        .map(|(index, (id, _))| (id, index))
        .collect();
    let runtime = RuntimeM2Particles::compile(
        &model,
        &node_indices,
        &TextureImages::default(),
        &HashMap::new(),
    );
    assert_eq!(
        runtime.emitters.len(),
        1,
        "geometry selection must not remove an emitter's simulation"
    );
    let mut meshes = Assets::<Mesh>::default();
    let mut world = World::new();
    let basis = world.spawn_empty().id();
    let mut queue = CommandQueue::default();
    let mut instances = {
        let mut commands = Commands::new(&mut queue, &world);
        runtime.spawn(&mut commands, &mut meshes, basis)
    };
    queue.apply(&mut world);
    let entity = instances.emitters[0].passes[0].entity;
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let poses = vec![Mat4::IDENTITY; model.nodes().len()];
    let visibility = vec![true; model.nodes().len()];
    let mut system =
        SystemState::<(Query<&mut AgeSpriteData>, Query<&mut SpriteEmitter>)>::new(&mut world);
    let mut previous_population = 0;
    for (time, change) in [
        (0., EffectChange::Continuous),
        (0.5, EffectChange::Continuous),
        (1., EffectChange::Continuous),
        (0., EffectChange::Reset),
    ] {
        let (mut data, mut statistics) = system.get_mut(&mut world).unwrap();
        runtime.update(
            &mut instances,
            EffectFrame {
                traversal: Default::default(),
                change,
                root_visible: true,
                pose: None,
                selection: Default::default(),
                time,
                sample: &sample,
                node_poses: &poses,
                node_visible: &visibility,
                basis_world: Mat4::IDENTITY,
            },
            &mut meshes,
            &mut data,
            &mut statistics,
        );
        let stats = statistics.get(entity).unwrap();
        let draw = data.get(entity).unwrap();
        assert!(draw.records.is_empty());
        assert!(draw.live_indices.is_empty());
        assert_eq!(stats.alive, instances.emitters[0].births.len());
        if time > 0. {
            assert!(stats.alive > previous_population);
            previous_population = stats.alive;
        } else {
            assert_eq!(stats.alive, 0);
        }
        assert_eq!(stats.dropped, 0);
    }
}

/// Body births place posed model joints once, independent of the emitter pivot.
#[test]
fn bone_births_use_posed_joint_origins_once_in_attached_and_detached_frames() {
    use wc3::model::profiles::m2::MeshProperties as SectionProperties;
    use wc3::model::{GeometryData, Mesh as ModelMesh, MeshInstance, MeshProperties};

    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let mut joint = builder.new_node("offset joint");
    joint.transform.pivot = Vec3::new(7., 8., 9.);
    let joint = builder.add_node(joint).unwrap();
    let mesh =
        builder
            .add_mesh(ModelMesh::new(GeometryData::empty()).with_properties(Some(
                MeshProperties::M2(SectionProperties {
                    bone_palette: Some(vec![joint]),
                    center_bone: None,
                    center: Vec3::ZERO,
                    sorting: None,
                }),
            )))
            .unwrap();
    let mut instance = MeshInstance::new(joint, mesh);
    instance.lod = Some(0);
    builder.add_instance(instance).unwrap();
    let model = builder.finish().unwrap();
    let source = BoneSource::compile(&model, &HashMap::from([(joint, 0)]));
    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    emitter.generator = ParticleGenerator::Bone;
    emitter.bones = Some(source);
    let basis = Mat4::from_scale_rotation_translation(
        Vec3::new(-2., 3., 1.),
        Quat::from_rotation_y(0.4),
        Vec3::new(20., -5., 3.),
    );
    let node = basis
        * Mat4::from_rotation_translation(Quat::from_rotation_x(0.7), Vec3::new(11., 2., -4.));
    let rest_point = Vec3::new(7., 8., 9.);
    for attached in [false, true] {
        emitter.flags.set_attached(attached);
        for point in [rest_point, rest_point + Vec3::new(2., -3., 1.)] {
            let birth = emitter
                .birth_posed(&values, 0., node, basis, Vec3::ZERO, 0, |candidate| {
                    assert_eq!(candidate.pivot, rest_point);
                    Some((point, None))
                })
                .unwrap()
                .0;
            let expected_local = (node.inverse() * basis).transform_point3(point);
            let expected = if attached {
                expected_local
            } else {
                basis.transform_point3(point)
            };
            assert!(
                birth.origin.abs_diff_eq(expected, 1e-4),
                "{attached}: {:?} != {expected:?}",
                birth.origin
            );
            let world = if attached {
                node.transform_point3(birth.origin)
            } else {
                birth.origin
            };
            assert!(world.abs_diff_eq(basis.transform_point3(point), 1e-4));
            let expected_direction = if attached {
                Vec3::Z
            } else {
                node.transform_vector3(Vec3::Z)
            };
            assert!(birth
                .velocity
                .normalize()
                .abs_diff_eq(expected_direction.normalize(), 1e-5));
        }
    }
    emitter.flags.set_attached(true);
    emitter.controls.as_mut().unwrap().z_source = values.add(3.);
    let birth = emitter
        .birth_posed(&values, 0., node, basis, Vec3::ZERO, 0, |_| {
            Some((rest_point, None))
        })
        .unwrap()
        .0;
    let aimed = (birth.origin - Vec3::Z * 3.).normalize();
    assert!(birth.velocity.normalize().abs_diff_eq(aimed, 1e-5));
}

#[test]
fn authored_sort_flag_selects_transport_policy_without_unknown_flag_diagnostic() {
    use super::super::fixture_tests::gravity_fixture;
    use wc3::model::conversion::ImportOptions;
    use wc3::model::io::m2 as semantic_m2;
    for sorted in [false, true] {
        let mut native = gravity_fixture(false, 0.0f32.to_bits());
        native.model.core_mut().particles[0]
            .flags
            .set_sort_particles(sorted);
        let model = semantic_m2::import(&native, &ImportOptions::default())
            .unwrap()
            .value
            .widen();
        let indices = model
            .nodes()
            .iter()
            .enumerate()
            .map(|(i, (id, _))| (id, i))
            .collect();
        let runtime = RuntimeM2Particles::compile(
            &model,
            &indices,
            &TextureImages::default(),
            &HashMap::new(),
        );
        assert_eq!(
            runtime.emitters[0].gpu_style.sort,
            if sorted {
                SortPolicy::Far
            } else {
                SortPolicy::None
            }
        );
        assert!(!runtime
            .diagnostics()
            .iter()
            .any(|(_, issue)| issue.kind == RenderDiagnosticKind::Unsupported
                && issue.message.contains("particle controls")));
    }
}

#[test]
fn natural_and_sorted_transport_orders_preserve_every_living_particle() {
    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    let births = [-1., -3., -2.].map(|z| {
        let mut birth = emitter
            .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
            .unwrap();
        birth.origin = Vec3::Z * z;
        birth.velocity = Vec3::ZERO;
        birth
    });
    let mut state = implosion_state(births.into());
    let model = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()))
        .finish()
        .unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let frame = EffectFrame {
        pose: None,
        root_visible: true,
        traversal: Default::default(),
        change: EffectChange::Continuous,
        time: 0.,
        selection: Default::default(),
        sample: &sample,
        node_poses: &[Mat4::IDENTITY],
        node_visible: &[true],
        basis_world: Mat4::IDENTITY,
    };
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    for (policy, expected) in [
        (SortPolicy::None, [-1., -3., -2.]),
        (SortPolicy::Far, [-3., -2., -1.]),
        (SortPolicy::Near, [-1., -2., -3.]),
    ] {
        emitter.gpu_style.sort = policy;
        emitter.draw(
            &mut state,
            &frame,
            Mat4::IDENTITY,
            &mut mesh,
            Mat4::IDENTITY,
        );
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("positions missing")
        };
        let centers: Vec<_> = positions
            .chunks_exact(4)
            .map(|quad| quad.iter().map(|p| p[2]).sum::<f32>() * 0.25)
            .collect();
        assert_eq!(centers, expected);
    }
}

#[test]
fn collapsed_attached_model_projection_keeps_live_birth_identity_and_finite_fallback() {
    use crate::child_coordinates::ChildCoordinates;
    use crate::CoordinatePolicy;
    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    emitter.flags.set_attached(true);
    let birth = emitter
        .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, 7)
        .unwrap();
    let sidecar = GeometryBirth {
        lease: None,
        ordinal: 7,
        initial_transform: Transform::IDENTITY,
        basis: ModelBasis::from_frame(Mat4::IDENTITY).unwrap(),
        omega: Vec3::ZERO,
        appearance: Default::default(),
    };
    let model = GeometryEmitter {
        template: ModelTemplate {
            model: Handle::default(),
            coordinates: ChildCoordinates {
                coordinates: CoordinatePolicy::Host,
                parent_basis: Transform::IDENTITY,
            },
        },
        minimum: Vec3::ZERO,
        maximum: Vec3::ZERO,
    };
    let collapsed =
        emitter.model_projection(&birth, &sidecar, &model, 0.5, Mat4::from_scale(Vec3::ZERO));
    assert!(collapsed.transform.is_none());
    assert_eq!(collapsed.ordinal, 7);
    assert_eq!(collapsed.age, 0.5);
    assert_eq!(collapsed.initial_transform, Transform::IDENTITY);
    let recovered = emitter.model_projection(
        &birth,
        &sidecar,
        &model,
        0.75,
        Mat4::from_translation(Vec3::X * 9.),
    );
    assert!(recovered.transform.is_some());
    assert_eq!(recovered.ordinal, collapsed.ordinal);
    emitter.flags.set_attached(false);
    assert!(emitter
        .model_projection(&birth, &sidecar, &model, 0.75, Mat4::from_scale(Vec3::ZERO))
        .transform
        .is_some());
}

#[test]
fn refraction_keeps_scene_fog_and_uses_data_texture_with_independent_uv_motion() {
    use super::super::fixture_tests::gravity_fixture;
    use crate::textures::{texture_requests, TextureColorSpace, TextureInput, TextureRole};
    use wc3::model::conversion::ImportOptions;
    use wc3::model::io::m2 as semantic_m2;
    let mut native = gravity_fixture(false, 0.0f32.to_bits());
    native.model.core_mut().particles[0]
        .flags
        .set_refraction(true);
    let model: Model<'_, Universal> = semantic_m2::import(&native, &ImportOptions::default())
        .unwrap()
        .value
        .widen();
    let requests = texture_requests(&model);
    let particle = requests
        .iter()
        .find(|request| matches!(request.input, Some(TextureInput::Particle { slot: 0, .. })))
        .unwrap();
    assert_eq!(particle.role, TextureRole::Data);
    assert_eq!(particle.color_space, TextureColorSpace::Linear);
    let image = Handle::<Image>::default();
    let images = TextureImages::resolve(&model, |_| Some(image.clone()));
    let nodes = model
        .nodes()
        .iter()
        .enumerate()
        .map(|(i, (id, _))| (id, i))
        .collect();
    let runtime = RuntimeM2Particles::compile(&model, &nodes, &images, &HashMap::new());
    let emitter = &runtime.emitters[0];
    assert!(emitter.flags.refraction());
    assert!(!emitter.flags.unfogged());
    assert_eq!(
        emitter.gpu_style.composition,
        AgeSpriteComposition::Refraction
    );
    assert_eq!(emitter.gpu_style.textures[0], Some(image));
    assert!(emitter.gpu_style.unlit);
    assert!(emitter
        .age_appearance
        .as_ref()
        .unwrap()
        .needs_particle_state());
    assert!(!runtime.diagnostics.iter().any(|(_, issue)| issue
        .message
        .contains("ordinary primary texture")
        || issue.message.contains("write main-view depth")));
}

#[test]
fn follow_default_knots_extrapolate_in_native_model_units() {
    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    emitter.flags.set_follow_position(true);
    emitter.follow = [Vec2::new(2.5, 0.7), Vec2::new(7., 0.9)];
    for (speed, response) in [(1., 0.6333333), (2.5, 0.7), (7., 0.9), (14., 1.)] {
        let translation = Mat4::from_translation(Vec3::X * speed);
        assert!(
            (emitter
                .follow_displacement(Mat4::IDENTITY, translation, 1.)
                .x
                - speed * response)
                .abs()
                < 2e-6
        );
        emitter.follow_unit_scale = 100.;
        let translation = Mat4::from_translation(Vec3::X * speed * 100.);
        assert!(
            (emitter
                .follow_displacement(Mat4::IDENTITY, translation, 1.)
                .x
                / 100.
                - speed * response)
                .abs()
                < 2e-6
        );
        emitter.follow_unit_scale = 1.;
    }
    emitter.follow = [Vec2::new(0., -2.), Vec2::new(1., -1.)];
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, Mat4::from_translation(Vec3::X), 1.),
        Vec3::ZERO
    );
    emitter.follow = [Vec2::new(0., 0.5), Vec2::new(1e-8, 0.9)];
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, Mat4::from_translation(Vec3::X), 1.),
        Vec3::ZERO
    );
}
#[test]
fn native_follow_flag_is_retained_as_a_known_compiled_control() {
    use super::super::fixture_tests::gravity_fixture;
    use wc3::model::conversion::ImportOptions;
    use wc3::model::io::m2 as semantic_m2;
    for enabled in [false, true] {
        let mut native = gravity_fixture(false, 0.0f32.to_bits());
        native.model.core_mut().particles[0]
            .flags
            .set_follow_position(enabled);
        let model = semantic_m2::import(&native, &ImportOptions::default())
            .unwrap()
            .value
            .widen();
        let indices = model
            .nodes()
            .iter()
            .enumerate()
            .map(|(index, (id, _))| (id, index))
            .collect();
        let runtime = RuntimeM2Particles::compile(
            &model,
            &indices,
            &TextureImages::default(),
            &HashMap::new(),
        );
        assert_eq!(runtime.emitters[0].flags.follow_position(), enabled);
        assert_eq!(
            runtime.emitters[0].follow_unit_scale, 1.,
            "unspecified native units retain numerical scale one"
        );
        assert!(!runtime
            .diagnostics()
            .iter()
            .any(|(_, issue)| issue.kind == RenderDiagnosticKind::Unsupported
                && issue.message.contains("particle controls")));
    }
}

#[test]
fn lifetime_scale_is_half_extent_while_tail_time_is_full_displacement() {
    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    emitter.head_tail = ParticleHeadTail::Both;
    emitter.gpu_style.head = true;
    emitter.gpu_style.tail = true;
    emitter.tail_length = 0.75;
    emitter.appearance.scale = Some(
        Curve::linear(
            [
                ModelTime::seconds(0.).unwrap(),
                ModelTime::seconds(1.).unwrap(),
            ],
            [Vec2::new(1., 2.), Vec2::new(3., 4.)],
        )
        .unwrap(),
    );
    let birth = Birth {
        time: 0.,
        lifetime: 2.,
        origin: Vec3::ZERO,
        velocity: Vec3::X * 2.,
        acceleration: Vec3::ZERO,
        spin: 0.,
        spin_rate: 0.,
        scale_variation: Vec2::ONE,
        random_cell: 0,
        size_scale: Vec2::ONE,
        appearance_index: 0,
    };
    let mut state = EmitterInstance {
        samples: Default::default(),
        mesh: Handle::default(),
        passes: Vec::new(),
        births: vec![birth],
        age_appearances: Vec::new(),
        models: None,
        order: Vec::new(),
        fraction: 0.,
        squirt: SquirtState::default(),
        ordinal: 0,
        previous_node: Mat4::IDENTITY,
        pose_scratch: Vec::new(),
        dropped: 0,
        skipped_seconds: 0.,
    };
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    builder.import_values(&values).unwrap();
    let model = builder.finish().unwrap();
    let program = AnimationProgram::compile(&model);
    let sample = program.sample(AnimationSample::default()).unwrap();
    let camera = Mat4::from_translation(Vec3::Z * 10.);
    let mut data = AgeSpriteData {
        records: Default::default(),
        live_indices: Default::default(),
        uniform: Default::default(),
        style: emitter.gpu_style.clone(),
    };
    for time in [0., 0.5, 1.] {
        // The scale curve is addressed by normalized age, not elapsed seconds.
        let half_extent = Vec2::new(1. + time as f32, 2. + time as f32);
        let center = Vec3::X * (2. * time as f32);
        let frame = EffectFrame {
            traversal: Default::default(),
            change: Default::default(),
            root_visible: true,
            pose: None,
            selection: Default::default(),
            time,
            sample: &sample,
            node_poses: &[Mat4::IDENTITY],
            node_visible: &[true],
            basis_world: Mat4::IDENTITY,
        };
        emitter.sample_gpu(&mut state, &frame, Mat4::IDENTITY, &mut data);
        assert_eq!(data.records.len(), 1);
        assert_eq!(&data.records[0].size_spin[..2], &half_extent.to_array());
        let head = gpu_sample_corners(&data.records[0], &data.uniform, camera, false);
        assert!(head[0].abs_diff_eq(
            center - half_extent.x * Vec3::X - half_extent.y * Vec3::Y,
            1e-6
        ));
        assert!(head[2].abs_diff_eq(
            center + half_extent.x * Vec3::X + half_extent.y * Vec3::Y,
            1e-6
        ));
        let tail = gpu_sample_corners(&data.records[0], &data.uniform, camera, true);
        let end = center - Vec3::X * 1.5;
        assert!(((tail[0] + tail[1]) * 0.5).abs_diff_eq(end, 1e-6));
        assert!(((tail[2] + tail[3]) * 0.5).abs_diff_eq(center, 1e-6));
        assert!(((tail[1] - tail[0]).length() - 2. * half_extent.x).abs() < 1e-6);
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        emitter.draw(&mut state, &frame, Mat4::IDENTITY, &mut mesh, camera);
        let Some(VertexAttributeValues::Float32x3(vertices)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("live head and tail geometry absent")
        };
        assert_eq!(vertices.len(), 8);
        for (expected, actual) in head.into_iter().chain(tail).zip(vertices) {
            assert!(expected.abs_diff_eq(Vec3::from_array(*actual), 1e-6));
        }
    }
}

#[test]
fn unresolved_native_child_emitter_keeps_the_parent_sprite_interpreter() {
    use super::super::fixture_tests::gravity_fixture;
    use wc3::model::common::ZeroVec;
    use wc3::model::conversion::ImportOptions;
    use wc3::model::formats::m2::{DynamicDocument, File, FileKey};
    use wc3::model::io::m2 as semantic_m2;

    for recursive in [false, true] {
        let mut native = gravity_fixture(false, 0.0f32.to_bits());
        if recursive {
            native.model.core_mut().particles[0].child_model_filename =
                ZeroVec::alloc_from_slice(b"unresolved-child.m2\0");
            // Strict native import requires every supporting model document.
            // Runtime adoption remains unbound: compile() receives no children.
            let child = gravity_fixture(false, 0.0f32.to_bits()).model;
            native.insert(
                FileKey::Path("unresolved-child.m2".into()),
                File::Document(Box::new(DynamicDocument::Flat256(child))),
            );
        }
        let model: Model<'_, Universal> = semantic_m2::import(&native, &ImportOptions::default())
            .unwrap()
            .value
            .widen();
        let node_indices = model
            .nodes()
            .iter()
            .enumerate()
            .map(|(index, (id, _))| (id, index))
            .collect();
        let compiled = RuntimeM2Particles::compile(
            &model,
            &node_indices,
            &TextureImages::default(),
            &HashMap::new(),
        );
        assert_eq!(
            compiled.emitters.len(),
            1,
            "an unsupported child cannot remove parent sprites"
        );
        let emitter = &compiled.emitters[0];
        assert!(emitter.model.is_none());
        assert!(emitter.gpu_style.head);
        assert_eq!(
            compiled.diagnostics.iter().any(|(_, issue)| {
                issue.kind == RenderDiagnosticKind::Resource
                    && issue
                        .message
                        .contains("adopted-emitter source is unresolved")
            }),
            recursive
        );
    }
}

#[test]
fn varied_birth_packet_preserves_the_established_random_draw_order() {
    let mut values = SlotValues::default();
    let mut emitter = emitter(&mut values);
    emitter.speed = values.add(7.);
    emitter.lifespan = values.add(3.);
    let controls = emitter.controls.as_mut().unwrap();
    controls.speed_variation = values.add(0.3);
    controls.lifespan_variation = 0.4;
    controls.width = values.add(2.);
    controls.length = values.add(5.);
    controls.gravity = values.add(Vec3::new(0.5, -1., -3.));
    emitter.appearance.scale_variation = Vec2::new(0.2, 0.7);
    emitter.flags.set_independent_scale(true);
    emitter.flags.set_random_texture(true);
    emitter.spin = [0.4, 0.8, 0.6, 0.2];
    emitter.atlas = [3, 5];
    let node = Mat4::from_scale_rotation_translation(
        Vec3::new(2., 3., 4.),
        Quat::from_rotation_z(0.3),
        Vec3::new(5., -2., 1.),
    );
    let birth = emitter
        .birth(&values, 2.5, node, Vec3::new(1., -2., 0.5), 17)
        .unwrap();
    let packet = [
        birth.lifetime,
        birth.origin.x,
        birth.origin.y,
        birth.origin.z,
        birth.velocity.x,
        birth.velocity.y,
        birth.velocity.z,
        birth.acceleration.x,
        birth.acceleration.y,
        birth.acceleration.z,
        birth.spin,
        birth.spin_rate,
        birth.scale_variation.x,
        birth.scale_variation.y,
        birth.size_scale.x,
        birth.size_scale.y,
    ];
    let expected = [
        3.150495, 8.294773, 2.1575747, 1., 2.120071, -2.569796, 27.664768, 1.8418971, -2.570489,
        -12., 0.8048676, 0.581381, 0.9193937, 0.9320074, 1.9999999, 2.9999998,
    ];
    for (actual, expected) in packet.into_iter().zip(expected) {
        assert!((actual - expected).abs() < 2e-5, "{actual} != {expected}");
    }
    assert_eq!(birth.random_cell, 14);
}

#[test]
fn frozen_birth_inputs_reuse_the_ordinary_packet_and_ignore_later_controls() {
    let mut values = SlotValues::default();
    let emitter = emitter(&mut values);
    let frozen = BirthInputs::sample(&emitter, &values);
    let frame = BirthFrame {
        time: 0.25,
        node: Mat4::from_rotation_translation(Quat::from_rotation_z(0.4), Vec3::new(2., 3., 4.)),
        basis: Mat4::from_scale(Vec3::splat(0.01)),
        inherited: Vec3::new(1., 2., 3.),
        ordinal: 17,
    };
    let ordinary = emitter
        .birth_posed(
            &values,
            frame.time,
            frame.node,
            frame.basis,
            frame.inherited,
            frame.ordinal,
            |_| None,
        )
        .unwrap()
        .0;
    let repeated = emitter.birth_sampled(frozen, frame, |_| None).unwrap().0;
    assert_eq!(ordinary.origin, repeated.origin);
    assert_eq!(ordinary.velocity, repeated.velocity);
    assert_eq!(ordinary.acceleration, repeated.acceleration);
    assert_eq!(ordinary.spin, repeated.spin);
    assert_eq!(ordinary.random_cell, repeated.random_cell);
    let controls = emitter.controls.as_ref().unwrap();
    values.set(emitter.speed, 0.).unwrap();
    values.set(controls.gravity, Vec3::Z * -9.).unwrap();
    values.set(controls.activation, false).unwrap();
    assert!(emitter
        .birth_posed(
            &values,
            frame.time,
            frame.node,
            frame.basis,
            frame.inherited,
            frame.ordinal,
            |_| None
        )
        .is_none());
    let retained = emitter.birth_sampled(frozen, frame, |_| None).unwrap().0;
    assert_eq!(ordinary.velocity, retained.velocity);
    assert_eq!(ordinary.acceleration, retained.acceleration);
}

#[test]
fn ordinary_birth_packets_match_the_pre_extraction_export() {
    // Recorded by the existing ignored Emitter::birth exporter before this
    // refactor. These fixed values do not call a second implementation oracle.
    let mut values = SlotValues::default();
    let emitter = emitter(&mut values);
    for (ordinal, expected) in [
        (0, Vec3::new(1.7775202, 3.043055, 8.317796)),
        (17, Vec3::new(0.6558661, -0.42448652, 9.879823)),
        (4095, Vec3::new(-0.29503617, 4.555508, 9.661248)),
    ] {
        let birth = emitter
            .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, ordinal)
            .unwrap();
        assert!(
            birth.velocity.abs_diff_eq(expected, 1e-5),
            "changed packet at ordinal {ordinal}: {:?}",
            birth.velocity
        );
        assert_eq!(birth.origin, Vec3::ZERO);
        assert_eq!(birth.acceleration, Vec3::ZERO);
        assert_eq!(birth.lifetime, 2.);
        assert_eq!(birth.scale_variation, Vec2::ONE);
    }
}

#[test]
fn adopted_catalogue_preserves_rejected_candidate_slots_and_never_promotes_fifth() {
    use super::super::fixture_tests::gravity_fixture;
    use wc3::model::common::ZeroVec;
    use wc3::model::conversion::ImportOptions;
    use wc3::model::formats::m2::records::ParticleGenerator as NativeGenerator;
    use wc3::model::formats::m2::{DynamicDocument, File, FileKey};
    use wc3::model::io::m2 as semantic_m2;

    let mut native = gravity_fixture(false, 0.0f32.to_bits());
    let core = native.model.core_mut();
    core.particles = vec![core.particles[0].clone(); 5];
    core.particles[0].flags.set_attached(true);
    core.particles[1].kind.emitter = NativeGenerator::Bone;
    core.particles[2].child_model_filename = ZeroVec::alloc_from_slice(b"nested.m2\0");
    core.particles[3].particle_model_filename = ZeroVec::alloc_from_slice(b"geometry.m2\0");
    for name in ["nested.m2", "geometry.m2"] {
        let child = gravity_fixture(false, 0.0f32.to_bits()).model;
        native.insert(
            FileKey::Path(name.into()),
            File::Document(Box::new(DynamicDocument::Flat256(child))),
        );
    }
    let model: Model<'_, Universal> = semantic_m2::import(&native, &ImportOptions::default())
        .unwrap()
        .value
        .widen();
    let candidates = model
        .features()
        .iter()
        .filter_map(|(feature, value)| {
            matches!(value.payload, NodeFeature::Particle(ParticleEmitter::M2(_)))
                .then_some(feature)
        })
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 5);
    let nodes = model
        .nodes()
        .iter()
        .enumerate()
        .map(|(index, (node, _))| (node, index))
        .collect();
    let runtime =
        RuntimeM2Particles::compile(&model, &nodes, &TextureImages::default(), &HashMap::new());
    assert!(
        runtime
            .emitters
            .iter()
            .any(|emitter| emitter.feature == candidates[4]),
        "the fifth is a supported ordinary sprite"
    );
    assert_eq!(
        runtime
            .templates
            .slots
            .iter()
            .map(|slot| slot.feature)
            .collect::<Vec<_>>(),
        candidates[..4]
    );
    assert!(
        runtime
            .templates
            .slots
            .iter()
            .all(|slot| slot.template.is_err()),
        "rejected slots consume their authored positions"
    );
}

#[test]
fn adopted_controls_freeze_time_zero_force_activation_and_share_scoped_images() {
    use super::super::fixture_tests::gravity_fixture;
    use wc3::model::common::ZeroVec;
    use wc3::model::conversion::ImportOptions;
    use wc3::model::formats::m2::records::LegacyValues;
    use wc3::model::io::m2 as semantic_m2;

    let mut native = gravity_fixture(false, 0.0f32.to_bits());
    let particle = &mut native.model.core_mut().particles[0];
    particle.emission_rate.base.timeline.timestamps = ZeroVec::alloc_from_slice(&[0, 500]);
    particle.emission_rate.values = LegacyValues::Values(ZeroVec::alloc_from_slice(&[12., 0.]));
    particle.enabled.values = LegacyValues::Values(ZeroVec::alloc_from_slice(&[0]));
    let model: Model<'_, Universal> = semantic_m2::import(&native, &ImportOptions::default())
        .unwrap()
        .value
        .widen();
    let mut images = Assets::<Image>::default();
    let image = images.add(Image::default());
    let bindings = TextureImages::resolve(&model, |_| Some(image.clone()));
    let nodes = model
        .nodes()
        .iter()
        .enumerate()
        .map(|(index, (node, _))| (node, index))
        .collect();
    let mut runtime = RuntimeM2Particles::compile(&model, &nodes, &bindings, &HashMap::new());
    let template = runtime.templates.slots[0].template.as_ref().unwrap();
    assert!(Arc::ptr_eq(template.kernel(), &runtime.emitters[0]));
    assert_eq!(
        template.kernel().gpu_style.textures[0].as_ref(),
        Some(&image)
    );
    let later = model
        .animation_sample(AnimationSample {
            clip_time: ModelTime::seconds(0.75).unwrap(),
            shared_time: ModelTime::seconds(0.75).unwrap(),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(later.animated(&runtime.emitters[0].rate), 0.);
    assert_eq!(template.rate, 12.);
    let node = Mat4::from_translation(Vec3::new(2., 3., 4.));
    assert!(runtime.emitters[0]
        .birth(&later, 0.75, node, Vec3::ZERO, 0)
        .is_none());
    let birth = template
        .birth(BirthFrame {
            time: 0.75,
            node,
            basis: Mat4::IDENTITY,
            inherited: Vec3::ZERO,
            ordinal: 0,
        })
        .unwrap()
        .0;
    assert_eq!(birth.origin, Vec3::new(2., 3., 4.));
    let replacement = images.add(Image::default());
    let mut image_visits = 0;
    runtime.map_images(&mut |_| {
        image_visits += 1;
        Some(replacement.clone())
    });
    let mapped = runtime.templates.slots[0].template.as_ref().unwrap();
    assert_eq!(
        image_visits, 1,
        "one shared immutable kernel has one image mapping"
    );
    assert!(Arc::ptr_eq(mapped.kernel(), &runtime.emitters[0]));
    assert_eq!(
        mapped.kernel().gpu_style.textures[0].as_ref(),
        Some(&replacement)
    );
}

fn adopted_pool(rate: f32, lifetime: f32, inherit: bool) -> adopted_pool::AdoptedPool {
    let mut values = SlotValues::default();
    let mut definition = emitter(&mut values);
    values.set(definition.rate, rate).unwrap();
    values.set(definition.speed, 0.).unwrap();
    values.set(definition.lifespan, lifetime).unwrap();
    values
        .set(definition.controls.as_ref().unwrap().activation, false)
        .unwrap();
    definition.flags.set_inherit_velocity(inherit);
    definition.inherit_velocity = 0.;
    let template = templates::FrozenTemplate::new(&Arc::new(definition), &values);
    adopted_pool::AdoptedPool::new(Arc::new(template))
}

fn drive(start: f64, end: f64, x: f32) -> adopted_pool::SourceDrive {
    adopted_pool::SourceDrive {
        start,
        end,
        placement: Mat4::from_translation(Vec3::X * x),
        basis: Mat4::IDENTITY,
        velocity: Vec3::ZERO,
        emission_scale: 1.,
    }
}

#[test]
fn adopted_sources_share_one_fractional_phase_without_a_bridge() {
    let mut pool = adopted_pool(1., 2., false);
    pool.finish(0.25);
    assert!(
        pool.storage.births.is_empty(),
        "templates have no autonomous emission"
    );
    let mut work = adopted_pool::AdoptionWork::default();
    pool.drive(drive(0., 0.25, 2.), &mut work);
    pool.drive(drive(0., 0.25, 9.), &mut work);
    assert!(pool.storage.births.is_empty());
    pool.drive(drive(0.25, 0.5, 2.), &mut work);
    pool.drive(drive(0.25, 0.5, 9.), &mut work);
    assert_eq!(pool.storage.births.len(), 1);
    assert_eq!(pool.storage.births[0].origin, Vec3::X * 9.);
    assert_eq!(pool.storage.births[0].time, 0.5);
    let mut independent = adopted_pool(1., 2., false);
    independent.drive(drive(0.25, 0.5, 9.), &mut work);
    assert!(
        independent.storage.births.is_empty(),
        "separate parent emitters do not share carry"
    );
}

#[test]
fn adopted_children_age_once_and_outlive_their_source() {
    let mut pool = adopted_pool(4., 0.5, false);
    let mut work = adopted_pool::AdoptionWork::default();
    for x in [1., 2., 3., 4.] {
        pool.drive(drive(0., 0.25, x), &mut work);
    }
    assert_eq!(pool.storage.births.len(), 4);
    assert!(pool.storage.births.iter().all(|birth| birth.time == 0.25));
    // No source survives after .25; independent child lifetimes continue.
    pool.finish(0.5);
    assert_eq!(pool.storage.births.len(), 4);
    pool.finish(0.75);
    assert!(pool.storage.births.is_empty());
}

#[test]
fn adopted_capacity_omits_births_without_phase_backlog() {
    let mut pool = adopted_pool(4096., 10., false);
    let mut work = adopted_pool::AdoptionWork::default();
    pool.drive(drive(0., 1., 1.), &mut work);
    assert_eq!(pool.storage.births.len(), adopted_pool::POOL_CAPACITY);
    pool.drive(drive(1., 2., 2.), &mut work);
    assert_eq!(pool.storage.dropped, 4096);
    pool.finish(11.);
    assert!(pool.storage.births.is_empty());
    let mut work = adopted_pool::AdoptionWork::default();
    pool.drive(drive(11., 11.25, 3.), &mut work);
    assert_eq!(pool.storage.births.len(), 1024);
    assert!(pool
        .storage
        .births
        .iter()
        .all(|birth| birth.origin == Vec3::X * 3.));
}

#[test]
fn adopted_inherited_velocity_is_flag_gated_and_variation_is_about_one() {
    for inherit in [false, true] {
        let mut pool = adopted_pool(4., 2., inherit);
        let mut source = drive(0., 0.25, 2.);
        source.basis = Mat4::from_scale(Vec3::splat(0.01));
        source.velocity = Vec3::X * 3.;
        pool.drive(source, &mut adopted_pool::AdoptionWork::default());
        assert_eq!(
            pool.storage.births[0].velocity,
            if inherit { Vec3::X * 3. } else { Vec3::ZERO }
        );
    }
}

#[test]
fn adopted_reset_clears_particles_and_fractional_phase() {
    let mut pool = adopted_pool(1., 2., false);
    let mut work = adopted_pool::AdoptionWork::default();
    pool.drive(drive(0., 0.75, 1.), &mut work);
    pool.reset();
    pool.drive(drive(0.75, 1., 2.), &mut work);
    assert!(pool.storage.births.is_empty());
    assert_eq!(pool.storage.dropped, 0);
}

#[test]
fn source_spans_preserve_trails_from_parents_born_and_dead_inside_one_update() {
    let mut values = SlotValues::default();
    let parent = emitter(&mut values);
    values.set(parent.speed, 0.).unwrap();
    values.set(parent.lifespan, 0.2).unwrap();
    let birth = parent
        .birth(
            &values,
            0.1,
            Mat4::from_translation(Vec3::X * 7.),
            Vec3::ZERO,
            0,
        )
        .unwrap();
    assert!(
        1. - birth.time >= f64::from(birth.lifetime),
        "normal endpoint publication omits this parent"
    );
    let mut snapshots = sources::SourceSpans::default();
    snapshots.begin(&[], Vec3::ZERO);
    snapshots.newborn(&birth, Vec3::ZERO, 0.1);
    let mut pool = adopted_pool(60., 2., false);
    let mut work = adopted_pool::AdoptionWork::default();
    for step in EmissionClock::new(0., 1.) {
        assert!(snapshots.drive(
            sources::SourceStep {
                emitter: &parent,
                start: step.start,
                end: step.end,
                frame_start: 0.,
                frame_end: 1.,
                follow: Vec3::ZERO,
                centers: [Vec3::ZERO; 2],
                node: Mat4::IDENTITY,
                basis: Mat4::IDENTITY
            },
            &mut work,
            |drive, work| pool.drive(drive, work)
        ));
    }
    pool.finish(1.);
    assert!(
        !pool.storage.births.is_empty(),
        "earlier surviving source steps leave independently living children"
    );
    assert!(pool
        .storage
        .births
        .iter()
        .all(|birth| birth.origin == Vec3::X * 7. && birth.time >= 0.1 && birth.time < 0.31));
}

#[test]
fn source_examination_budget_bounds_dead_sources_and_has_no_implicit_drive() {
    let mut values = SlotValues::default();
    let parent = emitter(&mut values);
    let birth = parent
        .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    let mut snapshots = sources::SourceSpans::default();
    snapshots.begin(&[birth.clone(), birth], Vec3::ZERO);
    let mut work = adopted_pool::AdoptionWork {
        source_checks: 1,
        child_births: 10,
        ..Default::default()
    };
    let mut drives = 0;
    assert!(!snapshots.drive(
        sources::SourceStep {
            emitter: &parent,
            start: 3.,
            end: 3.1,
            frame_start: 3.,
            frame_end: 3.1,
            follow: Vec3::ZERO,
            centers: [Vec3::ZERO; 2],
            node: Mat4::IDENTITY,
            basis: Mat4::IDENTITY
        },
        &mut work,
        |_, _| drives += 1
    ));
    assert_eq!(drives, 0);
    assert_eq!(work.source_checks, 0);
}

#[test]
fn undriven_expired_children_do_not_spend_motion_work() {
    let mut values = SlotValues::default();
    let mut kernel = emitter(&mut values);
    values.set(kernel.rate, 60.).unwrap();
    values.set(kernel.speed, 0.).unwrap();
    values.set(kernel.lifespan, 0.05).unwrap();
    kernel.flags.set_follow_position(true);
    kernel.follow = [Vec2::new(0., 1.), Vec2::new(1., 1.)];
    let template = templates::FrozenTemplate::new(&Arc::new(kernel), &values);
    let mut pool = adopted_pool::AdoptedPool::new(Arc::new(template));
    let mut work = adopted_pool::AdoptionWork::default();
    pool.begin_frame(Mat4::IDENTITY);
    pool.drive(drive(0., 1. / 60., 0.), &mut work);
    assert_eq!(pool.storage.births.len(), 1);
    // No surviving parent drives after this interval. At this point every child
    // has expired; charging even one motion check would spuriously omit time.
    work.motion_checks = 0;
    pool.advance_owner(1. / 60., 1., Mat4::from_translation(Vec3::X), 1., &mut work);
    assert!(pool.storage.births.is_empty());
    assert_eq!(pool.finish_frame(1.), 0.);
    assert_eq!(pool.storage.skipped_seconds, 0.);
}

#[test]
fn adopted_diagnostics_ignore_autonomous_squirt_without_hiding_other_controls() {
    use super::super::fixture_tests::gravity_fixture;
    use wc3::model::conversion::ImportOptions;
    use wc3::model::io::m2 as semantic_m2;
    for ground_clamp in [false, true] {
        let mut native = gravity_fixture(false, 0.0f32.to_bits());
        let flags = &mut native.model.core_mut().particles[0].flags;
        flags.set_squirt(true);
        flags.set_ground_clamp(ground_clamp);
        let model: Model<'_, Universal> = semantic_m2::import(&native, &ImportOptions::default())
            .unwrap()
            .value
            .widen();
        let nodes = model
            .nodes()
            .iter()
            .enumerate()
            .map(|(index, (node, _))| (node, index))
            .collect();
        let runtime =
            RuntimeM2Particles::compile(&model, &nodes, &TextureImages::default(), &HashMap::new());
        assert_eq!(
            runtime
                .diagnostics
                .iter()
                .any(|(_, issue)| issue.kind == RenderDiagnosticKind::Unsupported),
            ground_clamp
        );
        let template = runtime.templates.slots[0].template.as_ref().unwrap();
        assert_eq!(
            template
                .issues
                .iter()
                .any(|issue| issue.kind == RenderDiagnosticKind::Unsupported),
            ground_clamp,
            "template roles ignore autonomous squirt, while ground projection remains unsupported"
        );
    }
}

#[test]
fn undriven_long_lived_pools_skip_resident_scans_across_a_full_long_jump() {
    let mut pool = adopted_pool(4096., 1000., false);
    let mut work = adopted_pool::AdoptionWork::default();
    pool.begin_frame(Mat4::IDENTITY);
    pool.drive(drive(0., 1., 0.), &mut work);
    assert_eq!(pool.storage.births.len(), adopted_pool::POOL_CAPACITY);
    assert_eq!(pool.retirement_visits, 0);
    let mut steps = 0;
    for step in EmissionClock::new(1., 100.) {
        pool.advance_owner(step.start, step.end, Mat4::IDENTITY, 1., &mut work);
        steps += 1;
    }
    assert_eq!(
        steps, 4096,
        "exercise the entire permitted long-jump prefix"
    );
    assert_eq!(
        pool.retirement_visits, 0,
        "long-lived residents must not be revisited for every undriven quantum"
    );
    assert_eq!(pool.storage.births.len(), adopted_pool::POOL_CAPACITY);
    assert_eq!(work.motion_checks, adopted_pool::SOURCE_CHECKS);
    assert_eq!(pool.finish_frame(100.), 0.);
    pool.finish(1001.);
    assert!(pool.storage.births.is_empty());
    assert_eq!(pool.retirement_visits, adopted_pool::POOL_CAPACITY);
    pool.finish(2000.);
    assert_eq!(
        pool.retirement_visits,
        adopted_pool::POOL_CAPACITY,
        "empty pools stay constant work"
    );
}
#[test]
fn expiry_boundary_releases_capacity_before_the_next_adopted_admission() {
    let mut pool = adopted_pool(4096., 1., false);
    let mut work = adopted_pool::AdoptionWork::default();
    pool.drive(drive(0., 1., 0.), &mut work);
    let first = pool.storage.births[0].clone();
    let death = first.time + f64::from(first.lifetime);
    pool.finish(death.next_down());
    assert_eq!(pool.storage.births.len(), adopted_pool::POOL_CAPACITY);
    pool.finish(death);
    assert_eq!(pool.storage.births.len(), adopted_pool::POOL_CAPACITY - 1);
    let mut birth = drive(death, death + 1. / (4096. * 4096.), 3.);
    birth.emission_scale = 4096.;
    pool.drive(birth, &mut work);
    assert_eq!(pool.storage.births.len(), adopted_pool::POOL_CAPACITY);
    assert_eq!(pool.storage.dropped, 0);
    assert_eq!(pool.storage.births.last().unwrap().origin, Vec3::X * 3.);
    pool.reset();
    pool.finish(2000.);
    assert_eq!(pool.retirement_visits, 0);
    assert!(pool.storage.births.is_empty());
}
#[test]
fn expiry_wake_up_never_delays_the_existing_subtraction_boundary() {
    let mut values = SlotValues::default();
    let kernel = emitter(&mut values);
    let mut birth = kernel
        .birth(&values, 0., Mat4::IDENTITY, Vec3::ZERO, 0)
        .unwrap();
    for (time, lifetime) in [(0., 1.), (-1., 1.), (1e12, 0.25), (-1e12, 0.25)] {
        birth.time = time;
        birth.lifetime = lifetime;
        let expiry = time + f64::from(lifetime);
        for now in [expiry.next_down(), expiry, expiry.next_up()] {
            if now - birth.time >= f64::from(birth.lifetime) {
                assert!(
                    now >= birth.retirement_bound(),
                    "cached wake-up must precede actual retirement"
                );
            }
        }
    }
    // At cancellation, a negative value far below rounded birth+lifetime=0
    // already subtracts to the full lifetime. A next_down(sum)-only cache fails.
    birth.time = -1.;
    birth.lifetime = 1.;
    let now = -f64::EPSILON * 0.25;
    assert_eq!(now - birth.time, 1.);
    assert!(now >= birth.retirement_bound());
}
