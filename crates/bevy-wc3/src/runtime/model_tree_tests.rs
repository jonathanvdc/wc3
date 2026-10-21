use super::*;
use crate::compile_model;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use wc3::model::{profiles::Universal, Model, ModelMetadata};

fn prepared() -> crate::PreparedModel {
    let model = Model::<Universal>::empty(ModelMetadata::Mdlx(Default::default())).unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    compiled.prepare(
        &mut Assets::<Mesh>::default(),
        &mut Assets::<SkinnedMeshInverseBindposes>::default(),
    )
}
#[test]
fn pending_readiness_rotates_before_new_birth_admission_and_uses_real_frames() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));
    let root = world.spawn_empty().id();
    let mut limits = ModelParticleLimits {
        actors_per_tree: 4,
        birth_work_per_frame: 4,
        ..Default::default()
    };
    let budget = ActorBudget::new(root, limits);
    let a = world
        .spawn((
            PreparedModelRequest(prepared()),
            budget.reserve(FrameCount(0)).unwrap(),
        ))
        .id();
    let b = world
        .spawn((
            PreparedModelRequest(prepared()),
            budget.reserve(FrameCount(0)).unwrap(),
        ))
        .id();
    limits.birth_work_per_frame = 1;
    world.entity_mut(root).insert((
        limits,
        ModelActorTree {
            budget: budget.clone(),
            pending: [a, b].into(),
        },
    ));
    world.resource_mut::<FrameCount>().0 = 1;
    prepare_tree(&mut world, root);
    assert_eq!(
        world.get::<ModelActorTree>(root).unwrap().pending.front(),
        Some(&b)
    );
    assert_eq!(budget.stats(FrameCount(1)).birth_work, 1);
    assert!(
        budget.reserve(FrameCount(1)).is_err(),
        "ready retries precede fresh source work"
    );
    world.resource_mut::<FrameCount>().0 = 2;
    prepare_tree(&mut world, root);
    assert_eq!(
        world.get::<ModelActorTree>(root).unwrap().pending.front(),
        Some(&a)
    );
    assert!(
        world
            .get_mut::<ModelActorLease>(b)
            .unwrap()
            .admit_ready(FrameCount(2)),
        "factory shares its already-reserved readiness work"
    );
    assert_eq!(budget.stats(FrameCount(2)).birth_work, 1);
}
#[test]
fn stale_tree_generation_and_terminal_requests_release_ownership() {
    let mut world = World::new();
    let root = world.spawn_empty().id();
    let budget = ActorBudget::new(root, ModelParticleLimits::default());
    let request = world
        .spawn((
            budget.reserve(FrameCount(0)).unwrap(),
            ModelInstanceError::CyclicDependency {
                model: Handle::<ModelAsset>::default().id(),
                ancestor: root,
            },
        ))
        .id();
    world.entity_mut(root).insert(ModelActorTree {
        budget: budget.clone(),
        pending: [request].into(),
    });
    prepare_tree(&mut world, root);
    assert!(world.get_entity(request).is_err());
    assert_eq!(budget.stats(FrameCount(0)).actors, 0);
    let old = world.spawn(budget.reserve(FrameCount(0)).unwrap()).id();
    let replacement = ActorBudget::new(root, ModelParticleLimits::default());
    world.entity_mut(root).insert(ModelActorTree {
        budget: replacement,
        pending: Default::default(),
    });
    assert!(
        actor_frame(&mut world, old, true).is_none(),
        "an old lease cannot attach to a new generation at the same root entity"
    );
    world.despawn(old);
    assert_eq!(budget.stats(FrameCount(0)).actors, 0);
}
