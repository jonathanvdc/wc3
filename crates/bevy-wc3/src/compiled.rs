//! Owned compiled definitions and shared GPU asset preparation.
use crate::diagnostics::RenderDiagnostic;
use crate::effects::RuntimeParticles;
use crate::events::RuntimeEvents;
use crate::features::RuntimeFeatures;
use crate::image_replacements::ImageBinding;
use crate::lod::LodDefinition;
use crate::material::map_material_images;
use crate::material::SurfaceAnimation;
use crate::mesh_animation::MeshMaterialState;
use crate::physics::{PhysicsBodyInfo, PhysicsDefinition};
use crate::world::WorldRuntime;
use crate::TextureRequest;
use crate::{
    DoodadSet, ModelFeature, ModelMaterial, WorldEnvironment, WorldEnvironmentSample, WorldGroup,
    WorldLiquid, WorldPortal, WorldRegion, WorldSelection,
};
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::VisibilityClass;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wc3::model::systems::model_physics::Joint;
use wc3::model::Texture;
use wc3::model::{
    AnimationProgram, ClipId, DrawId, FeatureId, FeatureKind, InstanceId, MaterialId, MeshId,
    NodeId,
};
pub(crate) type MaterialCacheKey = (MaterialId, usize, MeshMaterialState, [u32; 4]);
pub(crate) type MaterialCache = HashMap<MaterialCacheKey, Handle<ModelMaterial>>;

pub(crate) use wc3::model::PoseNode as RuntimeNode;

#[derive(Clone)]
pub(crate) struct RuntimeSkin<B> {
    pub(crate) joints: Vec<usize>,
    pub(crate) binds: B,
}

/// Authored source of a rendered geometry selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawSource {
    /// An explicitly authored instance draw.
    Draw(DrawId),
}

/// Render-view role of an authored geometry pass.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DrawPurpose {
    /// Ordinary camera color/prepasses, with material-controlled shadow casting.
    #[default]
    Surface,
    /// Authored caster geometry excluded from camera color/depth/normal/motion passes.
    ShadowOnly,
}
impl DrawPurpose {
    /// Apply this role after inserting `Mesh3d`, whose hook adds its camera class.
    ///
    /// Other visibility classes, inherited visibility and render layers are retained.
    /// Ordinary light-shadow visibility queries `Mesh3d` independently of this class.
    /// Bevy's opt-in `NoCpuCulling` bypasses this routing and is unsupported for
    /// shadow-only meshes. GPU mesh preprocessing can remain enabled.
    /// `Surface` leaves the existing classes unchanged; apply the role again if
    /// `Mesh3d` is removed and reinserted.
    pub fn configure_visibility(self, visibility: &mut VisibilityClass) {
        if self == Self::ShadowOnly {
            visibility.retain(|class| *class != TypeId::of::<Mesh3d>());
        }
    }
}
#[derive(Clone)]
pub(crate) struct RuntimeDraw<M> {
    pub(crate) instance: InstanceId,
    pub(crate) source: DrawSource,
    pub(crate) mesh_id: MeshId,
    pub(crate) material_id: MaterialId,
    pub(crate) mesh_state: MeshMaterialState,
    pub(crate) world_state: [u32; 4],
    pub(crate) pass: usize,
    pub(crate) purpose: DrawPurpose,
    pub(crate) node: usize,
    pub(crate) skin: Option<usize>,
    pub(crate) lod: Option<u32>,
    pub(crate) mesh: M,
    pub(crate) material: ModelMaterial,
    pub(crate) animation: Arc<SurfaceAnimation>,
}

#[derive(Clone)]
pub(crate) struct RuntimeLiquid<M> {
    pub(crate) instance: InstanceId,
    pub(crate) node: usize,
    pub(crate) ordinal: usize,
    pub(crate) liquid_type: u32,
    pub(crate) material_id: Option<MaterialId>,
    pub(crate) pass: usize,
    pub(crate) mesh: M,
    pub(crate) surface: Option<(ModelMaterial, Arc<SurfaceAnimation>)>,
}

#[derive(Clone)]
pub(crate) struct ClipTiming {
    pub(crate) blend_duration: Option<f64>,
}

#[derive(Clone)]
pub(crate) struct RuntimeMount<M> {
    pub(crate) parent: usize,
    pub(crate) socket: Option<FeatureId>,
    pub(crate) transform: Transform,
    pub(crate) model: M,
    pub(crate) clips: HashMap<ClipId, ClipId>,
}

