//! Per-instance simulation and batched drawing for PRE2 emitters.
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;
use wc3::model::animation::Animatable;
use wc3::model::emitters::{Particle2FilterMode, Particle2Frames, ParticleEmitter2};
use wc3::model::materials::LayerFilterMode;

use crate::animation::{sample, sample_value, Wc3Animation};
use crate::material::{Wc3LayerMaterial, Wc3LayerState};

const MAX_PARTICLES: usize = 8192;

pub(crate) fn material(
    emitter: &ParticleEmitter2,
    texture: Option<Handle<Image>>,
) -> Wc3LayerMaterial {
    let filter = match emitter.filter_mode {
        Particle2FilterMode::Blend => LayerFilterMode::Blend,
        Particle2FilterMode::Additive => LayerFilterMode::Additive,
        Particle2FilterMode::Modulate => LayerFilterMode::Modulate,
        Particle2FilterMode::Modulate2x => LayerFilterMode::Modulate2x,
        Particle2FilterMode::AlphaKey => LayerFilterMode::Transparent,
        Particle2FilterMode::Unknown(_) => LayerFilterMode::Blend,
    };
    Wc3LayerMaterial {
        base: StandardMaterial {
            base_color_texture: texture,
            alpha_mode: if matches!(emitter.filter_mode, Particle2FilterMode::AlphaKey) {
                AlphaMode::Mask(0.5)
            } else {
                AlphaMode::Blend
            },
            unlit: emitter.node.flags.unshaded(),
            fog_enabled: !emitter.node.flags.unfogged(),
            depth_bias: emitter.priority_plane as f32,
            double_sided: true,
            cull_mode: None,
            ..default()
        },
        extension: Wc3LayerState {
            filter,
            no_depth_test: false,
            no_depth_set: true,
        },
    }
}

pub(crate) fn empty_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    // Bevy 0.19's mesh allocator does not allocate zero-byte meshes, but its
    // upload path still attempts to copy their vertex and index data.
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0; 3]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0; 3]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0; 2]; 3]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0; 4]; 3]);
    mesh.insert_indices(Indices::U32(vec![0, 1, 2]));
    mesh
}

#[derive(Clone)]
struct Particle {
    position: Vec3,
    velocity: Vec3,
    gravity: f32,
    age: f32,
    lifetime: f32,
    tail: bool,
    size_scale: f32,
}

#[derive(Component)]
pub(crate) struct Particle2State {
    pub(crate) root: Entity,
    pub(crate) node: Entity,
    pub(crate) definition: ParticleEmitter2,
    pub(crate) mesh: Handle<Mesh>,
    particles: Vec<Particle>,
    emission_remainder: f32,
    last_sequence: usize,
    last_elapsed_ms: f64,
    last_cycle: i64,
    last_squirt_key: Option<i32>,
    rng: u64,
}

