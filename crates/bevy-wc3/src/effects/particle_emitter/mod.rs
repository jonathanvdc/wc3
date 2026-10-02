//! Classic PREM model particles. Birth transforms and physical values are frozen
//! in world space; each child owns an independent sequence-zero animation.
use bevy::prelude::*;
use std::f32::consts::TAU;
use wc3::model::emitters::ParticleEmitter;

use crate::animation::pose::AnimatedNode;
use crate::animation::Wc3Animation;
use crate::animation::{sample, sample_value};
use crate::assets::loader::Wc3ModelAsset;
use crate::effects::simulation::{
    birth_animation, sequence_ended, simulation_delta, EmissionPhase, EmissionRounding, EmitterRng,
    SimulationClock,
};
use crate::instance::{Wc3ModelInstance, Wc3ModelOwner};

use crate::animation::pose::{resolve_pose, PoseInput};
use std::collections::{HashMap, HashSet};

mod spawn;
pub(crate) use spawn::spawn_particles;

const MAX_PARTICLES: usize = 1024;

#[derive(Component)]
pub(crate) struct ParticleState {
    root: Entity,
    node: Entity,
    definition: ParticleEmitter,
    model: Handle<Wc3ModelAsset>,
    particles: Vec<Particle>,
    emission: EmissionPhase,
    clock: SimulationClock,
    rng: EmitterRng,
}

struct Particle {
    entity: Entity,
    age: f64,
    lifetime: f64,
    origin: Vec3,
    velocity: Vec3,
    gravity: f32,
}

/// Used to initialize late-loaded child clocks and follow the emitter's pause
/// and speed without inheriting its transform or visibility.
#[derive(Component)]
pub(crate) struct ParticleModel {
    root: Entity,
    age: f64,
}

impl ParticleState {
    pub(crate) fn new(
        root: Entity,
        node: Entity,
        definition: ParticleEmitter,
        model: Handle<Wc3ModelAsset>,
    ) -> Self {
        Self {
            root,
            node,
            rng: EmitterRng::new(definition.node.object_id),
            definition,
            model,
            particles: Vec::new(),
            emission: EmissionPhase::default(),
            clock: SimulationClock::default(),
        }
    }
}

pub(crate) fn animate_particle_models(
    time: Res<Time>,
    mut animations: Query<&mut Wc3Animation>,
    particles: Query<(Entity, &ParticleModel)>,
) {
    for (entity, particle) in &particles {
        let Ok(parent) = animations.get(particle.root) else {
            continue;
        };
        let dt = simulation_delta(&time, parent);
        let playing = parent.playing;
        let speed = parent.speed;
        let Ok(mut animation) = animations.get_mut(entity) else {
            continue;
        };
        animation.elapsed_ms = (particle.age + dt) * 1000.0;
        // advance_animation already ran. Replace its result with the particle
        // age, including the partial update at birth and any loading delay.
        animation.playing = playing;
        animation.speed = speed;
    }
}

fn place(particle: &Particle, transform: &mut Transform) {
    let age = particle.age as f32;
    transform.translation =
        particle.origin + particle.velocity * age - Vec3::Z * (0.5 * particle.gravity * age * age);
}

type NodeTransforms<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        Option<&'static AnimatedNode>,
        Option<&'static ChildOf>,
        Option<&'static Visibility>,
    ),
>;

fn root_visible(mut entity: Entity, nodes: &NodeTransforms) -> bool {
    loop {
        let Ok((_, _, parent, visibility)) = nodes.get(entity) else {
            return true;
        };
        match visibility {
            Some(Visibility::Hidden) => return false,
            Some(Visibility::Visible) => return true,
            _ => {}
        }
        let Some(parent) = parent else { return true };
        entity = parent.parent();
    }
}

