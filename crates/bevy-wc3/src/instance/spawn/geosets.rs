use crate::lod::{LodGroup, Wc3LodState};
use crate::preparation::PreparedLayer;
use bevy::camera::visibility::DynamicSkinnedMeshBounds;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use std::collections::HashMap;
use wc3::model::animation::GeosetAnimation;
use wc3::model::materials::LayerFilterMode;

use super::{rig::Rig, PreparedModel};
use crate::materials::layers::AnimatedLayer;
use crate::materials::textures::Wc3TextureBindings;
use crate::materials::Wc3LayerMaterial;

pub(super) fn spawn_geosets(
    commands: &mut Commands,
    materials: &mut Assets<Wc3LayerMaterial>,
    prepared: &PreparedModel,
    root: Entity,
    rig: &Rig,
    bindings: &Wc3TextureBindings,
) {
    let layer_handles = instantiate_materials(materials, prepared, bindings);
    let selected = prepared.lod_levels[0];
    commands.entity(root).insert(Wc3LodState::new(prepared));
    let mut groups = HashMap::new();
    for geoset in &prepared.geosets {
        groups.entry(geoset.lod).or_insert_with(|| {
            let visibility = if geoset.lod.is_none_or(|level| level == selected) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            let group = commands
                .spawn((
                    Transform::default(),
                    visibility,
                    LodGroup {
                        root,
                        level: geoset.lod,
                    },
                ))
                .id();
            commands.entity(root).add_child(group);
            group
        });
    }
    for geoset in &prepared.geosets {
        let geoset_animation = prepared.geoset_animations[geoset.geoset_id].as_ref();
        let geoset_alpha = geoset_animation.map(|animation| animation.alpha.clone());
        let geoset_color = geoset_animation
            .filter(|animation| animation.flags.color())
            .map(|animation| animation.color.clone());
        let initial_visibility = if geoset_alpha
            .as_ref()
            .and_then(|alpha| alpha.value())
            .is_some_and(|alpha| *alpha <= 0.0)
        {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        for ((layer, material), mesh) in prepared.layers[geoset.material_id]
            .iter()
            .zip(&layer_handles[geoset.material_id])
            .zip(&geoset.meshes)
        {
            let material =
                instantiate_geoset_material(materials, layer, material, geoset_animation);
            let mut entity = commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                initial_visibility,
            ));
            entity.insert(AnimatedLayer {
                root,
                alpha: layer.alpha.clone(),
                geoset_alpha: geoset_alpha.clone(),
                geoset_color: geoset_color.clone(),
                texture_id: layer.texture_id.clone(),
            });
            let mut surface = layer.surface.clone();
            surface.root = root;
            entity.insert(surface);
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
            commands.entity(groups[&geoset.lod]).add_child(entity);
        }
    }
}

fn instantiate_materials(
    materials: &mut Assets<Wc3LayerMaterial>,
    prepared: &PreparedModel,
    bindings: &Wc3TextureBindings,
) -> Vec<Vec<Handle<Wc3LayerMaterial>>> {
    prepared
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
        .collect()
}

fn instantiate_geoset_material(
    materials: &mut Assets<Wc3LayerMaterial>,
    layer: &PreparedLayer,
    material: &Handle<Wc3LayerMaterial>,
    geoset_animation: Option<&GeosetAnimation>,
) -> Handle<Wc3LayerMaterial> {
    if let Some(animation) = geoset_animation {
        let geoset_alpha = Some(&animation.alpha);
        let mut material = materials
            .get(material)
            .cloned()
            .unwrap_or_else(|| layer.material.clone());
        let alpha = layer.alpha.value().copied().unwrap_or(1.0)
            * geoset_alpha
                .and_then(|alpha| alpha.value().copied())
                .unwrap_or(1.0);
        let [red, green, blue] = if animation.flags.color() {
            animation.color.value().copied().unwrap_or([1.0; 3])
        } else {
            [1.0; 3]
        };
        material.base.base_color = Color::linear_rgba(red, green, blue, alpha);
        if geoset_alpha
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
    }
}
