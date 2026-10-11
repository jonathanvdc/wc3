//! Spawn file-backed model instances once their source assets are ready.
mod cache;
mod ownership;
pub(crate) mod spawn;
pub(crate) use cache::{invalidate_prepared_models, PreparedModelCache};
use ownership::has_model_cycle;
pub(crate) use ownership::BlockedChildModel;
pub use ownership::{Wc3ModelOwner, Wc3OwnedModels};

use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;

use crate::animation::Wc3Animation;
use crate::assets::loader::Wc3ModelAsset;
use crate::instance::spawn::spawn_prepared_into;
use crate::materials::textures::Wc3TextureBindings;
use crate::materials::Wc3LayerMaterial;

/// Attach to an entity to spawn an MDX or MDL asset beneath it when loading finishes.
/// The entity remains the transform and animation root. [`Wc3Animation`] appears
/// after successful preparation and spawning; it is absent while loading or after
/// a preparation failure. The request supplies required transform and visibility
/// components, so callers can spawn it on its own.
///
/// Changing the handle on an already populated root does not rebuild the instance.
/// Spawn a new root when replacing a model. Source asset changes invalidate cached
/// preparation for future instances; existing instances retain their render assets.
#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Wc3ModelInstance(
    /// Model asset to instantiate beneath this root.
    pub Handle<Wc3ModelAsset>,
);

impl Wc3ModelInstance {
    /// Create an instance request for an asset loaded through Bevy.
    pub fn new(handle: Handle<Wc3ModelAsset>) -> Self {
        Self(handle)
    }
}

type UnloadedInstances<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Wc3ModelInstance,
        Option<&'static Wc3TextureBindings>,
        Option<&'static Wc3ModelOwner>,
    ),
    (Without<Wc3Animation>, Without<BlockedChildModel>),
>;

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_loaded_instances(
    mut commands: Commands,
    instances: UnloadedInstances,
    owners: Query<(Option<&Wc3ModelInstance>, Option<&Wc3ModelOwner>)>,
    sources: Res<Assets<Wc3ModelAsset>>,
    mut cache: ResMut<PreparedModelCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<Wc3LayerMaterial>>,
    mut inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    cache.retain_live(&sources);
    for (root, instance, bindings, owner) in &instances {
        let id = instance.0.id();
        if owner.is_some()
            && has_model_cycle(root, id, |entity| {
                let (model, owner) = owners.get(entity).ok()?;
                Some((model.map(|model| model.0.id()), owner.map(|owner| owner.0)))
            })
        {
            warn!("Skipping cyclic WC3 child model {:?}", instance.0);
            commands
                .entity(root)
                .insert((BlockedChildModel, Visibility::Hidden));
            continue;
        }
        let Some(asset) = sources.get(id) else {
            continue;
        };
        let prepared = match cache.get_or_prepare(id, asset, &mut meshes, &mut inverse_bindposes) {
            Ok(Some(prepared)) => prepared,
            Ok(None) => continue,
            Err(error) => {
                warn!("Could not prepare WC3 model {:?}: {error}", instance.0);
                continue;
            }
        };
        spawn_prepared_into(
            &mut commands,
            &mut meshes,
            &mut materials,
            prepared,
            root,
            bindings.cloned().unwrap_or_default(),
        );
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
