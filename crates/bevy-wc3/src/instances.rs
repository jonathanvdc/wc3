//! Asset-backed instance requests and shared GPU preparation.
use super::bounds::transform_bounds;
use super::child_coordinates::ChildCoordinates;
use super::effects::model_children::{ModelParticleChild, ParticleModelSubtree};
use super::events::EventClocks;
use super::runtime::populate_model_with_context;
use super::{ModelAsset, ModelMaterial, ModelRig, Playback, PoseOverrides, PreparedModel};
use crate::image_replacements::{effective_replacements, ImageOwners, ReplacementImages};
use bevy::asset::{
    AssetEvent, AssetId, AssetLoadError, DependencyLoadState, LoadState,
    RecursiveDependencyLoadState,
};
use bevy::ecs::system::SystemParam;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Load and populate a Universal model at an existing application entity.
///
/// Add this component with a typed model handle. Once ready, the plugin replaces
/// the request with [`super::ModelRig`] and [`ModelSource`].
/// Existing root playback and pose overrides are retained. Replacing a request
/// despawns only the old renderer-owned hierarchy, preserving application children.
#[derive(Component, Clone)]
#[require(Transform, Visibility)]
pub struct ModelInstance(pub Handle<ModelAsset>);

impl ModelInstance {
    /// Request a model asset on an existing or newly spawned application root.
    pub fn new(model: Handle<ModelAsset>) -> Self {
        Self(model)
    }
}

/// Source identity retained after an asset-backed instance has been populated.
#[derive(Component, Clone, Debug)]
pub struct ModelSource(pub Handle<ModelAsset>);

/// Most recent asset, dependency, or ownership failure for a model request.
/// Asset failures retain the request for reload/retry; success clears this component.
#[derive(Component, Clone, Debug)]
pub enum ModelInstanceError {
    /// The requested model itself could not be read or decoded.
    FailedAsset {
        /// Model whose load failed.
        model: AssetId<ModelAsset>,
        /// Bevy's structured failure, shared with its asset load state.
        error: Arc<AssetLoadError>,
    },
    /// A required recursive dependency failed after the model was decoded.
    FailedDependency {
        /// Root model waiting for this dependency tree.
        model: AssetId<ModelAsset>,
        /// Bevy's structured dependency failure.
        error: Arc<AssetLoadError>,
    },

    /// The same model already occurs in this attachment or particle owner chain.
    CyclicDependency {
        /// Model requested by the rejected child.
        model: AssetId<ModelAsset>,
        /// Existing ancestor that owns the model.
        ancestor: Entity,
    },
}

impl ModelInstanceError {
    fn same_failure(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::FailedAsset { model: a, error: x },
                Self::FailedAsset { model: b, error: y },
            )
            | (
                Self::FailedDependency { model: a, error: x },
                Self::FailedDependency { model: b, error: y },
            ) => a == b && Arc::ptr_eq(x, y),
            _ => false,
        }
    }
}
fn load_failure(
    model: AssetId<ModelAsset>,
    states: (LoadState, DependencyLoadState, RecursiveDependencyLoadState),
) -> Option<ModelInstanceError> {
    match states {
        (LoadState::Failed(error), _, _) => Some(ModelInstanceError::FailedAsset { model, error }),
        (_, DependencyLoadState::Failed(error), _)
        | (_, _, RecursiveDependencyLoadState::Failed(error)) => {
            Some(ModelInstanceError::FailedDependency { model, error })
        }
        _ => None,
    }
}