/// Owned render data compiled directly from a Universal graph.
///
/// Compilation does not export through a native format or retain the source graph. It does not
/// insert Bevy assets, or require an ECS world. Geometry and runtime definitions
/// outlive borrowed source bytes. [`Self::prepare`] inserts shared GPU assets.
#[derive(Clone)]
pub struct CompiledModel {
    pub(crate) image_template: Option<Arc<CompiledModel>>,
    pub(crate) image_bindings: Arc<[ImageBinding]>,
    pub(crate) mounts: Vec<RuntimeMount<Arc<CompiledModel>>>,
    pub(crate) physics: Arc<PhysicsDefinition>,
    pub(crate) events: Arc<RuntimeEvents>,
    pub(crate) nodes: Arc<[RuntimeNode]>,
    pub(crate) order: Arc<[usize]>,
    pub(crate) animation: Arc<AnimationProgram>,
    pub(crate) clips: Arc<[(ClipId, String)]>,
    pub(crate) skins: Vec<RuntimeSkin<Arc<[Mat4]>>>,
    pub(crate) draws: Vec<RuntimeDraw<Arc<Mesh>>>,
    pub(crate) diagnostics: Arc<[RenderDiagnostic]>,
    pub(crate) rest_bounds: Option<Aabb>,
    pub(crate) basis: Transform,
    pub(crate) up: Vec3,
    pub(crate) timing: Arc<HashMap<ClipId, ClipTiming>>,
    pub(crate) blend_time: f64,
    pub(crate) lod: Arc<LodDefinition>,
    pub(crate) has_animation: bool,
    pub(crate) pose_animated: bool,
    pub(crate) camera_sensitive: bool,
    pub(crate) features: Arc<RuntimeFeatures>,
    pub(crate) particles: Arc<RuntimeParticles>,
    pub(crate) world: Arc<WorldRuntime>,
    pub(crate) liquids: Vec<RuntimeLiquid<Arc<Mesh>>>,
}

/// Borrowed independent model mounted at one node of a prepared rig.
pub struct PreparedMount<'a> {
    /// Destination authored node identity.
    pub parent: NodeId,
    /// Attachment activation gate, when mounted through a feature.
    pub socket: Option<FeatureId>,
    /// Child authored coordinates into the carrying node's frame.
    pub transform: Transform,
    /// Prepared child definitions; geometry is shared with other occurrences.
    pub model: &'a PreparedModel,
}

/// Shared Bevy meshes and immutable definitions ready for repeated instantiation.
#[derive(Clone)]
pub struct PreparedModel {
    pub(crate) image_template: Option<Arc<PreparedModel>>,
    pub(crate) image_bindings: Arc<[ImageBinding]>,
    pub(crate) mounts: Arc<[RuntimeMount<PreparedModel>]>,
    pub(crate) physics: Arc<PhysicsDefinition>,
    pub(crate) events: Arc<RuntimeEvents>,
    pub(crate) nodes: Arc<[RuntimeNode]>,
    pub(crate) order: Arc<[usize]>,
    pub(crate) animation: Arc<AnimationProgram>,
    pub(crate) clips: Arc<[(ClipId, String)]>,
    pub(crate) skins: Arc<[RuntimeSkin<Handle<SkinnedMeshInverseBindposes>>]>,
    pub(crate) draws: Arc<[RuntimeDraw<Handle<Mesh>>]>,
    pub(crate) material_cache: Arc<Mutex<MaterialCache>>,
    pub(crate) diagnostics: Arc<[RenderDiagnostic]>,
    pub(crate) rest_bounds: Option<Aabb>,
    pub(crate) basis: Transform,
    pub(crate) up: Vec3,
    pub(crate) timing: Arc<HashMap<ClipId, ClipTiming>>,
    pub(crate) blend_time: f64,
    pub(crate) lod: Arc<LodDefinition>,
    pub(crate) has_animation: bool,
    pub(crate) pose_animated: bool,
    pub(crate) camera_sensitive: bool,
    pub(crate) features: Arc<RuntimeFeatures>,
    pub(crate) particles: Arc<RuntimeParticles>,
    pub(crate) world: Arc<WorldRuntime>,
    pub(crate) liquids: Arc<[RuntimeLiquid<Handle<Mesh>>]>,
}

impl CompiledModel {
    /// Logical image requests with their source declarations and interpretations.
    /// Requests include unresolved defaults, scoped material/effect uses and
    /// unscoped animated selections. Their IDs belong to this source model.
    pub fn image_requests(&self) -> impl Iterator<Item = TextureRequest<'_, 'static>> {
        self.image_bindings.iter().map(|binding| TextureRequest {
            input: binding.key.input,
            texture: Texture {
                resource: binding.key.resource,
                color_space: binding.key.declared,
                sampler: binding.key.sampler,
            },
            resource: &binding.resource,
            role: binding.role,
            color_space: binding.key.color_space,
            sampler: binding.key.sampler,
        })
    }
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
    /// Rendering coverage, resources, and work limits with typed kinds and semantic owners.
    pub fn diagnostics(&self) -> &[RenderDiagnostic] {
        &self.diagnostics
    }

