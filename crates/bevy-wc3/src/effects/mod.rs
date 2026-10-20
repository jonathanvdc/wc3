//! Semantic effect compilation, shared simulation clocks, and composed transports.
use crate::diagnostics::RenderIssue;
mod age_appearance;
mod age_sprites;
pub(crate) mod age_sprites_gpu;
mod atlas_sprites;
mod bones;
mod emission;
mod frame;
mod generators;
mod implosion;
pub(super) mod model_particles;

mod ribbons;
mod world;
use super::child_coordinates::CoordinatePolicy;
use super::events::EventTraversal;
use super::material::{ModelMaterial, SurfacePass};
use super::ModelAsset;
use crate::textures::TextureImages;
pub use age_sprites::SpriteEmitter;
use age_sprites::{AgeSpriteInstances, RuntimeAgeSprites};
use atlas_sprites::{AtlasParticleInstances, RuntimeAtlasParticles};
use bevy::prelude::*;
use emission::EffectClock;
pub(super) use frame::EffectFrame;
pub(crate) use model_particles::synchronize_model_particles;
pub use model_particles::ModelEmitter;
use model_particles::{ModelParticleInstances, RuntimeModelParticles};
pub use ribbons::RibbonEmitter;
use ribbons::{RibbonInstancesState, RuntimeRibbons};
use std::collections::HashMap;
use wc3::model::{ClipId, FeatureId, MaterialId, Model, NodeId};
pub(crate) use world::EffectWorld;

#[derive(Clone, Default)]
pub(super) struct RuntimeParticles {
    age: RuntimeAgeSprites,
    atlas: RuntimeAtlasParticles,
    ribbons: RuntimeRibbons,
    models: RuntimeModelParticles,

    diagnostics: Vec<(FeatureId, RenderIssue)>,
}
#[derive(Default)]
pub(super) struct ParticleInstances {
    clock: EffectClock,
    age: AgeSpriteInstances,
    atlas: AtlasParticleInstances,
    ribbons: RibbonInstancesState,
    models: ModelParticleInstances,
}
impl ParticleInstances {
    pub(super) fn effect_time(
        &self,
        raw: f64,
        clip: Option<ClipId>,
        traversal: EventTraversal,
    ) -> f64 {
        self.clock.project(raw, clip, traversal).0
    }
}
impl RuntimeParticles {
    pub(crate) fn append_remapped(
        &mut self,
        source: &Self,
        indices: &[usize],
        nodes: &HashMap<NodeId, NodeId>,
        clips: &HashMap<ClipId, ClipId>,
    ) {
        self.age.append_remapped(&source.age, indices, nodes, clips);
        self.atlas
            .append_remapped(&source.atlas, indices, nodes, clips);
        self.ribbons
            .append_remapped(&source.ribbons, indices, nodes, clips);
        self.models
            .append_remapped(&source.models, indices, nodes, clips);
        self.diagnostics.extend(source.diagnostics.iter().cloned());
    }

    pub(super) fn is_empty(&self) -> bool {
        self.age.is_empty()
            && self.atlas.is_empty()
            && self.ribbons.is_empty()
            && self.models.is_empty()
    }
    pub fn compile(
        model: &Model<'_>,
        nodes: &HashMap<NodeId, usize>,
        images: &TextureImages,
        materials: &HashMap<MaterialId, Vec<SurfacePass>>,
        children: &HashMap<FeatureId, Handle<ModelAsset>>,
        placements: &HashMap<FeatureId, CoordinatePolicy>,
    ) -> Self {
        let age = RuntimeAgeSprites::compile(model, nodes, images, materials);
        let atlas = RuntimeAtlasParticles::compile(model, nodes, images);
        let ribbons = RuntimeRibbons::compile(model, nodes, images, materials);
        let models = RuntimeModelParticles::compile(model, nodes, children, placements);
        let diagnostics = atlas
            .diagnostics()
            .iter()
            .chain(models.diagnostics())
            .chain(ribbons.diagnostics())
            .chain(age.diagnostics())
            .cloned()
            .collect();
        Self {
            age,
            atlas,
            ribbons,
            models,
            diagnostics,
        }
    }
    pub fn diagnostics(&self) -> &[(FeatureId, RenderIssue)] {
        &self.diagnostics
    }
    pub fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ModelMaterial>,
        basis: Entity,
    ) -> ParticleInstances {
        // Create age meshes before the resident transports, preserving entity order.
        let age = self.age.spawn(commands, meshes, materials, basis);
        ParticleInstances {
            clock: EffectClock::default(),
            atlas: self.atlas.spawn(commands, meshes, basis),
            ribbons: self.ribbons.spawn(commands, meshes, basis),
            models: self.models.spawn(commands, basis),
            age,
        }
    }
    pub fn update(
        &self,
        instances: &mut ParticleInstances,
        root: Entity,
        mut frame: EffectFrame<'_, '_, '_>,
        world: &mut EffectWorld<'_, '_>,
    ) {
        if !frame.time.is_finite() {
            return;
        }
        let (time, change) =
            instances
                .clock
                .observe(frame.time, frame.selection.clip, frame.traversal);
        frame.time = time;
        frame.change = change;
        self.atlas.update(
            &mut instances.atlas,
            &frame,
            &mut world.sprites,
            &mut world.age_statistics,
        );
        self.models.update(
            &mut instances.models,
            &mut world.commands,
            root,
            &frame,
            &world.models,
            &mut world.model_statistics,
        );
        self.ribbons.update(
            &mut instances.ribbons,
            &frame,
            &mut world.trails,
            &mut world.ribbon_statistics,
        );
        self.age.update(
            &mut instances.age,
            frame,
            &mut world.meshes,
            &mut world.age_sprites,
            &mut world.age_statistics,
            world.materials.bypass_change_detection(),
        );
    }
}

#[cfg(test)]
#[path = "fixture_tests.rs"]
mod fixture_tests;

pub(crate) mod blend;
pub(crate) mod records;
pub(crate) mod render;
pub(crate) mod simulation;
pub(crate) mod sprites;
pub(crate) mod trails;

mod ribbon_definition;

impl RuntimeParticles {
    pub(crate) fn map_images(
        &mut self,
        map: &mut impl FnMut(&Handle<Image>) -> Option<Handle<Image>>,
    ) {
        self.age.map_images(map);
        self.atlas.map_images(map);
        self.ribbons.map_images(map);
    }
}
