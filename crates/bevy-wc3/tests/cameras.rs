use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use bevy_wc3::{
    compile_universal_model, spawn_universal_model, UniversalCameraBinding, UniversalInstance,
    UniversalMaterial, UniversalPlayback,
};
use wc3::model::{conversion::ImportOptions, io, profiles::Universal, NodeFeature};

#[test]
fn spawned_instances_expose_authored_views_without_creating_bevy_cameras() {
    let source = io::mdlx::decode_mdl(
        include_str!("fixtures/cameras.mdl"),
        &ImportOptions::default(),
    )
    .unwrap()
    .value
    .widen::<Universal>();
    let cameras = source
        .features()
        .filter(|(_, feature)| matches!(feature.payload, NodeFeature::Camera(_)))
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    assert_eq!(cameras.len(), 2);
    let mut meshes = Assets::<Mesh>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let mut materials = Assets::<UniversalMaterial>::default();
    let compiled = compile_universal_model(&source, |_| None).unwrap();
    let prepared = compiled.prepare(&mut meshes, &mut bindposes);
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    let first = spawn_universal_model(
        &mut commands,
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::default(),
    );
    let second = spawn_universal_model(
        &mut commands,
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::default(),
    );
    queue.apply(&mut world);
    assert_eq!(world.query::<&Camera3d>().iter(&world).count(), 0);
    world
        .get_mut::<UniversalPlayback>(first)
        .unwrap()
        .seek(0.5)
        .unwrap();
    let sample = world
        .get::<UniversalInstance>(first)
        .unwrap()
        .sample_camera(
            cameras[0],
            world.get::<UniversalPlayback>(first).unwrap(),
            1.,
        )
        .unwrap()
        .unwrap();
    assert!(sample.eye.abs_diff_eq(Vec3::new(1.0, -10.0, 0.0), 1e-5));
    assert!(sample.target.abs_diff_eq(Vec3::X, 1e-5));
    assert!(sample
        .up
        .abs_diff_eq(Vec3::new(0.5_f32.sqrt(), 0., 0.5_f32.sqrt()), 1e-5));
    let other = world
        .get::<UniversalInstance>(second)
        .unwrap()
        .sample_camera(
            cameras[0],
            world.get::<UniversalPlayback>(second).unwrap(),
            1.,
        )
        .unwrap()
        .unwrap();
    assert_eq!(other.eye, Vec3::new(0., -10., 0.));
    let lens = sample
        .perspective_projection(
            UniversalCameraBinding {
                fov_multiplier: 0.75,
                ..UniversalCameraBinding::new(first, cameras[0])
            }
            .fov_multiplier,
        )
        .unwrap();
    assert!((lens.fov - 0.6).abs() < 1e-6);
    assert_eq!((lens.near, lens.far), (0.5, 100.0));
    let view = sample.world_transform(GlobalTransform::IDENTITY).unwrap();
    assert!((view.rotation * Vec3::NEG_Z).abs_diff_eq(Vec3::Y, 1e-5));
}