    /// Insert each shared geometry variant and inverse bind palette once.
    pub fn prepare(
        &self,
        meshes: &mut Assets<Mesh>,
        binds: &mut Assets<SkinnedMeshInverseBindposes>,
    ) -> PreparedModel {
        self.prepare_shared(meshes, binds, &mut HashMap::new(), &mut HashMap::new())
    }
    fn prepare_shared(
        &self,
        meshes: &mut Assets<Mesh>,
        binds: &mut Assets<SkinnedMeshInverseBindposes>,
        variants: &mut HashMap<*const Mesh, Handle<Mesh>>,
        palettes: &mut HashMap<(*const Mat4, usize), Handle<SkinnedMeshInverseBindposes>>,
    ) -> PreparedModel {
        PreparedModel {
            image_template: self.image_template.as_ref().map(|template| {
                Arc::new(template.prepare_shared(meshes, binds, variants, palettes))
            }),
            image_bindings: self.image_bindings.clone(),
            mounts: self
                .mounts
                .iter()
                .map(|mount| RuntimeMount {
                    parent: mount.parent,
                    socket: mount.socket,
                    transform: mount.transform,
                    model: mount
                        .model
                        .prepare_shared(meshes, binds, variants, palettes),
                    clips: mount.clips.clone(),
                })
                .collect::<Vec<_>>()
                .into(),
            physics: self.physics.clone(),
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
                    binds: palettes
                        .entry((skin.binds.as_ptr(), skin.binds.len()))
                        .or_insert_with(|| binds.add(skin.binds.to_vec()))
                        .clone(),
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
                    world_state: draw.world_state,
                    pass: draw.pass,
                    purpose: draw.purpose,
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

impl PreparedModel {
    /// Owned authored physics bodies available for optional secondary motion.
    pub fn physics_bodies(&self) -> impl Iterator<Item = &PhysicsBodyInfo> {
        self.physics.bodies()
    }

    /// Authored constraints available to external physics adapters.
    pub fn physics_joints(&self) -> impl Iterator<Item = &Joint> {
        self.physics.joints()
    }
    /// Inspect rigid component definitions without exposing runtime storage.
    pub fn mounts(&self) -> impl ExactSizeIterator<Item = PreparedMount<'_>> {
        self.mounts.iter().map(|mount| PreparedMount {
            parent: self.nodes[mount.parent].id,
            socket: mount.socket,
            transform: mount.transform,
            model: &mount.model,
        })
    }
    /// Number of rigid component models mounted beneath this rig.
    pub fn mounted_models(&self) -> usize {
        self.mounts.len()
    }

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
    /// Rendering coverage, resources, and work limits with typed kinds and semantic owners.
    pub fn diagnostics(&self) -> &[RenderDiagnostic] {
        &self.diagnostics
    }
    /// Static-world groups in authored model coordinates.
    pub fn world_groups(&self) -> &[WorldGroup] {
        &self.world.groups
    }
    /// Authored portal polygons and plane equations.
    pub fn world_portals(&self) -> &[WorldPortal] {
        &self.world.portals
    }
    /// External placement selection sets.
    pub fn doodad_sets(&self) -> &[DoodadSet] {
        &self.world.doodad_sets
    }
    /// Region-specific integration data.
    pub fn world_regions(&self) -> &[WorldRegion] {
        &self.world.regions
    }
    /// Authored liquid grids and external liquid-type identities.
    pub fn world_liquids(&self) -> &[WorldLiquid] {
        &self.world.liquid_definitions
    }
    /// Environmental volumes, without changing application-owned lighting or fog.
    pub fn world_environments(&self) -> &[WorldEnvironment] {
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
        selection: &WorldSelection,
        region: Option<InstanceId>,
    ) -> WorldEnvironmentSample {
        self.world.environment_at(position, selection, region)
    }
    /// Playback clips in stable source identity order.
    pub fn clips(&self) -> &[(ClipId, String)] {
        &self.clips
    }
    /// Authored integration points, including features without a visual interpreter.
    pub fn features(&self) -> &[(ModelFeature, FeatureKind)] {
        self.features.hooks()
    }
    /// Cameras available for opt-in playback on application-owned views.
    pub fn cameras(&self) -> impl Iterator<Item = ModelFeature> + '_ {
        self.features.camera_ids()
    }
}

impl CompiledModel {
    pub(crate) fn map_images(
        &mut self,
        map: &mut impl FnMut(&Handle<Image>) -> Option<Handle<Image>>,
    ) {
        for draw in &mut self.draws {
            map_material_images(&mut draw.material, map);
            Arc::make_mut(&mut draw.animation).map_images(map);
        }
        Arc::make_mut(&mut self.particles).map_images(map);
        for liquid in &mut self.liquids {
            if let Some((material, animation)) = &mut liquid.surface {
                map_material_images(material, map);
                Arc::make_mut(animation).map_images(map);
            }
        }
    }
}
impl PreparedModel {
    pub(crate) fn map_images(
        &mut self,
        map: &mut impl FnMut(&Handle<Image>) -> Option<Handle<Image>>,
    ) {
        for draw in Arc::make_mut(&mut self.draws) {
            map_material_images(&mut draw.material, map);
            Arc::make_mut(&mut draw.animation).map_images(map);
        }
        Arc::make_mut(&mut self.particles).map_images(map);
        for liquid in Arc::make_mut(&mut self.liquids) {
            if let Some((material, animation)) = &mut liquid.surface {
                map_material_images(material, map);
                Arc::make_mut(animation).map_images(map);
            }
        }
        self.material_cache = Arc::new(Mutex::new(HashMap::new()));
    }
}
