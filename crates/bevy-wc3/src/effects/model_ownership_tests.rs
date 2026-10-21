mod linked {
    use super::super::*;
    #[test]
    fn renderer_reload_or_root_death_immediately_removes_pending_and_populated_models() {
        let mut world = World::new();
        let root = world.spawn(Transform::from_xyz(100., 0., 0.)).id();
        let basis = world.spawn(ChildOf(root)).id();
        let emitter = world.spawn(ChildOf(basis)).id();
        let pending = world
            .spawn((ModelParticleOwner(emitter), Transform::from_xyz(2., 3., 4.)))
            .id();
        let ready = world
            .spawn((ModelParticleOwner(emitter), Transform::default()))
            .id();
        let geometry = world.spawn(ChildOf(ready)).id();
        assert!(
            world.get::<ChildOf>(pending).is_none(),
            "lifetime relation cannot move detached geometry"
        );
        world.despawn(basis);
        for entity in [emitter, pending, ready, geometry] {
            assert!(world.get_entity(entity).is_err());
        }
        assert!(
            world.get_entity(root).is_ok(),
            "reload preserves application-owned root"
        );
        let basis = world.spawn(ChildOf(root)).id();
        let emitter = world.spawn(ChildOf(basis)).id();
        let child = world.spawn(ModelParticleOwner(emitter)).id();
        world.despawn(root);
        assert!(world.get_entity(child).is_err());
    }
    #[test]
    fn death_updates_ownership_and_owner_reuse_cannot_resurrect_old_requests() {
        let mut world = World::new();
        let emitter = world.spawn_empty().id();
        let a = world.spawn(ModelParticleOwner(emitter)).id();
        let b = world.spawn(ModelParticleOwner(emitter)).id();
        world.despawn(a);
        assert_eq!(
            world.get::<ModelParticleChildren>(emitter).unwrap().0,
            vec![b]
        );
        world.despawn(emitter);
        assert!(world.get_entity(b).is_err());
        let replacement = world.spawn_empty().id();
        assert_ne!(emitter, replacement);
        assert!(world.get::<ModelParticleChildren>(replacement).is_none());
    }
}

mod queued {
    use super::super::*;
    use bevy::ecs::world::CommandQueue;
    #[derive(Component)]
    struct PendingModelRequest;
    #[test]
    fn deferred_birth_cannot_publish_a_request_after_its_owner_dies() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let mut queue = CommandQueue::default();
        let child = spawn_owned_model(
            &mut Commands::new(&mut queue, &world),
            owner,
            PendingModelRequest,
        );
        world.despawn(owner);
        queue.apply(&mut world);
        assert!(world.get_entity(child).is_err());
    }
    #[test]
    fn same_flush_owner_death_or_birth_cancellation_cannot_leave_a_request() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let mut queue = CommandQueue::default();
        let child = spawn_owned_model(
            &mut Commands::new(&mut queue, &world),
            owner,
            PendingModelRequest,
        );
        Commands::new(&mut queue, &world).entity(owner).despawn();
        queue.apply(&mut world);
        assert!(world.get_entity(child).is_err());
        let owner = world.spawn_empty().id();
        let child = spawn_owned_model(
            &mut Commands::new(&mut queue, &world),
            owner,
            PendingModelRequest,
        );
        Commands::new(&mut queue, &world).entity(child).despawn();
        queue.apply(&mut world);
        assert!(world.get_entity(child).is_err());
    }
}
