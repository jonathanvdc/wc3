//! Per-instance PRE2 spawning and lifetime bookkeeping; motion is evaluated on the GPU.
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;
use std::sync::Arc;
use wc3::model::emitters::{Particle2Frames, ParticleEmitter2};

use crate::effects::simulation::{
    birth_animation, sequence_ended, simulation_delta, EmissionPhase, EmissionRounding, EmitterRng,
    LiveRecords, SimulationClock,
};

use super::emission::SquirtTracker;
use super::render::{ParticleEmitterUniform, ParticleInstance, ParticleInstances};
use crate::animation::pose::{sample_emitter_transform, EmitterNodes};
use crate::animation::{sample, sample_value, Wc3Animation};

const MAX_PARTICLES: usize = 8192;

struct Particle {
    birth_time: f64,
    lifetime: f32,
}

#[derive(Component)]
pub(crate) struct Particle2State {
    pub(crate) root: Entity,
    pub(crate) node: Entity,
    pub(crate) definition: ParticleEmitter2,
    live: LiveRecords<ParticleInstance, Particle>,
    live_indices: Arc<Vec<u32>>,
    clock: SimulationClock,
    emission: EmissionPhase,
    squirt: SquirtTracker,
    rng: EmitterRng,
}

impl Particle2State {
    pub(crate) fn new(root: Entity, node: Entity, definition: ParticleEmitter2) -> Self {
        Self {
            root,
            node,
            rng: EmitterRng::new(definition.node.object_id),
            definition,
            live: LiveRecords::default(),
            live_indices: Arc::default(),
            clock: SimulationClock::default(),
            emission: EmissionPhase::default(),
            squirt: SquirtTracker::default(),
        }
    }

    fn spawn(&mut self, animation: &Wc3Animation, transform: &GlobalTransform, birth_time: f64) {
        if self.live.len() >= MAX_PARTICLES {
            return;
        }
        let width = sample_value(&self.definition.width, animation).abs();
        let length = sample_value(&self.definition.length, animation).abs();
        let latitude = sample_value(&self.definition.latitude, animation).to_radians();
        let variation = sample_value(&self.definition.variation, animation);
        let speed = sample_value(&self.definition.speed, animation);
        let gravity = sample_value(&self.definition.gravity, animation);
        let lifetime = self.definition.life_span;
        if !lifetime.is_finite() || lifetime <= 0.0 {
            return;
        }
        let local = Vec3::new(
            self.rng.signed() * width * 0.5,
            self.rng.signed() * length * 0.5,
            0.0,
        );
        let yaw = self.rng.signed() * latitude;
        let pitch = if self.definition.node.flags.line_emitter() {
            0.0
        } else {
            self.rng.signed() * latitude
        };
        let direction = Quat::from_rotation_z(FRAC_PI_2)
            * Quat::from_rotation_y(yaw)
            * Quat::from_rotation_x(pitch)
            * Vec3::Z;
        let speed = speed * (1.0 + self.rng.signed() * variation);
        let model_space = self.definition.node.flags.model_space();
        let world_scale = transform.affine().matrix3;
        let position = if model_space {
            local
        } else {
            transform.transform_point(local)
        };
        let velocity = if model_space {
            direction * speed
        } else {
            transform.affine().transform_vector3(direction * speed)
        };
        // Quad dimensions use the lengths of the transformed local axes, frozen
        // at birth in both spaces. ModelSpace later transforms particle motion,
        // while the shader applies these size multipliers to world-space offsets.
        let size_scale = Vec3::new(
            world_scale.x_axis.length(),
            world_scale.y_axis.length(),
            world_scale.z_axis.length(),
        );
        let gravity = gravity * world_scale.z_axis.length();
        let record = ParticleInstance::new(
            position,
            velocity,
            gravity,
            birth_time,
            lifetime,
            size_scale,
            self.definition.frames == Particle2Frames::Tail,
        );
        match self.definition.frames {
            Particle2Frames::Head | Particle2Frames::Tail => {
                self.insert_particle(record, birth_time, lifetime)
            }
            Particle2Frames::Both => {
                self.insert_particle(record, birth_time, lifetime);
                if self.live.len() < MAX_PARTICLES {
                    self.insert_particle(record.as_tail(), birth_time, lifetime);
                }
            }
            Particle2Frames::Unknown(_) => {}
        }
    }

