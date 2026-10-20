//! Render-layer inheritance limited to entities owned by a model instance.
use crate::effects::model_particles::ModelParticleChild;
use crate::ModelRig;
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::change_detection::Tick;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

/// A render entity whose default camera layers are selected by its model root.
#[derive(Component)]
pub(crate) struct ModelOwnedRenderEntity;

/// The exact write performed by this system, distinguishing application overrides.
#[derive(Component)]
struct InheritedModelLayers(Tick);

#[derive(SystemParam)]
pub(crate) struct LayerInputs<'w, 's> {
    roots: Query<'w, 's, Option<&'static RenderLayers>, With<ModelRig>>,
    new_roots: Query<'w, 's, Entity, Added<ModelRig>>,
    changed_layers: Query<
        'w,
        's,
        (
            Entity,
            Ref<'static, RenderLayers>,
            Option<&'static InheritedModelLayers>,
        ),
        Changed<RenderLayers>,
    >,
    new_render_entities: Query<'w, 's, Entity, Added<ModelOwnedRenderEntity>>,
    render_entities: Query<
        'w,
        's,
        (
            Entity,
            Option<Ref<'static, RenderLayers>>,
            Option<&'static InheritedModelLayers>,
        ),
        With<ModelOwnedRenderEntity>,
    >,
    parents: Query<'w, 's, &'static ChildOf>,
    particle_owners: Query<'w, 's, &'static ModelParticleChild>,
    changed_parents: Query<'w, 's, Entity, Changed<ChildOf>>,
    removed_layers: RemovedComponents<'w, 's, RenderLayers>,
    removed_parents: RemovedComponents<'w, 's, ChildOf>,
    removed_roots: RemovedComponents<'w, 's, ModelRig>,
}

/// Reconcile only when model ownership, hierarchy, or application layer choices change.
pub(crate) fn synchronize_model_layers(mut commands: Commands, mut inputs: LayerInputs<'_, '_>) {
    let roots = &inputs.roots;
    let render_entities = &inputs.render_entities;
    let mut removed = false;
    for entity in inputs.removed_layers.read() {
        removed |= roots.contains(entity) || render_entities.contains(entity);
    }
    for _ in inputs.removed_parents.read() {
        removed = true;
    }
    for _ in inputs.removed_roots.read() {
        removed = true;
    }
    let changed = inputs
        .changed_layers
        .iter()
        .any(|(entity, layers, inherited)| {
            (roots.contains(entity) || render_entities.contains(entity))
                && inherited.is_none_or(|inherited| inherited.0 != layers.last_changed())
        });
    if !removed
        && !changed
        && inputs.new_roots.is_empty()
        && inputs.new_render_entities.is_empty()
        && inputs.changed_parents.is_empty()
    {
        return;
    }
    // The parent count bounds malformed cycles without per-entity allocations.
    let hierarchy_limit = inputs.parents.iter().len() + inputs.particle_owners.iter().len() + 1;
    for (entity, current, inherited) in &inputs.render_entities {
        if let Some(current) = &current {
            if inherited.is_none_or(|inherited| inherited.0 != current.last_changed()) {
                // An explicit application write belongs to this render entity, including
                // a write of the same value that the model previously supplied.
                continue;
            }
        }
        let mut desired = RenderLayers::default();
        let mut ancestor = entity;
        for _ in 0..hierarchy_limit {
            ancestor = if let Ok(particle) = inputs.particle_owners.get(ancestor) {
                particle.parent_model()
            } else if let Ok(parent) = inputs.parents.get(ancestor) {
                parent.parent()
            } else {
                break;
            };
            if let Ok(Some(layers)) = inputs.roots.get(ancestor) {
                desired = layers.clone();
                break;
            }
        }
        if current.as_ref().is_some_and(|current| **current == desired)
            || (current.is_none() && desired == RenderLayers::default())
        {
            continue;
        }
        commands.queue(move |world: &mut World| {
            let tick = world.change_tick();
            if let Ok(mut entity) = world.get_entity_mut(entity) {
                entity.insert((desired, InheritedModelLayers(tick)));
            }
        });
    }
}
