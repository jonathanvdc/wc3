//! Semantic effect compilation, shared simulation clocks, and composed transports.
use crate::diagnostics::RenderIssue;
mod age_appearance;
pub(crate) mod age_sprites_gpu;
mod atlas_sprites;
mod bones;
mod emission;
mod frame;
mod generators;
mod implosion;
mod m2_model_geometry;
mod m2_models;
mod m2_particles;
pub(crate) mod model_children;
pub(crate) mod model_ownership;
pub(super) mod model_particles;

mod programmes;
mod ribbons;
mod world;
use super::events::EventTraversal;
use super::material::{ModelMaterial, SurfacePass};
use crate::child_models::{ChildModelBinding, ChildModelKey};
use crate::textures::TextureImages;
use atlas_sprites::{AtlasParticleInstances, RuntimeAtlasParticles};
use bevy::prelude::*;
pub(crate) use emission::EffectChange;
use emission::EffectClock;
pub(super) use frame::EffectFrame;
pub(crate) use m2_models::ModelOutputs;
pub(crate) use m2_particles::GeometryWorld;
pub use m2_particles::SpriteEmitter;
use m2_particles::{M2ParticleInstances, RuntimeM2Particles};
pub use model_particles::ModelEmitter;
use model_particles::{ModelParticleInstances, RuntimeModelParticles};
pub use programmes::{ParticleEffect, ParticleProgramEmitter, ParticleTextureRequest};
use programmes::{ProgramInstances, RuntimeProgramParticles};
pub use ribbons::RibbonEmitter;
use ribbons::{RibbonInstancesState, RuntimeRibbons};
use std::collections::HashMap;
use wc3::model::{AnimationSample, ClipId, FeatureId, MaterialId, Model, NodeId};
pub(crate) use world::EffectWorld;

#[derive(Clone, Default)]
pub(super) struct RuntimeParticles {
    m2: RuntimeM2Particles,
    atlas: RuntimeAtlasParticles,
    ribbons: RuntimeRibbons,
    models: RuntimeModelParticles,
    programmes: RuntimeProgramParticles,

    diagnostics: Vec<(FeatureId, RenderIssue)>,
}
#[derive(Default)]
pub(super) struct ParticleInstances {
    clock: EffectClock,
    m2: M2ParticleInstances,
    atlas: AtlasParticleInstances,
    ribbons: RibbonInstancesState,
    models: ModelParticleInstances,
    programmes: ProgramInstances,
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
        self.m2.append_remapped(&source.m2, indices, nodes, clips);
        self.atlas
            .append_remapped(&source.atlas, indices, nodes, clips);
        self.ribbons
            .append_remapped(&source.ribbons, indices, nodes, clips);
        self.models
            .append_remapped(&source.models, indices, nodes, clips);
        self.programmes
            .append_remapped(&source.programmes, indices, nodes, clips);
        self.diagnostics.extend(source.diagnostics.iter().cloned());
    }

    pub(super) fn is_empty(&self) -> bool {
        self.m2.is_empty()
            && self.atlas.is_empty()
            && self.ribbons.is_empty()
            && self.models.is_empty()
            && self.programmes.is_empty()
    }
    pub fn compile(
        model: &Model<'_>,
        nodes: &HashMap<NodeId, usize>,
        images: &TextureImages,
        materials: &HashMap<MaterialId, Vec<SurfacePass>>,
        children: &HashMap<ChildModelKey, ChildModelBinding>,
        programmes: &HashMap<FeatureId, ParticleEffect>,
    ) -> Self {
        let programmes = RuntimeProgramParticles::compile(model, nodes, programmes);
        let m2 =
            RuntimeM2Particles::compile_with_children(model, nodes, images, materials, children);
        let atlas = RuntimeAtlasParticles::compile(model, nodes, images);
        let ribbons = RuntimeRibbons::compile(model, nodes, images, materials);
        let models = RuntimeModelParticles::compile(model, nodes, children);
        let diagnostics = atlas
            .diagnostics()
            .iter()
            .chain(models.diagnostics())
            .chain(ribbons.diagnostics())
            .chain(m2.diagnostics())
            .chain(programmes.diagnostics())
            .cloned()
            .collect();
        Self {
            m2,
            atlas,
            ribbons,
            models,
            programmes,
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
        let m2 = self.m2.spawn(commands, meshes, materials, basis);
        ParticleInstances {
            clock: EffectClock::default(),
            programmes: self.programmes.spawn(commands, meshes, basis),
            atlas: self.atlas.spawn(commands, meshes, basis),
            ribbons: self.ribbons.spawn(commands, meshes, basis),
            models: self.models.spawn(commands, basis),
            m2,
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
        self.update_transports(instances, root, frame, world, true);
    }
    pub(crate) fn has_model_actors(&self) -> bool {
        self.m2.has_models() || !self.models.is_empty()
    }
    pub(crate) fn start_owned_clock(instances: &mut ParticleInstances, selection: AnimationSample) {
        instances
            .clock
            .observe(0., selection.clip, Default::default());
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_model_actors<'w, 's>(
        &self,
        instances: &mut ParticleInstances,
        root: Entity,
        mut frame: EffectFrame<'_, '_, '_>,
        world: &mut EffectWorld<'w, 's>,
        outputs: &mut ModelOutputs<'w, 's>,
        driving_camera: Option<Entity>,
        allow_births: bool,
    ) -> (f64, emission::EffectChange) {
        let (time, change) =
            instances
                .clock
                .observe(frame.time, frame.selection.clip, frame.traversal);
        frame.time = time;
        frame.change = change;
        self.models.update_limited(
            &mut instances.models,
            &mut world.commands,
            root,
            &frame,
            &world.models,
            &mut world.model_statistics,
            allow_births,
        );
        self.m2.update_with_models(
            &mut instances.m2,
            frame,
            &mut world.meshes,
            &mut world.age_sprites,
            &mut world.age_statistics,
            Some(GeometryWorld {
                commands: &mut world.commands,
                outputs,
                statistics: &mut world.model_statistics,
                root,
                driving_camera,
                allow_births,
            }),
        );
        (time, change)
    }
    pub(crate) fn update_after_models(
        &self,
        instances: &mut ParticleInstances,
        root: Entity,
        frame: EffectFrame<'_, '_, '_>,
        world: &mut EffectWorld<'_, '_>,
    ) {
        self.update_transports(instances, root, frame, world, false);
    }
    fn update_transports(
        &self,
        instances: &mut ParticleInstances,
        root: Entity,
        frame: EffectFrame<'_, '_, '_>,
        world: &mut EffectWorld<'_, '_>,
        simulate_models: bool,
    ) {
        self.atlas.update(
            &mut instances.atlas,
            &frame,
            &mut world.sprites,
            &mut world.age_statistics,
        );
        if simulate_models {
            self.models.update(
                &mut instances.models,
                &mut world.commands,
                root,
                &frame,
                &world.models,
                &mut world.model_statistics,
            );
        }
        self.ribbons.update(
            &mut instances.ribbons,
            &frame,
            &mut world.trails,
            &mut world.ribbon_statistics,
        );
        self.programmes.update(
            &mut instances.programmes,
            &frame,
            &mut world.age_sprites,
            &mut world.program_statistics,
        );
        if simulate_models {
            self.m2.update(
                &mut instances.m2,
                frame,
                &mut world.meshes,
                &mut world.age_sprites,
                &mut world.age_statistics,
                world.materials.bypass_change_detection(),
            );
        }
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
        self.m2.map_images(map);
        self.atlas.map_images(map);
        self.ribbons.map_images(map);
        self.programmes.map_images(map);
    }
}
