//! Render Universal semantic models through Bevy PBR.
//!
//! Add [`Wc3BevyPlugin`] beside Bevy's default plugins, then spawn
//! [`UniversalModelInstance`] with a model asset handle. Native asset loading
//! imports MDX/MDL, M2, WMO, M3 and Diablo sources into the same semantic renderer.
//! Applications provide archive identities, external companions and image choices
//! through [`UniversalResourceResolver`].
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_wc3::{UniversalModelInstance, Wc3BevyPlugin};
//! fn setup(mut commands: Commands, assets: Res<AssetServer>) {
//!     commands.spawn(UniversalModelInstance::new(assets.load("units/footman.mdx")));
//!     commands.spawn((Camera3d::default(),
//!         Transform::from_xyz(0., 350., 650.).looking_at(Vec3::ZERO, Vec3::Y)));
//!     commands.spawn(DirectionalLight::default());
//! }
//! App::new().add_plugins((DefaultPlugins, Wc3BevyPlugin))
//!     .add_systems(Startup, setup);
//! ```
//!
//! For in-memory graphs, use [`compile_universal_model`], prepare shared assets,
//! and call [`spawn_universal_model`]. Each instance owns its playback, rig and
//! changing material state. The compiler reports unsupported interpretations
//! through [`RenderDiagnostic`]; implemented behavior and fidelity limits are
//! described in the repository's rendering guides.
#![deny(missing_docs)]

mod assets;
mod effects;
mod error;
mod plugin;
mod schedule;
mod universal;

pub use universal::{
    compile_universal_model, compile_universal_model_with_options, spawn_universal_model,
    texture_requests, world_volume_weight, CompiledUniversalModel, ModelPlacementSpace,
    PreparedUniversalModel, RenderDiagnostic, SampledUniversalCamera, TextureColorSpace,
    TextureImageKey, TextureRequest, TextureRole, UniversalAnimationError, UniversalAssetSettings,
    UniversalAttachmentPoint, UniversalBakeSpace, UniversalBakedPart, UniversalBakedPose,
    UniversalBakedSource, UniversalCameraBinding, UniversalCompileOptions,
    UniversalControllerSelection, UniversalDoodadSet, UniversalDraw, UniversalDrawSource,
    UniversalEventDispatchStats, UniversalEventProgram, UniversalEventSource, UniversalFeature,
    UniversalInstance, UniversalLight, UniversalLightSettings, UniversalLod, UniversalLodOverride,
    UniversalLodSettings, UniversalLodState, UniversalMaterial, UniversalMaterialExtension,
    UniversalModelAsset, UniversalModelEvent, UniversalModelInstance, UniversalModelInstanceError,
    UniversalModelLoader, UniversalModelLoaderSettings, UniversalModelParticleEmitter,
    UniversalModelRebind, UniversalModelResourcePurpose, UniversalModelSource, UniversalNode,
    UniversalParticleEmitter, UniversalPathResolver, UniversalPlacementAppearance,
    UniversalPlayback, UniversalPoseBaker, UniversalPoseOptions, UniversalPoseOverrides,
    UniversalPreparedPart, UniversalResourceResolver, UniversalRibbonEmitter,
    UniversalWorldEnvironment, UniversalWorldEnvironmentSample, UniversalWorldGroup,
    UniversalWorldLiquid, UniversalWorldLiquidSurface, UniversalWorldPortal, UniversalWorldRegion,
    UniversalWorldSelection, WorldPortalPolicy,
};

pub use assets::BlpImageLoader;
pub use error::ModelError;
pub use plugin::Wc3BevyPlugin;
pub use schedule::Wc3Systems;
