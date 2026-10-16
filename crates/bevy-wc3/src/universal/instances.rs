//! Asset-backed instance requests and shared GPU preparation.
use super::bounds::transform_bounds;
use super::effects::model_particles::ModelParticleChild;
use super::events::EventClocks;
use super::placements::ChildModelPlacement;
use super::runtime::populate_universal_model;
use super::{
    PreparedUniversalModel, UniversalControllerSelection, UniversalInstance, UniversalMaterial,
    UniversalModelAsset, UniversalPlayback, UniversalPoseOverrides,
};
use bevy::asset::{
    AssetEvent, AssetId, AssetLoadError, DependencyLoadState, LoadState,
    RecursiveDependencyLoadState,
};
use bevy::ecs::system::SystemParam;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use wc3::model::AnimationSample;

/// Load and populate a Universal model at an existing application entity.
///
/// Add this component with a typed model handle. Once ready, the plugin replaces
/// the request with [`super::UniversalInstance`] and [`UniversalModelSource`].
/// Existing root playback and pose overrides are retained. Replacing a request
/// despawns only the old renderer-owned hierarchy, preserving application children.
#[derive(Component, Clone)]
#[require(Transform, Visibility)]
pub struct UniversalModelInstance(pub Handle<UniversalModelAsset>);

impl UniversalModelInstance {
    /// Request a model asset on an existing or newly spawned application root.
    pub fn new(model: Handle<UniversalModelAsset>) -> Self {
        Self(model)
    }
}

/// Source identity retained after an asset-backed instance has been populated.
#[derive(Component, Clone, Debug)]
pub struct UniversalModelSource(pub Handle<UniversalModelAsset>);

/// Most recent asset, dependency, or ownership failure for a model request.
/// Asset failures retain the request for reload/retry; success clears this component.
#[derive(Component, Clone, Debug)]
pub enum UniversalModelInstanceError {
    /// The requested model itself could not be read or decoded.
    FailedAsset {
        /// Model whose load failed.
        model: AssetId<UniversalModelAsset>,
        /// Bevy's structured failure, shared with its asset load state.
        error: Arc<AssetLoadError>,
    },
    /// A required recursive dependency failed after the model was decoded.
    FailedDependency {
        /// Root model waiting for this dependency tree.
        model: AssetId<UniversalModelAsset>,
        /// Bevy's structured dependency failure.
        error: Arc<AssetLoadError>,
    },

    /// The same model already occurs in this attachment or particle owner chain.
    CyclicDependency {
        /// Model requested by the rejected child.
        model: AssetId<UniversalModelAsset>,
        /// Existing ancestor that owns the model.
        ancestor: Entity,
    },
}

impl UniversalModelInstanceError {
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
    model: AssetId<UniversalModelAsset>,
    states: (LoadState, DependencyLoadState, RecursiveDependencyLoadState),
) -> Option<UniversalModelInstanceError> {
    match states {
        (LoadState::Failed(error), _, _) => {
            Some(UniversalModelInstanceError::FailedAsset { model, error })
        }
        (_, DependencyLoadState::Failed(error), _)
        | (_, _, RecursiveDependencyLoadState::Failed(error)) => {
            Some(UniversalModelInstanceError::FailedDependency { model, error })
        }
        _ => None,
    }
}

/// State migrated when a model definition replaces a populated instance.
/// Names must be nonempty and unique in both definitions to preserve identity.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct UniversalModelRebind {
    /// Selected animation migrated to the same uniquely named clip.
    pub clip_rebound: bool,
    /// Missing or ambiguous animation fell back to the first available clip.
    pub clip_reset: bool,
    /// Controller selections removed because their scoped identities are absent.
    pub controllers_reset: usize,
    /// Pose overrides migrated to uniquely named replacement nodes.
    pub poses_rebound: usize,
    /// Pose overrides removed because no unambiguous target remained.
    pub poses_reset: usize,
}
impl UniversalModelRebind {
    fn changed(self) -> bool {
        self.clip_rebound
            || self.clip_reset
            || self.controllers_reset > 0
            || self.poses_rebound > 0
            || self.poses_reset > 0
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
    previous: Option<&PreparedUniversalModel>,
    next: &PreparedUniversalModel,
    playback: &mut UniversalPlayback,
    overrides: Option<&UniversalPoseOverrides>,
) -> (UniversalModelRebind, Option<UniversalPoseOverrides>) {
    let mut report = UniversalModelRebind::default();
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
    let mut seen = HashSet::new();
    if let UniversalControllerSelection::Explicit(controllers) = &mut playback.controllers {
        controllers.retain(|id| {
            let valid = seen.insert(*id)
                && next
                    .animation
                    .sample(AnimationSample {
                        controllers: &[*id],
                        ..Default::default()
                    })
                    .is_ok();
            report.controllers_reset += usize::from(!valid);
            valid
        });
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
        UniversalPoseOverrides(poses)
    });
    if previous.is_some() {
        // Recreating the owned hierarchy must not replay historical events/births.
        playback.event_traversal.revision = playback.event_traversal.revision.wrapping_add(1);
        playback.event_traversal.origin = EventClocks {
            clip: playback.clip_time,
            timeline: playback.timeline_time,
            shared: playback.shared_time,
        };
        playback.event_traversal.include_origin = false;
    }
    (report, overrides)
}

