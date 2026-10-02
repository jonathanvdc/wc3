//! Opt-in playback of model-authored views on application-owned Bevy cameras.
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use wc3::model::scene::Camera as ModelCamera;
use wc3::model::V1800;

use crate::animation::pose::AnimatedNode;
use crate::animation::{sample, Wc3Animation};

/// Authored cameras on a spawned model's animation root, in source order.
/// This component does not create or activate Bevy cameras.
#[derive(Component)]
pub struct Wc3ModelCameras(pub(crate) Vec<ModelCamera<V1800>>);

impl Wc3ModelCameras {
    /// Camera names, base views, and all authored tracks, including unmapped lens tracks.
    pub fn definitions(&self) -> &[ModelCamera<V1800>] {
        &self.0
    }

    /// Samples a camera in model space using the supplied instance clock.
    pub fn sample(&self, index: usize, animation: &Wc3Animation) -> Option<Wc3CameraSample> {
        let camera = self.0.get(index)?;
        let translation = camera
            .translation
            .as_ref()
            .and_then(|track| sample(track, animation));
        let target_translation = camera
            .target_translation
            .as_ref()
            .and_then(|track| sample(track, animation));
        Some(Wc3CameraSample {
            position: Vec3::from_array(camera.position)
                + translation.map(Vec3::from_array).unwrap_or_default(),
            target: Vec3::from_array(camera.target_position)
                + target_translation.map(Vec3::from_array).unwrap_or_default(),
            roll: camera
                .rotation
                .as_ref()
                .and_then(|track| sample(track, animation))
                .unwrap_or(0.0),
            field_of_view: camera.field_of_view,
            near_clip: camera.near_clip,
            far_clip: camera.far_clip,
        })
    }
}

/// An evaluated authored view. Coordinates are model-local; angles are radians.
#[derive(Clone, Copy, Debug)]
pub struct Wc3CameraSample {
    pub position: Vec3,
    pub target: Vec3,
    /// Right-handed rotation about the eye-to-target direction.
    pub roll: f32,
    pub field_of_view: f32,
    pub near_clip: f32,
    pub far_clip: f32,
}

impl Wc3CameraSample {
    /// Builds a standard perspective lens with a configurable FOV multiplier.
    /// Returns `None` for invalid FOV or clip distances. Bevy owns viewport aspect
    /// updates and uses its normal infinite reverse-depth projection.
    pub fn perspective_projection(&self, fov_multiplier: f32) -> Option<PerspectiveProjection> {
        let fov = self.field_of_view * fov_multiplier;
        if !fov_multiplier.is_finite()
            || fov_multiplier <= 0.0
            || !fov.is_finite()
            || fov <= 0.0
            || fov >= PI
            || !self.near_clip.is_finite()
            || self.near_clip <= 0.0
            || !self.far_clip.is_finite()
            || self.far_clip <= self.near_clip
        {
            return None;
        }
        Some(PerspectiveProjection {
            fov,
            near: self.near_clip,
            far: self.far_clip,
            near_clip_plane: Vec4::new(0.0, 0.0, -1.0, -self.near_clip),
            ..default()
        })
    }

    /// Transforms eye, target, and Z up through the model root, then applies roll.
    /// Returns `None` for nonfinite or degenerate views. At a vertical view,
    /// transformed Y (then X) supplies a deterministic fallback up direction.
    /// The resulting camera has unit scale; clip distances are not scaled.
    pub fn world_transform(&self, root: &GlobalTransform) -> Option<Transform> {
        let affine = root.affine();
        let position = affine.transform_point3(self.position);
        let target = affine.transform_point3(self.target);
        if !position.is_finite() || !target.is_finite() || !self.roll.is_finite() {
            return None;
        }
        let forward = (target - position).try_normalize()?;
        let up = [Vec3::Z, Vec3::Y, Vec3::X].into_iter().find_map(|axis| {
            let up = affine.transform_vector3(axis);
            let projected = up - forward * up.dot(forward);
            (projected.length_squared() > 1e-10)
                .then(|| projected.try_normalize())
                .flatten()
        })?;
        let up = Quat::from_axis_angle(forward, self.roll) * up;
        Some(Transform::from_translation(position).looking_to(forward, up))
    }
}

/// Drives an existing perspective `Camera3d` from an authored model camera.
///
/// The plugin owns its transform, FOV, near/far distances, and standard near clip
/// plane. Activation, target, viewport, aspect ratio, and render settings remain
/// application-owned. Removing the binding stops playback. Invalid bindings keep
/// the last view and warn. The plugin never despawns the camera; application
/// hierarchy/ownership relationships still control its lifetime.
///
/// Both hierarchies must contain ordinary transforms, without animated WC3 nodes
/// or other bound cameras. A camera parent must allow a shear-free local camera
/// transform (rigid or uniform-scale parents are supported). Use an unparented
/// camera when in doubt. Visibility and modern lens tracks do not control views.
#[derive(Component, Clone, Copy, Debug)]
pub struct Wc3CameraBinding {
    pub model: Entity,
    pub camera_index: usize,
    /// Multiplies authored vertical FOV. Must produce a finite angle in (0, pi).
    pub fov_multiplier: f32,
}

