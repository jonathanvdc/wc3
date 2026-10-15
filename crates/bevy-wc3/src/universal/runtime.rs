//! Per-instance playback and pose evaluation over immutable semantic definitions.
use super::particles::{ParticleFrame, ParticleInstances, UniversalParticleEmitter};
use super::{
    features::{AttachmentQuery, LightQuery, RuntimeFeatureInstances},
    material::SurfaceAnimation,
    PreparedUniversalModel, RuntimeNode, SampledUniversalCamera, UniversalMaterial,
};
use crate::effects::particle_emitter2::ParticleInstances as SpriteInstances;
use crate::light::Wc3LightSettings;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::profiles::Universal;
use wc3::model::{
    AnimationSample, Billboard, ClipId, ControllerId, DrawId, InstanceId, MaterialId, MeshId,
    NodeId, Time as ModelTime, Transform as ModelTransform,
};
use wc3::model::{Error as AnimationError, FeatureId};

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
    /// Authored draw within that placement.
    pub draw: DrawId,
    /// Shared geometry identity.
    pub mesh: MeshId,
    /// Selected material identity.
    pub material: MaterialId,
    /// Pass ordinal in the selected material.
    pub pass: usize,
    /// Authored geometry detail level; absent means shared across levels.
    pub lod: Option<u32>,
}

/// Playback clocks and application-selected controller programmes, in seconds.
#[derive(Component, Clone, Debug)]
pub struct UniversalPlayback {
    /// Selected animation, or authored rest pose with global programmes.
    pub clip: Option<ClipId>,
    /// Elapsed local playback seconds, wrapped or clamped by clip policy.
    pub clip_time: f64,
    /// Absolute authoring timeline seconds.
    pub timeline_time: f64,
    /// Elapsed seconds for independent repeating clocks.
    pub shared_time: f64,
    /// Playback rate; negative values run backwards.
    pub speed: f64,
    /// Whether automatic clock advancement is paused.
    pub paused: bool,
    /// Explicit controller selection, in tie-breaking order.
    pub controllers: Vec<ControllerId>,
    /// Camera supplying billboard and camera-anchor poses.
    pub driving_camera: Option<Entity>,
    /// Selected authored LOD. Common geometry is always included.
    pub lod: u32,
}
impl UniversalPlayback {
    fn selection(
        &self,
        model: &PreparedUniversalModel,
    ) -> Result<AnimationSample<'_>, AnimationError> {
        let timing = self.clip.and_then(|id| model.timing.get(&id));
        let (timeline, interval) = timing.map_or((self.timeline_time, None), |timing| {
            let t = if timing.duration <= 0. {
                0.
            } else if timing.looping {
                self.clip_time.rem_euclid(timing.duration)
            } else {
                self.clip_time.clamp(0., timing.duration)
            };
            (
                timing.origin + t,
                Some((timing.origin, timing.origin + timing.duration)),
            )
        });
        Ok(AnimationSample {
            clip: self.clip,
            clip_time: ModelTime::seconds(self.clip_time)?,
            timeline_time: ModelTime::seconds(timeline)?,
            shared_time: ModelTime::seconds(self.shared_time)?,
            timeline_interval: interval
                .map(|(a, b)| Ok((ModelTime::seconds(a)?, ModelTime::seconds(b)?)))
                .transpose()?,
            controllers: &self.controllers,
        })
    }
}

