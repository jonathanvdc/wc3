use super::super::effects::ParticleInstances;
use super::super::events::EventState;
use super::super::features::RuntimeFeatureInstances;
use super::super::lod::LodDefinition;
use super::super::material::SurfaceAnimation;
use super::super::pose::{EvaluatedPose, PoseContext};
use super::super::transition::TransitionState;
use super::super::{
    PreparedUniversalModel, SampledUniversalCamera, UniversalDrawSource, UniversalMaterial,
};
use super::evaluation::EvaluationState;
use super::UniversalPlayback;
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
pub struct UniversalNode {
    /// Source hierarchy node.
    pub id: NodeId,
}

/// Source identities of a rendered geometry pass.
#[derive(Component, Clone, Copy, Debug)]
pub struct UniversalDraw {
    /// Authored geometry placement.
    pub instance: InstanceId,
    /// Explicit authored draw or implicit whole-instance geometry selection.
    pub source: UniversalDrawSource,
    /// Shared geometry identity.
    pub mesh: MeshId,
    /// Selected material identity.
    pub material: MaterialId,
    /// Pass ordinal in the selected material.
    pub pass: usize,
    /// Authored geometry detail level; absent means shared across levels.
    pub lod: Option<u32>,
}

/// Application-supplied local poses applied after authored animation sampling.
///
/// Physics, IK, turrets and gameplay controllers can override individual nodes
/// without replacing the animation evaluator. Remove an entry to resume sampling.
#[derive(Component, Default)]
pub struct UniversalPoseOverrides(pub HashMap<NodeId, ModelTransform>);

/// Most recent invalid playback selection; cleared after a successful evaluation.
#[derive(Component, Default)]
pub struct UniversalAnimationError(pub Option<String>);

/// Live instance with its own rig and evaluation storage, sharing prepared assets.
#[derive(Component)]
pub struct UniversalInstance {
    pub(super) events: EventState,
    pub(super) event_scratch: Vec<usize>,
    pub(super) model: PreparedUniversalModel,
    pub(super) nodes: Vec<Entity>,
    pub(super) poses: Vec<Mat4>,
    pub(super) evaluated: Vec<EvaluatedPose>,
    pub(super) visible: Vec<bool>,
    pub(super) basis_entity: Entity,
    pub(super) draws: Vec<(Entity, Option<u32>, Handle<UniversalMaterial>)>,
    pub(super) materials: Vec<(Handle<UniversalMaterial>, Arc<SurfaceAnimation>)>,
    pub(super) features: RuntimeFeatureInstances,
    pub(super) particles: ParticleInstances,
    pub(super) pose_initialized: bool,
    pub(super) transition: TransitionState,
    pub(super) evaluation: EvaluationState,
}
impl UniversalInstance {
    pub(in crate::universal) fn feature_instances(&self) -> &RuntimeFeatureInstances {
        &self.features
    }
    pub(in crate::universal) fn lod_definition(&self) -> &LodDefinition {
        &self.model.lod
    }
    pub(in crate::universal) fn lod_bounds(&self) -> Option<Aabb> {
        self.model.rest_bounds()
    }
    pub(in crate::universal) fn coordinate_entity(&self) -> Entity {
        self.basis_entity
    }
    pub(in crate::universal) fn sample_camera_current(
        &self,
        feature: FeatureId,
        playback: &UniversalPlayback,
        aspect: f32,
        overrides: &UniversalPoseOverrides,
        scratch: &mut Vec<usize>,
    ) -> Result<Option<SampledUniversalCamera>, AnimationError> {
        let selection = playback.selection(&self.model)?;
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
        let retimed = sample.at_times(
            selection.clip_time,
            selection.timeline_time,
            selection.shared_time,
        );
        let node = context.sample_node(index, &retimed, scratch).0;
        Ok(self
            .model
            .features
            .sample_camera(feature, &sample, node, self.model.up, aspect))
    }
    pub(in crate::universal) fn effect_time(&self, playback: &UniversalPlayback) -> f64 {
        self.particles.effect_time(
            playback.shared_time,
            playback.clip,
            playback.event_traversal,
        )
    }
    #[cfg(test)]
    pub(in crate::universal) fn evaluation_count(&self) -> usize {
        self.evaluation.samples
    }

    pub(in crate::universal) fn despawn_owned_subtree(&self, commands: &mut Commands) {
        commands.entity(self.basis_entity).despawn();
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
        playback: &UniversalPlayback,
        aspect: f32,
    ) -> Result<Option<SampledUniversalCamera>, AnimationError> {
        let sample = self
            .model
            .animation
            .sample(playback.selection(&self.model)?)?;
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
    pub fn model(&self) -> &PreparedUniversalModel {
        &self.model
    }
}
