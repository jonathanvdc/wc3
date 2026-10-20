//! Render-layer inheritance limited to entities owned by a model instance.
use crate::effects::model_particles::ModelParticleChild;
use crate::ModelRig;
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::change_detection::Tick;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashSet;

/// A render entity whose default camera layers are selected by its model root.
#[derive(Component)]
pub(crate) struct ModelOwnedRenderEntity;

/// The exact write performed by this system, distinguishing application overrides.
#[derive(Component)]
struct InheritedModelLayers(Tick);

type LayerChange = (
    Entity,
    Ref<'static, RenderLayers>,
    Option<&'static InheritedModelLayers>,
);
type RenderLayerState = (
    Entity,
    Option<Ref<'static, RenderLayers>>,
    Option<&'static InheritedModelLayers>,
);

#[derive(SystemParam)]
pub(crate) struct LayerInputs<'w, 's> {
    roots: Query<'w, 's, Option<&'static RenderLayers>, With<ModelRig>>,
    new_roots: Query<'w, 's, Entity, Added<ModelRig>>,
    changed_layers: Query<'w, 's, LayerChange, Changed<RenderLayers>>,
    new_render_entities: Query<'w, 's, Entity, Added<ModelOwnedRenderEntity>>,
    render_entities: Query<'w, 's, RenderLayerState, With<ModelOwnedRenderEntity>>,
    parents: Query<'w, 's, &'static ChildOf>,
    particle_owners: Query<'w, 's, &'static ModelParticleChild>,
    changed_parents: Query<'w, 's, Entity, Changed<ChildOf>>,
    removed_layers: RemovedComponents<'w, 's, RenderLayers>,
    removed_parents: RemovedComponents<'w, 's, ChildOf>,
    removed_roots: RemovedComponents<'w, 's, ModelRig>,
    #[cfg(test)]
    work: Option<ResMut<'w, LayerWork>>,
}

impl LayerInputs<'_, '_> {
    fn owner(&self, entity: Entity) -> Option<Entity> {
        self.particle_owners
            .get(entity)
            .map(ModelParticleChild::parent_model)
            .ok()
            .or_else(|| self.parents.get(entity).ok().map(ChildOf::parent))
    }
}

/// Reusable event scratch; ordinary frames perform no topology traversal.
#[derive(Default)]
pub(crate) struct LayerTopology {
    model_ancestors: HashSet<Entity>,
}

#[cfg(test)]
#[derive(Resource, Default)]
struct LayerWork {
    render_entities: usize,
}

/// Reconcile only when model ownership, hierarchy, or application layer choices change.
pub(crate) fn synchronize_model_layers(
    mut commands: Commands,
    mut inputs: LayerInputs<'_, '_>,
    mut topology: Local<LayerTopology>,
) {
    let roots = &inputs.roots;
    let render_entities = &inputs.render_entities;
    let mut removed = false;
    for entity in inputs.removed_layers.read() {
        removed |= roots.contains(entity) || render_entities.contains(entity);
    }
    let affects_layers = |entity: Entity| {
        topology.model_ancestors.contains(&entity)
            || roots.contains(entity)
            || render_entities.contains(entity)
    };
    let mut hierarchy_changed = inputs.changed_parents.iter().any(affects_layers);
    for entity in inputs.removed_parents.read() {
        hierarchy_changed |= affects_layers(entity);
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
        && !hierarchy_changed
    {
        return;
    }
    // Cache only the owner paths that contributed inherited selections. Every
    // way to change one of these paths changes a cached entity's parent or adds
    // a new root/render entity, so unrelated hierarchy churn needs no traversal.
    topology.model_ancestors.clear();
    let hierarchy_limit = inputs.parents.iter().len() + inputs.particle_owners.iter().len() + 1;
    for (entity, current, inherited) in &inputs.render_entities {
        topology.model_ancestors.insert(entity);
        #[cfg(test)]
        if let Some(work) = &mut inputs.work {
            work.render_entities += 1;
        }
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
            let Some(owner) = inputs.owner(ancestor) else {
                break;
            };
            ancestor = owner;
            topology.model_ancestors.insert(ancestor);
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

#[cfg(test)]
#[path = "layers_tests.rs"]
mod tests;
