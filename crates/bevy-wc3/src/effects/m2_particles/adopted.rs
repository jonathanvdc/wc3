//! Role-bound template readiness and sprite-draw ownership, never child actors.
use super::adopted_pool::{AdoptedPool, AdoptionWork};
use super::sources::{SourceSpans, SourceStep};
use super::templates::TemplateCatalogue;
use super::{Birth, MaterialPassInstance, SpriteEmitter};
use crate::child_coordinates::CoordinatePolicy;
use crate::child_models::ChildModelBinding;
use crate::diagnostics::{RenderDiagnostic, RenderDiagnosticKind, RenderDiagnosticSource};
use crate::effects::age_sprites_gpu::AgeSpriteData;
use crate::effects::frame::EffectFrame;
use crate::effects::model_budget::ActorFrame;
use crate::effects::render::EffectGeometry;
use crate::layers::ModelOwnedRenderEntity;
use crate::ModelAsset;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use std::sync::Arc;
use wc3::model::{FeatureId, NodeId};

/// Provenance colocated with one adopted pool's ordinary `SpriteEmitter` counts.
/// The latter's feature/node remain local to the owning parent model. These
/// feature/node IDs instead belong to `model`, which supplies the template.
#[derive(Component, Clone, Debug)]
pub struct AdoptedParticleTemplate {
    /// Independently loaded template model; images remain in this asset scope.
    pub model: Handle<ModelAsset>,
    /// Template feature in that model.
    pub feature: FeatureId,
    /// Authored template node, retained for provenance rather than placement.
    pub node: NodeId,
    /// Original first-four candidate slot, including preceding rejected slots.
    pub slot: u8,
}
/// Resource state of the one template model referenced by a parent emitter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdoptedTemplateStatus {
    /// Its model asset has not become available; no historical drives are queued.
    Pending,
    /// The catalogue is prepared, including any per-slot omissions.
    Ready,
    /// Child-frame conversion has no adopted-template interpretation.
    UnsupportedCoordinates,
}
/// Cold-path resource/coverage information attached to the parent emitter entity.
/// Its `SpriteEmitter` or `ModelEmitter` identifies the source in the owner model;
/// diagnostic feature/node IDs below belong to this component's `model` instead.
#[derive(Component, Clone, Debug)]
pub struct AdoptedParticleSource {
    /// Template model whose role was resolved by the normal resource provider.
    pub model: Handle<ModelAsset>,
    /// Current catalogue readiness, updated only when readiness/generation changes.
    pub status: AdoptedTemplateStatus,
    /// Accepted template slots, at most four.
    pub accepted: u8,
    /// Issues scoped to the template model. Rebuilt only for a new catalogue.
    pub diagnostics: Vec<RenderDiagnostic>,
}
#[derive(Clone, Copy, PartialEq)]
struct SampleKey {
    time: f64,
    node: Mat4,
    basis: Mat4,
}
pub(super) struct AdoptedSource {
    binding: ChildModelBinding,
    owner: Entity,
    feature: FeatureId,
    node: NodeId,
    selector: Handle<Mesh>,
    generation: Option<Arc<TemplateCatalogue>>,
    pools: Vec<AdoptedPool>,
    status: Option<AdoptedTemplateStatus>,
    pub sources: SourceSpans,
    pub ready_at: f64,
    pub active: bool,
    published: Option<SampleKey>,
    driving: bool,
    work: AdoptionWork,
}
impl AdoptedSource {
    pub(super) fn new(
        binding: ChildModelBinding,
        owner: Entity,
        feature: FeatureId,
        node: NodeId,
        selector: Handle<Mesh>,
    ) -> Self {
        Self {
            binding,
            owner,
            feature,
            node,
            selector,
            generation: None,
            pools: Vec::new(),
            status: None,
            sources: SourceSpans::default(),
            ready_at: 0.,
            active: false,
            published: None,
            driving: false,
            work: AdoptionWork::default(),
        }
    }
    fn destroy_draws(&mut self, commands: &mut Commands) {
        for pool in self.pools.drain(..) {
            for pass in pool.storage.passes {
                commands.entity(pass.entity).try_despawn();
            }
        }
        self.generation = None;
        self.published = None;
        self.sources.reset();
    }
    fn pending(&mut self, commands: &mut Commands) {
        if self.status != Some(AdoptedTemplateStatus::Pending) {
            self.destroy_draws(commands);
            commands.entity(self.owner).insert(AdoptedParticleSource {
                model: self.binding.model.clone(),
                status: AdoptedTemplateStatus::Pending,
                accepted: 0,
                diagnostics: Vec::new(),
            });
            self.status = Some(AdoptedTemplateStatus::Pending);
        }
        self.active = false;
    }
    /// No asset parsing/pose sampling here: borrow the immutable Universal-derived
    /// catalogue and keep its Arc alive as the reload identity, avoiding ABA.
    pub(super) fn refresh(
        &mut self,
        now: f64,
        assets: &Assets<ModelAsset>,
        commands: &mut Commands,
    ) {
        if self.binding.coordinates != CoordinatePolicy::Host {
            if self.status != Some(AdoptedTemplateStatus::UnsupportedCoordinates) {
                self.destroy_draws(commands);
                commands.entity(self.owner).insert(AdoptedParticleSource {
                    model: self.binding.model.clone(), status: AdoptedTemplateStatus::UnsupportedCoordinates, accepted: 0,
                    diagnostics: vec![RenderDiagnostic::new(RenderDiagnosticKind::Unsupported, RenderDiagnosticSource::Model, "Adopted particle templates use their source emitter frame and units; standalone child coordinate conversion is not interpreted")],
                });
                self.status = Some(AdoptedTemplateStatus::UnsupportedCoordinates);
            }
            self.active = false;
            return;
        }
        let Some(asset) = assets.get(&self.binding.model) else {
            self.pending(commands);
            return;
        };
        let catalogue = &asset.compiled.particles.m2.templates;
        if self
            .generation
            .as_ref()
            .is_some_and(|previous| Arc::ptr_eq(previous, catalogue))
        {
            self.active = !self.pools.is_empty();
            return;
        }
        self.destroy_draws(commands);
        let mut diagnostics = Vec::new();
        for (slot, entry) in catalogue.slots.iter().enumerate() {
            let template = match &entry.template {
                Ok(template) => template,
                Err(issue) => {
                    diagnostics.push(
                        issue
                            .clone()
                            .at(RenderDiagnosticSource::Feature(entry.feature)),
                    );
                    continue;
                }
            };
            diagnostics.extend(
                template
                    .issues
                    .iter()
                    .cloned()
                    .map(|issue| issue.at(RenderDiagnosticSource::Feature(entry.feature))),
            );
            let entity = commands
                .spawn((
                    EffectGeometry(self.selector.clone()),
                    ModelOwnedRenderEntity,
                    AgeSpriteData {
                        records: Default::default(),
                        live_indices: Default::default(),
                        uniform: Default::default(),
                        style: template.kernel().gpu_style.clone(),
                    },
                    SpriteEmitter {
                        feature: self.feature,
                        node: self.node,
                        alive: 0,
                        dropped: 0,
                        skipped_seconds: 0.,
                    },
                    AdoptedParticleTemplate {
                        model: self.binding.model.clone(),
                        feature: entry.feature,
                        node: template.kernel().node,
                        slot: slot as u8,
                    },
                    NotShadowCaster,
                    ChildOf(self.owner),
                ))
                .id();
            let mut simulation = AdoptedPool::new(template.clone());
            simulation
                .storage
                .passes
                .push(MaterialPassInstance { entity });
            self.pools.push(simulation);
        }
        diagnostics.push(RenderDiagnostic::new(
            RenderDiagnosticKind::Approximation, RenderDiagnosticSource::Model,
            "Adopted templates use common analytical M2 birth ages and motion, rather than native discrete per-drive age randomization and acceleration order",
        ));
        diagnostics.push(RenderDiagnostic::new(
            RenderDiagnosticKind::Unverified, RenderDiagnosticSource::Model,
            "Adopted template attached-parent framing, advanced image routing and exact native random distributions remain unverified",
        ));
        diagnostics.push(RenderDiagnostic::new(RenderDiagnosticKind::Budget, RenderDiagnosticSource::Model,
            "Adopted templates share 8192 child constructions, 32768 source examinations and 32768 child-motion checks per parent-emitter update; each pool retains at most 4096 children and owned trees also share emission-step limits. Omitted work is not queued"));
        commands.entity(self.owner).insert(AdoptedParticleSource {
            model: self.binding.model.clone(),
            status: AdoptedTemplateStatus::Ready,
            accepted: self.pools.len() as u8,
            diagnostics,
        });
        self.generation = Some(catalogue.clone());
        self.status = Some(AdoptedTemplateStatus::Ready);
        self.ready_at = now;
        self.active = !self.pools.is_empty();
    }
    pub(super) fn reset(&mut self, now: f64) {
        for pool in &mut self.pools {
            pool.reset();
        }
        self.sources.reset();
        self.ready_at = now;
        self.published = None;
    }
    /// Readiness refresh remains separate from population work. Paused unchanged
    /// updates keep both snapshots and sampled GPU records without copying births.
    pub(super) fn begin_frame(
        &mut self,
        frame: &EffectFrame<'_, '_, '_>,
        start: f64,
        previous: Mat4,
        node: Mat4,
        existing: &[Birth],
        follow: Vec3,
    ) {
        self.work = AdoptionWork::default();
        self.driving = self.active && frame.time > start && frame.time > self.ready_at;
        for pool in &mut self.pools {
            pool.begin_frame(if self.ready_at >= frame.time {
                node
            } else {
                previous
            });
        }
        if self.driving {
            self.sources.begin(existing, follow);
        }
    }
    pub(super) fn newborn(&mut self, birth: &Birth, follow: Vec3, fraction: f32) {
        if self.driving {
            self.sources.newborn(birth, follow, fraction);
        }
    }
    fn skip_source(&mut self, start: f64) {
        self.driving = false;
        for pool in &mut self.pools {
            pool.skip_source_from(start);
        }
    }
    pub(super) fn step(&mut self, step: SourceStep<'_>, actor: Option<&ActorFrame>) {
        if !self.driving || step.end <= self.ready_at {
            return;
        }
        if self
            .pools
            .iter()
            .any(|_| actor.is_some_and(|actor| !actor.step()))
        {
            self.skip_source(step.start);
            return;
        }
        let pools = &mut self.pools;
        let complete = self.sources.drive(step, &mut self.work, |drive, work| {
            for pool in pools.iter_mut() {
                pool.drive(drive, work);
            }
        });
        // Restored owner center/orientation, once after ALL A/B source drives.
        for pool in &mut self.pools {
            pool.advance_owner(
                step.start,
                step.end,
                step.node,
                step.emitter.follow_unit_scale,
                &mut self.work,
            );
        }
        if !complete {
            self.skip_source(step.start);
        }
    }
    pub(super) fn finish_frame(
        &mut self,
        frame: &EffectFrame<'_, '_, '_>,
        checked_end: f64,
        node: Mat4,
        actor: Option<&ActorFrame>,
    ) {
        if self.driving && checked_end < frame.time {
            self.skip_source(checked_end);
        }
        for pool in &mut self.pools {
            let omitted = pool.finish_frame(frame.time);
            pool.storage.previous_node = node;
            if let Some(actor) = actor {
                actor.budget.skip_seconds(omitted);
            }
        }
    }
    pub(super) fn publish(
        &mut self,
        frame: &EffectFrame<'_, '_, '_>,
        node: Mat4,
        gpu: &mut Query<&mut AgeSpriteData>,
        statistics: &mut Query<&mut SpriteEmitter>,
        commands: &mut Commands,
    ) {
        let key = SampleKey {
            time: frame.time,
            node,
            basis: frame.basis_world,
        };
        for pool in &mut self.pools {
            pool.finish(frame.time);
            let Some(entity) = pool.storage.passes.first().map(|pass| pass.entity) else {
                continue;
            };
            if self.published != Some(key) {
                if let Ok(mut data) = gpu.get_mut(entity) {
                    pool.publish(frame, node, &mut data);
                } else {
                    // Birth-stage draws may still be reserved in this same
                    // Commands batch. Publish into that entity on flush rather
                    // than delaying its first live frame or observing time twice.
                    let mut data = AgeSpriteData {
                        records: Default::default(),
                        live_indices: Default::default(),
                        uniform: Default::default(),
                        style: pool.template.kernel().gpu_style.clone(),
                    };
                    pool.publish(frame, node, &mut data);
                    commands.entity(entity).try_insert(data);
                }
            }
            if let Ok(mut stats) = statistics.get_mut(entity) {
                let state = &pool.storage;
                if stats.alive != state.births.len() {
                    stats.alive = state.births.len();
                }
                if stats.dropped != state.dropped {
                    stats.dropped = state.dropped;
                }
                if stats.skipped_seconds != state.skipped_seconds {
                    stats.skipped_seconds = state.skipped_seconds;
                }
            } else {
                // Keep the newly reserved draw's observable counts coherent
                // with the sample delivered in the same deferred publication.
                commands.entity(entity).try_insert(SpriteEmitter {
                    feature: self.feature,
                    node: self.node,
                    alive: pool.storage.births.len(),
                    dropped: pool.storage.dropped,
                    skipped_seconds: pool.storage.skipped_seconds,
                });
            }
        }
        self.published = Some(key);
    }
}
