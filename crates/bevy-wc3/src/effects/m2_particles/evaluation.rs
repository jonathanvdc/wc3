//! Shared occurrence-time pose and ordinary birth admission.
use super::adopted::AdoptedSource;
use super::birth::BirthPlan;
use super::squirt::{SquirtBurst, SquirtError, SquirtSample, SquirtUpdate};
use super::{
    interpolated_transform, random, Birth, BirthAppearance, Emitter, EmitterInstance,
    GeometryBirth, MAX_BIRTHS_PER_UPDATE,
};
use crate::effects::atlas_sprites::retimed_selection;
use crate::effects::emission::EffectChange;
use crate::effects::frame::EffectFrame;
use crate::effects::m2_model_geometry::{tumble, ModelBasis};
use crate::effects::model_budget::{ActorFrame, ModelActorLease};
use bevy::prelude::*;
use std::collections::HashMap;
use wc3::model::{ClipId, RetimedAnimation, Time as ModelTime};

/// Common occurrence-pose evaluation for continuous and keyed emission.
pub(super) struct BirthContext<'a, 'view, 'data> {
    pub(super) emitter: &'a Emitter,
    pub(super) clips: &'a HashMap<ClipId, (f64, bool)>,
    pub(super) allow_models: bool,
    pub(super) actor: Option<&'a ActorFrame>,
    pub(super) frame: EffectFrame<'a, 'view, 'data>,
    pub(super) previous_basis: Mat4,
    pub(super) node: Mat4,
    pub(super) start: f64,
    pub(super) inherited: Vec3,
    pub(super) follow: Vec3,
}
pub(super) struct OccurrenceFrame {
    pub(super) node: Mat4,
    pub(super) basis: Mat4,
    pub(super) fraction: f32,
    pub(super) visible: bool,
}
impl BirthContext<'_, '_, '_> {
    pub(super) fn sample_frame(
        &self,
        time: f64,
        sample: &RetimedAnimation<'_, '_, '_>,
        previous: Mat4,
        scratch: &mut Vec<usize>,
    ) -> OccurrenceFrame {
        let fraction = if self.frame.time > self.start {
            ((time - self.start) / (self.frame.time - self.start)) as f32
        } else {
            1.
        };
        let basis = interpolated_transform(self.previous_basis, self.frame.basis_world, fraction);
        let (node, visible) =
            if let Some(pose) = self.frame.pose {
                let (authored, visible) = pose
                    .at_offset(time - self.frame.time)
                    .sample_feature_node(self.emitter.node_index, sample, scratch);
                (self.emitter.emitter_frame(basis * authored), visible)
            } else {
                (
                    interpolated_transform(previous, self.node, fraction),
                    self.frame
                        .node_visible
                        .get(self.emitter.node_index)
                        .copied()
                        .unwrap_or(true),
                )
            };
        OccurrenceFrame {
            node,
            basis,
            fraction,
            visible,
        }
    }
    pub(super) fn available(
        &self,
        state: &EmitterInstance,
        used: usize,
        authorized: bool,
    ) -> usize {
        if !authorized {
            return 0;
        }
        MAX_BIRTHS_PER_UPDATE
            .saturating_sub(used)
            .min(self.emitter.capacity().saturating_sub(state.births.len()))
    }
    /// Both continuous and sampled-burst sources share bounded admission, model
    /// work and occurrence packet accounting. The placement function only picks
    /// when each already planned birth occurs.
    pub(super) fn admit(
        &self,
        state: &mut EmitterInstance,
        plan: BirthPlan,
        used: &mut usize,
        occurrence: impl Fn(usize) -> f64,
        mut adopted: Option<&mut AdoptedSource>,
    ) {
        let accepted = if self.emitter.model.is_some() && self.allow_models {
            self.actor
                .map_or(plan.accepted, |actor| plan.accepted.min(actor.remaining()))
        } else {
            plan.accepted
        };
        let work_omitted = plan.accepted - accepted;
        if let Some(actor) = self
            .actor
            .filter(|_| self.emitter.model.is_some() && self.allow_models)
        {
            actor.budget.omit_work(work_omitted as u64);
        }
        let omitted = plan.omitted.saturating_add(work_omitted as u64);
        state.dropped = state.dropped.saturating_add(omitted);
        let first = state.ordinal;
        for index in 0..accepted {
            self.push(state, occurrence(index), used, adopted.as_deref_mut());
        }
        state.ordinal = first.wrapping_add(plan.requested);
    }
    fn squirt_sample(
        &self,
        time: f64,
        state: &mut EmitterInstance,
    ) -> Result<SquirtSample, SquirtError> {
        let (clip, shared) =
            retimed_selection(&self.frame, time).ok_or(SquirtError::NonFiniteInput)?;
        let sample = self.frame.sample.at_times(clip, shared);
        let at = self.sample_frame(time, &sample, state.previous_node, &mut state.pose_scratch);
        let ended = self
            .frame
            .selection
            .clip
            .and_then(|id| self.clips.get(&id))
            .is_some_and(|(duration, looping)| !*looping && clip.as_seconds() >= *duration);
        let controls = self.emitter.controls.as_ref().expect("M2 controls");
        let allowed = self.frame.root_visible
            && at.visible
            && !ended
            && sample.animated(&controls.activation);
        SquirtSample::new(sample.animated(&self.emitter.rate), allowed)
    }
    /// Called once per actor interval; internal integration quanta never observe
    /// edges. Seek rebases the skipped destination before any real elapsed time.
    pub(super) fn observe_squirt(
        &self,
        state: &mut EmitterInstance,
        delta: f64,
    ) -> Result<Option<SquirtBurst>, SquirtError> {
        if self.frame.change == EffectChange::Seek {
            let destination = self.squirt_sample(self.start, state);
            let _ = state.squirt.observe(
                destination
                    .as_ref()
                    .copied()
                    .unwrap_or_else(|_| SquirtSample::inactive()),
                SquirtUpdate::Seek,
            );
            destination?;
        }
        if delta <= 0. {
            return Ok(state
                .squirt
                .observe(SquirtSample::inactive(), SquirtUpdate::Hold));
        }
        match self.squirt_sample(self.frame.time, state) {
            Ok(sample) => Ok(state.squirt.observe(sample, SquirtUpdate::Advance)),
            Err(error) => {
                let _ = state
                    .squirt
                    .observe(SquirtSample::inactive(), SquirtUpdate::Advance);
                Err(error)
            }
        }
    }
    pub(super) fn push(
        &self,
        state: &mut EmitterInstance,
        time: f64,
        births: &mut usize,
        adopted: Option<&mut AdoptedSource>,
    ) {
        let frame = &self.frame;
        let emitter = self.emitter;
        let ordinal = state.ordinal;
        state.ordinal = state.ordinal.wrapping_add(1);
        if emitter.model.is_some() && !self.allow_models {
            state.dropped = state.dropped.saturating_add(1);
            return;
        }
        let lease = if emitter.model.is_some() {
            match self.actor.map(ActorFrame::reserve).transpose() {
                Ok(lease) => lease,
                Err(_) => {
                    state.dropped = state.dropped.saturating_add(1);
                    return;
                }
            }
        } else {
            None
        };
        let Some((clip, shared)) = retimed_selection(frame, time) else {
            return;
        };
        if frame
            .selection
            .clip
            .and_then(|id| self.clips.get(&id))
            .is_some_and(|(duration, looping)| !*looping && clip.as_seconds() >= *duration)
        {
            return;
        }
        let sample = frame.sample.at_times(clip, shared);
        let at = self.sample_frame(time, &sample, state.previous_node, &mut state.pose_scratch);
        if !at.visible || !frame.root_visible {
            return;
        }
        let birth_basis = at.basis;
        let birth_node = at.node;
        let fraction = at.fraction;
        if state.births.len() >= emitter.capacity() || *births >= MAX_BIRTHS_PER_UPDATE {
            state.dropped = state.dropped.saturating_add(1);
            return;
        }
        *births += 1;
        if let Some((mut birth, appearance)) = emitter.birth_posed(
            &sample,
            time,
            birth_node,
            birth_basis,
            self.inherited,
            ordinal,
            |candidate| {
                if let Some(pose) = frame.pose {
                    return Some(pose.at_offset(time - frame.time).sample_joint_span(
                        candidate.node,
                        &sample,
                        &mut state.pose_scratch,
                    ));
                }
                let joint = |index: usize, pivot: Vec3| {
                    frame
                        .node_poses
                        .get(index)
                        .map(|matrix| matrix.transform_point3(pivot))
                };
                let child = joint(candidate.node, candidate.pivot)?;
                let parent = candidate
                    .parent
                    .and_then(|(index, pivot)| joint(index, pivot));
                Some((child, parent))
            },
        ) {
            let surviving = frame.time - birth.time < f64::from(birth.lifetime);
            // Geometry admission is part of accepting a source, even if its
            // independently living trail outlasts the endpoint-dead model birth.
            let geometry = if emitter.model.is_some() && state.models.is_some() {
                if !surviving && adopted.is_none() {
                    return;
                }
                let Some(sidecar) =
                    self.geometry_birth(&birth, birth_node, ordinal, appearance.as_ref(), lease)
                else {
                    state.dropped = state.dropped.saturating_add(1);
                    return;
                };
                Some(sidecar)
            } else {
                None
            };
            if let Some(source) = adopted {
                source.newborn(&birth, self.follow, fraction);
            }
            if surviving {
                // Newborns follow only emitter translation after occurrence.
                birth.origin += self.follow * (1. - fraction.clamp(0., 1.));
                if let (Some(sidecar), Some(models)) = (geometry, &mut state.models) {
                    birth.appearance_index = models.births.len() as u32;
                    models.births.push(sidecar);
                    state.births.push(birth);
                } else {
                    state.push_sprite(birth, appearance);
                }
            }
        }
    }
    fn geometry_birth(
        &self,
        birth: &Birth,
        node: Mat4,
        ordinal: u64,
        appearance: Option<&BirthAppearance>,
        lease: Option<ModelActorLease>,
    ) -> Option<GeometryBirth> {
        let emitter = self.emitter;
        let model = emitter.model.as_ref()?;
        let basis = ModelBasis::from_frame(node)?;
        let mut seed = emitter.seed ^ ordinal.wrapping_mul(0x9e3779b97f4a7c15);
        let unit = Vec3::new(random(&mut seed), random(&mut seed), random(&mut seed));
        let negate = if emitter.flags.negate_spin_random() {
            [
                random(&mut seed) < 0.5,
                random(&mut seed) < 0.5,
                random(&mut seed) < 0.5,
            ]
        } else {
            [false; 3]
        };
        let appearance = match appearance {
            Some(BirthAppearance::Age(value)) => *value,
            None => Default::default(),
        };
        let initial_size = emitter
            .appearance
            .scale
            .as_ref()
            .and_then(|curve| curve.sample(ModelTime::seconds(0.).unwrap()))
            .unwrap_or(Vec2::ONE);
        let twinkle = emitter
            .age_appearance
            .as_ref()
            .map_or(1., |style| style.sample(&appearance, 0.).scale);
        let position = if emitter.flags.attached() {
            node.transform_point3(birth.origin)
        } else {
            birth.origin
        };
        let initial_transform = basis.sample(
            position,
            initial_size,
            twinkle,
            model.template.coordinates.parent_basis.scale,
            Vec3::ZERO,
            0.,
        )?;
        Some(GeometryBirth {
            lease,
            initial_transform,
            ordinal,
            basis,
            omega: tumble(model.minimum, model.maximum, unit, negate),
            appearance,
        })
    }
}
