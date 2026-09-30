use bevy::camera::visibility::DynamicSkinnedMeshBounds;
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::prelude::*;

use crate::animation::AnimatedLayer;
use crate::material::Wc3LayerMaterial;
use crate::model::{ModelError, Wc3Model};

mod prepare;
mod rig;

pub(crate) use prepare::prepare_resolved_model;
pub use prepare::{prepare_model, PreparedModel};
use rig::{spawn_animation_root, spawn_rig};

/// Spawn an independently animated instance from prepared assets.
pub fn spawn_prepared_model(
    commands: &mut Commands,
    materials: &mut Assets<Wc3LayerMaterial>,
    prepared: &PreparedModel,
) -> Entity {
    let root = commands.spawn(Transform::default()).id();
    spawn_prepared_into(commands, materials, prepared, root);
    root
}

pub(crate) fn spawn_prepared_into(
    commands: &mut Commands,
    materials: &mut Assets<Wc3LayerMaterial>,
    prepared: &PreparedModel,
    root: Entity,
) {
    spawn_animation_root(commands, &prepared.model, root);
    let rig = spawn_rig(commands, &prepared.model, root, &prepared.inverse_bindposes);
    let layer_handles: Vec<Vec<_>> = prepared
        .layers
        .iter()
        .map(|layers| {
            layers
                .iter()
                .map(|layer| {
                    layer
                        .shared
                        .clone()
                        .unwrap_or_else(|| materials.add(layer.material.clone()))
                })
                .collect()
        })
        .collect();
    for geoset in &prepared.geosets {
        for (layer, material) in prepared.layers[geoset.material_id]
            .iter()
            .zip(&layer_handles[geoset.material_id])
        {
            let mut entity = commands.spawn((
                Mesh3d(geoset.mesh.clone()),
                MeshMaterial3d(material.clone()),
            ));
            if layer.shared.is_none() {
                entity.insert(AnimatedLayer {
                    root,
                    alpha: layer.alpha.clone(),
                    texture_id: layer.texture_id.clone(),
                    textures: prepared.textures.clone(),
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
            } else {
                commands.entity(root).add_child(entity);
            }
        }
    }
}

/// Spawn an independently animated instance. The texture resolver supplies
/// model texture handles; replaceable textures can be handled by the caller.
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
    Ok(spawn_prepared_model(commands, materials, &prepared))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::world::{CommandQueue, World};
    use wc3::model::animation::{Track, ValueKeyframe};
    use wc3::model::materials::Layer;

    #[test]
    fn prepared_assets_share_static_layers_and_isolate_animated_layers() {
        let bytes = include_bytes!("../../../wc3/tests/fixtures/mdl/quad_model.mdx");
        let mut source = Wc3Model::decode(bytes).unwrap();
        let mut material_records = source.model.materials();
        assert!(!material_records.is_empty());
        assert!(!material_records[0].layers.is_empty());
        material_records[0].layers[0].alpha.set_track(
            Track::linear(
                vec![
                    ValueKeyframe {
                        frame: 0,
                        value: 1.0,
                    },
                    ValueKeyframe {
                        frame: 100,
                        value: 0.0,
                    },
                ],
                None,
            )
            .unwrap(),
        );
        material_records[0].layers.push(Layer::new());
        source.model.set_materials(&material_records);

        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<Wc3LayerMaterial>::default();
        let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
        let prepared = prepare_model(&mut meshes, &mut materials, &mut bindposes, &source, |_| {
            None
        })
        .unwrap();
        let mesh_count = meshes.len();
        let static_count = materials.len();
        assert_eq!(static_count, 1);
        let bindpose_count = bindposes.len();
        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        spawn_prepared_model(&mut commands, &mut materials, &prepared);
        spawn_prepared_model(&mut commands, &mut materials, &prepared);
        assert_eq!(meshes.len(), mesh_count);
        assert_eq!(bindposes.len(), bindpose_count);
        assert_eq!(materials.len(), static_count + 2);
        assert!(!prepared.geosets.is_empty());
    }
}
