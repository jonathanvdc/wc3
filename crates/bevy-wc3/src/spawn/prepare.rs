use bevy::material::OpaqueRendererMethod;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use std::collections::HashMap;
use wc3::model::animation::{Animatable, GeosetAnimation};
use wc3::model::materials::{Layer, LayerFilterMode, ShaderType};
use wc3::model::{Model, V1800};

use super::rig::joint_ids;
use crate::asset::Wc3ModelAsset;
use crate::asset::{
    resolve_texture as resolve_texture_binding, ResolvedModelTextures, ResolvedTexture,
};
use crate::material::{Wc3LayerMaterial, Wc3LayerState};
use crate::material_animation::AnimatedSurface;
use crate::mesh::{build_mesh_with_uv, canonical_uv_coordinate};
use crate::model::{ModelError, Wc3Model};
use crate::model_resources::Wc3ModelResources;

pub(super) struct PreparedLayer {
    pub(super) alpha: Animatable<f32>,
    pub(super) texture_id: Animatable<u32>,
    pub(super) material: Wc3LayerMaterial,
    pub(super) surface: AnimatedSurface,
}

pub(super) struct PreparedGeoset {
    pub(super) meshes: Vec<Handle<Mesh>>,
    pub(super) material_id: usize,
    pub(super) geoset_id: usize,
}

/// Reusable Bevy assets for one model and one set of resolved textures.
/// Keep this value alive while spawning instances. Reprepare after changing
/// the source model or its texture resolution.
pub struct PreparedModel {
    pub(crate) model: Model<V1800>,
    pub(super) geosets: Vec<PreparedGeoset>,
    pub(super) geoset_animations: Vec<Option<GeosetAnimation>>,
    pub(super) layers: Vec<Vec<PreparedLayer>>,
    pub(super) textures: ResolvedModelTextures,
    pub(super) models: Wc3ModelResources,
    pub(super) inverse_bindposes: Handle<SkinnedMeshInverseBindposes>,
}

impl PreparedModel {
    /// Normalized diffuse binding shared by geosets and ribbon layers.
    pub(crate) fn layer_texture_id(&self, material: usize, layer: usize) -> &Animatable<u32> {
        &self.layers[material][layer].texture_id
    }

    /// Resolved attachment and Classic particle model references.
    pub fn model_resources(&self) -> &Wc3ModelResources {
        &self.models
    }
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
    prepare_model_with_resources(
        meshes,
        materials,
        inverse_bindposes,
        source,
        &mut resolve_texture,
        |_| None,
    )
}

/// Prepare textures and child model references through consumer-defined sources.
/// Model resolution does not spawn attachments or particles.
pub fn prepare_model_with_resources(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<Wc3LayerMaterial>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Model,
    mut resolve_texture: impl FnMut(&str) -> Option<Handle<Image>>,
    resolve_model: impl FnMut(&str) -> Option<Handle<Wc3ModelAsset>>,
) -> Result<PreparedModel, ModelError> {
    let models = Wc3ModelResources::resolve(&source.model, resolve_model);
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
    prepare_resolved_model(
        meshes,
        materials,
        inverse_bindposes,
        source,
        textures,
        models,
    )
}

pub(crate) fn prepare_resolved_model(
    meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<Wc3LayerMaterial>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Model,
    textures: ResolvedModelTextures,
    models: Wc3ModelResources,
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
        let material = &material_records[geoset.material_id as usize];
        let mut variants: HashMap<u32, Handle<Mesh>> = HashMap::new();
        let mut layer_meshes = Vec::new();
        for layer in &material.layers {
            let coordinate = canonical_uv_coordinate(geoset, layer.coordinate_id);
            let handle = if let Some(handle) = variants.get(&coordinate) {
                handle.clone()
            } else {
                let mesh =
                    build_mesh_with_uv(geoset, &joint_index, !joint_ids.is_empty(), coordinate)
                        .map_err(|error| ModelError(format!("geoset {geoset_id}: {error}")))?;
                let handle = meshes.add(mesh);
                variants.insert(coordinate, handle.clone());
                handle
            };
            layer_meshes.push(handle);
        }
        geosets.push(PreparedGeoset {
            meshes: layer_meshes,
            material_id: geoset.material_id as usize,
            geoset_id,
        });
    }
    let mut geoset_animations = vec![None; model.geosets().len()];
    for animation in model.geoset_animations() {
        if let Some(slot) = geoset_animations.get_mut(animation.geoset_id as usize) {
            *slot = Some(animation);
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
    let texture_animations = model.texture_animations();
    let layers = material_records
        .iter()
        .map(|material| {
            material
                .layers
                .iter()
                .map(|layer| {
                    if !matches!(
                        layer.shader_type(),
                        ShaderType::SD_LEGACY
                            | ShaderType::SD_FIXED_FUNCTION
                            | ShaderType::HD_DEFAULT_UNIT
                    ) {
                        warn!(
                            "Unsupported WC3 shader {:?}; rendering diffuse fallback",
                            layer.shader_type()
                        );
                    }
                    let texture_id = layer_texture_id(layer);
                    let value = build_layer_material(layer, &texture_id, &default_bitmaps);
                    PreparedLayer {
                        alpha: layer.alpha.clone(),
                        texture_id,
                        material: value,
                        surface: AnimatedSurface::new(
                            layer,
                            texture_animations
                                .get(layer.texture_animation_id as usize)
                                .cloned(),
                            material.priority_plane,
                        ),
                    }
                })
                .collect()
        })
        .collect();
    Ok(PreparedModel {
        model: model.clone(),
        geosets,
        geoset_animations,
        layers,
        textures,
        models,
        inverse_bindposes: inverse_bindposes.add(binds),
    })
}

fn layer_texture_id(layer: &Layer<V1800>) -> Animatable<u32> {
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
        // Bevy's Add shader path clears alpha. WC3's custom blend state needs
        // the sampled alpha as its source factor, so retain it with Blend.
        LayerFilterMode::Additive | LayerFilterMode::AddAlpha => AlphaMode::Blend,
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
            alpha_mode: if layer.shader_type() == ShaderType::HD_DEFAULT_UNIT
                && layer.filter_mode == LayerFilterMode::Transparent
            {
                AlphaMode::Mask(0.75)
            } else {
                layer_alpha_mode(layer.filter_mode)
            },
            opaque_render_method: OpaqueRendererMethod::Forward,
            unlit: layer.shading_flags.unshaded(),
            fog_enabled: !layer.shading_flags.unfogged(),
            double_sided: layer.shading_flags.two_sided(),
            cull_mode: if layer.shading_flags.two_sided() {
                None
            } else {
                Some(Face::Back)
            },
            ..default()
        },
        extension: Wc3LayerState {
            filter: layer.filter_mode,
            no_depth_test: layer.shading_flags.no_depth_test(),
            no_depth_set: layer.shading_flags.no_depth_set(),
            ..default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc3::model::materials::Texture;

    #[test]
    fn additive_layers_preserve_shader_alpha_for_custom_blending() {
        assert_eq!(
            layer_alpha_mode(LayerFilterMode::Additive),
            AlphaMode::Blend
        );
        assert_eq!(
            layer_alpha_mode(LayerFilterMode::AddAlpha),
            AlphaMode::Blend
        );
    }

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
