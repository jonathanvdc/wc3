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
pub(crate) mod model_budget;
pub(crate) mod model_children;
pub use model_budget::{ModelParticleLimits, ModelParticleTreeStats};
pub(crate) mod model_ownership;
pub(super) mod model_particles;

mod programmes;
mod ribbons;
mod world;
use super::events::EventTraversal;
use super::material::SurfacePass;
use crate::child_models::{ChildModelBinding, ChildModelKey};
use crate::textures::TextureImages;
use atlas_sprites::{AtlasParticleInstances, RuntimeAtlasParticles};
use bevy::prelude::*;
pub(crate) use emission::EffectChange;
use emission::EffectClock;
pub(super) use frame::EffectFrame;
use frame::EffectStart;
pub(crate) use m2_models::ModelOutputs;
pub use m2_particles::SpriteEmitter;
pub use m2_particles::{AdoptedParticleSource, AdoptedParticleTemplate, AdoptedTemplateStatus};
use m2_particles::{GeometryContext, M2ParticleInstances, M2World, RuntimeM2Particles};
use model_budget::ActorFrame;
pub use model_particles::ModelEmitter;
use model_particles::{ModelParticleInstances, RuntimeModelParticles};
pub use programmes::{
    ParticleEffect, ParticleProgramEmitter, ParticleTextureBinding, ParticleTextureRequest,
};
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
    actor: Option<ActorFrame>,
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
        basis: Entity,
    ) -> ParticleInstances {
        // Create age meshes before the resident transports, preserving entity order.
        let m2 = self.m2.spawn(commands, meshes, basis);
        ParticleInstances {
            clock: EffectClock::default(),
            actor: None,
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
    pub(crate) fn set_actor_frame(instances: &mut ParticleInstances, actor: Option<ActorFrame>) {
        instances.actor = actor;
    }
    pub(crate) fn seed_owned(
        &self,
        instances: &mut ParticleInstances,
        frame: EffectFrame<'_, '_, '_>,
        world: &mut EffectWorld<'_, '_>,
    ) {
        let start = EffectStart { frame };
        self.atlas.seed_owned(&mut instances.atlas, &start);
        self.ribbons.seed_owned(&mut instances.ribbons, &start);
        self.m2.seed_owned(
            &mut instances.m2,
            &start,
            &world.models,
            &mut world.commands,
        );
        self.models.seed_owned(&mut instances.models, &start);
        self.programmes
            .seed_owned(&mut instances.programmes, &start);
    }
    pub(crate) fn start_owned_clock(
        instances: &mut ParticleInstances,
        selection: AnimationSample,
        traversal: EventTraversal,
    ) {
        instances.clock.observe(0., selection.clip, traversal);
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
        actor: Option<&ActorFrame>,
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
            actor,
        );
        self.m2.update_bounded(
            &mut instances.m2,
            frame,
            &mut world.age_sprites,
            &mut world.age_statistics,
            M2World {
                commands: &mut world.commands,
                models: &world.models,
                geometry: Some(GeometryContext {
                    outputs,
                    statistics: &mut world.model_statistics,
                    root,
                    driving_camera,
                    allow_births,
                }),
            },
            actor,
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
        let actor = instances.actor.as_ref().filter(|actor| actor.owned);
        self.atlas.update_bounded(
            &mut instances.atlas,
            &frame,
            &mut world.sprites,
            &mut world.age_statistics,
            actor,
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
        self.ribbons.update_bounded(
            &mut instances.ribbons,
            &frame,
            &mut world.trails,
            &mut world.ribbon_statistics,
            actor,
        );
        self.programmes.update_bounded(
            &mut instances.programmes,
            &frame,
            &mut world.age_sprites,
            &mut world.program_statistics,
            actor,
        );
        if simulate_models {
            self.m2.update_bounded(
                &mut instances.m2,
                frame,
                &mut world.age_sprites,
                &mut world.age_statistics,
                M2World {
                    commands: &mut world.commands,
                    models: &world.models,
                    geometry: None,
                },
                actor,
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