    fn insert_particle(&mut self, record: ParticleInstance, birth_time: f64, lifetime: f32) {
        self.live.push(
            record,
            Particle {
                birth_time,
                lifetime,
            },
            MAX_PARTICLES,
        );
    }

    fn advance(
        &mut self,
        animation: &Wc3Animation,
        dt: f64,
        mut transform_at: impl FnMut(&Wc3Animation) -> GlobalTransform,
    ) {
        let reset = self.clock.observe(animation);
        if reset {
            self.emission.reset();
        }
        self.squirt
            .observe(&self.definition.emission_rate, animation, reset);
        self.clock.advance(dt);
        self.emit(animation, dt, &mut transform_at);
        self.retire(self.clock.time);
    }

    fn retire(&mut self, time: f64) {
        // PRE2 lifespan is fixed per emitter, so birth order is expiry order.
        // Visiting only expired particles keeps subframe births inexpensive.
        self.live
            .retire(|particle| time - particle.birth_time >= f64::from(particle.lifetime));
    }

    fn emit(
        &mut self,
        animation: &Wc3Animation,
        dt: f64,
        transform_at: &mut impl FnMut(&Wc3Animation) -> GlobalTransform,
    ) {
        if !animation.playing || dt <= 0.0 {
            return;
        }
        if sequence_ended(animation) {
            return;
        }
        let visible = self
            .definition
            .visibility
            .as_ref()
            .and_then(|track| sample(track, animation))
            .unwrap_or(1.0);
        if visible <= 0.0 {
            return;
        }
        let rate = f64::from(sample_value(&self.definition.emission_rate, animation).max(0.0));
        if !rate.is_finite() {
            return;
        }
        let squirt = self.definition.squirt_enabled();
        let burst_age = if squirt {
            let Some(age) = self
                .squirt
                .burst_age(&self.definition.emission_rate, animation, dt)
            else {
                return;
            };
            age
        } else {
            if rate <= 0.0 {
                return;
            }
            0.0
        };
        let schedule = if squirt {
            None
        } else {
            Some(
                self.emission
                    .continuous(rate, dt, MAX_PARTICLES, EmissionRounding::Quad),
            )
        };
        let count = schedule.as_ref().map_or_else(
            || self.emission.burst(rate, MAX_PARTICLES),
            |schedule| schedule.count as usize,
        );
        if count == 0 {
            return;
        }
        for i in 0..count {
            let birth_time = if squirt {
                self.clock.time - burst_age
            } else {
                let schedule = schedule.as_ref().unwrap();
                // Preserve PRE2's clamped subframe offset arithmetic.
                self.clock.time - dt + schedule.offset(i)
            };
            let birth_animation = birth_animation(animation, self.clock.time - birth_time);
            self.retire(birth_time);
            let transform = transform_at(&birth_animation);
            self.spawn(&birth_animation, &transform, birth_time);
        }
    }
}

pub(crate) fn update_particles(
    time: Res<Time>,
    animations: Query<&Wc3Animation>,
    nodes: EmitterNodes,
    mut emitters: Query<(&mut Particle2State, &mut ParticleInstances)>,
) {
    for (mut state, mut instances) in &mut emitters {
        let (Ok(animation), Ok((transform, _, _, _))) =
            (animations.get(state.root), nodes.get(state.node))
        else {
            continue;
        };
        let dt = simulation_delta(&time, animation);
        let node = state.node;
        let root = state.root;
        state.advance(animation, dt, |birth_animation| {
            sample_emitter_transform(node, root, birth_animation, &nodes).unwrap_or(*transform)
        });
        // Physical draw indices change only when the head or live count changes.
        if state.live_indices.len() != state.live.records.len()
            || (!state.live.records.is_empty()
                && state.live_indices.first().copied() != Some(state.live.records.index(0)))
        {
            state.live_indices = Arc::new(
                (0..state.live.records.len())
                    .map(|i| state.live.records.index(i))
                    .collect(),
            );
        }
        instances.records = state.live.records.clone();
        instances.live_indices = state.live_indices.clone();
        instances.uniform =
            ParticleEmitterUniform::new(&state.definition, transform, state.clock.time);
    }
}

#[cfg(test)]
#[path = "simulation_tests.rs"]
mod tests;
