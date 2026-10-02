use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;

use crate::assets::model::{ModelError, Wc3Model};
use crate::attachment::spawn_attachments;
use crate::camera::Wc3ModelCameras;
use crate::effects::particle_emitter::spawn_particles;
use crate::effects::particle_emitter2::spawn_particles2;
use crate::effects::ribbon_emitter::spawn_ribbons;
use crate::light::spawn_lights;
use crate::materials::textures::Wc3TextureBindings;
use crate::materials::Wc3LayerMaterial;

mod geosets;
mod rig;

use crate::preparation::{prepare_model, PreparedModel};
use geosets::spawn_geosets;
pub use rig::Wc3NodeEntities;
use rig::{spawn_animation_root, spawn_rig};

/// Spawn an independently animated instance from prepared assets.
pub fn spawn_prepared_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<Wc3LayerMaterial>,
    prepared: &PreparedModel,
) -> Entity {
    spawn_prepared_model_with_bindings(
        commands,
        meshes,
        materials,
        prepared,
        Wc3TextureBindings::default(),
    )
}

/// Spawn a prepared model with consumer-defined texture choices.
pub fn spawn_prepared_model_with_bindings(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<Wc3LayerMaterial>,
    prepared: &PreparedModel,
    bindings: Wc3TextureBindings,
) -> Entity {
    let root = commands
        .spawn((Transform::default(), Visibility::default()))
        .id();
    spawn_prepared_into(commands, meshes, materials, prepared, root, bindings);
    root
}

pub(crate) fn spawn_prepared_into(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<Wc3LayerMaterial>,
    prepared: &PreparedModel,
    root: Entity,
    bindings: Wc3TextureBindings,
) {
    let bindings = initialize_root(commands, prepared, root, bindings);
    spawn_animation_root(commands, &prepared.model, root);
    commands
        .entity(root)
        .insert(Wc3ModelCameras(prepared.model.cameras()));
    let rig = spawn_rig(commands, &prepared.model, root, &prepared.inverse_bindposes);
    let node_entities = &rig.by_object_id;
    spawn_lights(commands, prepared, root, node_entities);
    spawn_attachments(commands, prepared, root, node_entities);
    spawn_particles(commands, prepared, root, node_entities);
    spawn_particles2(commands, meshes, prepared, root, node_entities, &bindings);
    spawn_ribbons(commands, meshes, prepared, root, node_entities, &bindings);
    spawn_geosets(commands, materials, prepared, root, &rig, &bindings);
}

fn initialize_root(
    commands: &mut Commands,
    prepared: &PreparedModel,
    root: Entity,
    bindings: Wc3TextureBindings,
) -> Wc3TextureBindings {
    let bindings = bindings.with_defaults(prepared.textures.clone());
    commands
        .entity(root)
        .insert((bindings.clone(), prepared.models.clone()));
    // Preserve consumer visibility when spawning into an existing root.
    commands.entity(root).insert_if_new(Visibility::default());
    bindings
}

/// Spawn an independently animated instance. The resolver supplies literal
/// bitmap paths; replaceable IDs can be bound on the returned root entity.
pub fn spawn_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<Wc3LayerMaterial>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Model,
    resolve_texture: impl FnMut(&str) -> Option<Handle<Image>>,
) -> Result<Entity, ModelError> {
    let prepared = prepare_model(meshes, inverse_bindposes, source, resolve_texture)?;
    Ok(spawn_prepared_model(commands, meshes, materials, &prepared))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
