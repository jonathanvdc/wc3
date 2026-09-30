use bevy::camera::visibility::DynamicSkinnedMeshBounds;
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::prelude::*;
use std::collections::HashMap;
use wc3::model::materials::{Layer, LayerFilterMode, Material};
use wc3::model::{Model, V1800};

use super::animation::{AnimatedLayer, AnimatedNode, Wc3Animation};
use super::material::{Wc3LayerMaterial, Wc3LayerState};
use super::mesh::build_mesh;
use super::model::{ModelError, Wc3Model};

struct Rig {
    joint_index: HashMap<u32, u16>,
    joints: Vec<Entity>,
    inverse_bindposes: Handle<SkinnedMeshInverseBindposes>,
}

/// Spawn an independently animated instance. The texture resolver supplies
/// model texture handles; replaceable textures can be handled by the caller.
pub fn spawn_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<Wc3LayerMaterial>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Model,
    mut resolve_texture: impl FnMut(&str) -> Option<Handle<Image>>,
) -> Result<Entity, ModelError> {
    let model = &source.model;
    let root = spawn_animation_root(commands, model);
    let rig = spawn_rig(commands, inverse_bindposes, model, root);
    let textures = resolve_textures(model, &mut resolve_texture);
    let material_records = model.materials();
    for geoset in model.geosets() {
        if geoset
            .try_level_of_detail()
            .is_ok_and(|lod| lod != 0 && lod != u32::MAX)
        {
            continue;
        }
        if geoset.vertices().is_empty() || geoset.face_indices().is_empty() {
            continue;
        }
        let mesh = meshes.add(build_mesh(
            &geoset,
            &rig.joint_index,
            !rig.joints.is_empty(),
        )?);
        let Some(material) = material_records.get(geoset.material_id as usize) else {
            continue;
        };
        spawn_layers(commands, materials, root, &rig, mesh, material, &textures);
    }
    Ok(root)
}

fn spawn_animation_root(commands: &mut Commands, model: &Model<V1800>) -> Entity {
    commands
        .spawn((
            Wc3Animation {
                sequence: 0,
                elapsed_ms: 0.0,
                speed: 1.0,
                playing: true,
                sequences: model.sequences(),
                global_sequences: model.global_sequences(),
            },
            Transform::default(),
        ))
        .id()
}

fn spawn_rig(
    commands: &mut Commands,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    model: &Model<V1800>,
    root: Entity,
) -> Rig {
    let bones = model.bones();
    let mut source_nodes: Vec<_> = bones.iter().map(|bone| bone.node.clone()).collect();
    source_nodes.extend(model.helpers());
    let pivots = model.pivot_points();
    let mut nodes = HashMap::new();
    for node in &source_nodes {
        let pivot = pivots
            .get(node.object_id as usize)
            .copied()
            .unwrap_or([0.0; 3]);
        let parent_pivot = pivots
            .get(node.parent_id as usize)
            .copied()
            .unwrap_or([0.0; 3]);
        let entity = commands
            .spawn((
                AnimatedNode {
                    root,
                    pivot: Vec3::from_array(pivot),
                    parent_pivot: Vec3::from_array(parent_pivot),
                    translation: node.translation.clone(),
                    rotation: node.rotation.clone(),
                    scaling: node.scaling.clone(),
                },
                Transform::from_translation(
                    Vec3::from_array(pivot) - Vec3::from_array(parent_pivot),
                ),
            ))
            .id();
        nodes.insert(node.object_id, entity);
    }
    for node in &source_nodes {
        let parent = nodes.get(&node.parent_id).copied().unwrap_or(root);
        commands.entity(parent).add_child(nodes[&node.object_id]);
    }
    let mut joint_ids: Vec<u32> = bones.iter().map(|bone| bone.node.object_id).collect();
    joint_ids.sort_unstable();
    let joint_index = joint_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index as u16))
        .collect();
    let joints = joint_ids.iter().map(|id| nodes[id]).collect();
    let binds: Vec<Mat4> = joint_ids
        .iter()
        .map(|id| {
            let pivot = pivots.get(*id as usize).copied().unwrap_or([0.0; 3]);
            Mat4::from_translation(-Vec3::from_array(pivot))
        })
        .collect();
    Rig {
        joint_index,
        joints,
        inverse_bindposes: inverse_bindposes.add(binds),
    }
}

fn resolve_textures(
    model: &Model<V1800>,
    resolve: &mut impl FnMut(&str) -> Option<Handle<Image>>,
) -> Vec<Option<Handle<Image>>> {
    model
        .textures()
        .iter()
        .map(|texture| {
            if texture.replaceable_id == 0 {
                resolve(&texture.path.text())
            } else {
                None
            }
        })
        .collect()
}

fn layer_texture_id(layer: &Layer<V1800>) -> wc3::model::animation::Animatable<u32> {
    layer
        .texture_slots()
        .iter()
        .find(|slot| slot.texture_type == 0)
        .map(|slot| slot.texture_id.clone())
        .unwrap_or_else(|| layer.texture_id.clone())
}

fn layer_alpha_mode(mode: LayerFilterMode) -> AlphaMode {
    match mode {
        LayerFilterMode::None => AlphaMode::Opaque,
        LayerFilterMode::Transparent => AlphaMode::Mask(0.5),
        LayerFilterMode::Blend => AlphaMode::Blend,
        LayerFilterMode::Additive | LayerFilterMode::AddAlpha => AlphaMode::Add,
        LayerFilterMode::Modulate | LayerFilterMode::Modulate2x => AlphaMode::Multiply,
        LayerFilterMode::Unknown(_) => AlphaMode::Blend,
    }
}

fn spawn_layers(
    commands: &mut Commands,
    materials: &mut Assets<Wc3LayerMaterial>,
    root: Entity,
    rig: &Rig,
    mesh: Handle<Mesh>,
    material: &Material<V1800>,
    textures: &[Option<Handle<Image>>],
) {
    for layer in &material.layers {
        let texture_id = layer_texture_id(layer);
        let alpha = layer.alpha.value().copied().unwrap_or(1.0);
        let texture = texture_id
            .value()
            .and_then(|id| textures.get(*id as usize))
            .cloned()
            .flatten();
        let material_handle = materials.add(Wc3LayerMaterial {
            base: StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, alpha),
                base_color_texture: texture,
                alpha_mode: layer_alpha_mode(layer.filter_mode),
                double_sided: layer.shading_flags.two_sided(),
                cull_mode: if layer.shading_flags.two_sided() {
                    None
                } else {
                    Some(bevy::render::render_resource::Face::Back)
                },
                ..default()
            },
            extension: Wc3LayerState {
                filter: layer.filter_mode,
                no_depth_test: layer.shading_flags.no_depth_test(),
                no_depth_set: layer.shading_flags.no_depth_set(),
            },
        });
        let entity = commands
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material_handle),
                AnimatedLayer {
                    root,
                    alpha: layer.alpha.clone(),
                    texture_id,
                    textures: textures.to_vec(),
                },
            ))
            .id();
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
