//! Parent emission, adopted source observation and transport publication.
use super::evaluation::BirthContext;
use super::sources::SourceStep;
use super::{
    signed, ImplosionInterval, M2ParticleInstances, M2World, RuntimeM2Particles, SpriteEmitter,
    MAX_BIRTHS_PER_UPDATE,
};
use crate::effects::age_sprites_gpu::AgeSpriteData;
use crate::effects::atlas_sprites::retimed_selection;
use crate::effects::emission::{EffectChange, EmissionClock};
use crate::effects::frame::EffectFrame;
use crate::effects::implosion::bounded_end;
use crate::effects::m2_models::ModelProjectionContext;
use crate::effects::model_budget::ActorFrame;
use bevy::prelude::*;
use wc3::model::values::model::ParticleGenerator;

impl RuntimeM2Particles {
    #[cfg(test)]
    pub fn update(
        &self,
        instances: &mut M2ParticleInstances,
        frame: EffectFrame<'_, '_, '_>,
        meshes: &mut Assets<Mesh>,
        gpu_sprites: &mut Query<&mut AgeSpriteData>,
        statistics: &mut Query<&mut SpriteEmitter>,
    ) {
        let had_births: Vec<_> = instances
            .emitters
            .iter()
            .map(|state| !state.births.is_empty())
            .collect();
        self.update_inner(instances, frame, gpu_sprites, statistics, None, None);
        for ((emitter, state), had_births) in self
            .emitters
            .iter()
            .zip(&mut instances.emitters)
            .zip(had_births)
        {
            // The CPU oracle must preserve the resident sentinel on empty-to-empty
            // updates, while publishing it when the final live birth expires.
            if state.passes.is_empty() && (had_births || !state.births.is_empty()) {
                if let Some(mut mesh) = meshes.get_mut(&state.mesh) {
                    let node = emitter.emitter_frame(
                        frame.basis_world * frame.feature_matrix(emitter.node_index),
                    );
                    emitter.draw(state, &frame, node, &mut mesh, Mat4::IDENTITY);
                }
            }
        }
    }
    pub fn update_bounded(
        &self,
        instances: &mut M2ParticleInstances,
        frame: EffectFrame<'_, '_, '_>,
        gpu_sprites: &mut Query<&mut AgeSpriteData>,
        statistics: &mut Query<&mut SpriteEmitter>,
        world: M2World,
        actor: Option<&ActorFrame>,
    ) {
        self.update_inner(
            instances,
            frame,
            gpu_sprites,
            statistics,
            Some(world),
            actor,
        );
    }
    fn update_inner(
        &self,
        instances: &mut M2ParticleInstances,
        frame: EffectFrame<'_, '_, '_>,
        gpu_sprites: &mut Query<&mut AgeSpriteData>,
        statistics: &mut Query<&mut SpriteEmitter>,
        mut world: Option<M2World>,
        actor: Option<&ActorFrame>,
    ) {
        if !frame.time.is_finite() {
            return;
        }
        let previous = instances.previous_time.unwrap_or(frame.time);
        let backwards = frame.time < previous || frame.change == EffectChange::Reset;
        let start = if backwards { frame.time } else { previous };
        let delta = frame.time - start;
        for (index, (emitter, state)) in self
            .emitters
            .iter()
            .zip(&mut instances.emitters)
            .enumerate()
        {
            let mut adopted = instances.adopted.get_mut(&index);
            if let (Some(source), Some(services)) = (adopted.as_deref_mut(), world.as_mut()) {
                source.refresh(frame.time, services.models, services.commands);
            }
            if world.is_none() {
                adopted = None;
            }
            let had_births = !state.births.is_empty();
            let node =
                emitter.emitter_frame(frame.basis_world * frame.feature_matrix(emitter.node_index));
            if instances.previous_time.is_none() || backwards {
                state.births.clear();
                state.age_appearances.clear();
                if let Some(models) = &mut state.models {
                    models.births.clear();
                    if let Some(world) = world.as_mut().filter(|world| world.geometry.is_some()) {
                        models.children.reset(world.commands);
                    }
                }
                state.fraction = 0.;
                state.ordinal = 0;
                state.previous_node = node;
                state.dropped = 0;
                state.skipped_seconds = 0.;
            }
            if let Some(source) = adopted.as_deref_mut() {
                if backwards {
                    source.reset(frame.time);
                }
            }
            if frame.change == EffectChange::Seek {
                state.previous_node = node;
            }
            let follow = emitter.follow_displacement(state.previous_node, node, delta);
            if let Some(source) = adopted.as_deref_mut() {
                source.begin_frame(
                    &frame,
                    start,
                    state.previous_node,
                    node,
                    &state.births,
                    follow,
                );
            }
            state.retire(frame.time);
            if follow != Vec3::ZERO {
                for birth in &mut state.births {
                    birth.origin += follow;
                }
            }
            let inherited = if delta > 0. {
                (node.w_axis.truncate() - state.previous_node.w_axis.truncate()) / delta as f32
                    * emitter.inherit_velocity
            } else {
                Vec3::ZERO
            };
            let mut births_this_update = 0;
            let births = BirthContext {
                emitter,
                clips: &self.clips,
                frame,
                allow_models: world
                    .as_ref()
                    .and_then(|world| world.geometry.as_ref())
                    .is_none_or(|geometry| geometry.allow_births),
                actor,
                previous_basis: instances.previous_basis.unwrap_or(frame.basis_world),
                node,
                start,
                inherited,
                follow,
            };

            let mut clock = EmissionClock::new(start, frame.time);
            let mut checked_end = start;
            for step in clock.by_ref() {
                if actor
                    .filter(|actor| actor.owned || emitter.model.is_some())
                    .is_some_and(|actor| !actor.step())
                {
                    break;
                }
                checked_end = step.end;
                let at = step.start;
                let end = step.end;
                let tick = step.tick;
                let rate_time = step.sample_time;
                let Some((clip, shared)) = retimed_selection(&frame, rate_time) else {
                    continue;
                };
                let sample = frame.sample.at_times(clip, shared);
                let ended = frame
                    .selection
                    .clip
                    .and_then(|id| self.clips.get(&id))
                    .is_some_and(|(duration, looping)| !*looping && clip.as_seconds() >= *duration);
                let visible = if let Some(pose) = frame.pose {
                    pose.sample_feature_node(emitter.node_index, &sample, &mut state.pose_scratch)
                        .1
                } else {
                    frame
                        .node_visible
                        .get(emitter.node_index)
                        .copied()
                        .unwrap_or(true)
                };
                let controls = emitter.controls.as_ref().expect("M2 controls");
                let active = if visible && !ended && sample.animated(&controls.activation) {
                    1.
                } else {
                    0.
                };
                let base_rate = sample.animated(&emitter.rate).max(0.) * active;
                let mut rate_seed = emitter.seed ^ (tick as u64).wrapping_mul(0x517cc1b727220a95);
                let rate = (base_rate + signed(&mut rate_seed) * emitter.rate_variation * active)
                    .max(0.) as f64;
                if rate > 0. {
                    let prior = state.fraction;
                    let amount = prior + (end - at) * rate;
                    let count = amount.floor() as usize;
                    state.fraction = amount - count as f64;
                    let available = MAX_BIRTHS_PER_UPDATE
                        .saturating_sub(births_this_update)
                        .min(emitter.capacity().saturating_sub(state.births.len()));
                    let local_accepted = count.min(available);
                    let accepted = if emitter.model.is_some() && births.allow_models {
                        births.actor.map_or(local_accepted, |actor| {
                            local_accepted.min(actor.remaining())
                        })
                    } else {
                        local_accepted
                    };
                    if let Some(actor) = births
                        .actor
                        .filter(|_| emitter.model.is_some() && births.allow_models)
                    {
                        actor.budget.omit_work((local_accepted - accepted) as u64);
                    }
                    let omitted = count - accepted;
                    state.dropped = state.dropped.saturating_add(omitted as u64);
                    for index in 0..accepted {
                        let time = at + (index as f64 + 1. - prior) / rate;
                        births.push(state, time, &mut births_this_update, adopted.as_deref_mut());
                    }
                    state.ordinal = state.ordinal.wrapping_add(omitted as u64);
                }
                if let Some(source) = adopted.as_deref_mut() {
                    let (clip, shared) =
                        retimed_selection(&frame, step.end).unwrap_or((clip, shared));
                    let sample = frame.sample.at_times(clip, shared);
                    let at = births.sample_frame(
                        step.end,
                        &sample,
                        state.previous_node,
                        &mut state.pose_scratch,
                    );
                    source.step(
                        SourceStep {
                            emitter,
                            start: step.start,
                            end: step.end,
                            frame_start: start,
                            frame_end: frame.time,
                            follow,
                            centers: [
                                state.previous_node.w_axis.truncate(),
                                node.w_axis.truncate(),
                            ],
                            node: at.node,
                            basis: at.basis,
                        },
                        actor,
                    );
                }
            }
            if let (Some(source), Some(services)) = (adopted, world.as_mut()) {
                source.finish_frame(&frame, checked_end, node, actor);
                source.publish(&frame, node, gpu_sprites, statistics, services.commands);
            }
            let emission_omitted = (frame.time - checked_end).max(0.);
            if let Some(actor) = actor.filter(|actor| actor.owned || emitter.model.is_some()) {
                actor.budget.skip_seconds(emission_omitted);
            }
            if emitter.generator == ParticleGenerator::Sphere && emitter.flags.implosion_filter() {
                checked_end = bounded_end(start, checked_end, state.births.len());
            }
            let retirement_omitted = (frame.time - checked_end).max(0.);
            state.skipped_seconds += emission_omitted.max(retirement_omitted);
            let centers = [
                state.previous_node.w_axis.truncate(),
                node.w_axis.truncate(),
            ];
            emitter.retire_implosion(
                state,
                ImplosionInterval {
                    start,
                    end: frame.time,
                    checked_end,
                    centers,
                    follow,
                },
            );

            let node_changed = state.previous_node != node;
            state.previous_node = node;
            let redraw = backwards
                || delta != 0.
                || instances.previous_basis != Some(frame.basis_world)
                || node_changed
                || (!had_births && !state.births.is_empty());
            if emitter.model.is_none() && redraw && (had_births || !state.births.is_empty()) {
                if let Some(pass) = state.passes.first() {
                    if let Ok(mut data) = gpu_sprites.get_mut(pass.entity) {
                        emitter.sample_gpu(state, &frame, node, &mut data);
                    }
                }
            }
            if let (Some(model), Some(models), Some(world), Some(pass)) = (
                &emitter.model,
                &mut state.models,
                world.as_mut(),
                state.passes.first(),
            ) {
                if let Some(geometry) = world.geometry.as_mut() {
                    models.projections.clear();
                    for birth in &state.births {
                        if let Some(sidecar) =
                            models.births.get_mut(birth.appearance_index as usize)
                        {
                            let mut projection =
                                emitter.model_projection(birth, sidecar, model, frame.time, node);
                            projection.lease = sidecar.lease.take();
                            models.projections.push(projection);
                        }
                    }
                    models.children.synchronize(
                        world.commands,
                        ModelProjectionContext {
                            root: geometry.root,
                            owner: pass.entity,
                            visible: frame.root_visible,
                            driving_camera: geometry.driving_camera,
                        },
                        &model.template,
                        geometry.outputs,
                        models.projections.drain(..),
                    );
                    if let Ok(mut stats) = geometry.statistics.get_mut(pass.entity) {
                        if stats.alive != state.births.len() {
                            stats.alive = state.births.len();
                        }
                        if stats.omitted != state.dropped {
                            stats.omitted = state.dropped;
                        }
                        if stats.skipped_seconds != state.skipped_seconds {
                            stats.skipped_seconds = state.skipped_seconds;
                        }
                    }
                }
            }
            if let Some(pass) = state.passes.first() {
                if let Ok(mut stats) = statistics.get_mut(pass.entity) {
                    if stats.alive != state.births.len() {
                        stats.alive = state.births.len();
                    }
                    if stats.dropped != state.dropped {
                        stats.dropped = state.dropped;
                    }
                    if stats.skipped_seconds != state.skipped_seconds {
                        stats.skipped_seconds = state.skipped_seconds;
                    }
                }
            }
        }
        instances.previous_time = Some(frame.time);
        instances.previous_basis = Some(frame.basis_world);
    }
}
