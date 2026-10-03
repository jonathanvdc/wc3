//! Load, animate, and render Warcraft III MDX and MDL models in Bevy.
//!
//! # Load a model
//!
//! Register [`Wc3BevyPlugin`] alongside Bevy's default plugins, then attach
//! [`Wc3ModelInstance`] to each desired root. Instances are populated after their
//! assets load. Geometry and bind poses are shared; animation and materials are
//! private to each instance.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_wc3::{Wc3BevyPlugin, Wc3ModelInstance};
//!
//! fn spawn_model(mut commands: Commands, assets: Res<AssetServer>) {
//!     commands.spawn((
//!         Wc3ModelInstance::new(assets.load("units/footman.mdx")),
//!         Transform::default(),
//!     ));
//! }
//!
//! App::new()
//!     .add_plugins((DefaultPlugins, Wc3BevyPlugin))
//!     .add_systems(Startup, spawn_model);
//! ```
//!
//! Supply a Bevy camera and scene lighting to render the model. Bitmap paths
//! resolve beside the model first, then from the asset root, trying the literal
//! filename and then BLP, DDS, PNG, and TGA extensions at each location. BLP decoding is
//! included. [`Wc3TextureBindings`] supplies replaceable images and slot overrides.
//!
//! # Work with an instance
//!
//! Query [`Wc3Animation`] on the root to select sequences, blend poses, pause,
//! change speed, or seek. [`Wc3Attachments`] exposes attachment mounts;
//! [`Wc3ModelOwner`] ties detached child lifetimes to their owner. Model particles,
//! quad particles, and ribbons follow instance playback and ownership.
//!
//! [`Wc3Lod`] selects authored geometry levels. [`Wc3LodSettings`] provides global
//! quality presets and automatic selection controls; [`Wc3LodOverride`] supplies
//! instance overrides and [`Wc3LodState`] exposes the selected level.
//!
//! [`Wc3LightSettings`] configures imported scene lights. [`Wc3ModelCameras`]
//! exposes authored views, and [`Wc3CameraBinding`] plays them on an existing
//! camera. Read [`Wc3ModelEvent`] messages to interpret model event names in your
//! application. Use [`Wc3Systems`] to order application systems around these stages.
//!
//! # Custom sources
//!
//! Decode bytes with [`Wc3Model::decode`], prepare shared assets with
//! [`prepare_model`] or [`prepare_model_with_resources`], and spawn instances with
//! [`spawn_prepared_model`] or [`spawn_prepared_model_with_bindings`]. Resolvers
//! supply image and child-model handles without requiring file-backed sources.
//!
//! # Rendering semantics
//!
//! Meshes integrate with Bevy PBR and WC3 layer blend/depth states. Reforged
//! DefaultUnit texture roles, geoset tint/alpha, UV animation, and node flags are
//! evaluated by the renderer. Game-specific texture selection belongs to the
//! application. Lighting and pass ordering use Bevy semantics; exact Warcraft III
//! appearance is unverified. The repository's `docs/` guides explain the renderer
//! and feature-specific limits.

mod animation;
mod assets;
mod attachment;
mod camera;
mod effects;
mod event;
mod instance;
mod light;
mod lod;
mod materials;
mod plugin;
mod preparation;
mod schedule;

pub use animation::pose::{Wc3DefaultNodeCamera, Wc3NodeCamera};
pub use animation::Wc3Animation;
pub use assets::{
    BlpImageLoader, ModelError, Wc3Model, Wc3ModelAsset, Wc3ModelLoader, Wc3ModelResources,
};
pub use attachment::{Wc3AttachmentPoint, Wc3Attachments};
pub use camera::{Wc3CameraBinding, Wc3CameraSample, Wc3ModelCameras};
pub use event::Wc3ModelEvent;
pub use instance::spawn::{
    spawn_model, spawn_prepared_model, spawn_prepared_model_with_bindings, Wc3NodeEntities,
};
pub use instance::{Wc3ModelInstance, Wc3ModelOwner, Wc3OwnedModels};
pub use light::{Wc3Light, Wc3LightSettings};
pub use lod::{Wc3Lod, Wc3LodOverride, Wc3LodSettings, Wc3LodState};
pub use materials::textures::{Wc3TextureBindings, Wc3TextureSlot};
pub use materials::{Wc3LayerMaterial, Wc3LayerState};
pub use plugin::Wc3BevyPlugin;
pub use preparation::{prepare_model, prepare_model_with_resources, PreparedModel};
pub use schedule::Wc3Systems;
