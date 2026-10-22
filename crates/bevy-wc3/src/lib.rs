//! Render Universal semantic models through Bevy PBR.
//!
//! Add [`Wc3BevyPlugin`] beside Bevy's default plugins, then spawn
//! [`ModelInstance`] with a model asset handle. Native asset loading
//! imports MDX/MDL, M2 and WMO sources into the same semantic renderer.
//! Applications provide archive identities, external companions and image choices
//! through [`ResourceResolver`].
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_wc3::{ModelInstance, Wc3BevyPlugin};
//! fn setup(mut commands: Commands, assets: Res<AssetServer>) {
//!     commands.spawn(ModelInstance::new(assets.load("units/footman.mdx")));
//!     commands.spawn((Camera3d::default(),
//!         Transform::from_xyz(0., 350., 650.).looking_at(Vec3::ZERO, Vec3::Y)));
//!     commands.spawn(DirectionalLight::default());
//! }
//! App::new().add_plugins((DefaultPlugins, Wc3BevyPlugin))
//!     .add_systems(Startup, setup);
//! ```
//!
//! For in-memory graphs, use [`compile_model`], prepare shared assets,
//! and call [`spawn_model`]. Each instance owns its playback, rig and
//! changing material state. The compiler reports unsupported interpretations
//! through [`RenderDiagnostic`]; implemented behavior and fidelity limits are
//! described in the repository's rendering guides.
#![deny(missing_docs)]

mod animation_targets;
/// Resource-driven model assembly with application-defined appearance selection.
pub mod assembly;
mod assets;
mod baking;
mod bounds;
mod camera;
mod child_coordinates;
mod child_models;
mod compiled;
mod compiler;
mod composition;
mod coordinates;
mod diagnostics;
mod effects;
mod error;
mod events;
mod features;
mod geometry;
mod hierarchy;
mod image_replacements;
pub use image_replacements::ImageReplacements;
mod instances;
mod layers;
mod lod;
mod material;
mod mesh_animation;
pub(crate) mod mesh_attributes;
mod mesh_interner;
mod physics;
mod plugin;
mod pose;
mod runtime;
mod schedule;
mod skin_bounds;
mod textures;
mod transition;
mod world;
mod world_lighting;
/// World of Warcraft character and equipment assembly.
pub mod wow;

pub use assets::{
    AssetSettings, BlpImageLoader, ModelAsset, ModelLoader, ModelLoaderSettings, PathResolver,
    ResourceResolver,
};
pub use baking::{
    BakeSpace, BakedPart, BakedPose, BakedSource, PoseBaker, PoseOptions, PreparedPart,
};
pub use camera::CameraBinding;
pub use child_coordinates::CoordinatePolicy;
pub use child_models::{ChildModelBinding, ChildModelKey, ChildModelRole};
pub use compiled::{CompiledModel, DrawPurpose, DrawSource, PreparedModel, PreparedMount};
pub use compiler::{compile_model, compile_model_with_options, CompileOptions};
pub use composition::{DrawSelection, JointMap, ModelPart, PartBinding};
pub use diagnostics::{
    RenderDiagnostic, RenderDiagnosticKind, RenderDiagnosticSeverity, RenderDiagnosticSource,
};
pub use effects::age_sprites_gpu::ParticleRefraction;
pub use effects::{
    AdoptedParticleSource, AdoptedParticleTemplate, AdoptedTemplateStatus, ModelEmitter,
    ModelParticleLimits, ModelParticleTreeStats, ParticleEffect, ParticleProgramAttribute,
    ParticleProgramControlError, ParticleProgramControls, ParticleProgramEmitter,
    ParticleTextureBinding, ParticleTextureRequest, RibbonEmitter, SpriteEmitter,
};
pub use error::ModelError;
pub use events::{EventDispatchStats, EventProgram, EventSource, ModelEvent};
pub use features::{
    AttachmentState, ModelAttachment, ModelFeature, ModelLight, ModelLightSettings, SampledCamera,
};
pub use instances::{ModelInstance, ModelInstanceError, ModelRebind, ModelSource};
pub use lod::{Lod, LodOverride, LodSettings, LodState};
pub use material::{ModelMaterial, ModelMaterialExtension};
pub use physics::{
    PhysicsBodyInfo, PhysicsContactError, PhysicsContactIssue, PhysicsContactLimits,
    PhysicsContactPolicy, PhysicsContactStats, PhysicsFrame, PhysicsSimulation,
};
pub use plugin::Wc3BevyPlugin;
pub use runtime::{
    spawn_model, ModelDraw, ModelNode, ModelRig, Playback, PlaybackError, PoseOverrides,
};
pub use schedule::Wc3Systems;
pub use textures::{
    texture_requests, TextureColorSpace, TextureInput, TextureRequest, TextureRequestKey,
    TextureRole,
};
pub use world::{
    world_volume_weight, DoodadSet, PortalView, WorldEnvironment, WorldEnvironmentSample,
    WorldGroup, WorldLiquid, WorldLiquidSurface, WorldPortal, WorldPortalPolicy, WorldRegion,
    WorldSelection,
};

pub(crate) use camera::animate_cameras;
pub(crate) use features::restart_attachments;
pub(crate) use instances::{invalidate_models, spawn_loaded_models, PreparedModelCache};
pub(crate) use lod::update_lod;
pub(crate) use runtime::{
    advance_animation, animate_instances, apply_lod_visibility, dispatch_events, simulate_particles,
};
pub(crate) use skin_bounds::update_skin_bounds;
pub(crate) use world::apply_world_selection;

#[cfg(test)]
mod tests;
