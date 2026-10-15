//! Shared meshes, bind poses, and material templates prepared before instantiation.
use crate::assets::model::Wc3Model;
use crate::materials::runtime::MaterialTemplateId;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::conversion::ExportOptions;
use wc3::model::formats::mdlx::animation::{Animatable, GeosetAnimation};
use wc3::model::formats::mdlx::animation::{Sequence, TextureAnimation};
use wc3::model::formats::mdlx::emitters::{ParticleEmitter, ParticleEmitter2, RibbonEmitter};
use wc3::model::formats::mdlx::materials::ShaderType;
use wc3::model::formats::mdlx::materials::{Material, Texture};
use wc3::model::formats::mdlx::scene::EventObject;
use wc3::model::formats::mdlx::scene::{Attachment, Camera, Light};
use wc3::model::formats::mdlx::{Document, V1800};
use wc3::model::io::mdlx as semantic_mdlx;
use wc3::model::{profiles::Mdlx, Model};

mod compilation;
use compilation::RuntimeRecords;
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
use crate::assets::model::ModelError;
#[cfg(test)]
use crate::assets::model::Wc3Document;
use crate::assets::resources::Wc3ModelResources;
use crate::lod::LodBounds;
use crate::materials::animation::AnimatedSurface;
use crate::materials::Wc3LayerMaterial;

#[derive(Clone)]
pub(crate) struct PreparedLayer {
    pub(crate) template_id: MaterialTemplateId,
    pub(crate) alpha: Shared<Animatable<'static, f32>>,
    pub(crate) texture_id: Shared<Animatable<'static, u32>>,
    pub(crate) material: Wc3LayerMaterial,
    pub(crate) surface: AnimatedSurface,
}

#[derive(Clone)]
pub(crate) struct PreparedGeoset<M> {
    pub(crate) meshes: Vec<M>,
    pub(crate) material_id: usize,
    pub(crate) geoset_id: usize,
    pub(crate) lod: Option<u32>,
}

/// An owned runtime definition and reusable Bevy assets for a model.
/// Compilation owns the required runtime definitions; instances share them.
/// The decoded source container can be dropped immediately after preparation.
/// Meshes, texture provenance, animation, and scene features remain available.
/// Keep this value alive while spawning instances. Reprepare after changing
/// the source model or its texture resolution.
#[derive(Clone)]
pub struct PreparedModel<M = Handle<Mesh>, B = Handle<SkinnedMeshInverseBindposes>> {
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
    pub(crate) geosets: Vec<PreparedGeoset<M>>,
    pub(crate) lod_levels: Vec<u32>,
    pub(crate) lod_bounds: LodBounds,
    pub(crate) geoset_animations: Vec<Option<Shared<GeosetAnimation<'static>>>>,
    pub(crate) layers: Vec<Vec<PreparedLayer>>,
    pub(crate) textures: ResolvedModelTextures,
    pub(crate) models: Wc3ModelResources,
    pub(crate) inverse_bindposes: B,
    pub(crate) joint_bindings: Vec<(u32, Mat4)>,
}

/// Owned CPU meshes and runtime definitions, ready for insertion into Bevy assets.
/// No encoded input or decoded model is retained.
pub type CompiledModel = PreparedModel<Arc<Mesh>, Arc<[Mat4]>>;

impl<M, B> PreparedModel<M, B> {
    /// Source animation metadata retained for runtime playback and inspection.
    pub fn sequences(&self) -> &[Sequence] {
        &self.sequences
    }
    /// Model-space center and conservative bounding radius.
    pub fn bounds(&self) -> (Vec3, f32) {
        (self.lod_bounds.center, self.lod_bounds.radius)
    }
    /// Authored texture slots, including paths, flags, and replacement IDs.
    pub fn texture_definitions(&self) -> &[Texture] {
        &self.texture_definitions
    }
    /// Resolved child-model slots.
    pub fn model_resources(&self) -> &Wc3ModelResources {
        &self.models
    }
    /// Sorted authored geometry levels.
    pub fn lod_levels(&self) -> &[u32] {
        &self.lod_levels
    }
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
}

/// Insert an already compiled model asset into Bevy's mesh and bind-pose stores.
/// Dependencies and runtime definitions remain shared with the compiled asset.
/// No decoding, geometry compilation, asset loading, or spawning occurs here.
pub fn prepare_model_asset(
    meshes: &mut Assets<Mesh>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    asset: &Wc3ModelAsset,
) -> PreparedModel {
    asset.compiled.prepare(meshes, inverse_bindposes)
}