impl Default for UniversalPlayback {
    fn default() -> Self {
        Self {
            clip: None,
            clip_time: 0.,
            timeline_time: 0.,
            shared_time: 0.,
            speed: 1.,
            paused: false,
            controllers: Vec::new(),
            driving_camera: None,
            lod: 0,
        }
    }
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
    model: PreparedUniversalModel,
    nodes: Vec<Entity>,
    poses: Vec<Mat4>,
    visible: Vec<bool>,
    basis_entity: Entity,
    draws: Vec<(Entity, Option<u32>, Handle<UniversalMaterial>)>,
    materials: Vec<(Handle<UniversalMaterial>, Arc<SurfaceAnimation>)>,
    features: RuntimeFeatureInstances,
    particles: ParticleInstances,
    pose_initialized: bool,
}
impl UniversalInstance {
    pub(super) fn despawn_owned_subtree(&self, commands: &mut Commands) {
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

fn hierarchical(node: &RuntimeNode) -> bool {
    node.inheritance.translation
        && node.inheritance.rotation
        && node.inheritance.scale
        && matches!(
            node.billboard,
            Billboard::None | Billboard::Locked { enabled: false, .. }
        )
        && !node.camera_anchored
}

fn local_transform(rest: ModelTransform) -> Transform {
    let rotation = Quat::from_array(rest.rotation.0).normalize();
    let pivot = Vec3::from_array(rest.pivot);
    let scale = Vec3::from_array(rest.scale);
    let translation = Vec3::from_array(rest.translation) + pivot - rotation * (scale * pivot);
    Transform {
        translation,
        rotation,
        scale,
    }
}

fn local_matrix(rest: ModelTransform) -> Mat4 {
    local_transform(rest).to_matrix()
}

fn inherited(parent: Mat4, node: &RuntimeNode) -> Mat4 {
    if node.inheritance.translation && node.inheritance.rotation && node.inheritance.scale {
        return parent;
    }
    let (scale, rotation, translation) = parent.to_scale_rotation_translation();
    Mat4::from_scale_rotation_translation(
        if node.inheritance.scale {
            scale
        } else {
            Vec3::ONE
        },
        if node.inheritance.rotation {
            rotation
        } else {
            Quat::IDENTITY
        },
        if node.inheritance.translation {
            translation
        } else {
            Vec3::ZERO
        },
    )
}

/// Spawn a directly compiled Universal model beneath an application-owned root.
///
/// Register [`crate::Wc3BevyPlugin`] to advance playback. The root owns its rig and
/// draws through Bevy hierarchy lifetime; meshes and inverse binds are shared.
pub fn spawn_universal_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<UniversalMaterial>,
    model: &PreparedUniversalModel,
    transform: Transform,
) -> Entity {
    let root = commands
        .spawn((
            Name::new("Universal model"),
            transform,
            Visibility::default(),
        ))
        .id();
    populate_universal_model(commands, meshes, materials, model, root);
    root
}

pub(super) fn populate_universal_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<UniversalMaterial>,
    model: &PreparedUniversalModel,
    root: Entity,
) {
    let basis_entity = commands
        .spawn((
            Name::new("Model coordinate frame"),
            model.basis,
            Visibility::default(),
            ChildOf(root),
        ))
        .id();
    let particles = model
        .particles
        .spawn(commands, meshes, materials, basis_entity);
    let mut nodes = Vec::with_capacity(model.nodes.len());
    let mut poses = vec![Mat4::IDENTITY; model.nodes.len()];
    let mut visible = vec![true; model.nodes.len()];
    for &index in model.order.iter() {
        let node = &model.nodes[index];
        poses[index] = node
            .parent
            .map_or(Mat4::IDENTITY, |parent| inherited(poses[parent], node))
            * local_matrix(node.rest);
        visible[index] = node.visible && node.parent.is_none_or(|parent| visible[parent]);
    }
    for (index, node) in model.nodes.iter().enumerate() {
        nodes.push(
            commands
                .spawn((
                    Name::new(node.name.clone()),
                    UniversalNode { id: node.id },
                    if hierarchical(node) {
                        local_transform(node.rest)
                    } else {
                        Transform::from_matrix(poses[index])
                    },
                    if visible[index] {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                    ChildOf(basis_entity),
                ))
                .id(),
        );
    }
    for (index, node) in model.nodes.iter().enumerate() {
        if hierarchical(node) {
            if let Some(parent) = node.parent {
                commands.entity(nodes[index]).insert(ChildOf(nodes[parent]));
            }
        }
    }
    let feature_nodes = model
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id, nodes[i]))
        .collect();
    let feature_instances = model.features.spawn(commands, &feature_nodes);
    let lod = model
        .draws
        .iter()
        .filter_map(|draw| draw.lod)
        .min()
        .unwrap_or(0);
    let mut material_cache = HashMap::new();
    let mut animated_materials = Vec::new();
    let mut draw_entities = Vec::new();
    for draw in model.draws.iter() {
        let material = material_cache
            .entry((draw.material_id, draw.pass, draw.mesh_state))
            .or_insert_with(|| {
                if draw.animation.is_animated() {
                    let handle = materials.add(draw.material.clone());
                    animated_materials.push((handle.clone(), draw.animation.clone()));
                    handle
                } else {
                    model
                        .material_cache
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .entry((draw.material_id, draw.pass, draw.mesh_state))
                        .or_insert_with(|| materials.add(draw.material.clone()))
                        .clone()
                }
            })
            .clone();
        let mut entity = commands.spawn((
            UniversalDraw {
                instance: draw.instance,
                draw: draw.draw,
                mesh: draw.mesh_id,
                material: draw.material_id,
                pass: draw.pass,
                lod: draw.lod,
            },
            Mesh3d(draw.mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            if draw.lod.is_none_or(|level| level == lod) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
            ChildOf(nodes[draw.node]),
        ));
        if let Some(skin) = draw.skin {
            let skin = &model.skins[skin];
            entity.insert((
                SkinnedMesh {
                    inverse_bindposes: skin.binds.clone(),
                    joints: skin.joints.iter().map(|&i| nodes[i]).collect(),
                },
                NoFrustumCulling,
            ));
        }
        draw_entities.push((entity.id(), draw.lod, material));
    }
    commands.entity(root).insert((
        UniversalInstance {
            model: model.clone(),
            nodes,
            poses,
            visible,
            basis_entity,
            draws: draw_entities,
            materials: animated_materials,
            features: feature_instances,
            particles,
            pose_initialized: false,
        },
        UniversalPlayback {
            lod,
            ..Default::default()
        },
        UniversalAnimationError::default(),
    ));
    commands
        .entity(root)
        .insert_if_new(UniversalPoseOverrides::default());
}

