//! Bevy integration for Warcraft III models.
//!
//! Load an MDX or MDL through Bevy's `AssetServer` and attach `Wc3ModelInstance` to
//! each desired root entity. The plugin loads literal bitmap paths and shares
//! prepared meshes; texture bindings and materials belong to each instance.
//!
//! `Wc3Model::decode`, `prepare_model`, and `spawn_prepared_model` remain
//! available for callers with custom model or texture sources.

mod animation;
mod assets;
mod attachment;
mod camera;
mod effects;
mod instance;
mod light;
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
pub use instance::spawn::{
    spawn_model, spawn_prepared_model, spawn_prepared_model_with_bindings, Wc3NodeEntities,
};
pub use instance::{Wc3ModelInstance, Wc3ModelOwner, Wc3OwnedModels};
pub use light::{Wc3Light, Wc3LightSettings};
pub use materials::textures::{Wc3TextureBindings, Wc3TextureSlot};
pub use materials::{Wc3LayerMaterial, Wc3LayerState};
pub use plugin::Wc3BevyPlugin;
pub use preparation::{prepare_model, prepare_model_with_resources, PreparedModel};
pub use schedule::Wc3Systems;
