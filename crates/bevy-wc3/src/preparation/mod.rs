//! Shared meshes, bind poses, and material templates prepared before instantiation.
use crate::materials::runtime::MaterialTemplateId;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::animation::{Animatable, GeosetAnimation};
use wc3::model::animation::{Sequence, TextureAnimation};
use wc3::model::chunks::{
    AttachmentsChunk, CamerasChunk, LightsChunk, ParticleEmitters2Chunk, ParticleEmittersChunk,
    RibbonEmittersChunk,
};
use wc3::model::chunks::{
    EventObjectsChunk, GeosetAnimationsChunk, MaterialsChunk, TextureAnimationsChunk,
};
use wc3::model::emitters::{ParticleEmitter, ParticleEmitter2, RibbonEmitter};
use wc3::model::materials::ShaderType;
use wc3::model::materials::{Material, Texture};
use wc3::model::scene::EventObject;
use wc3::model::scene::{Attachment, Camera, Light};
use wc3::model::V1800;

mod materials;
pub mod shared;
use shared::{RuntimeNode, Shared};
mod pose;
pub use pose::{
    BakedModelPart, BakedModelPose, PreparedPart, Wc3MaterialSnapshot, Wc3PartId, Wc3PoseBaker,
    Wc3PoseOptions, Wc3TextureColorSpace, Wc3TextureReference, Wc3TextureRole,
};
pub(crate) mod mesh;
pub(crate) mod rig;
use self::materials::{build_layer_material, layer_texture_id};
use self::mesh::{build_mesh_with_uv, canonical_uv_coordinate};
use self::rig::{inverse_bind_matrices, joint_ids, node_order};
use crate::assets::loader::Wc3ModelAsset;
use crate::assets::loader::{
    resolve_texture as resolve_texture_binding, ResolvedModelTextures, ResolvedTexture,
};
use crate::assets::model::{ModelError, Wc3Model};
use crate::assets::resources::Wc3ModelResources;
use crate::lod::LodBounds;
use crate::materials::animation::AnimatedSurface;
use crate::materials::Wc3LayerMaterial;

pub(crate) struct PreparedLayer {
    pub(crate) template_id: MaterialTemplateId,
    pub(crate) alpha: Shared<Animatable<'static, f32>>,
    pub(crate) texture_id: Shared<Animatable<'static, u32>>,
    pub(crate) material: Wc3LayerMaterial,
    pub(crate) surface: AnimatedSurface,
}

pub(crate) struct PreparedGeoset {
    pub(crate) meshes: Vec<Handle<Mesh>>,
    pub(crate) material_id: usize,
    pub(crate) geoset_id: usize,
    pub(crate) lod: Option<u32>,
}

/// An owned runtime definition and reusable Bevy assets for a model.
/// Preparation copies the runtime definitions once; instances share those definitions.
/// The decoded source container can be dropped immediately after preparation.
/// Meshes, texture provenance, animation, and scene features remain available.
/// Keep this value alive while spawning instances. Reprepare after changing
/// the source model or its texture resolution.
pub struct PreparedModel {
    pub(crate) nodes: Vec<RuntimeNode>,
    pub(crate) node_order: Vec<usize>,
    pub(crate) sequences: Vec<Sequence>,
    pub(crate) global_sequences: Vec<u32>,
    pub(crate) blend_time: u32,
    pub(crate) pivots: Vec<[f32; 3]>,
    pub(crate) cameras: Vec<Shared<Camera<'static, V1800>>>,
    pub(crate) lights: Vec<Shared<Light<'static, V1800>>>,
    pub(crate) attachments: Vec<Shared<Attachment<'static>>>,
    pub(crate) particles: Vec<Shared<ParticleEmitter<'static>>>,
    pub(crate) particles2: Vec<Shared<ParticleEmitter2<'static>>>,
    pub(crate) ribbons: Vec<Shared<RibbonEmitter<'static>>>,
    pub(crate) material_definitions: Vec<Shared<Material<'static, V1800>>>,
    pub(crate) texture_animations: Vec<Shared<TextureAnimation<'static>>>,
    pub(crate) texture_definitions: Vec<Texture>,
    pub(crate) events: Arc<[Shared<EventObject<'static>>]>,
    pub(crate) geosets: Vec<PreparedGeoset>,
    pub(crate) lod_levels: Vec<u32>,
    pub(crate) lod_bounds: LodBounds,
    pub(crate) geoset_animations: Vec<Option<Shared<GeosetAnimation<'static>>>>,
    pub(crate) layers: Vec<Vec<PreparedLayer>>,
    pub(crate) textures: ResolvedModelTextures,
    pub(crate) models: Wc3ModelResources,
    pub(crate) inverse_bindposes: Handle<SkinnedMeshInverseBindposes>,
    pub(crate) joint_bindings: Vec<(u32, Mat4)>,
}

impl PreparedModel {
    /// Enumerate mesh passes in source order, preserving original source indices.
    pub fn parts(&self) -> impl Iterator<Item = PreparedPart<'_>> {
        self.geosets.iter().flat_map(|geoset| {
            geoset
                .meshes
                .iter()
                .enumerate()
                .map(move |(layer, mesh)| PreparedPart {
                    id: Wc3PartId {
                        geoset: geoset.geoset_id,
                        material: geoset.material_id,
                        layer,
                    },
                    mesh,
                    lod: geoset.lod,
                })
        })
    }

    /// Sorted authored levels with drawable geometry; larger numbers are coarser.
    /// Models containing only common geometry expose level zero.
    pub fn lod_levels(&self) -> &[u32] {
        &self.lod_levels
    }

    /// Resolved attachment and Classic particle model references.
    pub fn model_resources(&self) -> &Wc3ModelResources {
        &self.models
    }
}

