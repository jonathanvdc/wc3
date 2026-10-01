use bevy::camera::visibility::DynamicSkinnedMeshBounds;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::prelude::*;
use wc3::model::materials::LayerFilterMode;

use crate::animation::AnimatedLayer;
use crate::attachment::spawn_attachments;
use crate::material::Wc3LayerMaterial;
use crate::model::{ModelError, Wc3Model};
use crate::particle_emitter::ParticleState;
use crate::particle_emitter2::{Particle2State, ParticleInstances, ParticleTextureSlot};
use crate::ribbon_emitter::{RibbonInstances, RibbonLayer, RibbonState};
use crate::texture_bindings::Wc3TextureBindings;

mod prepare;
mod rig;

pub(crate) use prepare::prepare_resolved_model;
pub use prepare::{prepare_model, prepare_model_with_resources, PreparedModel};
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
    let bindings = bindings.with_defaults(prepared.textures.clone());
    commands
        .entity(root)
        .insert((bindings.clone(), prepared.models.clone()));
    // Preserve consumer visibility when spawning into an existing root.
    commands.entity(root).insert_if_new(Visibility::default());
    spawn_animation_root(commands, &prepared.model, root);
    let rig = spawn_rig(commands, &prepared.model, root, &prepared.inverse_bindposes);
    let node_entities = &rig.by_object_id;
    spawn_attachments(commands, prepared, root, node_entities);
    for (index, definition) in prepared.model.particle_emitters().into_iter().enumerate() {
        let Some(model) = prepared.model_resources().particle(index) else {
            continue;
        };
        let Some(&node) = node_entities.get(&definition.node.object_id) else {
            continue;
        };
        commands.spawn((
            ParticleState::new(root, node, definition, model),
            ChildOf(root),
        ));
    }
    for (emitter_id, definition) in prepared.model.particle_emitters2().into_iter().enumerate() {
        let Some(&node) = node_entities.get(&definition.node.object_id) else {
            continue;
        };
        let texture = bindings.particle(emitter_id);
        let mesh = meshes.add(Rectangle::new(2.0, 2.0));
        let render = ParticleInstances {
            records: Default::default(),
            live_indices: Default::default(),
            uniform: Default::default(),
            texture,
            filter: definition.filter_mode,
            priority_plane: definition.priority_plane,
            sort_far: definition.node.flags.sort_prims_far_z(),
        };
        let entity = commands
            .spawn((
                Mesh3d(mesh.clone()),
                render,
                Particle2State::new(root, node, definition),
                ParticleTextureSlot(emitter_id),
                Transform::default(),
                NoFrustumCulling,
            ))
            .id();
        commands.entity(root).add_child(entity);
    }
    let source_materials = prepared.model.materials();
    let texture_animations = prepared.model.texture_animations();
    for definition in prepared.model.ribbon_emitters() {
        let Some(&node) = node_entities.get(&definition.node.object_id) else {
            continue;
        };
        let Some(material) = source_materials.get(definition.material_id as usize) else {
            warn!(
                "Ribbon {} references missing material {}",
                definition.node.object_id, definition.material_id
            );
            continue;
        };
        let emitter = commands
            .spawn((
                RibbonState::new(root, node, definition.clone()),
                ChildOf(root),
            ))
            .id();
        let mesh = meshes.add(Rectangle::new(2.0, 2.0));
        for (layer_index, layer) in material.layers.iter().enumerate() {
            let mut layer = layer.clone();
            // Prepared layers normalize Classic and HD diffuse texture bindings.
            layer.texture_id = prepared.layers[definition.material_id as usize][layer_index]
                .texture_id
                .clone();
            let texture = bindings.bitmap(layer.texture_id.value().copied().unwrap_or(0) as usize);
            let flags = layer.shading_flags;
            commands.spawn((
                Mesh3d(mesh.clone()),
                RibbonInstances {
                    records: Default::default(),
                    live_indices: Default::default(),
                    uniform: Default::default(),
                    texture,
                    filter: layer.filter_mode,
                    priority_plane: material.priority_plane,
                    emitter,
                    layer_index,
                    sort_far: material.render_mode.sort_primitives_far_z(),
                    sort_near: material.render_mode.sort_primitives_near_z(),
                    no_depth_test: flags.no_depth_test(),
                    no_depth_set: flags.no_depth_set(),
                    two_sided: flags.two_sided() || material.render_mode.two_sided(),
                },
                RibbonLayer {
                    emitter,
                    texture_animation: texture_animations
                        .get(layer.texture_animation_id as usize)
                        .cloned(),
                    definition: layer,
                },
                Transform::default(),
                NoFrustumCulling,
                ChildOf(root),
            ));
        }
    }
    let layer_handles: Vec<Vec<_>> = prepared
        .layers
        .iter()
        .map(|layers| {
            layers
                .iter()
                .map(|layer| {
                    let mut value = layer.material.clone();
                    let texture_id = layer.texture_id.value().copied().unwrap_or(0);
                    value.base.base_color_texture = bindings.bitmap(texture_id as usize);
                    materials.add(value)
                })
                .collect()
        })
        .collect();
    for geoset in &prepared.geosets {
        let geoset_alpha = prepared.geoset_alphas[geoset.geoset_id].clone();
        let initial_visibility = if geoset_alpha
            .as_ref()
            .and_then(|alpha| alpha.value())
            .is_some_and(|alpha| *alpha <= 0.0)
        {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        for (layer, material) in prepared.layers[geoset.material_id]
            .iter()
            .zip(&layer_handles[geoset.material_id])
        {
            let material = if geoset_alpha.is_some() {
                let mut material = materials
                    .get(material)
                    .cloned()
                    .unwrap_or_else(|| layer.material.clone());
                let alpha = layer.alpha.value().copied().unwrap_or(1.0)
                    * geoset_alpha
                        .as_ref()
                        .and_then(|alpha| alpha.value().copied())
                        .unwrap_or(1.0);
                material.base.base_color = Color::srgba(1.0, 1.0, 1.0, alpha);
                if geoset_alpha
                    .as_ref()
                    .and_then(|alpha| alpha.value().copied())
                    .is_some_and(|alpha| (0.0..1.0).contains(&alpha))
                    && matches!(
                        layer.material.extension.filter,
                        LayerFilterMode::None | LayerFilterMode::Transparent
                    )
                {
                    material.base.alpha_mode = AlphaMode::AlphaToCoverage;
                }
                materials.add(material)
            } else {
                material.clone()
            };
            let mut entity = commands.spawn((
                Mesh3d(geoset.mesh.clone()),
                MeshMaterial3d(material),
                initial_visibility,
            ));
            {
                entity.insert(AnimatedLayer {
                    root,
                    alpha: layer.alpha.clone(),
                    geoset_alpha: geoset_alpha.clone(),
                    texture_id: layer.texture_id.clone(),
                });
            }
            let entity = entity.id();
            if !rig.joints.is_empty() {
                commands.entity(entity).insert((
                    SkinnedMesh {
                        inverse_bindposes: rig.inverse_bindposes.clone(),
                        joints: rig.joints.clone(),
                    },
                    DynamicSkinnedMeshBounds,
                ));
            }
            // Joint matrices already contain world transforms; skinning ignores
            // this mesh transform. Parenting supplies visibility and ownership.
            commands.entity(root).add_child(entity);
        }
    }
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
    let prepared = prepare_model(
        meshes,
        materials,
        inverse_bindposes,
        source,
        resolve_texture,
    )?;
    Ok(spawn_prepared_model(commands, meshes, materials, &prepared))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
