use super::super::features::{AttachmentQuery, LightQuery, ModelLightSettings};
use super::super::hierarchy::{current_world, model_world, HierarchyData};
use super::super::pose::{
    as_transform, evaluate_pose, hierarchical, local_transform, EvaluatedPose,
};
use super::super::{ModelMaterial, WorldLiquidSurface};
use super::mounts::MountedModel;
use super::{
    ControllerSelection, ModelDraw, ModelNode, ModelRig, Playback, PlaybackError, PoseOverrides,
};
use crate::physics::PhysicsSimulation;
use bevy::ecs::query::QueryItem;
use bevy::math::Affine3A;
use bevy::prelude::*;
use std::collections::HashMap;
use wc3::model::{ClipId, Transform as ModelTransform};

#[derive(Default)]
pub(super) struct EvaluationState {
    pub(super) initialized: bool,
    pub(super) clip: Option<ClipId>,
    pub(super) controllers: ControllerSelection,
    pub(super) clocks: [f64; 3],
    pub(super) lod: u32,
    pub(super) driving_camera: Option<Entity>,
    pub(super) has_settings: bool,
    camera_frame: Option<Affine3A>,
    camera_failure: Option<&'static str>,
    #[cfg(test)]
    pub(super) samples: usize,
}
impl EvaluationState {
    pub(super) fn animation_changed(&self, playback: &Playback, animated: bool) -> bool {
        !self.initialized
            || self.clip != playback.clip
            || self.controllers != playback.controllers
            || (animated
                && self.clocks
                    != [
                        playback.clip_time,
                        playback.timeline_time,
                        playback.shared_time,
                    ])
    }
    pub(super) fn remember(
        &mut self,
        playback: &Playback,
        has_settings: bool,
        camera: Option<Affine3A>,
    ) {
        self.initialized = true;
        self.clip = playback.clip;
        self.controllers.clone_from(&playback.controllers);
        self.clocks = [
            playback.clip_time,
            playback.timeline_time,
            playback.shared_time,
        ];
        self.lod = playback.lod;
        self.driving_camera = playback.driving_camera;
        self.has_settings = has_settings;
        self.camera_frame = camera;
        #[cfg(test)]
        {
            self.samples += 1;
        }
    }
}

type AnimatedData = (
    Entity,
    &'static mut ModelRig,
    Ref<'static, Playback>,
    Ref<'static, PoseOverrides>,
    &'static mut PlaybackError,
    Option<Ref<'static, ModelLightSettings>>,
    Option<Ref<'static, PhysicsSimulation>>,
    Option<&'static MountedModel>,
);
type AnimatedInstances<'world, 'state> = Query<'world, 'state, AnimatedData>;

#[derive(Default)]
pub(crate) struct InstanceOrder {
    entities: Vec<Entity>,
    parents: HashMap<Entity, Entity>,
    bases: HashMap<Entity, Entity>,
}

type DrawVisibility<'w, 's> = Query<
    'w,
    's,
    &'static mut Visibility,
    (
        Or<(With<ModelDraw>, With<WorldLiquidSurface>)>,
        Without<ModelNode>,
    ),
>;

fn update_draw_visibility(
    draws: &[(Entity, Option<u32>, Handle<ModelMaterial>)],
    lod: u32,
    materials: &Assets<ModelMaterial>,
    visibility: &mut DrawVisibility<'_, '_>,
) {
    for (entity, level, material) in draws {
        if let Ok(mut current) = visibility.get_mut(*entity) {
            let has_opacity = materials
                .get(material)
                .is_some_and(|material| material.base.base_color.alpha() > 0.);
            let next = if has_opacity && level.is_none_or(|level| level == lod) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if *current != next {
                *current = next;
            }
        }
    }
}

pub(crate) fn apply_lod_visibility(
    mut instances: Query<(&mut ModelRig, &Playback)>,
    materials: Res<Assets<ModelMaterial>>,
    mut draws: DrawVisibility<'_, '_>,
) {
    for (mut instance, playback) in &mut instances {
        if instance.evaluation.lod != playback.lod {
            update_draw_visibility(&instance.draws, playback.lod, &materials, &mut draws);
            instance.evaluation.lod = playback.lod;
        }
    }
}

type NodeTransformData = (&'static mut Transform, &'static mut Visibility);
type NodeOutputs<'w, 's> = Query<'w, 's, NodeTransformData, With<ModelNode>>;
type MountEvaluation<'w, 's> = (
    Local<'s, Vec<(Entity, Transform)>>,
    Local<'s, InstanceOrder>,
    Query<'w, 's, (), With<MountedModel>>,
);