impl Wc3CameraBinding {
    /// Plays the authored FOV directly.
    pub fn new(model: Entity, camera_index: usize) -> Self {
        Self {
            model,
            camera_index,
            fov_multiplier: 1.0,
        }
    }

    /// Uses a 0.75 FOV multiplier for portrait framing. Game fidelity is unverified.
    pub fn portrait(model: Entity, camera_index: usize) -> Self {
        Self {
            fov_multiplier: 0.75,
            ..Self::new(model, camera_index)
        }
    }
}

type CameraHierarchy<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        Option<&'static ChildOf>,
        Has<AnimatedNode>,
        Has<Wc3CameraBinding>,
    ),
>;

type CameraOutputs<'w, 's> =
    Query<'w, 's, (&'static mut Transform, &'static mut Projection), With<Camera3d>>;

fn current_world(
    entity: Entity,
    hierarchy: &CameraHierarchy,
) -> Result<GlobalTransform, &'static str> {
    let mut chain = Vec::new();
    let mut seen = HashSet::new();
    let mut next = Some(entity);
    while let Some(entity) = next {
        if !seen.insert(entity) {
            return Err("cyclic transform hierarchy");
        }
        let (transform, parent, animated, bound) = hierarchy
            .get(entity)
            .map_err(|_| "missing hierarchy transform")?;
        if animated || bound {
            return Err("camera hierarchy depends on an animated WC3 node or bound camera");
        }
        chain.push(*transform);
        next = parent.map(ChildOf::parent);
    }
    let mut world = GlobalTransform::IDENTITY;
    for transform in chain.into_iter().rev() {
        world = world.mul_transform(transform);
    }
    Ok(world)
}

fn local_view(view: Transform, parent: GlobalTransform) -> Result<Transform, &'static str> {
    let affine = parent.affine();
    if !affine.is_finite() || affine.matrix3.determinant().abs() < 1e-10 {
        return Err("singular or nonfinite camera parent transform");
    }
    let local_matrix = Mat4::from(affine.inverse()) * view.to_matrix();
    let local = Transform::from_matrix(local_matrix);
    if !local.to_matrix().is_finite() || !local.to_matrix().abs_diff_eq(local_matrix, 1e-4) {
        return Err("camera parent requires a sheared camera transform");
    }
    Ok(local)
}

pub(crate) fn animate_cameras(
    roots: Query<(&Wc3ModelCameras, &Wc3Animation)>,
    cameras: Query<(Entity, &Wc3CameraBinding, Option<&ChildOf>), With<Camera3d>>,
    mut transforms: ParamSet<(CameraHierarchy, CameraOutputs)>,
    mut warnings: Local<HashMap<Entity, &'static str>>,
) {
    let mut updates = Vec::new();
    let mut present = HashSet::new();
    for (entity, binding, parent) in &cameras {
        present.insert(entity);
        let result = (|| {
            {
                let outputs = transforms.p1();
                let (_, projection) = outputs
                    .get(entity)
                    .map_err(|_| "camera transform or projection is missing")?;
                if !matches!(projection, Projection::Perspective(_)) {
                    return Err("binding requires a perspective projection");
                }
            }
            let (definitions, animation) = roots
                .get(binding.model)
                .map_err(|_| "model cameras or animation are not available")?;
            let sample = definitions
                .sample(binding.camera_index, animation)
                .ok_or("camera index is out of range")?;
            let lens = sample
                .perspective_projection(binding.fov_multiplier)
                .ok_or("invalid authored lens or FOV multiplier")?;
            let hierarchy = transforms.p0();
            let root = current_world(binding.model, &hierarchy)?;
            let world = sample
                .world_transform(&root)
                .ok_or("nonfinite or degenerate camera view")?;
            let local = match parent {
                Some(parent) => local_view(world, current_world(parent.parent(), &hierarchy)?)?,
                None => world,
            };
            Ok((local, lens))
        })();
        match result {
            Ok(update) => {
                warnings.remove(&entity);
                updates.push((entity, update));
            }
            Err(reason) => {
                if warnings.insert(entity, reason) != Some(reason) {
                    warn!("WC3 camera {entity:?} cannot play model {:?} camera {}: {reason}; retaining the previous view", binding.model, binding.camera_index);
                }
            }
        }
    }
    warnings.retain(|entity, _| present.contains(entity));
    let mut outputs = transforms.p1();
    for (entity, (local, lens)) in updates {
        let Ok((mut transform, mut projection)) = outputs.get_mut(entity) else {
            continue;
        };
        let Projection::Perspective(perspective) = &mut *projection else {
            continue;
        };
        *transform = local;
        perspective.fov = lens.fov;
        perspective.near = lens.near;
        perspective.far = lens.far;
        perspective.near_clip_plane = lens.near_clip_plane;
    }
}

#[cfg(test)]
#[path = "camera_tests.rs"]
mod tests;