pub(crate) fn advance_universal_animation(
    time: Res<Time>,
    mut query: Query<&mut UniversalPlayback>,
) {
    for mut playback in &mut query {
        if !playback.paused {
            let delta = time.delta_secs_f64() * playback.speed;
            if delta == 0. {
                continue;
            }
            playback.clip_time += delta;
            playback.timeline_time += delta;
            playback.shared_time += delta;
        }
    }
}

type AnimatedInstances<'world, 'state> = Query<
    'world,
    'state,
    (
        &'static mut UniversalInstance,
        Ref<'static, UniversalPlayback>,
        Ref<'static, UniversalPoseOverrides>,
        &'static mut UniversalAnimationError,
        Option<&'static Wc3LightSettings>,
    ),
>;

pub(crate) fn animate_universal_instances(
    mut instances: AnimatedInstances<'_, '_>,
    globals: Query<Ref<GlobalTransform>>,
    mut transforms: Query<(&mut Transform, &mut Visibility), With<UniversalNode>>,
    mut draws: Query<&mut Visibility, (With<UniversalDraw>, Without<UniversalNode>)>,
    mut materials: ResMut<Assets<UniversalMaterial>>,
    mut lights: LightQuery<'_, '_>,
    mut attachments: AttachmentQuery<'_, '_>,
) {
    for (mut instance, playback, overrides, mut error, settings) in &mut instances {
        let settings = settings.copied().unwrap_or_default();
        let camera_changed = instance.model.camera_sensitive
            && (playback.is_changed()
                || globals
                    .get(instance.basis_entity)
                    .is_ok_and(|value| value.is_changed())
                || playback
                    .driving_camera
                    .and_then(|id| globals.get(id).ok())
                    .is_some_and(|value| value.is_changed()));
        let pose_dirty = !instance.pose_initialized
            || overrides.is_changed()
            || (instance.model.pose_animated && playback.is_changed())
            || camera_changed;
        let selection = playback.selection(&instance.model);
        let animation = instance.model.animation.clone();
        let sample = match selection.and_then(|selection| animation.sample(selection)) {
            Ok(sample) => {
                if error.0.is_some() {
                    error.0 = None;
                }
                sample
            }
            Err(failure) => {
                let message = failure.to_string();
                if error.0.as_ref() != Some(&message) {
                    error.0 = Some(message);
                }
                continue;
            }
        };
        let camera = playback
            .driving_camera
            .and_then(|entity| globals.get(entity).ok())
            .and_then(|camera| {
                let root = globals.get(instance.basis_entity).ok()?;
                if root.affine().matrix3.determinant().abs() <= 1e-12 {
                    return None;
                }
                Some(root.affine().inverse() * camera.affine())
            });
        let UniversalInstance {
            model,
            poses,
            visible,
            nodes,
            draws: draw_entities,
            materials: material_bindings,
            features: feature_instances,
            pose_initialized,
            ..
        } = &mut *instance;
        if pose_dirty {
            for &index in model.order.iter() {
                let node = &model.nodes[index];
                let pose = overrides
                    .0
                    .get(&node.id)
                    .copied()
                    .unwrap_or_else(|| ModelTransform {
                        translation: sample
                            .property(&node.id.translation::<Universal>())
                            .unwrap_or(node.rest.translation),
                        rotation: sample
                            .property(&node.id.rotation::<Universal>())
                            .unwrap_or(node.rest.rotation),
                        scale: sample
                            .property(&node.id.scale::<Universal>())
                            .unwrap_or(node.rest.scale),
                        pivot: node.rest.pivot,
                    });
                let local = local_matrix(pose);
                let mut matrix = node
                    .parent
                    .map_or(Mat4::IDENTITY, |parent| inherited(poses[parent], node))
                    * local;
                if let Some(camera) = camera.filter(|_| !hierarchical(node)) {
                    let (scale, rotation, _) = matrix.to_scale_rotation_translation();
                    let pivot = Vec3::from_array(pose.pivot);
                    let mut anchor = matrix.transform_point3(pivot);
                    if node.camera_anchored {
                        anchor += Vec3::from(camera.translation);
                    }
                    let rotation = match node.billboard {
                        Billboard::Spherical => Quat::from_mat3(&camera.matrix3.into()),
                        Billboard::Locked {
                            enabled: true,
                            x,
                            y,
                            z,
                        } => {
                            let target = Quat::from_mat3(&camera.matrix3.into());
                            let (tx, ty, tz) = target.to_euler(EulerRot::XYZ);
                            let (rx, ry, rz) = rotation.to_euler(EulerRot::XYZ);
                            Quat::from_euler(
                                EulerRot::XYZ,
                                if x { rx } else { tx },
                                if y { ry } else { ty },
                                if z { rz } else { tz },
                            )
                        }
                        _ => rotation,
                    };
                    matrix = Mat4::from_scale_rotation_translation(
                        scale,
                        rotation,
                        anchor - rotation * (scale * pivot),
                    );
                }
                poses[index] = matrix;
                let node_visible = sample
                    .property(&node.id.visibility::<Universal>())
                    .unwrap_or(node.visible)
                    && node.parent.is_none_or(|parent| visible[parent]);
                visible[index] = node_visible;
                if let Ok((mut transform, mut visibility)) = transforms.get_mut(nodes[index]) {
                    let value =
                        Transform::from_matrix(if hierarchical(node) { local } else { matrix });
                    if *transform != value {
                        *transform = value;
                    }
                    let value = if node_visible {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                    if *visibility != value {
                        *visibility = value;
                    }
                }
            }
            *pose_initialized = true;
        }
        for (handle, binding) in material_bindings.iter() {
            binding.apply_to_assets(&sample, handle, &mut materials);
        }
        for (entity, lod, material) in draw_entities.iter() {
            if let Ok(mut visibility) = draws.get_mut(*entity) {
                let has_opacity = materials
                    .get(material)
                    .is_some_and(|material| material.base.base_color.alpha() > 0.);
                let value = if has_opacity && lod.is_none_or(|level| level == playback.lod) {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if *visibility != value {
                    *visibility = value;
                }
            }
        }
        model.features.animate(
            &sample,
            feature_instances,
            &settings,
            &mut lights,
            &mut attachments,
        );
    }
}

pub(crate) fn simulate_universal_particles(
    mut instances: Query<(&mut UniversalInstance, &UniversalPlayback)>,
    globals: Query<&GlobalTransform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut stats: Query<&mut UniversalParticleEmitter>,
    mut sprites: Query<&mut SpriteInstances>,
) {
    for (mut instance, playback) in &mut instances {
        if instance.model.particles.is_empty() {
            continue;
        }
        let animation = instance.model.animation.clone();
        let Ok(selection) = playback.selection(&instance.model) else {
            continue;
        };
        let Ok(sample) = animation.sample(selection) else {
            continue;
        };
        let Ok(basis) = globals.get(instance.basis_entity) else {
            continue;
        };
        let camera = playback
            .driving_camera
            .and_then(|camera| globals.get(camera).ok())
            .map(GlobalTransform::to_matrix);
        let UniversalInstance {
            model,
            poses,
            visible,
            particles,
            ..
        } = &mut *instance;
        model.particles.update(
            particles,
            ParticleFrame {
                time: playback.shared_time,
                selection,
                sample: &sample,
                node_poses: poses,
                node_visible: visible,
                basis_world: basis.to_matrix(),
                camera_world: camera,
            },
            &mut meshes,
            &mut stats,
            &mut sprites,
        );
    }
}
