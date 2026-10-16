//! Direct compilation and instantiation of the universal semantic graph.
mod baking;
pub use baking::{
    UniversalBakeSpace, UniversalBakedPart, UniversalBakedPose, UniversalBakedSource,
    UniversalPoseBaker, UniversalPoseOptions, UniversalPreparedPart,
};
mod camera;
mod hierarchy;
pub(crate) use camera::animate_universal_cameras;
pub use camera::UniversalCameraBinding;
mod bounds;
mod skin_bounds;
pub(crate) use skin_bounds::update_skin_bounds;
mod compiler;
use bevy::camera::primitives::Aabb;
mod coordinates;
pub use compiler::{
    compile_universal_model, compile_universal_model_with_options, UniversalCompileOptions,
};
mod assets;
mod effects;
mod events;
use events::RuntimeEvents;
pub use events::{
    UniversalEventDispatchStats, UniversalEventProgram, UniversalEventSource, UniversalModelEvent,
};
mod features;
mod geometry;
mod material;
mod mesh_interner;
mod textures;
pub use textures::{
    texture_requests, TextureColorSpace, TextureImageKey, TextureRequest, TextureRole,
};
mod mesh_animation;
use mesh_animation::MeshMaterialState;
pub(crate) mod mesh_attributes;
pub(crate) use effects::synchronize_model_particles;
pub use effects::UniversalModelParticleEmitter;
mod lod;
mod pose;
mod transition;
pub(crate) use lod::update_universal_lod;
use lod::LodDefinition;
pub use lod::{UniversalLod, UniversalLodOverride, UniversalLodSettings, UniversalLodState};
mod runtime;
mod world;
use effects::RuntimeParticles;
pub use effects::UniversalParticleEmitter;
pub use effects::UniversalRibbonEmitter;
pub(crate) use world::apply_universal_world_selection;
use world::WorldRuntime;
pub use world::{
    world_volume_weight, UniversalDoodadSet, UniversalWorldEnvironment,
    UniversalWorldEnvironmentSample, UniversalWorldGroup, UniversalWorldLiquid,
    UniversalWorldLiquidSurface, UniversalWorldPortal, UniversalWorldRegion,
    UniversalWorldSelection, WorldPortalPolicy,
};
mod placements;
pub use placements::ModelPlacementSpace;
mod instances;

pub(crate) use instances::{
    invalidate_universal_models, spawn_loaded_universal_models, UniversalPreparedCache,
};
pub use instances::{
    UniversalModelInstance, UniversalModelInstanceError, UniversalModelRebind, UniversalModelSource,
};

pub(crate) use features::restart_universal_attachments;
use features::RuntimeFeatures;
pub use features::{
    SampledUniversalCamera, UniversalAttachmentPoint, UniversalFeature, UniversalLight,
    UniversalLightSettings, UniversalPlacementAppearance,
};

pub use assets::{
    UniversalAssetSettings, UniversalModelAsset, UniversalModelLoader,
    UniversalModelLoaderSettings, UniversalModelResourcePurpose, UniversalPathResolver,
    UniversalResourceResolver,
};

use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use material::SurfaceAnimation;
pub use material::{UniversalMaterial, UniversalMaterialExtension};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wc3::model::{
    AnimationProgram, Billboard, ClipId, ControllerId, DrawId, FeatureId, FeatureKind, Inheritance,
    InstanceId, MaterialId, MeshId, NodeId, Transform as ModelTransform,
};

pub(crate) use runtime::apply_universal_lod_visibility;
pub(crate) use runtime::{
    advance_universal_animation, animate_universal_instances, dispatch_universal_events,
    simulate_universal_particles,
};
pub use runtime::{
    spawn_universal_model, UniversalAnimationError, UniversalControllerSelection, UniversalDraw,
    UniversalInstance, UniversalNode, UniversalPlayback, UniversalPoseOverrides,
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
    billboard_convention: pose::BillboardConvention,
    visible: bool,
    camera_anchored: bool,
}

#[derive(Clone)]
struct RuntimeSkin<B> {
    joints: Vec<usize>,
    binds: B,
}

/// Authored source of a rendered geometry selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UniversalDrawSource {
    /// An explicitly authored instance draw.
    Draw(DrawId),
    /// The complete primary geometry of a slot-bound Diablo instance.
    WholeInstance,
}