pub(crate) fn animate_instances(
    mut instances: AnimatedInstances<'_, '_>,
    mut transforms: ParamSet<(Query<HierarchyData>, NodeOutputs)>,
    mut draws: DrawVisibility<'_, '_>,
    mut materials: ResMut<Assets<ModelMaterial>>,
    mut lights: LightQuery<'_, '_>,
    mut attachments: AttachmentQuery<'_, '_>,
    (mut hierarchy_scratch, mut ordering, mounted): MountEvaluation<'_, '_>,
) {
    let material_changed = materials.is_changed();
    let has_mounts = !mounted.is_empty();
    ordering.entities.clear();
    ordering.parents.clear();
    ordering.bases.clear();
    if has_mounts {
        for (entity, instance, _, _, _, _, _, mount) in instances.iter() {
            ordering.entities.push(entity);
            ordering.bases.insert(entity, instance.basis_entity);
            if let Some(mount) = mount {
                ordering.parents.insert(entity, mount.parent);
            }
        }
        let InstanceOrder {
            entities, parents, ..
        } = &mut *ordering;
        entities.sort_unstable_by_key(|entity| {
            let mut cursor = *entity;
            let mut depth = 0;
            while let Some(parent) = parents.get(&cursor) {
                depth += 1;
                if depth > parents.len() {
                    break;
                }
                cursor = *parent;
            }
            depth
        });
    }
    let mut apply = |(
        _,
        mut instance,
        playback,
        overrides,
        mut error,
        settings,
        physics_settings,
        mount,
    ): QueryItem<'_, '_, AnimatedData>| {
        let physics_active = physics_settings.is_some() && !instance.model.physics.is_empty();
        let physics_dirty = physics_active || instance.physics.is_initialized();
        if !physics_active {
            instance.physics.clear();
        }
        let has_settings = settings.is_some();
        let settings_changed = settings.as_ref().is_some_and(|value| value.is_changed())
            || has_settings != instance.evaluation.has_settings;
        let settings = settings.as_deref().copied().unwrap_or_default();
        let animation_changed = instance
            .evaluation
            .animation_changed(&playback, instance.model.has_animation);
        let camera_result = if instance.model.camera_sensitive {
            playback.driving_camera.map_or(Ok(None), |driver| {
                let hierarchy = transforms.p0();
                let camera = current_world(driver, &hierarchy, &mut hierarchy_scratch, true)?;
                let root = if mount.is_some() {
                    model_world(instance.basis_entity, &hierarchy, &mut hierarchy_scratch)
                } else {
                    current_world(
                        instance.basis_entity,
                        &hierarchy,
                        &mut hierarchy_scratch,
                        false,
                    )
                }?;
                if root.affine().matrix3.determinant().abs() <= 1e-12 {
                    return Err("singular model frame for driving camera");
                }
                Ok(Some(root.affine().inverse() * camera.affine()))
            })
        } else {
            Ok(None)
        };
        let failure = camera_result.as_ref().err().copied();
        if failure != instance.evaluation.camera_failure {
            if let Some(message) = failure {
                warn!("Universal driving camera: {message}");
            }
            instance.evaluation.camera_failure = failure;
        }
        let camera = camera_result.unwrap_or(None);
        let camera_changed = instance.model.camera_sensitive
            && (instance.evaluation.camera_frame != camera
                || instance.evaluation.driving_camera != playback.driving_camera);
        let transition_dirty = instance
            .transition
            .needs_update(playback.transition_request, playback.transition_clock);
        let pose_dirty = physics_dirty
            || transition_dirty
            || !instance.pose_initialized
            || overrides.is_changed()
            || (instance.model.pose_animated && animation_changed)
            || camera_changed;
        let light_input_changed = instance.features.has_changed_light_input(&mut lights);
        let evaluation_dirty = !instance.evaluation.initialized
            || animation_changed
            || pose_dirty
            || settings_changed
            || light_input_changed;
        let draws_dirty =
            evaluation_dirty || material_changed || instance.evaluation.lod != playback.lod;
        if !evaluation_dirty {
            if draws_dirty {
                update_draw_visibility(&instance.draws, playback.lod, &materials, &mut draws);
            }
            instance.evaluation.lod = playback.lod;
            return;
        }
        let timing = instance.model.timing.clone();
        let selection = playback.selection_from_timing(&timing);
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
                return;
            }
        };
        let blend_time = playback.blend_duration(&instance.model);
        let node_count = instance.model.nodes.len();
        instance.transition.prepare(
            playback.transition_request,
            playback.transition_clock,
            blend_time,
            node_count,
        );
        instance
            .evaluation
            .remember(&playback, has_settings, camera);
        let mut physics_frame = None;
        let local_physics =
            physics_settings
                .as_deref()
                .filter(|_| physics_active)
                .map(|settings| {
                    let mut local = settings.clone();
                    let hierarchy = transforms.p0();
                    let child =
                        model_world(instance.basis_entity, &hierarchy, &mut hierarchy_scratch);
                    if let Ok(child) = child {
                        physics_frame = Some(child.affine());
                        if let Some(source) = mount
                            .filter(|mount| mount.inherited_physics.as_ref() == Some(settings))
                            .and_then(|mount| mount.inherited_from)
                            .and_then(|source| ordering.bases.get(&source))
                        {
                            if let Ok(source) =
                                model_world(*source, &hierarchy, &mut hierarchy_scratch)
                            {
                                if child.affine().matrix3.determinant().abs() > 1e-20 {
                                    let frame = child.affine().inverse() * source.affine();
                                    local.gravity = frame.transform_vector3(settings.gravity);
                                    local.acceleration =
                                        frame.transform_vector3(settings.acceleration);
                                }
                            }
                        }
                    }
                    local
                });
        let ModelRig {
            model,
            poses,
            evaluated,
            visible,
            nodes,
            draws: draw_entities,
            materials: material_bindings,
            features: feature_instances,
            pose_initialized,
            transition,
            physics,
            physics_locals,
            ..
        } = &mut *instance;
        if pose_dirty {
            if physics_active {
                physics_locals.resize(model.nodes.len(), ModelTransform::default());
            }
            let mut output = transforms.p1();
            for &index in model.order.iter() {
                let node = &model.nodes[index];
                let pose = overrides
                    .0
                    .get(&node.id)
                    .copied()
                    .unwrap_or_else(|| node.sample(&sample));
                let pose = if overrides.0.contains_key(&node.id) {
                    pose
                } else {
                    transition.blend(index, pose)
                };
                transition.record(index, pose);
                if physics_active {
                    physics_locals[index] = pose;
                }
                evaluated[index] = evaluate_pose(
                    node,
                    pose,
                    node.parent
                        .map_or_else(EvaluatedPose::default, |parent| evaluated[parent]),
                    camera,
                );
                let matrix = evaluated[index].matrix;
                poses[index] = matrix;
                let node_visible = node.sample_visibility(&sample)
                    && node.parent.is_none_or(|parent| visible[parent]);
                visible[index] = node_visible;
                if let Ok((mut transform, mut visibility)) = output.get_mut(nodes[index]) {
                    // Physics consumes the authored cache first; publish only
                    // the final pose below so paused rigs do not mark every
                    // skeleton Transform changed twice per frame.
                    if !physics_active {
                        let value = if hierarchical(node) {
                            local_transform(pose)
                        } else {
                            as_transform(evaluated[index])
                        };
                        if *transform != value {
                            *transform = value;
                        }
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
            if let Some(settings) = local_physics.as_ref().filter(|_| physics_active) {
                physics.synchronize_playback(playback.event_traversal.revision);
                physics.rebase(
                    &model.physics,
                    settings,
                    playback.shared_time,
                    physics_frame,
                );
                let simulated = physics.step(
                    &model.physics,
                    settings,
                    playback.shared_time,
                    poses,
                    &overrides.0,
                );
                // Apply in hierarchy order: a simulated parent changes the local
                // frame of its children before their body targets are resolved.
                for &index in model.order.iter() {
                    let node = &model.nodes[index];
                    let mut pose = physics_locals[index];
                    if let Some(matrix) = simulated.get(index).copied().flatten() {
                        let parent = node.parent.map_or(Mat4::IDENTITY, |parent| poses[parent]);
                        if hierarchical(node) && parent.determinant().abs() > 1e-8 {
                            let local = parent.inverse() * matrix;
                            let (scale, rotation, translation) =
                                local.to_scale_rotation_translation();
                            let pivot = Vec3::from_array(pose.pivot);
                            if scale.is_finite() && rotation.is_finite() && translation.is_finite()
                            {
                                pose.translation =
                                    (translation - pivot + rotation * (scale * pivot)).to_array();
                                pose.rotation.0 = rotation.to_array();
                                pose.scale = scale.to_array();
                            }
                        }
                    }
                    evaluated[index] = evaluate_pose(
                        node,
                        pose,
                        node.parent
                            .map_or_else(EvaluatedPose::default, |parent| evaluated[parent]),
                        camera,
                    );
                    poses[index] = evaluated[index].matrix;
                    if let Ok((mut transform, _)) = output.get_mut(nodes[index]) {
                        let value = if hierarchical(node) {
                            local_transform(pose)
                        } else {
                            as_transform(evaluated[index])
                        };
                        if *transform != value {
                            *transform = value;
                        }
                    }
                }
            }
            *pose_initialized = true;
            transition.finish();
        }
        for (handle, binding) in material_bindings.iter() {
            binding.apply_to_assets(&sample, handle, materials.bypass_change_detection());
        }
        if draws_dirty {
            update_draw_visibility(draw_entities, playback.lod, &materials, &mut draws);
        }
        model.features.animate(
            &sample,
            feature_instances,
            &settings,
            &mut lights,
            &mut attachments,
        );
    };
    if has_mounts {
        for entity in &ordering.entities {
            if let Ok(instance) = instances.get_mut(*entity) {
                apply(instance);
            }
        }
    } else {
        for instance in &mut instances {
            apply(instance);
        }
    }
}
