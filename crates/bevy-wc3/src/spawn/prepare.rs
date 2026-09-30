use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::HashMap;
use wc3::model::animation::Animatable;
use wc3::model::materials::{Layer, LayerFilterMode};
use wc3::model::{Model, V1800};

use super::rig::joint_ids;
use crate::material::{Wc3LayerMaterial, Wc3LayerState};
use crate::mesh::build_mesh;
use crate::model::{ModelError, Wc3Model};

pub(super) struct PreparedLayer {
    pub(super) alpha: Animatable<f32>,
    pub(super) texture_id: Animatable<u32>,
    pub(super) material: Wc3LayerMaterial,
    pub(super) shared: Option<Handle<Wc3LayerMaterial>>,
}

pub(super) struct PreparedGeoset {
    pub(super) mesh: Handle<Mesh>,
    pub(super) material_id: usize,
    pub(super) geoset_id: usize,
}

/// Reusable Bevy assets for one model and one set of resolved textures.
/// Keep this value alive while spawning instances. Reprepare after changing
/// the source model or its texture resolution.
pub struct PreparedModel {
    pub(super) model: Model<V1800>,
    pub(super) geosets: Vec<PreparedGeoset>,
    pub(super) geoset_alphas: Vec<Option<Animatable<f32>>>,
    pub(super) layers: Vec<Vec<PreparedLayer>>,
    pub(super) textures: Vec<Option<Handle<Image>>>,
    pub(super) inverse_bindposes: Handle<SkinnedMeshInverseBindposes>,
}

/// Build shareable meshes, bind poses, and static layer materials once.
/// Animated layers retain a template and receive a private material per spawn.
pub fn prepare_model(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<Wc3LayerMaterial>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Model,
    mut resolve_texture: impl FnMut(&str) -> Option<Handle<Image>>,
) -> Result<PreparedModel, ModelError> {
    let textures = resolve_textures(&source.model, &mut resolve_texture);
    prepare_resolved_model(meshes, materials, inverse_bindposes, source, textures)
}

pub(crate) fn prepare_resolved_model(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<Wc3LayerMaterial>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Model,
    textures: Vec<Option<Handle<Image>>>,
) -> Result<PreparedModel, ModelError> {
    let model = &source.model;
    let joint_ids = joint_ids(model);
    let joint_index: HashMap<_, _> = joint_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index as u16))
        .collect();
    let material_records = model.materials();
    let mut geosets = Vec::new();
    for (geoset_id, geoset) in model.geosets().iter().enumerate() {
        if geoset
            .try_level_of_detail()
            .is_ok_and(|lod| lod != 0 && lod != u32::MAX)
            || geoset.vertices().is_empty()
            || geoset.face_indices().is_empty()
        {
            continue;
        }
        if material_records.get(geoset.material_id as usize).is_none() {
            continue;
        }
        let mesh = build_mesh(geoset, &joint_index, !joint_ids.is_empty())
            .map_err(|error| ModelError(format!("geoset {geoset_id}: {error}")))?;
        let mesh = meshes.add(mesh);
        geosets.push(PreparedGeoset {
            mesh,
            material_id: geoset.material_id as usize,
            geoset_id,
        });
    }
    let mut geoset_alphas = vec![None; model.geosets().len()];
    for animation in model.geoset_animations() {
        if let Some(alpha) = geoset_alphas.get_mut(animation.geoset_id as usize) {
            *alpha = Some(animation.alpha);
        }
    }
    let pivots = model.pivot_points();
    let binds = joint_ids
        .iter()
        .map(|id| {
            let pivot = pivots.get(*id as usize).copied().unwrap_or([0.0; 3]);
            Mat4::from_translation(-Vec3::from_array(pivot))
        })
        .collect::<Vec<_>>();
    let layers = material_records
        .iter()
        .map(|material| {
            material
                .layers
                .iter()
                .map(|layer| {
                    let texture_id = layer_texture_id(layer);
                    let value = build_layer_material(layer, &texture_id, &textures);
                    let shared = if layer.alpha.track().is_none() && texture_id.track().is_none() {
                        Some(materials.add(value.clone()))
                    } else {
                        None
                    };
                    PreparedLayer {
                        alpha: layer.alpha.clone(),
                        texture_id,
                        material: value,
                        shared,
                    }
                })
                .collect()
        })
        .collect();
    Ok(PreparedModel {
        model: model.clone(),
        geosets,
        geoset_alphas,
        layers,
        textures,
        inverse_bindposes: inverse_bindposes.add(binds),
    })
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

fn build_layer_material(
    layer: &Layer<V1800>,
    texture_id: &Animatable<u32>,
    textures: &[Option<Handle<Image>>],
) -> Wc3LayerMaterial {
    let alpha = layer.alpha.value().copied().unwrap_or(1.0);
    let texture = texture_id
        .value()
        .and_then(|id| textures.get(*id as usize))
        .cloned()
        .flatten();
    Wc3LayerMaterial {
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
    }
}
