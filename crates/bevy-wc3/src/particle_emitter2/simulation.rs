//! Per-instance PRE2 spawning and lifetime bookkeeping; motion is evaluated on the GPU.
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;
use std::sync::Arc;
use wc3::model::animation::Animatable;
use wc3::model::emitters::{Particle2Frames, ParticleEmitter2};

use super::render::{ParticleEmitterUniform, ParticleInstance, ParticleInstances};
use crate::animation::{sample, sample_value, track_time, Wc3Animation};

const MAX_PARTICLES: usize = 8192;

struct Particle {
    slot: u32,
    birth_time: f64,
    lifetime: f32,
}

#[derive(Component)]
pub(crate) struct Particle2State {
    pub(crate) root: Entity,
    pub(crate) node: Entity,
    pub(crate) definition: ParticleEmitter2,
    particles: Vec<Particle>,
    records: Arc<Vec<ParticleInstance>>,
    live_indices: Arc<Vec<u32>>,
    free_slots: Vec<u32>,
    simulation_time: f64,
    emission_remainder: f32,
    last_sequence: usize,
    last_elapsed_ms: f64,
    last_cycle: i64,
    last_squirt_key: Option<i32>,
    rng: u64,
}

impl Particle2State {
    pub(crate) fn new(root: Entity, node: Entity, definition: ParticleEmitter2) -> Self {
        Self {
            root,
            node,
            rng: u64::from(definition.node.object_id).wrapping_add(1),
            definition,
            particles: Vec::new(),
            records: Arc::default(),
            live_indices: Arc::default(),
            free_slots: Vec::new(),
            simulation_time: 0.0,
            emission_remainder: 0.0,
            last_sequence: usize::MAX,
            last_elapsed_ms: 0.0,
            last_cycle: -1,
            last_squirt_key: None,
        }
    }

    fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u32 << 24) as f32
    }

    fn signed(&mut self) -> f32 {
        self.random() * 2.0 - 1.0
    }

    fn spawn(&mut self, animation: &Wc3Animation, transform: &GlobalTransform) {
        if self.particles.len() >= MAX_PARTICLES {
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
            self.signed() * width * 0.5,
            self.signed() * length * 0.5,
            0.0,
        );
        let yaw = self.signed() * latitude;
        let pitch = if self.definition.node.flags.line_emitter() {
            0.0
        } else {
            self.signed() * latitude
        };
        let direction = Quat::from_rotation_z(FRAC_PI_2)
            * Quat::from_rotation_y(yaw)
            * Quat::from_rotation_x(pitch)
            * Vec3::Z;
        let speed = speed * (1.0 + self.signed() * variation);
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
        let size_scale = world_scale.x_axis.length();
        let gravity = gravity * world_scale.z_axis.length();
        let record = ParticleInstance::new(
            position,
            velocity,
            gravity,
            self.simulation_time,
            lifetime,
            size_scale,
            self.definition.frames == Particle2Frames::Tail,
        );
        match self.definition.frames {
            Particle2Frames::Head | Particle2Frames::Tail => self.insert_particle(record, lifetime),
            Particle2Frames::Both => {
                self.insert_particle(record, lifetime);
                if self.particles.len() < MAX_PARTICLES {
                    self.insert_particle(record.as_tail(), lifetime);
                }
            }
            Particle2Frames::Unknown(_) => {}
        }
    }

    fn insert_particle(&mut self, record: ParticleInstance, lifetime: f32) {
        let slot = self.free_slots.pop().unwrap_or(self.records.len() as u32);
        let records = Arc::make_mut(&mut self.records);
        if slot as usize == records.len() {
            records.push(record);
        } else {
            records[slot as usize] = record;
        }
        self.particles.push(Particle {
            slot,
            birth_time: self.simulation_time,
            lifetime,
        });
    }

    fn advance(&mut self, animation: &Wc3Animation, transform: &GlobalTransform, dt: f64) {
        let reset =
            animation.sequence != self.last_sequence || animation.elapsed_ms < self.last_elapsed_ms;
        let cycle = if let Some(global_id) = self
            .definition
            .emission_rate
            .track()
            .and_then(|track| track.global_sequence_id())
        {
            let length = animation
                .global_sequences
                .get(global_id as usize)
                .copied()
                .unwrap_or(0);
            if length == 0 {
                0
            } else {
                (animation.elapsed_ms / length as f64).floor() as i64
            }
        } else {
            animation
                .sequences
                .get(animation.sequence)
                .map(|sequence| {
                    let length = sequence.interval[1].saturating_sub(sequence.interval[0]);
                    if sequence.flags.non_looping() || length == 0 {
                        0
                    } else {
                        (animation.elapsed_ms / length as f64).floor() as i64
                    }
                })
                .unwrap_or(0)
        };
        if reset {
            self.emission_remainder = 0.0;
            self.last_squirt_key = None;
        }
        if cycle != self.last_cycle {
            self.last_squirt_key = None;
        }
        self.last_sequence = animation.sequence;
        self.last_elapsed_ms = animation.elapsed_ms;
        self.last_cycle = cycle;
        self.simulation_time += dt;
        self.particles.retain(|particle| {
            let alive = self.simulation_time - particle.birth_time < f64::from(particle.lifetime);
            if !alive {
                self.free_slots.push(particle.slot);
            }
            alive
        });
        if !animation.playing || dt <= 0.0 {
            return;
        }
        if animation
            .sequences
            .get(animation.sequence)
            .is_some_and(|sequence| {
                sequence.flags.non_looping()
                    && animation.elapsed_ms
                        >= sequence.interval[1].saturating_sub(sequence.interval[0]) as f64
            })
        {
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
        let rate = sample_value(&self.definition.emission_rate, animation).max(0.0);
        if self.definition.squirt_enabled() {
            let key = emission_key(&self.definition.emission_rate, animation).or(Some(0));
            if self.last_squirt_key != key {
                self.emission_remainder += rate;
                self.last_squirt_key = key;
            }
        } else {
            self.emission_remainder += rate * dt as f32;
        }
        let count = (self.emission_remainder.floor() as usize).min(MAX_PARTICLES);
        self.emission_remainder = (self.emission_remainder - count as f32).min(1.0);
        for _ in 0..count {
            self.spawn(animation, transform);
        }
    }
}

