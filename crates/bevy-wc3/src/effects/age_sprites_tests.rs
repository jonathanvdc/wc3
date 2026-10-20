use super::*;
use bevy::math::{Quat, Vec2, Vec3};
use std::f32::consts::TAU;
use wc3::model::profiles::Universal;
use wc3::model::ModelMetadata;
use wc3::model::{AnimationProgram, AnimationSample};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
fn emitter(slot_values: &mut SlotValues) -> Emitter {
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
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
        flags,
        drag: 0.,
        wind: Vec3::ZERO,
        inherit_velocity: 0.,
        follow: [Vec2::ZERO; 2],
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
            composition: 0,
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
        fraction: 0.,
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
        &mut gpu_query.query_mut(&mut gpu_world),
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
        &mut gpu_query.query_mut(&mut gpu_world),
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
fn emitter_follow_interpolates_speed_knots_and_preserves_local_motion() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
    emitter.follow = [Vec2::new(2., 0.25), Vec2::new(6., 0.75)];
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
    emitter.follow = [Vec2::new(2., 0.5); 2];
    assert_eq!(
        emitter.follow_displacement(Mat4::IDENTITY, translated(4.), 1.),
        Vec3::X * 2.
    );
}

#[test]
fn world_particles_follow_only_translation_after_their_birth() {
    let mut slot_values = SlotValues::default();
    let mut emitter = emitter(&mut slot_values);
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
    let mut gpu_world = World::new();
    let mut gpu_query = gpu_world.query::<&mut AgeSpriteData>();
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
    };
    runtime.update(
        &mut instances,
        frame,
        &mut meshes,
        &mut gpu_query.query_mut(&mut gpu_world),
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
    let mut side = (right * cos + up * sin) * width * 0.5;
    let mut vertical = (up * cos - right * sin) * height * 0.5;
    if tail {
        let extent = velocity * record.velocity_tail[3];
        side = extent
            .cross(camera.w_axis.truncate() - position)
            .normalize_or_zero()
            * width
            * 0.5;
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
        order: Vec::new(),
        fraction: 0.,
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
        order: Vec::new(),
        fraction: 0.,
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
    println!("8192 head+tail particles: CPU indexed corners {corners:.6}s; compact samples {samples:.6}s; preparation ratio {:.2}x (per-view render sorting excluded)", corners / samples);
    println!(
        "live payload: {} compact bytes vs {} indexed-corner bytes, excluding draw-order buffers",
        data.records.len() * size_of::<AgeSpriteRecord>(),
        MAX_PARTICLES * 2 * 204
    );
}
