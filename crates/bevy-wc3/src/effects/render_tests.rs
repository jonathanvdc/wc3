use super::*;
use bevy::render::view::visibility::RenderVisibleEntitiesClass;
use std::any::TypeId;

#[test]
fn effect_selection_obeys_each_views_cpu_and_gpu_mesh_visibility() {
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
    left.classes.insert(TypeId::of::<Mesh3d>(), cpu);
    let mut gpu = RenderVisibleEntitiesClass::default();
    gpu.entities_gpu_culling.insert(second, render_second);
    right.classes.insert(TypeId::of::<Mesh3d>(), gpu);

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
    visible.classes.insert(TypeId::of::<Mesh3d>(), meshes);
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