impl Particle2State {
    pub(crate) fn new(
        root: Entity,
        node: Entity,
        definition: ParticleEmitter2,
        mesh: Handle<Mesh>,
    ) -> Self {
        Self {
            root,
            node,
            rng: u64::from(definition.node.object_id).wrapping_add(1),
            definition,
            mesh,
            particles: Vec::new(),
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
        let make = |tail| Particle {
            position,
            velocity,
            gravity,
            age: 0.0,
            lifetime,
            tail,
            size_scale,
        };
        match self.definition.frames {
            Particle2Frames::Head => self.particles.push(make(false)),
            Particle2Frames::Tail => self.particles.push(make(true)),
            Particle2Frames::Both => {
                self.particles.push(make(false));
                if self.particles.len() < MAX_PARTICLES {
                    self.particles.push(make(true));
                }
            }
            Particle2Frames::Unknown(_) => {}
        }
    }

    fn advance(&mut self, animation: &Wc3Animation, transform: &GlobalTransform, dt: f32) {
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
        for particle in &mut self.particles {
            particle.age += dt;
            particle.velocity.z -= particle.gravity * dt;
            particle.position += particle.velocity * dt;
        }
        self.particles
            .retain(|particle| particle.age < particle.lifetime);
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
            self.emission_remainder += rate * dt;
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
    let time = if let Some(id) = track.global_sequence_id() {
        let length = *animation.global_sequences.get(id as usize)? as f64;
        if length > 0.0 {
            animation.elapsed_ms.rem_euclid(length)
        } else {
            0.0
        }
    } else {
        let sequence = animation.sequences.get(animation.sequence)?;
        let start = sequence.interval[0] as f64;
        let length = (sequence.interval[1] as f64 - start).max(0.0);
        start
            + if sequence.flags.non_looping() {
                animation.elapsed_ms.clamp(0.0, length)
            } else if length > 0.0 {
                animation.elapsed_ms.rem_euclid(length)
            } else {
                0.0
            }
    };
    track
        .step_keys()
        .and_then(|keys| {
            keys.iter()
                .take_while(|key| key.frame as f64 <= time)
                .last()
                .map(|key| key.frame)
        })
        .or_else(|| {
            track.linear_keys().and_then(|keys| {
                keys.iter()
                    .take_while(|key| key.frame as f64 <= time)
                    .last()
                    .map(|key| key.frame)
            })
        })
        .or_else(|| {
            track.hermite_keys().and_then(|keys| {
                keys.iter()
                    .take_while(|key| key.frame as f64 <= time)
                    .last()
                    .map(|key| key.frame)
            })
        })
        .or_else(|| {
            track.bezier_keys().and_then(|keys| {
                keys.iter()
                    .take_while(|key| key.frame as f64 <= time)
                    .last()
                    .map(|key| key.frame)
            })
        })
}

fn stage(definition: &ParticleEmitter2, life: f32) -> ([f32; 4], f32, usize, f32) {
    let middle = definition.time.clamp(0.001, 0.999);
    let (a, b, t, phase) = if life < middle {
        (0, 1, life / middle, 0)
    } else {
        (1, 2, (life - middle) / (1.0 - middle), 1)
    };
    let color_a = definition.segment_colors[a];
    let color_b = definition.segment_colors[b];
    let alpha_a = definition.alpha[a] as f32 / 255.0;
    let alpha_b = definition.alpha[b] as f32 / 255.0;
    let color = [
        color_a[0] + (color_b[0] - color_a[0]) * t,
        color_a[1] + (color_b[1] - color_a[1]) * t,
        color_a[2] + (color_b[2] - color_a[2]) * t,
        alpha_a + (alpha_b - alpha_a) * t,
    ];
    let scaling = definition.particle_scaling;
    let size = scaling[a] + (scaling[b] - scaling[a]) * t;
    (color, size, phase, t)
}

fn atlas_uv(definition: &ParticleEmitter2, tail: bool, phase: usize, t: f32) -> [[f32; 2]; 4] {
    let rows = definition.rows.max(1);
    let columns = definition.columns.max(1);
    let interval = definition.uv_animations[usize::from(tail) * 2 + phase];
    let first = interval[0];
    let count = interval[1].saturating_sub(first);
    let cell = if count > 0 {
        let offset = ((count as f32 * interval[2] as f32 * t).floor() as u32) % count;
        first.saturating_add(offset)
    } else {
        first
    }
    .min(rows.saturating_mul(columns).saturating_sub(1));
    let u = (cell % columns) as f32 / columns as f32;
    let v = (cell / columns) as f32 / rows as f32;
    let du = 1.0 / columns as f32;
    let dv = 1.0 / rows as f32;
    [[u, v + dv], [u + du, v + dv], [u + du, v], [u, v]]
}

fn rebuild_mesh(
    mesh: &mut Mesh,
    state: &Particle2State,
    transform: &GlobalTransform,
    root: &GlobalTransform,
    camera: &GlobalTransform,
) {
    let mut positions = Vec::with_capacity(state.particles.len() * 4);
    let mut normals = Vec::with_capacity(state.particles.len() * 4);
    let mut colors = Vec::with_capacity(state.particles.len() * 4);
    let mut uvs = Vec::with_capacity(state.particles.len() * 4);
    let mut indices = Vec::with_capacity(state.particles.len() * 6);
    let right = camera.right().as_vec3();
    let up = camera.up().as_vec3();
    let forward = camera.forward().as_vec3();
    let mut particles: Vec<_> = state.particles.iter().collect();
    if state.definition.node.flags.sort_prims_far_z() {
        let eye = camera.translation();
        particles.sort_by(|a, b| {
            let world_position = |particle: &Particle| {
                if state.definition.node.flags.model_space() {
                    transform.transform_point(particle.position)
                } else {
                    particle.position
                }
            };
            let a = world_position(a).distance_squared(eye);
            let b = world_position(b).distance_squared(eye);
            b.total_cmp(&a)
        });
    }
    for particle in particles {
        let life = (particle.age / particle.lifetime).clamp(0.0, 1.0);
        let (color, scale, phase, phase_t) = stage(&state.definition, life);
        if color[3] <= 0.0 || scale <= 0.0 {
            continue;
        }
        let mut center = if state.definition.node.flags.model_space() {
            transform.transform_point(particle.position)
        } else {
            particle.position
        };
        let velocity = if state.definition.node.flags.model_space() {
            transform.affine().transform_vector3(particle.velocity)
        } else {
            particle.velocity
        };
        if particle.tail {
            center -= velocity * state.definition.tail_length * 0.5;
        }
        let size = scale * particle.size_scale;
        let (side, vertical) = if particle.tail {
            let axis = velocity.normalize_or_zero();
            let side = axis.cross(forward).normalize_or_zero();
            (
                side * size * 0.5,
                velocity * state.definition.tail_length * 0.5,
            )
        } else if state.definition.node.flags.xy_quad() {
            let axis = velocity.normalize_or_zero();
            (
                Vec3::new(-axis.y, axis.x, 0.0) * size * 0.5,
                axis * size * 0.5,
            )
        } else {
            (right * size * 0.5, up * size * 0.5)
        };
        let corners = [
            center - side - vertical,
            center + side - vertical,
            center + side + vertical,
            center - side + vertical,
        ];
        let base = positions.len() as u32;
        let inverse_root = root.affine().inverse();
        positions.extend(corners.map(|corner| inverse_root.transform_point3(corner).to_array()));
        normals.extend(
            [root.affine()
                .inverse()
                .transform_vector3(-forward)
                .normalize_or_zero()
                .to_array(); 4],
        );
        colors.extend([color; 4]);
        uvs.extend(atlas_uv(&state.definition, particle.tail, phase, phase_t));
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    if positions.is_empty() {
        positions.resize(3, [0.0; 3]);
        normals.resize(3, [0.0; 3]);
        uvs.resize(3, [0.0; 2]);
        colors.resize(3, [0.0; 4]);
        indices.extend([0, 1, 2]);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
}

pub(crate) fn update_particles(
    time: Res<Time>,
    animations: Query<&Wc3Animation>,
    nodes: Query<&GlobalTransform>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    mut emitters: Query<&mut Particle2State>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let camera = cameras.iter().next();
    for mut state in &mut emitters {
        let (Ok(animation), Ok(transform), Ok(root)) = (
            animations.get(state.root),
            nodes.get(state.node),
            nodes.get(state.root),
        ) else {
            continue;
        };
        let dt = if animation.playing {
            (time.delta_secs_f64() * animation.speed).max(0.0) as f32
        } else {
            0.0
        };
        state.advance(animation, transform, dt);
        if let (Some(camera), Some(mut mesh)) = (camera, meshes.get_mut(&state.mesh)) {
            rebuild_mesh(&mut mesh, &state, transform, root, camera);
        }
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
        let mut state = Particle2State::new(root, root, emitter(), Handle::default());
        let animation = animation();
        state.advance(&animation, &GlobalTransform::default(), 0.15);
        assert_eq!(state.particles.len(), 2);
        state.advance(&animation, &GlobalTransform::default(), 0.15);
        assert_eq!(state.particles.len(), 6);
        assert_eq!(
            state
                .particles
                .iter()
                .filter(|particle| particle.tail)
                .count(),
            3
        );
        let mut mesh = empty_mesh();
        rebuild_mesh(
            &mut mesh,
            &state,
            &GlobalTransform::default(),
            &GlobalTransform::default(),
            &GlobalTransform::default(),
        );
        assert_eq!(mesh.count_vertices(), 24);
    }

    #[test]
    fn particle_mesh_never_has_zero_sized_gpu_buffers() {
        let root = Entity::from_bits(1);
        let state = Particle2State::new(root, root, emitter(), Handle::default());
        let mut mesh = empty_mesh();
        assert_eq!(mesh.count_vertices(), 3);
        assert_eq!(mesh.indices().unwrap().len(), 3);
        rebuild_mesh(
            &mut mesh,
            &state,
            &GlobalTransform::default(),
            &GlobalTransform::default(),
            &GlobalTransform::default(),
        );
        assert_eq!(mesh.count_vertices(), 3);
        assert_eq!(mesh.indices().unwrap().len(), 3);
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
        let mut state = Particle2State::new(root, root, definition, Handle::default());
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
    fn lifetime_stage_interpolates_color_alpha_and_size() {
        let definition = emitter();
        let (color, size, phase, _) = stage(&definition, definition.time * 0.5);
        assert_eq!(phase, 0);
        assert_eq!(color[0], 0.5);
        assert_eq!(color[1], 0.5);
        assert_eq!(size, 1.5);
        assert!(color[3] < 1.0);
    }

    #[test]
    fn atlas_interval_uses_exclusive_end_and_repeat_count() {
        let mut definition = emitter();
        definition.uv_animations[0] = [0, 4, 2];
        assert_eq!(atlas_uv(&definition, false, 0, 0.0)[3], [0.0, 0.0]);
        assert_eq!(atlas_uv(&definition, false, 0, 0.25)[3], [0.0, 0.5]);
        assert_eq!(atlas_uv(&definition, false, 0, 0.5)[3], [0.0, 0.0]);
    }
}