/// State migrated when a model definition replaces a populated instance.
/// Names must be nonempty and unique in both definitions to preserve identity.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ModelRebind {
    /// Selected animation migrated to the same uniquely named clip.
    pub clip_rebound: bool,
    /// Missing or ambiguous animation fell back to the first available clip.
    pub clip_reset: bool,
    /// Pose overrides migrated to uniquely named replacement nodes.
    pub poses_rebound: usize,
    /// Pose overrides removed because no unambiguous target remained.
    pub poses_reset: usize,
}
impl ModelRebind {
    fn changed(self) -> bool {
        self.clip_rebound || self.clip_reset || self.poses_rebound > 0 || self.poses_reset > 0
    }
}
fn unique_names<'a, T: Copy>(
    items: impl Iterator<Item = (&'a str, T)>,
) -> HashMap<&'a str, Option<T>> {
    let mut names = HashMap::new();
    for (name, id) in items.filter(|(name, _)| !name.is_empty()) {
        names
            .entry(name)
            .and_modify(|entry| *entry = None)
            .or_insert(Some(id));
    }
    names
}
fn rebind_state(
    previous: Option<&PreparedModel>,
    next: &PreparedModel,
    playback: &mut Playback,
    overrides: Option<&PoseOverrides>,
) -> (ModelRebind, Option<PoseOverrides>) {
    let mut report = ModelRebind::default();
    if let Some(id) = playback
        .clip
        .filter(|id| !next.clips.iter().any(|(candidate, _)| candidate == id))
    {
        let replacement = previous.and_then(|previous| {
            let name = previous
                .clips
                .iter()
                .find(|(candidate, _)| *candidate == id)?
                .1
                .as_str();
            let old = unique_names(previous.clips.iter().map(|(id, name)| (name.as_str(), *id)));
            let new = unique_names(next.clips.iter().map(|(id, name)| (name.as_str(), *id)));
            old.get(name).copied().flatten()?;
            new.get(name).copied().flatten()
        });
        report.clip_rebound = replacement.is_some();
        report.clip_reset = replacement.is_none();
        playback.clip = replacement.or_else(|| next.clips.first().map(|(id, _)| *id));
    }
    let overrides = overrides.map(|overrides| {
        let existing: HashSet<_> = next.nodes.iter().map(|node| node.id).collect();
        let new_names = unique_names(next.nodes.iter().map(|node| (node.name.as_str(), node.id)));
        let old_names = previous.map(|previous| {
            unique_names(
                previous
                    .nodes
                    .iter()
                    .map(|node| (node.name.as_str(), node.id)),
            )
        });
        let old_ids: HashMap<_, _> = previous
            .into_iter()
            .flat_map(|previous| {
                previous
                    .nodes
                    .iter()
                    .map(|node| (node.id, node.name.as_str()))
            })
            .collect();
        let mut poses = HashMap::with_capacity(overrides.0.len());
        for (&id, &pose) in &overrides.0 {
            if existing.contains(&id) {
                poses.insert(id, pose);
                continue;
            }
            let replacement = old_ids.get(&id).and_then(|name| {
                old_names.as_ref()?.get(*name).copied().flatten()?;
                new_names.get(*name).copied().flatten()
            });
            if let Some(id) = replacement {
                poses.insert(id, pose);
                report.poses_rebound += 1;
            } else {
                report.poses_reset += 1;
            }
        }
        PoseOverrides(poses)
    });
    if previous.is_some() {
        // Recreating the owned hierarchy must not replay historical events/births.
        playback.event_traversal.revision = playback.event_traversal.revision.wrapping_add(1);
        playback.event_traversal.origin = EventClocks {
            clip: playback.clip_time,
            shared: playback.shared_time,
        };
        playback.event_traversal.include_origin = false;
    }
    (report, overrides)
}

#[derive(Resource, Default)]
pub(crate) struct PreparedModelCache(HashMap<AssetId<ModelAsset>, PreparedModel>);

pub(crate) fn invalidate_models(
    mut events: MessageReader<AssetEvent<ModelAsset>>,
    mut cache: ResMut<PreparedModelCache>,
    mut image_cache: ResMut<ReplacementImages>,
    mut commands: Commands,
    sources: Query<(Entity, &ModelSource), Without<ModelInstance>>,
) {
    for event in events.read() {
        if let AssetEvent::Added { id } | AssetEvent::Modified { id } | AssetEvent::Removed { id } =
            event
        {
            cache.0.remove(id);
            image_cache.invalidate(*id);
            if matches!(event, AssetEvent::Modified { .. }) {
                for (root, source) in &sources {
                    if source.0.id() == *id {
                        commands
                            .entity(root)
                            .insert(ModelInstance(source.0.clone()));
                    }
                }
            }
        }
    }
}