/// Build shareable meshes and bind poses once. Each instance receives private
/// layer materials so its texture bindings can change independently.
///
/// The resolver receives literal bitmap paths and may return `None` for missing
/// images. Replaceable IDs are supplied when spawning or through instance texture
/// bindings. This convenience function leaves child-model resources unresolved;
/// use [`prepare_document_with_resources`] to resolve attachments and PREM models.
///
/// # Errors
///
/// Returns [`ModelError`] when geometry cannot be prepared, including unsupported
/// influence counts or unavailable texture-coordinate sets.
#[cfg(test)]
pub(crate) fn prepare_document(
    meshes: &mut Assets<Mesh>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Document,
    resolve_texture: impl FnMut(&str) -> Option<Handle<Image>>,
) -> Result<PreparedModel, ModelError> {
    prepare_document_with_resources(meshes, inverse_bindposes, source, resolve_texture, |_| None)
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
/// Returns [`ModelError`] for geometry preparation failures, as in [`prepare_document`].
#[cfg(test)]
pub(crate) fn prepare_document_with_resources(
    meshes: &mut Assets<Mesh>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Document,
    resolve_texture: impl FnMut(&str) -> Option<Handle<Image>>,
    resolve_model: impl FnMut(&str) -> Option<Handle<Wc3ModelAsset>>,
) -> Result<PreparedModel, ModelError> {
    let compiled = compile_document(source.model.as_borrowed(), resolve_texture, resolve_model)?;
    Ok(compiled.prepare(meshes, inverse_bindposes))
}

/// Consume a decoded model, resolving dependencies and compiling owned runtime data.
/// Geometry stays borrowed while meshes are built. Runtime records move owned
/// buffers and copy only borrowed buffers. No Bevy asset stores are needed.
///
///
/// # Errors
/// Returns an error for geometry that cannot be compiled, including unsupported
/// influence counts. No mesh or bind-pose assets are inserted on failure.
pub(crate) fn compile_document(
    model: Document<'_, V1800>,
    mut resolve_texture: impl FnMut(&str) -> Option<Handle<Image>>,
    resolve_model: impl FnMut(&str) -> Option<Handle<Wc3ModelAsset>>,
) -> Result<CompiledModel, ModelError> {
    let models = Wc3ModelResources::resolve_document(&model, resolve_model);
    let textures = ResolvedModelTextures {
        bitmaps: model
            .textures()
            .map(|bitmap| resolve_texture_binding(bitmap, &mut resolve_texture))
            .collect(),
        particles: model
            .particle_emitters2()
            .map(|emitter| ResolvedTexture {
                replaceable_id: emitter.replaceable_id,
                bitmap_id: (emitter.replaceable_id == 0).then_some(emitter.texture_id as usize),
                ..default()
            })
            .collect(),
    };
    compile_resolved_model(model, textures, models)
}

pub(crate) fn compile_resolved_model(
    model: Document<'_, V1800>,
    textures: ResolvedModelTextures,
    models: Wc3ModelResources,
) -> Result<CompiledModel, ModelError> {
    let default_bitmaps: Vec<_> = textures
        .bitmaps
        .iter()
        .map(|binding| binding.default.clone())
        .collect();
    let joint_ids = joint_ids(&model);
    let joint_index: HashMap<_, _> = joint_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index as u16))
        .collect();
    let material_records: Vec<_> = model.materials().collect();
    let mut geosets = Vec::new();
    for (geoset_id, geoset) in model.geosets().enumerate() {
        if geoset.vertices().is_empty() || geoset.face_indices().is_empty() {
            continue;
        }
        if material_records.get(geoset.material_id as usize).is_none() {
            continue;
        }
        let material = &material_records[geoset.material_id as usize];
        let mut variants: HashMap<u32, Arc<Mesh>> = HashMap::new();
        let mut layer_meshes = Vec::new();
        for layer in &material.layers {
            let coordinate = canonical_uv_coordinate(geoset, layer.coordinate_id);
            let handle = if let Some(handle) = variants.get(&coordinate) {
                handle.clone()
            } else {
                let mesh =
                    build_mesh_with_uv(geoset, &joint_index, !joint_ids.is_empty(), coordinate)
                        .map_err(|error| ModelError(format!("geoset {geoset_id}: {error}")))?;
                let handle = Arc::new(mesh);
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
    let geoset_count = model.geosets().count();
    let binds = inverse_bind_matrices(&model, &joint_ids);
    let lod_bounds = LodBounds::from_model(&model);
    let sequences = model.sequences().cloned().collect();
    let global_sequences = model.global_sequences();
    let blend_time = model.model_info().map_or(0, |info| info.blend_time);
    let pivots = model.pivot_points();
    let texture_definitions = model.textures().cloned().collect();
    let records = RuntimeRecords::consume(model);
    let nodes = records.nodes;
    let material_records = records.materials;
    let texture_animations = records.texture_animations;
    let mut geoset_animations = vec![None; geoset_count];
    for animation in records.geoset_animations {
        if let Some(slot) = geoset_animations.get_mut(animation.geoset_id as usize) {
            *slot = Some(animation);
        }
    }
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
    let node_order = node_order(&nodes);
    Ok(PreparedModel {
        lod_levels,
        lod_bounds,
        events: records.events.into(),
        nodes,
        node_order,
        sequences,
        global_sequences,
        blend_time,
        pivots,
        cameras: records.cameras,
        lights: records.lights,
        attachments: records.attachments,
        particles: records.particles,
        particles2: records.particles2,
        ribbons: records.ribbons,
        material_definitions: material_records,
        texture_animations,
        texture_definitions,
        geosets,
        geoset_animations,
        layers,
        textures,
        models,
        joint_bindings: joint_ids.into_iter().zip(binds.iter().copied()).collect(),
        inverse_bindposes: binds.into(),
    })
}

#[cfg(test)]
mod tests;

impl CompiledModel {
    /// Insert compiled geometry into Bevy's asset stores. Call once and reuse the
    /// returned prepared model for all instances. Runtime definitions stay shared.
    pub fn prepare(
        &self,
        meshes: &mut Assets<Mesh>,
        inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    ) -> PreparedModel {
        let mut variants = HashMap::new();
        let geosets = self
            .geosets
            .iter()
            .map(|geoset| PreparedGeoset {
                meshes: geoset
                    .meshes
                    .iter()
                    .map(|mesh| {
                        variants
                            .entry(Arc::as_ptr(mesh))
                            .or_insert_with(|| meshes.add((**mesh).clone()))
                            .clone()
                    })
                    .collect(),
                material_id: geoset.material_id,
                geoset_id: geoset.geoset_id,
                lod: geoset.lod,
            })
            .collect();
        PreparedModel {
            nodes: self.nodes.clone(),
            node_order: self.node_order.clone(),
            sequences: self.sequences.clone(),
            global_sequences: self.global_sequences.clone(),
            blend_time: self.blend_time,
            pivots: self.pivots.clone(),
            cameras: self.cameras.clone(),
            lights: self.lights.clone(),
            attachments: self.attachments.clone(),
            particles: self.particles.clone(),
            particles2: self.particles2.clone(),
            ribbons: self.ribbons.clone(),
            material_definitions: self.material_definitions.clone(),
            texture_animations: self.texture_animations.clone(),
            texture_definitions: self.texture_definitions.clone(),
            events: self.events.clone(),
            lod_levels: self.lod_levels.clone(),
            lod_bounds: self.lod_bounds,
            geoset_animations: self.geoset_animations.clone(),
            layers: self.layers.clone(),
            textures: self.textures.clone(),
            models: self.models.clone(),
            joint_bindings: self.joint_bindings.clone(),
            geosets,
            inverse_bindposes: inverse_bindposes.add(self.inverse_bindposes.to_vec()),
        }
    }
}

/// Compile the default semantic model into owned meshes and runtime definitions.
/// The current WC3 renderer lowers strictly to its V1800 runtime vocabulary.
/// Unsupported semantic data produces an error instead of being discarded.
pub fn compile_model(
    model: Model<'_, Mdlx>,
    resolve_texture: impl FnMut(&str) -> Option<Handle<Image>>,
    resolve_model: impl FnMut(&str) -> Option<Handle<Wc3ModelAsset>>,
) -> Result<CompiledModel, ModelError> {
    let document = lower_model(&model)?;
    compile_document(document, resolve_texture, resolve_model)
}

pub(crate) fn lower_model<'a>(
    model: &'a Model<'_, Mdlx>,
) -> Result<Document<'a, V1800>, ModelError> {
    semantic_mdlx::plan::<V1800>(model, &ExportOptions::default())
        .and_then(|plan| plan.execute())
        .map(|conversion| conversion.value)
        .map_err(|error| ModelError(error.to_string()))
}

/// Prepare shared Bevy assets from a semantic Warcraft III graph.
pub fn prepare_model(
    meshes: &mut Assets<Mesh>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Model,
    resolve_texture: impl FnMut(&str) -> Option<Handle<Image>>,
) -> Result<PreparedModel, ModelError> {
    prepare_model_with_resources(meshes, inverse_bindposes, source, resolve_texture, |_| None)
}

/// Prepare a semantic graph using application-defined dependency resolvers.
pub fn prepare_model_with_resources(
    meshes: &mut Assets<Mesh>,
    inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    source: &Wc3Model,
    resolve_texture: impl FnMut(&str) -> Option<Handle<Image>>,
    resolve_model: impl FnMut(&str) -> Option<Handle<Wc3ModelAsset>>,
) -> Result<PreparedModel, ModelError> {
    let document = lower_model(&source.model)?;
    let compiled = compile_document(document, resolve_texture, resolve_model)?;
    Ok(compiled.prepare(meshes, inverse_bindposes))
}
