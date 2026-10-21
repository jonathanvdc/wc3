use super::*;
use bevy::render::view::visibility::RenderVisibleEntitiesClass;
use std::any::TypeId;

#[test]
fn effect_selection_obeys_each_views_procedural_visibility() {
    let mut world = World::new();
    let first: MainEntity = world.spawn_empty().id().into();
    let second: MainEntity = world.spawn_empty().id().into();
    let hidden: MainEntity = world.spawn_empty().id().into();
    let render_first = world.spawn_empty().id();
    let render_second = world.spawn_empty().id();
    // Bevy's per-view selections already account for disjoint camera layers.
    // Exercise both its CPU and GPU visibility stores: custom effect draws must
    // obey the same selection irrespective of the preprocessing path.
    let mut left = RenderVisibleEntities::default();
    let mut right = RenderVisibleEntities::default();
    let mut cpu = RenderVisibleEntitiesClass::default();
    cpu.update_cpu_culled_entities(&[(render_first, first)]);
    left.classes.insert(TypeId::of::<EffectGeometry>(), cpu);
    let mut gpu = RenderVisibleEntitiesClass::default();
    gpu.entities_gpu_culling.insert(second, render_second);
    right.classes.insert(TypeId::of::<EffectGeometry>(), gpu);

    assert!(effect_visible(&left, first));
    assert!(!effect_visible(&left, second));
    assert!(!effect_visible(&left, hidden));
    assert!(!effect_visible(&right, first));
    assert!(effect_visible(&right, second));
    assert!(!effect_visible(&right, hidden));
    assert!(!effect_visible(&RenderVisibleEntities::default(), first));
}

#[test]
fn cpu_visibility_membership_uses_main_world_identity() {
    let mut world = World::new();
    let mut entities: Vec<MainEntity> = (0..5).map(|_| world.spawn_empty().id().into()).collect();
    entities.sort();
    let mut meshes = RenderVisibleEntitiesClass::default();
    meshes.update_cpu_culled_entities(&[
        (world.spawn_empty().id(), entities[0]),
        (world.spawn_empty().id(), entities[2]),
        (world.spawn_empty().id(), entities[4]),
    ]);
    let mut visible = RenderVisibleEntities::default();
    visible
        .classes
        .insert(TypeId::of::<EffectGeometry>(), meshes);
    for (index, entity) in entities.into_iter().enumerate() {
        assert_eq!(effect_visible(&visible, entity), index % 2 == 0);
    }
}

#[test]
fn effect_binding_invalidates_for_reloaded_images_and_reallocated_storage() {
    let binding = EffectBinding {
        layout: BindGroupLayoutId::new(),
        image: (TextureViewId::new(), SamplerId::new()),
        extra_images: Some([
            (TextureViewId::new(), SamplerId::new()),
            (TextureViewId::new(), SamplerId::new()),
        ]),
        uniform: BufferId::new(),
        records: BufferId::new(),
    };
    assert_eq!(binding, binding);
    assert_ne!(
        binding,
        EffectBinding {
            layout: BindGroupLayoutId::new(),
            ..binding
        }
    );
    assert_ne!(
        binding,
        EffectBinding {
            image: (TextureViewId::new(), binding.image.1),
            ..binding
        }
    );
    assert_ne!(
        binding,
        EffectBinding {
            image: (binding.image.0, SamplerId::new()),
            ..binding
        }
    );
    assert_ne!(
        binding,
        EffectBinding {
            records: BufferId::new(),
            ..binding
        }
    );
    assert_ne!(
        binding,
        EffectBinding {
            uniform: BufferId::new(),
            ..binding
        }
    );
    for index in 0..2 {
        let mut changed = binding;
        changed.extra_images.as_mut().unwrap()[index].0 = TextureViewId::new();
        assert_ne!(binding, changed);
        let mut changed = binding;
        changed.extra_images.as_mut().unwrap()[index].1 = SamplerId::new();
        assert_ne!(binding, changed);
    }
    assert_ne!(
        binding,
        EffectBinding {
            extra_images: None,
            ..binding
        }
    );
}

#[test]
fn empty_or_hidden_frames_suspend_resident_draws_without_discarding_source_identity() {
    let source = Arc::new(vec![7u32, 4, 2]);
    let weak = Arc::downgrade(&source);
    let mut draw = DrawState::new(&source);
    assert_eq!(draw.instances, 3);
    drop(source);
    draw.suspend();
    assert_eq!(
        draw.instances, 0,
        "resident GPU records must not imply a live draw"
    );
    let retained = weak
        .upgrade()
        .expect("source identity must survive suspension");
    assert!(Arc::ptr_eq(&retained, &draw.source));
    let resumed = DrawState::new(&retained);
    assert_eq!(resumed.instances, 3);
    let empty = DrawState::new(&Arc::new(Vec::<u32>::new()));
    assert_eq!(empty.instances, 0);
}

#[test]
fn order_revisions_detect_motion_population_and_uniform_changes_without_scanning_records() {
    let first = Arc::new(vec![1u32, 2]);
    let same = RecordSource::Sampled(first.clone());
    let source = RecordSource::Sampled(first);
    let different = RecordSource::Sampled(Arc::new(vec![1u32, 2]));
    let indices = Arc::new(vec![0u32, 1]);
    let replaced = Arc::new(vec![0u32, 1]);
    let uniform: Arc<[u8]> = Arc::from([0u8; 16]);
    let changed_uniform: Arc<[u8]> = Arc::from([1u8; 16]);
    let revision = OrderRevision {
        records: &source,
        indices: &indices,
        uniform: &uniform,
    };
    assert!(revision.same(&OrderRevision {
        records: &same,
        indices: &indices,
        uniform: &uniform
    }));
    assert!(!revision.same(&OrderRevision {
        records: &different,
        indices: &indices,
        uniform: &uniform
    }));
    assert!(!revision.same(&OrderRevision {
        records: &source,
        indices: &replaced,
        uniform: &uniform
    }));
    assert!(!revision.same(&OrderRevision {
        records: &source,
        indices: &indices,
        uniform: &changed_uniform
    }));
}

#[test]
fn reset_ring_lineages_cannot_reuse_a_repeated_cursor_as_the_same_source() {
    let first: RecordRing<u32> = vec![1, 2].into();
    let second: RecordRing<u32> = vec![3, 4].into();
    assert_eq!(first.cursor(), second.cursor());
    let source = |ring: &RecordRing<u32>| RecordSource::Ring {
        cursor: ring.cursor(),
        lineage: ring.lineage().clone(),
    };
    assert!(source(&first).same(&source(&first.clone())));
    assert!(!source(&first).same(&source(&second)));
}
