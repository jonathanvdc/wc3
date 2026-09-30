use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::HashMap;
use wc3::model::animation::Animatable;
use wc3::model::materials::{Layer, LayerFilterMode};
use wc3::model::{Model, V1800};

use super::rig::joint_ids;
use crate::asset::{
    resolve_texture as resolve_texture_binding, ResolvedModelTextures, ResolvedTexture,
};
use crate::material::{Wc3LayerMaterial, Wc3LayerState};
use crate::mesh::build_mesh;
use crate::model::{ModelError, Wc3Model};

pub(super) struct PreparedLayer {
    pub(super) alpha: Animatable<f32>,
    pub(super) texture_id: Animatable<u32>,
    pub(super) material: Wc3LayerMaterial,
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
    pub(super) textures: ResolvedModelTextures,
    pub(super) inverse_bindposes: Handle<SkinnedMeshInverseBindposes>,
}

/// Build shareable meshes and bind poses once. Each instance receives private
/// layer materials so its texture bindings can change independently.
pub fn prepare_model(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<Wc3LayerMaterial>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Model,
    mut resolve_texture: impl FnMut(&str) -> Option<Handle<Image>>,
) -> Result<PreparedModel, ModelError> {
    let textures = ResolvedModelTextures {
        bitmaps: source
            .model
            .textures()
            .iter()
            .map(|bitmap| resolve_texture_binding(bitmap, &mut resolve_texture))
            .collect(),
        particles: source
            .model
            .particle_emitters2()
            .iter()
            .map(|emitter| ResolvedTexture {
                replaceable_id: emitter.replaceable_id,
                bitmap_id: (emitter.replaceable_id == 0).then_some(emitter.texture_id as usize),
                ..default()
            })
            .collect(),
    };
    prepare_resolved_model(meshes, materials, inverse_bindposes, source, textures)
}

pub(crate) fn prepare_resolved_model(
    meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<Wc3LayerMaterial>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Model,
    textures: ResolvedModelTextures,
) -> Result<PreparedModel, ModelError> {
    let model = &source.model;
    let default_bitmaps: Vec<_> = textures
        .bitmaps
        .iter()
        .map(|binding| binding.default.clone())
        .collect();
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
                    let value = build_layer_material(layer, &texture_id, &default_bitmaps);
                    PreparedLayer {
                        alpha: layer.alpha.clone(),
                        texture_id,
                        material: value,
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

#[cfg(test)]
mod tests {
    use super::*;
    use wc3::model::materials::Texture;

    #[test]
    fn bitmap_keeps_replaceable_id_and_only_resolves_explicit_path() {
        let mut bitmap = Texture::new("").unwrap();
        bitmap.replaceable_id = 11;
        let mut requested = Vec::new();
        let binding = resolve_texture_binding(&bitmap, &mut |path| {
            requested.push(path.to_owned());
            None
        });
        assert!(requested.is_empty());
        assert_eq!(binding.replaceable_id, 11);

        bitmap.path.set_text("Custom/Cliff.blp").unwrap();
        resolve_texture_binding(&bitmap, &mut |path| {
            requested.push(path.to_owned());
            None
        });
        assert_eq!(requested, ["Custom/Cliff.blp"]);
    }
}