type ModelHierarchy<'world, 'state> = Query<
    'world,
    'state,
    (
        Option<&'static ChildOf>,
        Option<&'static ModelParticleChild>,
        Option<&'static ModelSource>,
        Option<&'static ModelInstance>,
    ),
>;

fn pending_ancestor(root: Entity, hierarchy: &ModelHierarchy<'_, '_>) -> bool {
    let mut current = hierarchy
        .get(root)
        .ok()
        .and_then(|(parent, particle, _, _)| {
            particle
                .map(ModelParticleChild::parent_model)
                .or_else(|| parent.map(ChildOf::parent))
        });
    let mut visited = HashSet::new();
    while let Some(entity) = current {
        if !visited.insert(entity) {
            return false;
        }
        let Ok((parent, particle, _, request)) = hierarchy.get(entity) else {
            return false;
        };
        if request.is_some() {
            return true;
        }
        current = particle
            .map(ModelParticleChild::parent_model)
            .or_else(|| parent.map(ChildOf::parent));
    }
    false
}

fn cyclic_ancestor(
    root: Entity,
    model: AssetId<ModelAsset>,
    hierarchy: &ModelHierarchy<'_, '_>,
) -> Option<Entity> {
    let owner = |entity| {
        hierarchy
            .get(entity)
            .ok()
            .and_then(|(parent, particle, _, _)| {
                particle
                    .map(ModelParticleChild::parent_model)
                    .or_else(|| parent.map(ChildOf::parent))
            })
    };
    let mut current = owner(root);
    let mut visited = HashSet::new();
    while let Some(entity) = current {
        if !visited.insert(entity) {
            return Some(entity);
        }
        if let Ok((_, _, source, request)) = hierarchy.get(entity) {
            if source.is_some_and(|source| source.0.id() == model)
                || request.is_some_and(|request| request.0.id() == model)
            {
                return Some(entity);
            }
        }
        current = owner(entity);
    }
    None
}

type ModelRequests<'world, 'state> = Query<
    'world,
    'state,
    (
        Entity,
        &'static ModelInstance,
        Option<&'static Playback>,
        Option<&'static ModelRig>,
        Option<&'static ChildCoordinates>,
        Option<&'static PoseOverrides>,
        Option<&'static ModelInstanceError>,
        Has<ParticleModelSubtree>,
        Option<&'static ModelParticleChild>,
    ),
>;

#[derive(SystemParam)]
pub(crate) struct LoaderResources<'w> {
    server: Option<Res<'w, AssetServer>>,
    assets: Res<'w, Assets<ModelAsset>>,
    meshes: ResMut<'w, Assets<Mesh>>,
    binds: ResMut<'w, Assets<SkinnedMeshInverseBindposes>>,
    materials: ResMut<'w, Assets<ModelMaterial>>,
    cache: ResMut<'w, PreparedModelCache>,
    image_cache: ResMut<'w, ReplacementImages>,
    images: ResMut<'w, Assets<Image>>,
}

