use super::super::effects::ParticleInstances;
use super::super::events::EventState;
use super::super::features::RuntimeFeatureInstances;
use super::super::lod::LodDefinition;
use super::super::material::SurfaceAnimation;
use super::super::pose::{EvaluatedPose, PoseContext};
use super::super::transition::TransitionState;
use super::super::{DrawPurpose, DrawSource, ModelMaterial, PreparedModel, SampledCamera};
use super::evaluation::EvaluationState;
use super::Playback;
use crate::physics::PhysicsState;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::{
    Error as AnimationError, FeatureId, InstanceId, MaterialId, MeshId, NodeId,
    Transform as ModelTransform,
};

/// Semantic identity of a spawned node, independent of native object numbers.
#[derive(Component, Clone, Copy, Debug)]
pub struct ModelNode {
    /// Source hierarchy node.
    pub id: NodeId,
}

/// Source identities of a rendered geometry pass.
#[derive(Component, Clone, Copy, Debug)]
pub struct ModelDraw {
    /// Authored geometry placement.
    pub instance: InstanceId,
    /// Explicit authored draw or implicit whole-instance geometry selection.
    pub source: DrawSource,
    /// Shared geometry identity.
    pub mesh: MeshId,
    /// Selected material identity.
    pub material: MaterialId,
    /// Pass ordinal in the selected material.
    pub pass: usize,
    /// Camera surface or dedicated shadow-only geometry.
    pub purpose: DrawPurpose,
    /// Authored geometry detail level; absent means shared across levels.
    pub lod: Option<u32>,
}

/// Application-supplied local poses applied after authored animation sampling.
///
/// Physics, IK, turrets and gameplay controllers can override individual nodes
/// without replacing the animation evaluator. Remove an entry to resume sampling.
#[derive(Component, Default)]
pub struct PoseOverrides(pub HashMap<NodeId, ModelTransform>);

/// Most recent invalid playback selection; cleared after a successful evaluation.
#[derive(Component, Default)]
pub struct PlaybackError(pub Option<String>);

/// Live instance with its own rig and evaluation storage, sharing prepared assets.
#[derive(Component)]
pub struct ModelRig {
    pub(super) events: EventState,
    pub(super) event_scratch: Vec<usize>,
    pub(super) model: PreparedModel,
    pub(super) nodes: Vec<Entity>,
    pub(super) mounts: Vec<Entity>,
    pub(super) poses: Vec<Mat4>,
    pub(super) evaluated: Vec<EvaluatedPose>,
    pub(super) visible: Vec<bool>,
    pub(super) basis_entity: Entity,
    pub(super) draws: Vec<(Entity, Option<u32>, Handle<ModelMaterial>)>,
    pub(super) materials: Vec<(Handle<ModelMaterial>, Arc<SurfaceAnimation>)>,
    pub(super) features: RuntimeFeatureInstances,
    pub(super) particles: ParticleInstances,
    pub(super) pose_initialized: bool,
    pub(super) transition: TransitionState,
    pub(super) evaluation: EvaluationState,
    pub(super) physics: PhysicsState,
    pub(super) physics_locals: Vec<ModelTransform>,
}
impl ModelRig {
    /// This rig's live external programme entity for an authored feature.
    /// The entity owns programme statistics and controls. Mounted and
    /// particle-owned child rigs have independent lookups; replacement may
    /// despawn the returned entity. Missing or foreign features return `None`.
    pub fn particle_program(&self, feature: FeatureId) -> Option<Entity> {
        self.model
            .particles
            .programme_entity(&self.particles, feature)
    }

    pub(crate) fn feature_instances(&self) -> &RuntimeFeatureInstances {
        &self.features
    }
    pub(crate) fn lod_definition(&self) -> &LodDefinition {
        &self.model.lod
    }
    pub(crate) fn lod_bounds(&self) -> Option<Aabb> {
        self.model.rest_bounds()
    }
    pub(crate) fn coordinate_entity(&self) -> Entity {
        self.basis_entity
    }
    pub(crate) fn sample_camera_current(
        &self,
        feature: FeatureId,
        playback: &Playback,
        aspect: f32,
        overrides: &PoseOverrides,
        scratch: &mut Vec<usize>,
    ) -> Result<Option<SampledCamera>, AnimationError> {
        let selection = playback.selection()?;
        let sample = self.model.animation.sample(selection)?;
        let Some(camera) = self
            .model
            .features
            .camera_ids()
            .find(|camera| camera.id == feature)
        else {
            return Ok(None);
        };
        let Some(index) = self
            .model
            .nodes
            .iter()
            .position(|node| node.id == camera.node)
        else {
            return Ok(None);
        };
        let context = PoseContext {
            nodes: &self.model.nodes,
            transition: self.transition.preview(
                playback.transition_request,
                playback.transition_clock,
                playback.blend_duration(&self.model),
            ),
            overrides: &overrides.0,
            camera: None,
        };
        let retimed = sample.at_times(selection.clip_time, selection.shared_time);
        let node = context.sample_node(index, &retimed, scratch).0;
        Ok(self
            .model
            .features
            .sample_camera(feature, &sample, node, self.model.up, aspect))
    }
    pub(crate) fn effect_time(&self, playback: &Playback) -> f64 {
        self.particles.effect_time(
            playback.shared_time,
            playback.clip,
            playback.event_traversal,
        )
    }
    #[cfg(test)]
    pub(crate) fn evaluation_count(&self) -> usize {
        self.evaluation.samples
    }

    pub(crate) fn despawn_owned_subtree(&self, commands: &mut Commands) {
        commands.entity(self.basis_entity).despawn();
    }
    /// Composed rigid equipment roots, ordered like [`PreparedModel::mounts`].
    ///
    /// Each root has an independent [`ModelRig`] and [`Playback`]. This list does
    /// not include automatically loaded attachment-point or particle children.
    pub fn mounts(&self) -> &[Entity] {
        &self.mounts
    }
    /// Resolve a semantic node to its Bevy entity for attachments and inspection.
    pub fn node(&self, id: NodeId) -> Option<Entity> {
        self.model
            .nodes
            .iter()
            .position(|node| node.id == id)
            .map(|i| self.nodes[i])
    }
    /// Sample an authored view in model coordinates using the current evaluated pose.
    /// Call after node pose evaluation for a view matching the current frame.
    pub fn sample_camera(
        &self,
        feature: FeatureId,
        playback: &Playback,
        aspect: f32,
    ) -> Result<Option<SampledCamera>, AnimationError> {
        let sample = self.model.animation.sample(playback.selection()?)?;
        let Some(camera) = self
            .model
            .features
            .camera_ids()
            .find(|camera| camera.id == feature)
        else {
            return Ok(None);
        };
        let Some(index) = self
            .model
            .nodes
            .iter()
            .position(|node| node.id == camera.node)
        else {
            return Ok(None);
        };
        Ok(self.model.features.sample_camera(
            feature,
            &sample,
            self.poses[index],
            self.model.up,
            aspect,
        ))
    }
    /// Shared definitions and observable renderer limitations.
    pub fn model(&self) -> &PreparedModel {
        &self.model
    }
}
