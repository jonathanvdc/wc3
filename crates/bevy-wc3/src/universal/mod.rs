//! Direct compilation and instantiation of the universal semantic graph.
mod compiler;
mod coordinates;
pub use compiler::compile_universal_model;
mod assets;
mod atlas_particles;
mod features;
mod geometry;
mod material;
mod textures;
pub use textures::{
    texture_requests, TextureColorSpace, TextureImageKey, TextureRequest, TextureRole,
};
mod mesh_animation;
use mesh_animation::MeshMaterialState;
pub(crate) mod mesh_attributes;
mod particles;
mod runtime;
use particles::RuntimeParticles;
pub use particles::UniversalParticleEmitter;
mod instances;

pub(crate) use instances::{
    invalidate_universal_models, spawn_loaded_universal_models, UniversalPreparedCache,
};
pub use instances::{UniversalModelInstance, UniversalModelSource};

use features::RuntimeFeatures;
pub use features::{
    SampledUniversalCamera, UniversalAttachmentPoint, UniversalFeature, UniversalLight,
};

pub use assets::{
    UniversalAssetSettings, UniversalModelAsset, UniversalModelLoader, UniversalPathResolver,
    UniversalResourceResolver,
};

use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use material::SurfaceAnimation;
pub use material::{UniversalMaterial, UniversalMaterialExtension};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wc3::model::{
    AnimationProgram, Billboard, ClipId, DrawId, FeatureId, FeatureKind, Inheritance, InstanceId,
    MaterialId, MeshId, NodeId, Transform as ModelTransform,
};

pub(crate) use runtime::{
    advance_universal_animation, animate_universal_instances, simulate_universal_particles,
};
pub use runtime::{
    spawn_universal_model, UniversalAnimationError, UniversalDraw, UniversalInstance,
    UniversalNode, UniversalPlayback, UniversalPoseOverrides,
};

/// An observable limitation attached to its semantic source identity.
#[derive(Clone, Debug)]
pub struct RenderDiagnostic {
    /// Source object when this diagnostic concerns an attached feature.
    pub feature: Option<FeatureId>,
    /// Source material when this diagnostic concerns a surface.
    pub material: Option<MaterialId>,
    /// Implemented approximation or unresolved rendering behavior.
    pub message: String,
}

type MaterialCacheKey = (MaterialId, usize, MeshMaterialState);
type MaterialCache = HashMap<MaterialCacheKey, Handle<UniversalMaterial>>;

#[derive(Clone)]
struct RuntimeNode {
    id: NodeId,
    name: String,
    parent: Option<usize>,
    rest: ModelTransform,
    inheritance: Inheritance,
    billboard: Billboard,
    visible: bool,
    camera_anchored: bool,
}

#[derive(Clone)]
struct RuntimeSkin<B> {
    joints: Vec<usize>,
    binds: B,
}

#[derive(Clone)]
struct RuntimeDraw<M> {
    instance: InstanceId,
    draw: DrawId,
    mesh_id: MeshId,
    material_id: MaterialId,
    mesh_state: MeshMaterialState,
    pass: usize,
    node: usize,
    skin: Option<usize>,
    lod: Option<u32>,
    mesh: M,
    material: UniversalMaterial,
    animation: Arc<SurfaceAnimation>,
}

#[derive(Clone, Copy)]
struct ClipTiming {
    origin: f64,
    duration: f64,
    looping: bool,
}

/// Owned render data compiled directly from a Universal graph.
///
/// Compilation does not export through a native format, retain the source graph,
/// insert Bevy assets, or require an ECS world. Geometry and runtime definitions
/// outlive borrowed source bytes. [`Self::prepare`] inserts shared GPU assets.
#[derive(Clone)]
pub struct CompiledUniversalModel {
    nodes: Arc<[RuntimeNode]>,
    order: Arc<[usize]>,
    animation: Arc<AnimationProgram>,
    clips: Arc<[(ClipId, String)]>,
    skins: Vec<RuntimeSkin<Arc<[Mat4]>>>,
    draws: Vec<RuntimeDraw<Arc<Mesh>>>,
    diagnostics: Arc<[RenderDiagnostic]>,
    basis: Transform,
    up: Vec3,
    timing: Arc<HashMap<ClipId, ClipTiming>>,
    pose_animated: bool,
    camera_sensitive: bool,
    features: Arc<RuntimeFeatures>,
    particles: Arc<RuntimeParticles>,
}