fn sample_emitter_transform(
    entity: Entity,
    root: Entity,
    animation: &Wc3Animation,
    nodes: &NodeTransforms,
) -> Option<GlobalTransform> {
    let lookup = |entity| {
        let (transform, node, parent, _) = nodes.get(entity).ok()?;
        let sampled = node.filter(|node| node.root == root);
        Some(PoseInput {
            world: None,
            anchor: node
                .filter(|node| node.root != root)
                .and_then(AnimatedNode::camera_anchor),
            local: sampled.map_or(*transform, |node| node.sample_transform(animation)),
            parent: parent.map(ChildOf::parent),
            node: sampled.map(|node| node.pose_sample(node.camera)),
        })
    };
    resolve_pose(entity, &lookup, &mut HashMap::new(), &mut HashSet::new())
        .map(|pose| GlobalTransform::from(Mat4::from(pose.affine)))
}

pub(crate) fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    animations: Query<&Wc3Animation>,
    mut transforms: ParamSet<(NodeTransforms, Query<(&mut Transform, &mut ParticleModel)>)>,
    sources: Res<Assets<Wc3ModelAsset>>,
    mut emitters: Query<&mut ParticleState>,
) {
    for mut state in &mut emitters {
        let Ok(animation) = animations.get(state.root) else {
            continue;
        };
        let dt = simulation_delta(&time, animation);
        if state.clock.observe(animation) {
            for particle in state.particles.drain(..) {
                commands.entity(particle.entity).despawn();
            }
            state.emission.reset();
        }
        state.clock.advance(dt);
        let root_visible = root_visible(state.root, &transforms.p0());
        {
            let mut models = transforms.p1();
            state.particles.retain_mut(|particle| {
                particle.age += dt;
                if particle.age >= particle.lifetime {
                    commands.entity(particle.entity).despawn();
                    return false;
                }
                let Ok((mut transform, mut model)) = models.get_mut(particle.entity) else {
                    return false;
                };
                commands.entity(particle.entity).insert(if root_visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
                model.age = particle.age;
                place(particle, &mut transform);
                true
            });
        }
        let nodes = transforms.p0();
        let visible = state
            .definition
            .visibility
            .as_ref()
            .and_then(|track| sample(track, animation))
            .unwrap_or(1.0);
        let ended = sequence_ended(animation);
        if dt <= 0.0
            || ended
            || visible <= 0.1
            || !root_visible
            || !sources.contains(state.model.id())
        {
            continue;
        }
        let rate = f64::from(sample_value(&state.definition.emission_rate, animation));
        if !rate.is_finite() || rate <= 0.0 {
            continue;
        }
        let schedule = state
            .emission
            .continuous(rate, dt, MAX_PARTICLES, EmissionRounding::Model);
        for index in 0..schedule.count as usize {
            if state.particles.len() >= MAX_PARTICLES {
                break;
            }
            let age = schedule.age(index);
            let birth = birth_animation(animation, age);
            let lifetime = f64::from(sample_value(&state.definition.life_span, &birth));
            let speed = sample_value(&state.definition.initial_velocity, &birth);
            let gravity = sample_value(&state.definition.gravity, &birth);
            // PREM latitude is interpreted directly in radians; longitude is ignored.
            let latitude = sample_value(&state.definition.latitude, &birth);
            if !lifetime.is_finite()
                || lifetime <= age
                || !speed.is_finite()
                || !gravity.is_finite()
                || !latitude.is_finite()
            {
                continue;
            }
            let Some(world) = sample_emitter_transform(state.node, state.root, &birth, &nodes)
            else {
                continue;
            };
            let (scale, rotation, origin) = world.to_scale_rotation_translation();
            let yaw = state.rng.random() * TAU;
            let pitch = (state.rng.random() * 2.0 - 1.0) * latitude;
            let direction = Quat::from_rotation_z(yaw) * Quat::from_rotation_y(pitch) * Vec3::Z;
            let velocity = (rotation * direction) * speed * scale;
            let mut transform = Transform {
                translation: origin,
                rotation: Quat::from_rotation_z(state.rng.random() * TAU),
                scale,
            };
            let entity = commands
                .spawn((
                    Wc3ModelInstance::new(state.model.clone()),
                    Wc3ModelOwner(state.root),
                    ParticleModel {
                        root: state.root,
                        age,
                    },
                ))
                .id();
            let particle = Particle {
                entity,
                age,
                lifetime,
                origin,
                velocity,
                gravity: gravity * scale.z,
            };
            place(&particle, &mut transform);
            commands.entity(entity).insert(transform);
            state.particles.push(particle);
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
