use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use bevy_wc3::{
    prepare_model, spawn_prepared_model, Wc3Animation, Wc3CameraBinding, Wc3LayerMaterial,
    Wc3Model, Wc3ModelCameras,
};
use std::f32::consts::FRAC_PI_4;

#[test]
fn spawned_instances_expose_authored_views_without_creating_bevy_cameras() {
    let source = Wc3Model::decode_mdl(include_str!("fixtures/cameras.mdl")).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let mut materials = Assets::<Wc3LayerMaterial>::default();
    let prepared = prepare_model(&mut meshes, &mut bindposes, &source, |_| None).unwrap();
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    let first = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    let second = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    queue.apply(&mut world);
    assert_eq!(world.query::<&Camera3d>().iter(&world).count(), 0);
    world.get_mut::<Wc3Animation>(first).unwrap().seek(500.0);
    let cameras = world.get::<Wc3ModelCameras>(first).unwrap();
    assert_eq!(cameras.definitions()[0].name.text(), "Portrait");
    assert_eq!(cameras.definitions()[1].name.text(), "Alternate");
    let sample = cameras
        .sample(0, world.get::<Wc3Animation>(first).unwrap())
        .unwrap();
    assert!(sample
        .position
        .abs_diff_eq(Vec3::new(1.0, -10.0, 0.0), 1e-5));
    assert!(sample.target.abs_diff_eq(Vec3::X, 1e-5));
    assert!((sample.roll - FRAC_PI_4).abs() < 1e-5);
    let other = world
        .get::<Wc3ModelCameras>(second)
        .unwrap()
        .sample(0, world.get::<Wc3Animation>(second).unwrap())
        .unwrap();
    assert_eq!(other.position, Vec3::new(0.0, -10.0, 0.0));
    let lens = sample
        .perspective_projection(Wc3CameraBinding::portrait(first, 0).fov_multiplier)
        .unwrap();
    assert!((lens.fov - 0.6).abs() < 1e-6);
    assert_eq!((lens.near, lens.far), (0.5, 100.0));
    let view = sample.world_transform(&GlobalTransform::IDENTITY).unwrap();
    assert!((view.rotation * Vec3::NEG_Z).abs_diff_eq(Vec3::Y, 1e-5));
}
