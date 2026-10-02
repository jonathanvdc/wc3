//! Asset, material, renderer, and main-world system registration.
use bevy::asset::embedded_asset;
use bevy::prelude::*;

use crate::animation::advance_animation;
use crate::animation::pose::animate_nodes;
use crate::assets::{BlpImageLoader, Wc3ModelAsset, Wc3ModelLoader};
use crate::attachment::animate_attachments;
use crate::camera::animate_cameras;
use crate::effects::particle_emitter::{
    animate_particle_models, update_particles as update_model_particles,
};
use crate::effects::particle_emitter2::textures::update_particle_textures;
use crate::effects::particle_emitter2::{
    update_particles as update_quad_particles, ParticleRenderPlugin,
};
use crate::effects::ribbon_emitter::{update_ribbons, RibbonRenderPlugin};
use crate::event::{dispatch_events, Wc3ModelEvent};
use crate::instance::{spawn_loaded_instances, PreparedModelCache};
use crate::light::animate_lights;
use crate::materials::animation::animate_surface;
use crate::materials::layers::animate_layers;
use crate::materials::Wc3LayerMaterial;
use crate::schedule::{configure, Wc3Systems};

/// Registers model/BLP loaders, WC3 materials, animation, and effect rendering.
///
/// Add alongside Bevy's `DefaultPlugins`. Applications supply cameras, scene
/// lighting, and replaceable textures. Use [`crate::Wc3Systems`] to order systems
/// around the plugin's loading, animation, and simulation stages.
pub struct Wc3BevyPlugin;

impl Plugin for Wc3BevyPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/wc3_mesh.wgsl");
        embedded_asset!(app, "shaders/wc3_material.wgsl");
        embedded_asset!(app, "shaders/wc3_material_prepass.wgsl");
        embedded_asset!(app, "shaders/wc3_prepass.wgsl");
        embedded_asset!(app, "shaders/wc3_particle.wgsl");
        embedded_asset!(app, "shaders/wc3_ribbon.wgsl");
        app.init_asset::<Wc3ModelAsset>();
        app.init_asset_loader::<Wc3ModelLoader>();
        app.init_asset_loader::<BlpImageLoader>();
        app.init_resource::<PreparedModelCache>();
        app.add_plugins(MaterialPlugin::<Wc3LayerMaterial>::default());
        app.add_plugins(ParticleRenderPlugin);
        app.add_plugins(RibbonRenderPlugin);
        configure(app);
        app.add_message::<Wc3ModelEvent>();
        app.add_systems(
            PostUpdate,
            dispatch_events.in_set(Wc3Systems::DispatchEvents),
        );
        app.add_systems(
            Update,
            spawn_loaded_instances.in_set(Wc3Systems::SpawnInstances),
        );
        app.add_systems(
            Update,
            update_particle_textures.in_set(Wc3Systems::BindTextures),
        );
        app.add_systems(
            Update,
            advance_animation.in_set(Wc3Systems::AdvanceAnimation),
        );
        app.add_systems(
            Update,
            (
                animate_particle_models,
                animate_attachments,
                animate_lights,
                animate_layers,
                animate_surface,
            )
                .chain()
                .in_set(Wc3Systems::AnimateInstances),
        );
        app.add_systems(
            PostUpdate,
            animate_cameras.in_set(Wc3Systems::AnimateCameras),
        );
        app.add_systems(
            PostUpdate,
            animate_nodes.in_set(Wc3Systems::EvaluateNodePoses),
        );
        app.add_systems(
            PostUpdate,
            update_model_particles.in_set(Wc3Systems::SimulateModelParticles),
        );
        app.add_systems(
            PostUpdate,
            (update_quad_particles, update_ribbons).in_set(Wc3Systems::SimulateEffects),
        );
    }
}