#[derive(Clone)]
struct RuntimeDraw<M> {
    instance: InstanceId,
    source: UniversalDrawSource,
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

#[derive(Clone)]
struct RuntimeLiquid<M> {
    instance: InstanceId,
    node: usize,
    ordinal: usize,
    liquid_type: u32,
    material_id: Option<MaterialId>,
    pass: usize,
    mesh: M,
    surface: Option<(UniversalMaterial, Arc<SurfaceAnimation>)>,
}

#[derive(Clone)]
struct ClipTiming {
    origin: f64,
    duration: f64,
    looping: bool,
    controllers: Arc<[ControllerId]>,
    blend_duration: Option<f64>,
}

/// Owned render data compiled directly from a Universal graph.
///
/// Compilation does not export through a native format, retain the source graph,
/// insert Bevy assets, or require an ECS world. Geometry and runtime definitions
/// outlive borrowed source bytes. [`Self::prepare`] inserts shared GPU assets.
#[derive(Clone)]
pub struct CompiledUniversalModel {
    events: Arc<RuntimeEvents>,
    nodes: Arc<[RuntimeNode]>,
    order: Arc<[usize]>,
    animation: Arc<AnimationProgram>,
    clips: Arc<[(ClipId, String)]>,
    skins: Vec<RuntimeSkin<Arc<[Mat4]>>>,
    draws: Vec<RuntimeDraw<Arc<Mesh>>>,
    diagnostics: Arc<[RenderDiagnostic]>,
    rest_bounds: Option<Aabb>,
    basis: Transform,
    up: Vec3,
    timing: Arc<HashMap<ClipId, ClipTiming>>,
    blend_time: f64,
    lod: Arc<LodDefinition>,
    has_animation: bool,
    pose_animated: bool,
    camera_sensitive: bool,
    features: Arc<RuntimeFeatures>,
    particles: Arc<RuntimeParticles>,
    world: Arc<WorldRuntime>,
    liquids: Vec<RuntimeLiquid<Arc<Mesh>>>,
}

/// Shared Bevy meshes and immutable definitions ready for repeated instantiation.
#[derive(Clone)]
pub struct PreparedUniversalModel {
    events: Arc<RuntimeEvents>,
    nodes: Arc<[RuntimeNode]>,
    order: Arc<[usize]>,
    animation: Arc<AnimationProgram>,
    clips: Arc<[(ClipId, String)]>,
    skins: Arc<[RuntimeSkin<Handle<SkinnedMeshInverseBindposes>>]>,
    draws: Arc<[RuntimeDraw<Handle<Mesh>>]>,
    material_cache: Arc<Mutex<MaterialCache>>,
    diagnostics: Arc<[RenderDiagnostic]>,
    rest_bounds: Option<Aabb>,
    basis: Transform,
    up: Vec3,
    timing: Arc<HashMap<ClipId, ClipTiming>>,
    blend_time: f64,
    lod: Arc<LodDefinition>,
    has_animation: bool,
    pose_animated: bool,
    camera_sensitive: bool,
    features: Arc<RuntimeFeatures>,
    particles: Arc<RuntimeParticles>,
    world: Arc<WorldRuntime>,
    liquids: Arc<[RuntimeLiquid<Handle<Mesh>>]>,
}

impl CompiledUniversalModel {
    /// Playback clips in stable source identity order.
    pub fn clips(&self) -> &[(ClipId, String)] {
        &self.clips
    }

    /// Bounds in application-root Bevy coordinates, including frame conversion.
    /// Combines authored bounds with rest geometry, skins and liquids across all
    /// detail levels. Animated poses, billboards and effect/child envelopes can
    /// extend beyond these bounds; this is not a runtime culling guarantee.
    pub fn rest_bounds(&self) -> Option<Aabb> {
        self.rest_bounds
    }
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
            events: self.events.clone(),
            nodes: self.nodes.clone(),
            order: self.order.clone(),
            animation: self.animation.clone(),
            clips: self.clips.clone(),
            diagnostics: self.diagnostics.clone(),
            rest_bounds: self.rest_bounds,
            basis: self.basis,
            up: self.up,
            timing: self.timing.clone(),
            blend_time: self.blend_time,
            lod: self.lod.clone(),
            has_animation: self.has_animation,
            pose_animated: self.pose_animated,
            camera_sensitive: self.camera_sensitive,
            features: self.features.clone(),
            particles: self.particles.clone(),
            world: self.world.clone(),
            liquids: self
                .liquids
                .iter()
                .map(|liquid| RuntimeLiquid {
                    instance: liquid.instance,
                    node: liquid.node,
                    ordinal: liquid.ordinal,
                    liquid_type: liquid.liquid_type,
                    material_id: liquid.material_id,
                    pass: liquid.pass,
                    mesh: variants
                        .entry(Arc::as_ptr(&liquid.mesh))
                        .or_insert_with(|| meshes.add((*liquid.mesh).clone()))
                        .clone(),
                    surface: liquid.surface.clone(),
                })
                .collect::<Vec<_>>()
                .into(),
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
                    source: draw.source,
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
    /// Authored coordinate frame and units mapped into the application's Bevy frame.
    pub fn model_to_bevy(&self) -> Transform {
        self.basis
    }

    /// Bounds in application-root Bevy coordinates, including frame conversion.
    /// Combines authored bounds with rest geometry, skins and liquids across all
    /// detail levels. Animated poses, billboards and effect/child envelopes can
    /// extend beyond these bounds; this is not a runtime culling guarantee.
    pub fn rest_bounds(&self) -> Option<Aabb> {
        self.rest_bounds
    }
    /// Rendering approximations and unresolved features, with semantic provenance.
    pub fn diagnostics(&self) -> &[RenderDiagnostic] {
        &self.diagnostics
    }
    /// Static-world groups in authored model coordinates.
    pub fn world_groups(&self) -> &[UniversalWorldGroup] {
        &self.world.groups
    }
    /// Authored portal polygons and plane equations.
    pub fn world_portals(&self) -> &[UniversalWorldPortal] {
        &self.world.portals
    }
    /// External placement selection sets.
    pub fn doodad_sets(&self) -> &[UniversalDoodadSet] {
        &self.world.doodad_sets
    }
    /// Region-specific integration data.
    pub fn world_regions(&self) -> &[UniversalWorldRegion] {
        &self.world.regions
    }
    /// Authored liquid grids and external liquid-type identities.
    pub fn world_liquids(&self) -> &[UniversalWorldLiquid] {
        &self.world.liquid_definitions
    }
    /// Environmental volumes, without changing application-owned lighting or fog.
    pub fn world_environments(&self) -> &[UniversalWorldEnvironment] {
        &self.world.environments
    }
    /// Model's authored ambient color, if present.
    pub fn world_ambient(&self) -> Option<[u8; 4]> {
        self.world.ambient
    }
    /// Sample environment volumes at a point in model coordinates.
    pub fn world_environment_at(
        &self,
        position: Vec3,
        selection: &UniversalWorldSelection,
        region: Option<InstanceId>,
    ) -> UniversalWorldEnvironmentSample {
        self.world.environment_at(position, selection, region)
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
