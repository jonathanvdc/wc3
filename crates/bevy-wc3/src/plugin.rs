//! Registration and scheduling for the Universal renderer.
use crate::assets::BlpImageLoader;
use crate::effects::render::configure_effect_views;
use crate::effects::sprites::ParticleRenderPlugin;
use crate::effects::trails::RibbonRenderPlugin;
use crate::schedule::{configure, Wc3Systems};
use crate::universal::{
    advance_universal_animation, animate_universal_cameras, animate_universal_instances,
    apply_universal_lod_visibility, apply_universal_world_selection, dispatch_universal_events,
    invalidate_universal_models, restart_universal_attachments, simulate_universal_particles,
    spawn_loaded_universal_models, synchronize_model_particles, update_skin_bounds,
    update_universal_lod, UniversalAssetSettings, UniversalLodSettings, UniversalMaterial,
    UniversalModelAsset, UniversalModelEvent, UniversalModelLoader, UniversalPreparedCache,
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
        embedded_asset!(app, "shaders/universal_material.wgsl");
        embedded_asset!(app, "shaders/universal_hd.wgsl");
        embedded_asset!(app, "shaders/universal_material_prepass.wgsl");
        embedded_asset!(app, "shaders/wc3_particle.wgsl");
        embedded_asset!(app, "shaders/wc3_ribbon.wgsl");
        app.init_asset::<UniversalModelAsset>();
        app.init_resource::<UniversalAssetSettings>();
        let resolver = app
            .world()
            .resource::<UniversalAssetSettings>()
            .resolver
            .clone();
        app.register_asset_loader(UniversalModelLoader::new(resolver));
        app.init_asset_loader::<BlpImageLoader>();
        app.init_resource::<UniversalPreparedCache>();
        app.init_resource::<UniversalLodSettings>();
        app.add_plugins((
            MaterialPlugin::<UniversalMaterial>::default(),
            ParticleRenderPlugin,
            RibbonRenderPlugin,
        ));
        app.add_message::<UniversalModelEvent>();
        configure(app);
        app.add_systems(
            Update,
            (invalidate_universal_models, spawn_loaded_universal_models)
                .chain()
                .in_set(Wc3Systems::SpawnInstances),
        );
        app.add_systems(
            Update,
            (advance_universal_animation, synchronize_model_particles)
                .chain()
                .in_set(Wc3Systems::AdvanceAnimation),
        );
        app.add_systems(
            PostUpdate,
            animate_universal_cameras.in_set(Wc3Systems::AnimateCameras),
        );
        app.add_systems(
            PostUpdate,
            (restart_universal_attachments, animate_universal_instances)
                .chain()
                .in_set(Wc3Systems::EvaluateNodePoses),
        );
        app.add_systems(
            PostUpdate,
            simulate_universal_particles.in_set(Wc3Systems::SimulateEffects),
        );
        app.add_systems(
            PostUpdate,
            dispatch_universal_events.in_set(Wc3Systems::DispatchEvents),
        );
        app.add_systems(
            PostUpdate,
            (
                update_universal_lod,
                apply_universal_lod_visibility,
                apply_universal_world_selection,
            )
                .chain()
                .in_set(Wc3Systems::SelectLod),
        );
        app.add_systems(
            PostUpdate,
            update_skin_bounds
                .after(VisibilitySystems::CalculateBounds)
                .after(VisibilitySystems::VisibilityPropagate)
                .after(Wc3Systems::SelectLod)
                .before(VisibilitySystems::CheckVisibility),
        );
        app.add_systems(PostUpdate, configure_effect_views);
    }
}
