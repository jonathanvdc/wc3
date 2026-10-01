//! Classic PREM model particles. Birth transforms and physical values are frozen
//! in world space; each child owns an independent sequence-zero animation.
use bevy::prelude::*;
use std::f32::consts::TAU;
use wc3::model::emitters::ParticleEmitter;

use crate::animation::{sample, sample_value, AnimatedNode, Wc3Animation};
use crate::asset::Wc3ModelAsset;
use crate::instance::{Wc3ModelInstance, Wc3ModelOwner};

const MAX_PARTICLES: usize = 1024;

#[derive(Component)]
pub(crate) struct ParticleState {
    root: Entity,
    node: Entity,
    definition: ParticleEmitter,
    model: Handle<Wc3ModelAsset>,
    particles: Vec<Particle>,
    remainder: f64,
    last_sequence: usize,
    last_elapsed_ms: f64,
    rng: u64,
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
            rng: u64::from(definition.node.object_id) + 1,
            definition,
            model,
            particles: Vec::new(),
            remainder: 0.0,
            last_sequence: usize::MAX,
            last_elapsed_ms: 0.0,
        }
    }

    fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u32 << 24) as f32
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

fn simulation_delta(time: &Time, animation: &Wc3Animation) -> f64 {
    let dt = time.delta_secs_f64() * animation.speed;
    if animation.playing && dt.is_finite() {
        dt.max(0.0)
    } else {
        0.0
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
    mut entity: Entity,
    root: Entity,
    animation: &Wc3Animation,
    nodes: &NodeTransforms,
) -> Option<GlobalTransform> {
    let mut world = GlobalTransform::IDENTITY;
    loop {
        let (transform, node, parent, _) = nodes.get(entity).ok()?;
        let local = node
            .filter(|node| node.root == root)
            .map_or(*transform, |node| node.sample_transform(animation));
        world = GlobalTransform::from(local) * world;
        let Some(parent) = parent else {
            return Some(world);
        };
        entity = parent.parent();
    }
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
        if state.last_sequence != animation.sequence || animation.elapsed_ms < state.last_elapsed_ms
        {
            for particle in state.particles.drain(..) {
                commands.entity(particle.entity).despawn();
            }
            state.remainder = 0.0;
        }
        state.last_sequence = animation.sequence;
        state.last_elapsed_ms = animation.elapsed_ms;
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
        let ended = animation
            .sequences
            .get(animation.sequence)
            .is_some_and(|sequence| {
                sequence.flags.non_looping()
                    && animation.elapsed_ms
                        >= f64::from(sequence.interval[1].saturating_sub(sequence.interval[0]))
            });
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
        let remainder = state.remainder;
        let emission = remainder + rate * dt;
        if !emission.is_finite() {
            state.remainder = 0.0;
            continue;
        }
        let tolerance = 16.0 * f64::EPSILON * emission.max(1.0);
        let count = ((emission + tolerance).floor() as usize).min(MAX_PARTICLES);
        state.remainder = (emission - (emission + tolerance).floor()).clamp(0.0, 1.0);
        for index in 0..count {
            if state.particles.len() >= MAX_PARTICLES {
                break;
            }
            let age = dt - ((index as f64 + 1.0 - remainder) / rate).clamp(0.0, dt);
            let mut birth = animation.clone();
            birth.elapsed_ms -= age * 1000.0;
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
            let yaw = state.random() * TAU;
            let pitch = (state.random() * 2.0 - 1.0) * latitude;
            let direction = Quat::from_rotation_z(yaw) * Quat::from_rotation_y(pitch) * Vec3::Z;
            let velocity = (rotation * direction) * speed * scale;
            let mut transform = Transform {
                translation: origin,
                rotation: Quat::from_rotation_z(state.random() * TAU),
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
mod tests {
    use super::*;
    use bevy::camera::visibility::VisibilityPlugin;
    use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
    use bevy::transform::TransformSystems;
    use std::time::Duration;
    use wc3::model::scene::Node;

    use crate::animation::{advance_animation, animate_layers, animate_nodes};
    use crate::instance::{spawn_loaded_instances, PreparedModelCache, Wc3OwnedModels};
    use crate::material::Wc3LayerMaterial;
    use crate::model::Wc3Model;

    fn setup() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins((TransformPlugin, VisibilityPlugin));
        app.insert_resource(Time::<()>::default());
        app.init_resource::<Assets<Wc3ModelAsset>>();
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<Wc3LayerMaterial>>();
        app.init_resource::<Assets<SkinnedMeshInverseBindposes>>();
        app.init_resource::<PreparedModelCache>();
        app.add_systems(
            Update,
            (
                spawn_loaded_instances,
                advance_animation,
                animate_particle_models,
                animate_nodes,
                animate_layers,
            )
                .chain(),
        );
        app.add_systems(
            PostUpdate,
            update_particles.before(TransformSystems::Propagate),
        );
        let source = Wc3Model::decode_mdl(include_str!(
            "../tests/fixtures/attachment_capture_child.mdl"
        ))
        .unwrap();
        let model = app
            .world_mut()
            .resource_mut::<Assets<Wc3ModelAsset>>()
            .add(Wc3ModelAsset {
                source,
                textures: default(),
                models: default(),
            });
        let root = app
            .world_mut()
            .spawn((
                Wc3Animation {
                    sequence: 0,
                    elapsed_ms: 0.0,
                    speed: 1.0,
                    playing: true,
                    sequences: Vec::new(),
                    global_sequences: Vec::new(),
                },
                Transform::default(),
                Visibility::default(),
            ))
            .id();
        let node = app
            .world_mut()
            .spawn((Transform::from_xyz(2.0, 0.0, 0.0), ChildOf(root)))
            .id();
        let mut definition =
            ParticleEmitter::new(Node::new("Emitter", 0).unwrap(), "child.mdl").unwrap();
        definition.emission_rate = 4.0.into();
        definition.life_span = 1.0.into();
        definition.initial_velocity = 4.0.into();
        definition.gravity = 2.0.into();
        let emitter = app
            .world_mut()
            .spawn((
                ParticleState::new(root, node, definition, model),
                ChildOf(root),
            ))
            .id();
        step(&mut app, 0.0);
        (app, root, emitter)
    }

    fn step(app: &mut App, seconds: f64) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f64(seconds));
        app.update();
    }

    #[test]
    fn births_are_subframe_and_children_have_independent_clocks() {
        let (mut app, _, emitter) = setup();
        step(&mut app, 0.625);
        let state = app.world().get::<ParticleState>(emitter).unwrap();
        assert_eq!(state.particles.len(), 2);
        let first = state.particles[0].entity;
        let second = state.particles[1].entity;
        assert_eq!(state.particles[0].age, 0.375);
        assert_eq!(state.particles[1].age, 0.125);
        let transform = app.world().get::<Transform>(first).unwrap();
        assert!((transform.translation.z - 1.359375).abs() < 1e-6);
        assert_eq!(transform.translation.x, 2.0);
        assert!(app.world().get::<ChildOf>(first).is_none());
        // Zero-time render warmup loads models without advancing their age.
        step(&mut app, 0.0);
        assert_eq!(
            app.world().get::<Wc3Animation>(first).unwrap().elapsed_ms,
            375.0
        );
        assert_eq!(
            app.world().get::<Wc3Animation>(second).unwrap().elapsed_ms,
            125.0
        );
        assert_eq!(
            app.world()
                .resource::<Assets<SkinnedMeshInverseBindposes>>()
                .len(),
            1
        );
    }

    #[test]
    fn world_motion_pause_speed_expiry_and_owner_cleanup() {
        let (mut app, root, emitter) = setup();
        step(&mut app, 0.25);
        let particle = app.world().get::<ParticleState>(emitter).unwrap().particles[0].entity;
        app.world_mut()
            .get_mut::<Transform>(root)
            .unwrap()
            .translation
            .x = 10.0;
        app.world_mut()
            .get_mut::<Wc3Animation>(root)
            .unwrap()
            .playing = false;
        step(&mut app, 0.5);
        assert_eq!(
            app.world().get::<Transform>(particle).unwrap().translation,
            Vec3::new(2.0, 0.0, 0.0)
        );
        assert_eq!(
            app.world()
                .get::<Wc3Animation>(particle)
                .unwrap()
                .elapsed_ms,
            0.0
        );
        {
            let mut animation = app.world_mut().get_mut::<Wc3Animation>(root).unwrap();
            animation.playing = true;
            animation.speed = 2.0;
        }
        step(&mut app, 0.25);
        assert_eq!(
            app.world().get::<Transform>(particle).unwrap().translation,
            Vec3::new(2.0, 0.0, 1.75)
        );
        assert_eq!(
            app.world()
                .get::<Wc3Animation>(particle)
                .unwrap()
                .elapsed_ms,
            500.0
        );
        step(&mut app, 0.25);
        assert!(app.world().get_entity(particle).is_err());
        let owned: Vec<_> = app
            .world()
            .get::<Wc3OwnedModels>(root)
            .unwrap()
            .iter()
            .collect();
        assert!(!owned.is_empty());
        app.world_mut().despawn(root);
        assert!(owned
            .into_iter()
            .all(|entity| app.world().get_entity(entity).is_err()));
        assert!(app.world().get_entity(emitter).is_err());
    }

    #[test]
    fn hidden_emitter_stops_births_but_existing_particles_age() {
        let (mut app, root, emitter) = setup();
        step(&mut app, 0.25);
        app.world_mut()
            .get_mut::<Visibility>(root)
            .unwrap()
            .clone_from(&Visibility::Hidden);
        step(&mut app, 0.25);
        let state = app.world().get::<ParticleState>(emitter).unwrap();
        assert_eq!(state.particles.len(), 1);
        assert_eq!(state.particles[0].age, 0.25);
    }

    #[test]
    fn backward_seek_retires_old_particles() {
        let (mut app, root, emitter) = setup();
        step(&mut app, 0.5);
        let old: Vec<_> = app
            .world()
            .get::<ParticleState>(emitter)
            .unwrap()
            .particles
            .iter()
            .map(|p| p.entity)
            .collect();
        app.world_mut()
            .get_mut::<Wc3Animation>(root)
            .unwrap()
            .elapsed_ms = 0.0;
        step(&mut app, 0.0);
        assert!(app
            .world()
            .get::<ParticleState>(emitter)
            .unwrap()
            .particles
            .is_empty());
        assert!(old
            .into_iter()
            .all(|entity| app.world().get_entity(entity).is_err()));
    }
    #[test]
    fn moving_nodes_are_sampled_at_birth() {
        let (mut app, root, emitter) = setup();
        let node = app.world().get::<ParticleState>(emitter).unwrap().node;
        let source =
            Wc3Model::decode_mdl(include_str!("../tests/fixtures/prem_capture.mdl")).unwrap();
        app.world_mut()
            .get_mut::<Wc3Animation>(root)
            .unwrap()
            .sequences = source.model.sequences();
        let definition = source.model.particle_emitters().remove(0);
        app.world_mut().entity_mut(node).insert(AnimatedNode {
            root,
            pivot: Vec3::ZERO,
            parent_pivot: Vec3::ZERO,
            translation: definition.node.translation,
            rotation: None,
            scaling: None,
        });
        step(&mut app, 0.625);
        let state = app.world().get::<ParticleState>(emitter).unwrap();
        assert_eq!(state.particles.len(), 2);
        assert_eq!(state.particles[0].origin.x, -1.5);
        assert_eq!(state.particles[1].origin.x, 0.0);
    }

    #[test]
    fn recursive_particle_models_are_blocked() {
        use crate::instance::BlockedChildModel;
        let (mut app, root, emitter) = setup();
        let handle = app
            .world()
            .get::<ParticleState>(emitter)
            .unwrap()
            .model
            .clone();
        app.world_mut()
            .entity_mut(root)
            .insert(Wc3ModelInstance::new(handle));
        step(&mut app, 0.25);
        let child = app.world().get::<ParticleState>(emitter).unwrap().particles[0].entity;
        step(&mut app, 0.0);
        assert!(app.world().get::<BlockedChildModel>(child).is_some());
        assert!(app.world().get::<Wc3Animation>(child).is_none());
    }
}
