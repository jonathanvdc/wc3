//! Bevy integration for Warcraft III models.
//!
//! Load an MDX or MDL through Bevy's `AssetServer` and attach `Wc3ModelInstance` to
//! each desired root entity. The plugin loads literal bitmap paths and shares
//! prepared meshes; texture bindings and materials belong to each instance.
//!
//! `Wc3Model::decode`, `prepare_model`, and `spawn_prepared_model` remain
//! available for callers with custom model or texture sources.

mod animation;
mod asset;
mod instance;
mod material;
mod mesh;
mod model;
mod model_resources;
mod particle_emitter2;
mod spawn;
mod texture_bindings;

use bevy::asset::embedded_asset;
use bevy::prelude::*;

pub use animation::Wc3Animation;
pub use asset::Wc3ModelAsset;
pub use instance::{Wc3ModelInstance, Wc3ModelOwner, Wc3OwnedModels};
pub use material::{Wc3LayerMaterial, Wc3LayerState};
pub use model::{ModelError, Wc3Model};
pub use model_resources::Wc3ModelResources;
pub use spawn::{
    prepare_model, prepare_model_with_resources, spawn_model, spawn_prepared_model,
    spawn_prepared_model_with_bindings, PreparedModel, Wc3NodeEntities,
};
pub use texture_bindings::{Wc3TextureBindings, Wc3TextureSlot};

pub struct Wc3BevyPlugin;

impl Plugin for Wc3BevyPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/wc3_mesh.wgsl");
        embedded_asset!(app, "shaders/wc3_prepass.wgsl");
        embedded_asset!(app, "shaders/wc3_particle.wgsl");
        app.init_asset::<Wc3ModelAsset>();
        app.init_asset_loader::<asset::Wc3ModelLoader>();
        app.init_asset_loader::<asset::BlpImageLoader>();
        app.init_resource::<instance::PreparedModelCache>();
        app.add_plugins(MaterialPlugin::<Wc3LayerMaterial>::default());
        app.add_plugins(particle_emitter2::ParticleRenderPlugin);
        app.add_systems(Update, instance::spawn_loaded_instances);
        app.add_systems(
            Update,
            texture_bindings::update_particle_textures.after(instance::spawn_loaded_instances),
        );
        app.add_systems(
            Update,
            (
                animation::advance_animation,
                animation::animate_nodes,
                animation::animate_layers,
            )
                .chain()
                .after(instance::spawn_loaded_instances),
        );
        app.add_systems(
            PostUpdate,
            particle_emitter2::update_particles.after(bevy::transform::TransformSystems::Propagate),
        );
    }
}