/// Shared Bevy meshes and immutable definitions ready for repeated instantiation.
#[derive(Clone)]
pub struct PreparedUniversalModel {
    nodes: Arc<[RuntimeNode]>,
    order: Arc<[usize]>,
    animation: Arc<AnimationProgram>,
    clips: Arc<[(ClipId, String)]>,
    skins: Arc<[RuntimeSkin<Handle<SkinnedMeshInverseBindposes>>]>,
    draws: Arc<[RuntimeDraw<Handle<Mesh>>]>,
    material_cache: Arc<Mutex<MaterialCache>>,
    diagnostics: Arc<[RenderDiagnostic]>,
    basis: Transform,
    up: Vec3,
    timing: Arc<HashMap<ClipId, ClipTiming>>,
    pose_animated: bool,
    camera_sensitive: bool,
    features: Arc<RuntimeFeatures>,
    particles: Arc<RuntimeParticles>,
}

impl CompiledUniversalModel {
    /// Rendering approximations and unresolved features, with semantic provenance.
    pub fn diagnostics(&self) -> &[RenderDiagnostic] {
        &self.diagnostics
    }

    /// Insert each shared geometry variant and inverse bind palette once.
    pub fn prepare(
        &self,
        meshes: &mut Assets<Mesh>,
        binds: &mut Assets<SkinnedMeshInverseBindposes>,
    ) -> PreparedUniversalModel {
        let mut variants = HashMap::new();
        PreparedUniversalModel {
            nodes: self.nodes.clone(),
            order: self.order.clone(),
            animation: self.animation.clone(),
            clips: self.clips.clone(),
            diagnostics: self.diagnostics.clone(),
            basis: self.basis,
            up: self.up,
            timing: self.timing.clone(),
            pose_animated: self.pose_animated,
            camera_sensitive: self.camera_sensitive,
            features: self.features.clone(),
            particles: self.particles.clone(),
            material_cache: Arc::new(Mutex::new(HashMap::new())),
            skins: self
                .skins
                .iter()
                .map(|skin| RuntimeSkin {
                    joints: skin.joints.clone(),
                    binds: binds.add(skin.binds.to_vec()),
                })
                .collect::<Vec<_>>()
                .into(),
            draws: self
                .draws
                .iter()
                .map(|draw| RuntimeDraw {
                    instance: draw.instance,
                    draw: draw.draw,
                    mesh_id: draw.mesh_id,
                    material_id: draw.material_id,
                    mesh_state: draw.mesh_state,
                    pass: draw.pass,
                    node: draw.node,
                    skin: draw.skin,
                    lod: draw.lod,
                    mesh: variants
                        .entry(Arc::as_ptr(&draw.mesh))
                        .or_insert_with(|| meshes.add((*draw.mesh).clone()))
                        .clone(),
                    material: draw.material.clone(),
                    animation: draw.animation.clone(),
                })
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

impl PreparedUniversalModel {
    /// Rendering approximations and unresolved features, with semantic provenance.
    pub fn diagnostics(&self) -> &[RenderDiagnostic] {
        &self.diagnostics
    }
    /// Playback clips in stable source identity order.
    pub fn clips(&self) -> &[(ClipId, String)] {
        &self.clips
    }
    /// Authored integration points, including features without a visual interpreter.
    pub fn features(&self) -> &[(UniversalFeature, FeatureKind)] {
        self.features.hooks()
    }
    /// Cameras available for opt-in playback on application-owned views.
    pub fn cameras(&self) -> impl Iterator<Item = UniversalFeature> + '_ {
        self.features.camera_ids()
    }
}

#[cfg(test)]
mod tests;
