use super::*;
use bevy::transform::TransformPlugin;

#[derive(Component)]
struct Spawned;

#[derive(Resource, Default)]
struct Observed(Vec<&'static str>);

#[test]
fn update_consumers_see_spawned_entities_and_advanced_state() {
    let mut app = App::new();
    configure(&mut app);
    app.init_resource::<Observed>();
    // Register consumers before producers so insertion order cannot satisfy the contract.
    app.add_systems(
        Update,
        (
            (|entities: Query<&Spawned>, mut observed: ResMut<Observed>| {
                assert_eq!(entities.iter().count(), 1);
                assert!(observed.0.contains(&"advance"));
                observed.0.push("animate");
            })
            .in_set(Wc3Systems::AnimateInstances),
            (|entities: Query<&Spawned>, mut observed: ResMut<Observed>| {
                assert_eq!(entities.iter().count(), 1);
                observed.0.push("bind");
            })
            .in_set(Wc3Systems::BindTextures),
            (|entities: Query<&Spawned>, mut observed: ResMut<Observed>| {
                assert_eq!(entities.iter().count(), 1);
                observed.0.push("advance");
            })
            .in_set(Wc3Systems::AdvanceAnimation),
            (|mut commands: Commands| {
                commands.spawn(Spawned);
            })
            .in_set(Wc3Systems::SpawnInstances),
        ),
    );
    app.update();
    let observed = &app.world().resource::<Observed>().0;
    assert!(observed.contains(&"bind"));
    assert!(observed.contains(&"animate"));
}

#[test]
fn pose_and_model_particles_precede_propagation_and_quad_effects_follow_it() {
    let mut app = App::new();
    app.add_plugins(TransformPlugin);
    configure(&mut app);
    app.init_resource::<Observed>();
    app.world_mut()
        .spawn((Spawned, Transform::default(), GlobalTransform::default()));
    app.add_systems(
        PostUpdate,
        (
            (|transforms: Query<&GlobalTransform, With<Spawned>>,
              mut observed: ResMut<Observed>| {
                assert_eq!(transforms.single().unwrap().translation(), Vec3::X);
                observed.0.push("effects");
            })
            .in_set(Wc3Systems::SimulateEffects),
            (|transforms: Query<(&Transform, &GlobalTransform), With<Spawned>>,
              mut observed: ResMut<Observed>| {
                let (local, global) = transforms.single().unwrap();
                assert_eq!(local.translation, Vec3::X);
                assert_eq!(global.translation(), Vec3::ZERO);
                observed.0.push("model particles");
            })
            .in_set(Wc3Systems::SimulateModelParticles),
            (|mut transforms: Query<&mut Transform, With<Spawned>>| {
                transforms.single_mut().unwrap().translation = Vec3::X;
            })
            .in_set(Wc3Systems::EvaluateNodePoses),
        ),
    );
    app.update();
    assert_eq!(
        app.world().resource::<Observed>().0,
        ["model particles", "effects"]
    );
}