#[derive(Resource, Default)]
pub(crate) struct UniversalPreparedCache(
    HashMap<AssetId<UniversalModelAsset>, PreparedUniversalModel>,
);

pub(crate) fn invalidate_universal_models(
    mut events: MessageReader<AssetEvent<UniversalModelAsset>>,
    mut cache: ResMut<UniversalPreparedCache>,
    mut commands: Commands,
    sources: Query<(Entity, &UniversalModelSource), Without<UniversalModelInstance>>,
) {
    for event in events.read() {
        if let AssetEvent::Added { id } | AssetEvent::Modified { id } | AssetEvent::Removed { id } =
            event
        {
            cache.0.remove(id);
            if matches!(event, AssetEvent::Modified { .. }) {
                for (root, source) in &sources {
                    if source.0.id() == *id {
                        commands
                            .entity(root)
                            .insert(UniversalModelInstance(source.0.clone()));
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
        Option<&'static UniversalModelSource>,
        Option<&'static UniversalModelInstance>,
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
    model: AssetId<UniversalModelAsset>,
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
        &'static UniversalModelInstance,
        Option<&'static UniversalPlayback>,
        Option<&'static UniversalInstance>,
        Option<&'static ChildModelPlacement>,
        Option<&'static UniversalPoseOverrides>,
        Option<&'static UniversalModelInstanceError>,
    ),
>;

#[derive(SystemParam)]
pub(crate) struct LoaderResources<'w> {
    server: Option<Res<'w, AssetServer>>,
    assets: Res<'w, Assets<UniversalModelAsset>>,
    meshes: ResMut<'w, Assets<Mesh>>,
    binds: ResMut<'w, Assets<SkinnedMeshInverseBindposes>>,
    materials: ResMut<'w, Assets<UniversalMaterial>>,
    cache: ResMut<'w, UniversalPreparedCache>,
}

pub(crate) fn spawn_loaded_universal_models(
    mut commands: Commands,
    requests: ModelRequests<'_, '_>,
    hierarchy: ModelHierarchy<'_, '_>,
    resources: LoaderResources<'_>,
) {
    let LoaderResources {
        server,
        assets,
        mut meshes,
        mut binds,
        mut materials,
        mut cache,
    } = resources;
    for (root, request, playback, previous, placement, overrides, failure) in &requests {
        // Ancestor replacement may despawn this renderer-owned request. Defer
        // surviving application children until that replacement has completed.
        if pending_ancestor(root, &hierarchy) {
            continue;
        }
        if let Some(ancestor) = cyclic_ancestor(root, request.0.id(), &hierarchy) {
            commands
                .entity(root)
                .remove::<UniversalModelInstance>()
                .insert(UniversalModelInstanceError::CyclicDependency {
                    model: request.0.id(),
                    ancestor,
                });
            continue;
        }
        // Asset-backed requests wait for their recursive defaults. Assets inserted
        // directly by applications have no server load state and are already usable.
        if let Some(server) = &server {
            if let Some(states) = server.get_load_states(request.0.id()) {
                if let Some(next) = load_failure(request.0.id(), states) {
                    if failure.is_none_or(|failure| !failure.same_failure(&next)) {
                        commands.entity(root).insert(next);
                    }
                    continue;
                }
                if !server.is_loaded_with_dependencies(request.0.id()) {
                    continue;
                }
            }
        }
        let Some(source) = assets.get(&request.0) else {
            continue;
        };
        let prepared = cache
            .0
            .entry(request.0.id())
            .or_insert_with(|| source.compiled.prepare(&mut meshes, &mut binds));
        let mut rebound_playback = playback.cloned().unwrap_or_default();
        if placement.is_some() && playback.is_none() && previous.is_none() {
            if let Some((clip, _)) = prepared.clips.first() {
                rebound_playback.play_immediately(*clip);
            }
        }
        let (report, rebound_overrides) = rebind_state(
            previous.map(UniversalInstance::model),
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
            previous.despawn_owned_subtree(&mut commands);
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
            populate_universal_model(&mut commands, &mut meshes, &mut materials, &mounted, root);
        } else {
            populate_universal_model(&mut commands, &mut meshes, &mut materials, prepared, root);
        }
        if playback.is_some() || previous.is_some() || placement.is_some() {
            commands.entity(root).insert(rebound_playback);
        }
        commands
            .entity(root)
            .remove::<UniversalModelInstance>()
            .remove::<UniversalModelInstanceError>()
            .insert(UniversalModelSource(request.0.clone()));
    }
}

#[cfg(test)]
#[path = "instances_tests.rs"]
mod tests;