fn emission_key(value: &Animatable<f32>, animation: &Wc3Animation) -> Option<i32> {
    let track = value.track()?;
    let (time, interval) = track_time(track, animation)?;
    track.key_frame_at_or_before_in(time, interval)
}

pub(crate) fn update_particles(
    time: Res<Time>,
    animations: Query<&Wc3Animation>,
    nodes: Query<&GlobalTransform>,
    mut emitters: Query<(&mut Particle2State, &mut ParticleInstances)>,
) {
    for (mut state, mut instances) in &mut emitters {
        let (Ok(animation), Ok(transform)) = (animations.get(state.root), nodes.get(state.node))
        else {
            continue;
        };
        let dt = if animation.playing {
            (time.delta_secs_f64() * animation.speed).max(0.0)
        } else {
            0.0
        };
        state.advance(animation, transform, dt);
        if state
            .live_indices
            .iter()
            .copied()
            .ne(state.particles.iter().map(|particle| particle.slot))
        {
            state.live_indices = Arc::new(
                state
                    .particles
                    .iter()
                    .map(|particle| particle.slot)
                    .collect(),
            );
        }
        instances.records = state.records.clone();
        instances.live_indices = state.live_indices.clone();
        instances.uniform =
            ParticleEmitterUniform::new(&state.definition, transform, state.simulation_time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc3::model::animation::{Sequence, Track, ValueKeyframe};
    use wc3::model::scene::Node;

    fn animation() -> Wc3Animation {
        Wc3Animation {
            sequence: 0,
            elapsed_ms: 0.0,
            speed: 1.0,
            playing: true,
            sequences: vec![Sequence::new("Stand", [0, 1000]).unwrap()],
            global_sequences: vec![],
        }
    }

    fn emitter() -> ParticleEmitter2 {
        let mut emitter = ParticleEmitter2::new(Node::new("Particles", 0).unwrap());
        emitter.life_span = 2.0;
        emitter.emission_rate = 10.0.into();
        emitter.frames = Particle2Frames::Both;
        emitter.time = 0.5;
        emitter.particle_scaling = [1.0, 2.0, 0.0];
        emitter.alpha = [255, 128, 0];
        emitter.segment_colors = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        emitter.rows = 2;
        emitter.columns = 2;
        emitter
    }

    #[test]
    fn emission_is_fractional_and_head_tail_are_independent() {
        let root = Entity::from_bits(1);
        let mut state = Particle2State::new(root, root, emitter());
        let animation = animation();
        state.advance(&animation, &GlobalTransform::default(), 0.15);
        assert_eq!(state.particles.len(), 2);
        state.advance(&animation, &GlobalTransform::default(), 0.15);
        assert_eq!(state.particles.len(), 6);
        assert_eq!(
            state
                .particles
                .iter()
                .filter(|particle| state.records[particle.slot as usize].is_tail())
                .count(),
            3
        );
        assert_eq!(state.records.len(), 6);
    }

    #[test]
    fn no_live_particles_produce_no_gpu_instances() {
        let root = Entity::from_bits(1);
        let state = Particle2State::new(root, root, emitter());
        assert!(state.records.is_empty());
    }

    #[test]
    fn squirt_retriggers_when_sequence_loops() {
        let root = Entity::from_bits(1);
        let mut definition = emitter();
        definition.frames = Particle2Frames::Head;
        definition.set_squirt_enabled(true);
        definition.emission_rate = Animatable::Animated(
            Track::step(
                vec![
                    ValueKeyframe {
                        frame: 0,
                        value: 2.0,
                    },
                    ValueKeyframe {
                        frame: 500,
                        value: 3.0,
                    },
                ],
                None,
            )
            .unwrap(),
        );
        let mut state = Particle2State::new(root, root, definition);
        let mut animation = animation();
        state.advance(&animation, &GlobalTransform::default(), 0.01);
        assert_eq!(state.particles.len(), 2);
        animation.elapsed_ms = 600.0;
        state.advance(&animation, &GlobalTransform::default(), 0.01);
        assert_eq!(state.particles.len(), 5);
        animation.elapsed_ms = 1100.0;
        state.advance(&animation, &GlobalTransform::default(), 0.01);
        assert_eq!(state.particles.len(), 7);
    }

    #[test]
    fn particles_keep_spawn_records_and_reuse_expired_slots() {
        let root = Entity::from_bits(1);
        let mut definition = emitter();
        definition.frames = Particle2Frames::Head;
        definition.gravity = 10.0.into();
        let mut state = Particle2State::new(root, root, definition);
        let mut animation = animation();
        state.advance(&animation, &GlobalTransform::default(), 0.1);
        let records = state.records.clone();
        animation.playing = false;
        state.advance(&animation, &GlobalTransform::default(), 1.0);
        assert!(Arc::ptr_eq(&records, &state.records));
        assert_eq!(state.particles.len(), 1);
        state.advance(&animation, &GlobalTransform::default(), 1.0);
        assert!(state.particles.is_empty());
        animation.playing = true;
        state.advance(&animation, &GlobalTransform::default(), 0.1);
        assert_eq!(state.records.len(), 1);
        assert_eq!(state.particles[0].slot, 0);
        assert_ne!(records[0], state.records[0]);
    }
    #[test]
    fn update_respects_pause_speed_and_keeps_clock_across_sequence_reset() {
        use std::time::Duration;
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        app.add_systems(Update, update_particles);
        let mut animation = animation();
        animation.speed = 2.0;
        let root = app
            .world_mut()
            .spawn((animation, GlobalTransform::default()))
            .id();
        let definition = emitter();
        let entity = app
            .world_mut()
            .spawn((
                Particle2State::new(root, root, definition.clone()),
                ParticleInstances {
                    records: Arc::default(),
                    live_indices: Arc::default(),
                    uniform: ParticleEmitterUniform::default(),
                    texture: None,
                    filter: definition.filter_mode,
                    priority_plane: 0,
                    sort_far: false,
                },
            ))
            .id();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f64(0.25));
        app.update();
        assert_eq!(
            app.world()
                .get::<Particle2State>(entity)
                .unwrap()
                .simulation_time,
            0.5
        );
        let records = app
            .world()
            .get::<ParticleInstances>(entity)
            .unwrap()
            .records
            .clone();
        app.world_mut()
            .get_mut::<Wc3Animation>(root)
            .unwrap()
            .playing = false;
        app.update();
        assert_eq!(
            app.world()
                .get::<Particle2State>(entity)
                .unwrap()
                .simulation_time,
            0.5
        );
        assert!(Arc::ptr_eq(
            &records,
            &app.world()
                .get::<ParticleInstances>(entity)
                .unwrap()
                .records
        ));
        let mut animation = app.world_mut().get_mut::<Wc3Animation>(root).unwrap();
        animation.playing = true;
        animation.elapsed_ms = 0.0;
        animation.sequence = 1;
        app.update();
        let state = app.world().get::<Particle2State>(entity).unwrap();
        assert_eq!(state.simulation_time, 1.0);
        assert_eq!(state.particles[0].birth_time, 0.5);
        assert_eq!(state.records[0], records[0]);
    }

    #[test]
    fn head_tail_pair_never_exceeds_particle_limit() {
        let root = Entity::from_bits(1);
        let mut state = Particle2State::new(root, root, emitter());
        let animation = animation();
        let transform = GlobalTransform::default();
        for _ in 0..MAX_PARTICLES / 2 {
            state.spawn(&animation, &transform);
        }
        let records = state.records.clone();
        state.spawn(&animation, &transform);
        assert_eq!(state.particles.len(), MAX_PARTICLES);
        assert_eq!(state.records.len(), MAX_PARTICLES);
        assert!(Arc::ptr_eq(&records, &state.records));
    }
}