/// The ordinary loader and same-frame particle factory share one checked population path.
#[derive(SystemParam)]
pub(crate) struct OwnedModelFactory<'w, 's> {
    commands: Commands<'w, 's>,
    requests: ModelRequests<'w, 's>,
    hierarchy: ModelHierarchy<'w, 's>,
    owners: ImageOwners<'w, 's>,
    resources: LoaderResources<'w>,
}
impl OwnedModelFactory<'_, '_> {
    /// Populate only this requested root; pending assets leave its request intact.
    pub(crate) fn populate(&mut self, root: Entity) {
        let Self {
            commands,
            requests,
            hierarchy,
            owners,
            resources,
        } = self;
        let Ok((_, request, playback, previous, placement, overrides, failure, owned, particle)) =
            requests.get(root)
        else {
            return;
        };
        let LoaderResources {
            server,
            assets,
            meshes,
            binds,
            materials,
            cache,
            image_cache,
            images,
        } = resources;
        // Ancestor replacement may despawn this renderer-owned request. Defer
        // surviving application children until that replacement has completed.
        if pending_ancestor(root, hierarchy) {
            return;
        }
        if let Some(ancestor) = cyclic_ancestor(root, request.0.id(), hierarchy) {
            commands.entity(root).remove::<ModelInstance>().insert(
                ModelInstanceError::CyclicDependency {
                    model: request.0.id(),
                    ancestor,
                },
            );
            return;
        }
        // Asset-backed requests wait for their recursive defaults. Assets inserted
        // directly by applications have no server load state and are already usable.
        if let Some(server) = &server {
            if let Some(states) = server.get_load_states(request.0.id()) {
                if let Some(next) = load_failure(request.0.id(), states) {
                    if failure.is_none_or(|failure| !failure.same_failure(&next)) {
                        commands.entity(root).insert(next);
                    }
                    return;
                }
                if !server.is_loaded_with_dependencies(request.0.id()) {
                    return;
                }
            }
        }
        let Some(source) = assets.get(&request.0) else {
            return;
        };
        let prepared = cache
            .0
            .entry(request.0.id())
            .or_insert_with(|| source.compiled.prepare(meshes, binds));
        let replacements = effective_replacements(root, owners);
        let Some((prepared, bound)) =
            image_cache.prepare(request.0.id(), prepared, replacements, images)
        else {
            return;
        };
        let prepared = &prepared;
        commands.entity(root).insert(bound);
        let mut rebound_playback = playback.cloned().unwrap_or_default();
        if placement.is_some() && playback.is_none() && previous.is_none() {
            if let Some((clip, _)) = prepared.clips.first() {
                rebound_playback.play_immediately(*clip);
            }
        }
        if let Some(particle) = particle.filter(|particle| particle.first_clip_pending) {
            if rebound_playback.clip.is_none() {
                rebound_playback.clip = prepared.clips.first().map(|(clip, _)| *clip);
            }
            commands.entity(root).insert(ModelParticleChild {
                first_clip_pending: false,
                ..*particle
            });
        }
        let (report, rebound_overrides) = rebind_state(
            previous.map(ModelRig::model),
            prepared,
            &mut rebound_playback,
            overrides,
        );
        if previous.is_some() || report.changed() {
            commands.entity(root).insert(report);
        }
        if let Some(overrides) = rebound_overrides {
            commands.entity(root).insert(overrides);
        }
        if let Some(previous) = previous {
            previous.despawn_owned_subtree(commands);
        }
        if let Some(placement) = placement {
            let mut mounted = prepared.clone();
            mounted.basis = placement.basis(prepared.basis);
            mounted.rest_bounds = prepared.rest_bounds.map(|bounds| {
                transform_bounds(
                    bounds,
                    mounted.basis.to_matrix() * prepared.basis.to_matrix().inverse(),
                )
            });
            populate_model_with_context(
                commands,
                meshes,
                materials,
                &mounted,
                root,
                owned,
                rebound_playback.clone(),
            );
        } else {
            populate_model_with_context(
                commands,
                meshes,
                materials,
                prepared,
                root,
                owned,
                rebound_playback.clone(),
            );
        }
        if playback.is_some() || previous.is_some() || placement.is_some() || owned {
            commands.entity(root).insert(rebound_playback);
        }
        commands
            .entity(root)
            .remove::<ModelInstance>()
            .remove::<ModelInstanceError>()
            .insert(ModelSource(request.0.clone()));
    }
}

pub(crate) fn spawn_loaded_models(
    mut factory: OwnedModelFactory<'_, '_>,
    mut pending: Local<Vec<Entity>>,
) {
    pending.clear();
    pending.extend(factory.requests.iter().map(|request| request.0));
    for &root in pending.iter() {
        factory.populate(root);
    }
}

#[cfg(test)]
#[path = "instances_tests.rs"]
mod tests;