/// Prepare an already loaded model asset, retaining its resolved textures and
/// child-model resources. No asset loading or entity spawning occurs here.
///
/// # Errors
/// Returns [`ModelError`] for geometry preparation failures, as in [`prepare_model`].
pub fn prepare_model_asset(
    meshes: &mut Assets<Mesh>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    asset: &Wc3ModelAsset,
) -> Result<PreparedModel, ModelError> {
    prepare_resolved_model(
        meshes,
        inverse_bindposes,
        &asset.source,
        asset.textures.clone(),
        asset.models.clone(),
    )
}

/// Build shareable meshes and bind poses once. Each instance receives private
/// layer materials so its texture bindings can change independently.
///
/// The resolver receives literal bitmap paths and may return `None` for missing
/// images. Replaceable IDs are supplied when spawning or through instance texture
/// bindings. This convenience function leaves child-model resources unresolved;
/// use [`prepare_model_with_resources`] to resolve attachments and PREM models.
///
/// # Errors
///
/// Returns [`ModelError`] when geometry cannot be prepared, including unsupported
/// influence counts or unavailable texture-coordinate sets.
pub fn prepare_model(
    meshes: &mut Assets<Mesh>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Model,
    resolve_texture: impl FnMut(&str) -> Option<Handle<Image>>,
) -> Result<PreparedModel, ModelError> {
    prepare_model_with_resources(meshes, inverse_bindposes, source, resolve_texture, |_| None)
}

/// Prepare shared geometry, material templates, and dependencies through
/// consumer-defined image and child-model resolvers. This does not spawn entities.
///
/// Resolvers receive authored resource paths; the application chooses their lookup
/// policy. Missing child models retain their resource slots. Meshes and bind poses
/// can be reused by multiple calls to [`crate::spawn_prepared_model`].
///
/// # Errors
///
/// Returns [`ModelError`] for geometry preparation failures, as in [`prepare_model`].
pub fn prepare_model_with_resources(
    meshes: &mut Assets<Mesh>,
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
            .map(|bitmap| resolve_texture_binding(bitmap, &mut resolve_texture))
            .collect(),
        particles: source
            .model
            .particle_emitters2()
            .map(|emitter| ResolvedTexture {
                replaceable_id: emitter.replaceable_id,
                bitmap_id: (emitter.replaceable_id == 0).then_some(emitter.texture_id as usize),
                ..default()
            })
            .collect(),
    };
    prepare_resolved_model(meshes, inverse_bindposes, source, textures, models)
}

pub(crate) fn prepare_resolved_model(
    meshes: &mut Assets<Mesh>,
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
    let material_records = model.owned_records::<MaterialsChunk<V1800>>();
    let mut geosets = Vec::new();
    for (geoset_id, geoset) in model.geosets().enumerate() {
        if geoset.vertices().is_empty() || geoset.face_indices().is_empty() {
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
        if layer_meshes.is_empty() {
            continue;
        }
        geosets.push(PreparedGeoset {
            meshes: layer_meshes,
            material_id: geoset.material_id as usize,
            geoset_id,
            lod: geoset
                .try_level_of_detail()
                .ok()
                .filter(|&lod| lod != u32::MAX),
        });
    }
    let mut geoset_animations = vec![None; model.geosets().count()];
    for animation in model.owned_records::<GeosetAnimationsChunk>() {
        if let Some(slot) = geoset_animations.get_mut(animation.geoset_id as usize) {
            *slot = Some(animation);
        }
    }
    let binds = inverse_bind_matrices(model, &joint_ids);
    let texture_animations = model.owned_records::<TextureAnimationsChunk>();
    let layers = material_records
        .iter()
        .map(|material| {
            material
                .layers
                .iter()
                .enumerate()
                .map(|(index, _)| {
                    let layer = material.select(move |material| &material.layers[index]);
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
                    let texture_id = layer.map(layer_texture_id);
                    let value = build_layer_material(&layer, &texture_id, &default_bitmaps);
                    PreparedLayer {
                        template_id: MaterialTemplateId::new(),
                        alpha: layer.map(|layer| &layer.alpha),
                        texture_id,
                        material: value,
                        surface: AnimatedSurface::new(
                            &layer,
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
    let mut lod_levels: Vec<_> = geosets.iter().filter_map(|geoset| geoset.lod).collect();
    lod_levels.sort_unstable();
    lod_levels.dedup();
    if lod_levels.is_empty() {
        lod_levels.push(0);
    }
    let nodes = model.runtime_nodes();
    let node_order = node_order(&nodes);
    Ok(PreparedModel {
        lod_levels,
        lod_bounds: LodBounds::from_model(model),
        events: model.owned_records::<EventObjectsChunk>().into(),
        nodes,
        node_order,
        sequences: model.sequences().cloned().collect(),
        global_sequences: model.global_sequences(),
        blend_time: model.model_info().map_or(0, |info| info.blend_time),
        pivots: model.pivot_points(),
        cameras: model.owned_records::<CamerasChunk<V1800>>(),
        lights: model.owned_records::<LightsChunk<V1800>>(),
        attachments: model.owned_records::<AttachmentsChunk>(),
        particles: model.owned_records::<ParticleEmittersChunk>(),
        particles2: model.owned_records::<ParticleEmitters2Chunk>(),
        ribbons: model.owned_records::<RibbonEmittersChunk>(),
        material_definitions: material_records,
        texture_animations,
        texture_definitions: model.textures().cloned().collect(),
        geosets,
        geoset_animations,
        layers,
        textures,
        models,
        joint_bindings: joint_ids.into_iter().zip(binds.iter().copied()).collect(),
        inverse_bindposes: inverse_bindposes.add(binds),
    })
}

#[cfg(test)]
mod tests;
