use super::*;
use crate::CoordinatePolicy;
use bevy::ecs::system::SystemState;
fn template() -> ModelTemplate {
    ModelTemplate {
        model: Handle::default(),
        coordinates: ChildCoordinates {
            coordinates: CoordinatePolicy::Host,
            parent_basis: Transform::IDENTITY,
        },
    }
}
fn projection(ordinal: u64, age: f64, x: f32) -> ModelProjection {
    ModelProjection {
        lease: None,
        ordinal,
        birth: 0.,
        lifetime: 10.,
        age,
        transform: Some(Transform::from_xyz(x, 2., 3.)),
        initial_transform: Transform::from_xyz(1., 2., 3.),
        visible: true,
    }
}
fn synchronize(
    world: &mut World,
    children: &mut ModelChildren,
    owner: Entity,
    samples: impl IntoIterator<Item = ModelProjection>,
) {
    let mut state = SystemState::<(Commands, ModelOutputs)>::new(world);
    let (mut commands, mut outputs) = state.get_mut(world).unwrap();
    children.synchronize(
        &mut commands,
        ModelProjectionContext {
            root: owner,
            owner,
            visible: true,
            driving_camera: None,
        },
        &template(),
        &mut outputs,
        samples,
    );
    state.apply(world);
}
#[test]
fn birth_identity_survives_compaction_and_repeated_paused_projection() {
    let mut world = World::new();
    let owner = world.spawn_empty().id();
    let mut children = ModelChildren::default();
    synchronize(
        &mut world,
        &mut children,
        owner,
        [projection(4, 0., 1.), projection(9, 0., 2.)],
    );
    let first = children.slots[&ParticleKey {
        epoch: 0,
        ordinal: 4,
    }]
        .entity;
    let second = children.slots[&ParticleKey {
        epoch: 0,
        ordinal: 9,
    }]
        .entity;
    synchronize(&mut world, &mut children, owner, [projection(9, 0.5, 7.)]);
    assert!(world.get_entity(first).is_err());
    assert_eq!(children.slots.len(), 1);
    assert_eq!(
        children.slots[&ParticleKey {
            epoch: 0,
            ordinal: 9
        }]
            .entity,
        second
    );
    assert_eq!(world.get::<Transform>(second).unwrap().translation.x, 7.);
    assert_eq!(world.get::<Playback>(second).unwrap().shared_time, 0.5);
    synchronize(&mut world, &mut children, owner, [projection(9, 0.5, 7.)]);
    assert_eq!(
        children.slots[&ParticleKey {
            epoch: 0,
            ordinal: 9
        }]
            .entity,
        second
    );
}
#[test]
fn reset_retains_no_old_request_and_reused_ordinal_is_a_new_birth() {
    let mut world = World::new();
    let owner = world.spawn_empty().id();
    let mut children = ModelChildren::default();
    synchronize(&mut world, &mut children, owner, [projection(0, 0., 1.)]);
    let old = children.slots[&ParticleKey {
        epoch: 0,
        ordinal: 0,
    }]
        .entity;
    let mut state = SystemState::<Commands>::new(&mut world);
    children.reset(&mut state.get_mut(&mut world).unwrap());
    state.apply(&mut world);
    assert!(world.get_entity(old).is_err());
    synchronize(&mut world, &mut children, owner, [projection(0, 0., 2.)]);
    assert_ne!(
        children.slots[&ParticleKey {
            epoch: 1,
            ordinal: 0
        }]
            .entity,
        old
    );
}

#[test]
fn temporarily_invalid_projection_preserves_actor_clock_and_last_finite_transform() {
    use crate::effects::model_children::synchronize_child;
    let mut world = World::new();
    let owner = world.spawn(Playback::default()).id();
    let mut children = ModelChildren::default();
    synchronize(&mut world, &mut children, owner, [projection(3, 0., 4.)]);
    let entity = children.slots[&ParticleKey {
        epoch: 0,
        ordinal: 3,
    }]
        .entity;
    let mut hidden = projection(3, 0.5, 99.);
    hidden.transform = None;
    synchronize(&mut world, &mut children, owner, [hidden]);
    world.get_mut::<Playback>(owner).unwrap().shared_time = 0.5;
    assert!(synchronize_child(&mut world, entity));
    assert_eq!(world.get::<Transform>(entity).unwrap().translation.x, 4.);
    assert_eq!(
        *world.get::<Visibility>(entity).unwrap(),
        Visibility::Hidden
    );
    assert_eq!(world.get::<Playback>(entity).unwrap().shared_time, 0.5);
    synchronize(&mut world, &mut children, owner, [projection(3, 0.8, 9.)]);
    assert_eq!(
        children.slots[&ParticleKey {
            epoch: 0,
            ordinal: 3
        }]
            .entity,
        entity
    );
    assert_eq!(world.get::<Transform>(entity).unwrap().translation.x, 9.);
    assert_eq!(
        *world.get::<Visibility>(entity).unwrap(),
        Visibility::Inherited
    );
    assert_eq!(world.get::<Playback>(entity).unwrap().shared_time, 0.8);
}
#[test]
fn first_invalid_projection_uses_hidden_finite_birth_frame() {
    let mut world = World::new();
    let owner = world.spawn_empty().id();
    let mut children = ModelChildren::default();
    let mut sample = projection(1, 0.2, 99.);
    sample.transform = None;
    let initial_transform = sample.initial_transform;
    synchronize(&mut world, &mut children, owner, [sample]);
    let entity = children.slots[&ParticleKey {
        epoch: 0,
        ordinal: 1,
    }]
        .entity;
    assert_eq!(*world.get::<Transform>(entity).unwrap(), initial_transform);
    assert_eq!(
        *world.get::<Visibility>(entity).unwrap(),
        Visibility::Hidden
    );
    assert!(!world.get::<ModelParticleChild>(entity).unwrap().visible);
}
