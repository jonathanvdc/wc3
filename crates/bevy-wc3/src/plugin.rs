//! Registration and scheduling for the Universal renderer.
use crate::assets::BlpImageLoader;
use crate::effects::age_sprites_gpu::AgeSpriteRenderPlugin;
use crate::effects::model_ownership::ModelParticleBirths;
use crate::effects::render::EffectGeometryPlugin;
use crate::effects::sprites::ParticleRenderPlugin;
use crate::effects::trails::RibbonRenderPlugin;
use crate::image_replacements::{refresh_image_replacements, ReplacementImages};
use crate::layers::synchronize_model_layers;
use crate::runtime::{
    publish_affine_poses, simulate_model_particles, synchronize_model_cameras,
    synchronize_mounted_models,
};
use crate::schedule::{configure, Wc3Systems};
use crate::{
    advance_animation, animate_cameras, animate_instances, apply_lod_visibility,
    apply_world_selection, dispatch_events, invalidate_models, restart_attachments,
    simulate_particles, spawn_loaded_models, update_lod, update_skin_bounds, AssetSettings,
    LodSettings, ModelAsset, ModelEvent, ModelLoader, ModelMaterial, PreparedModelCache,
};
use bevy::asset::embedded_asset;
use bevy::camera::visibility::VisibilitySystems;
use bevy::prelude::*;

/// Registers native model and BLP loaders, Universal PBR materials and effects.
/// Add beside Bevy's `DefaultPlugins`; applications supply cameras and lighting.
/// [`Wc3Systems`] exposes scheduling boundaries for application integration.
pub struct Wc3BevyPlugin;
impl Plugin for Wc3BevyPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/wc3_mesh.wgsl");
        embedded_asset!(app, "shaders/extra_skinning.wgsl");
        embedded_asset!(app, "shaders/wc3_prepass.wgsl");
        embedded_asset!(app, "shaders/material.wgsl");
        embedded_asset!(app, "shaders/combiner.wgsl");
        embedded_asset!(app, "shaders/surface_images.wgsl");
        embedded_asset!(app, "shaders/world_composition.wgsl");
        embedded_asset!(app, "shaders/model_io.wgsl");
        embedded_asset!(app, "shaders/hd.wgsl");
        embedded_asset!(app, "shaders/material_prepass.wgsl");
        embedded_asset!(app, "shaders/effect_lighting.wgsl");
        embedded_asset!(app, "shaders/wc3_particle.wgsl");
        embedded_asset!(app, "shaders/wc3_age_particle.wgsl");
        embedded_asset!(app, "shaders/wc3_ribbon.wgsl");
        app.init_asset::<ModelAsset>();
        app.init_resource::<AssetSettings>();
        let resolver = app.world().resource::<AssetSettings>().resolver.clone();
        app.register_asset_loader(ModelLoader::new(resolver));
        app.init_asset_loader::<BlpImageLoader>();
        app.init_resource::<PreparedModelCache>();
        app.init_resource::<ModelParticleBirths>();
        app.init_resource::<ReplacementImages>();
        app.init_resource::<LodSettings>();
        app.add_plugins((
            MaterialPlugin::<ModelMaterial>::default(),
            EffectGeometryPlugin,
            ParticleRenderPlugin,
            AgeSpriteRenderPlugin,
            RibbonRenderPlugin,
        ));
        app.add_message::<ModelEvent>();
        configure(app);
        app.add_systems(
            Update,
            (
                invalidate_models,
                refresh_image_replacements,
                spawn_loaded_models,
            )
                .chain()
                .in_set(Wc3Systems::SpawnInstances),
        );
        app.add_systems(
            Update,
            (synchronize_mounted_models, advance_animation)
                .chain()
                .in_set(Wc3Systems::AdvanceAnimation),
        );
        app.add_systems(
            PostUpdate,
            animate_cameras.in_set(Wc3Systems::AnimateCameras),
        );
        app.add_systems(
            PostUpdate,
            (
                synchronize_model_cameras,
                restart_attachments,
                animate_instances,
            )
                .chain()
                .in_set(Wc3Systems::EvaluateNodePoses),
        );
        app.add_systems(
            PostUpdate,
            simulate_model_particles.in_set(Wc3Systems::SimulateModelParticles),
        );
        app.add_systems(
            PostUpdate,
            publish_affine_poses.in_set(Wc3Systems::PublishAffinePoses),
        );
        app.add_systems(
            PostUpdate,
            simulate_particles.in_set(Wc3Systems::SimulateEffects),
        );
        app.add_systems(
            PostUpdate,
            dispatch_events.in_set(Wc3Systems::DispatchEvents),
        );
        app.add_systems(
            PostUpdate,
            (update_lod, apply_lod_visibility, apply_world_selection)
                .chain()
                .in_set(Wc3Systems::SelectLod),
        );
        app.add_systems(
            PostUpdate,
            synchronize_model_layers
                .after(Wc3Systems::SimulateEffects)
                .before(VisibilitySystems::CheckVisibility),
        );
        app.add_systems(
            PostUpdate,
            update_skin_bounds
                .after(VisibilitySystems::CalculateBounds)
                .after(VisibilitySystems::VisibilityPropagate)
                .after(Wc3Systems::SelectLod)
                .before(VisibilitySystems::CheckVisibility),
        );
    }
}
