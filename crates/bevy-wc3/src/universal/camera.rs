//! Opt-in authored views evaluated from current clocks and current application transforms.
use super::hierarchy::{current_world, HierarchyData};
use super::{SampledUniversalCamera, UniversalInstance, UniversalPlayback, UniversalPoseOverrides};
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;
use wc3::model::FeatureId;

/// Bind an application-owned perspective camera to an authored semantic view.
/// Activation, viewport and render target remain application-owned. Both root
/// and camera-parent hierarchies must use ordinary transforms; an animated rig
/// node or another bound camera in either hierarchy is rejected to avoid cycles.
#[derive(Component, Clone, Copy, Debug)]
pub struct UniversalCameraBinding {
    /// Root carrying the spawned model instance and playback.
    pub model: Entity,
    /// Authored camera feature identity.
    pub camera: FeatureId,
    /// Multiplier applied to the authored vertical field of view.
    pub fov_multiplier: f32,
}
impl UniversalCameraBinding {
    /// Bind with the authored field of view unchanged.
    pub fn new(model: Entity, camera: FeatureId) -> Self {
        Self {
            model,
            camera,
            fov_multiplier: 1.,
        }
    }
}
impl SampledUniversalCamera {
    /// Constructs a finite perspective lens; Bevy continues managing aspect ratio.
    pub fn perspective_projection(self, multiplier: f32) -> Option<PerspectiveProjection> {
        let fov = self.vertical_field_of_view * multiplier;
        let [near, far] = self.clipping;
        if !multiplier.is_finite()
            || multiplier <= 0.
            || !fov.is_finite()
            || fov <= 0.
            || fov >= PI
            || !near.is_finite()
            || near <= 0.
            || !far.is_finite()
            || far <= near
        {
            return None;
        }
        Some(PerspectiveProjection {
            fov,
            near,
            far,
            near_clip_plane: Vec4::new(0., 0., -1., -near),
            ..Default::default()
        })
    }
    /// Projects authored eye, target and rolled up into a finite world view.
    pub fn world_transform(self, root: GlobalTransform) -> Option<Transform> {
        let eye = root.transform_point(self.eye);
        let target = root.transform_point(self.target);
        let forward = (target - eye).try_normalize()?;
        if !eye.is_finite() || !target.is_finite() {
            return None;
        }
        let up = [self.up, Vec3::Y, Vec3::X, Vec3::Z]
            .into_iter()
            .find_map(|up| {
                let up = root.affine().transform_vector3(up);
                (up - forward * up.dot(forward)).try_normalize()
            })?;
        Some(Transform::from_translation(eye).looking_to(forward, up))
    }
}

type Outputs<'w, 's> =
    Query<'w, 's, (&'static mut Transform, &'static mut Projection), With<Camera3d>>;
#[derive(Default)]
pub(crate) struct CameraScratch {
    chain: Vec<(Entity, Transform)>,
    pose: Vec<usize>,
    updates: Vec<(Entity, Transform, PerspectiveProjection)>,
    present: Vec<Entity>,
    warnings: HashMap<Entity, &'static str>,
}
fn local_view(view: Transform, parent: GlobalTransform) -> Result<Transform, &'static str> {
    let affine = parent.affine();
    if !affine.is_finite() || affine.matrix3.determinant().abs() < 1e-10 {
        return Err("singular camera parent transform");
    }
    let matrix = Mat4::from(affine.inverse()) * view.to_matrix();
    let local = Transform::from_matrix(matrix);
    if !local.to_matrix().is_finite() || !local.to_matrix().abs_diff_eq(matrix, 1e-4) {
        return Err("camera parent requires a sheared local transform");
    }
    Ok(local)
}
pub(crate) fn animate_universal_cameras(
    models: Query<(
        &UniversalInstance,
        &UniversalPlayback,
        &UniversalPoseOverrides,
    )>,
    cameras: Query<(Entity, &UniversalCameraBinding, &Camera, Option<&ChildOf>), With<Camera3d>>,
    mut transforms: ParamSet<(Query<HierarchyData>, Outputs)>,
    mut scratch: Local<CameraScratch>,
) {
    scratch.updates.clear();
    scratch.present.clear();
    for (entity, binding, camera, parent) in &cameras {
        scratch.present.push(entity);
        let result = (|| {
            let aspect = {
                let outputs = transforms.p1();
                let (_, projection) = outputs
                    .get(entity)
                    .map_err(|_| "camera transform or projection missing")?;
                let Projection::Perspective(projection) = projection else {
                    return Err("binding requires a perspective camera");
                };
                camera
                    .viewport
                    .as_ref()
                    .map_or(projection.aspect_ratio, |viewport| {
                        viewport.physical_size.x as f32 / viewport.physical_size.y.max(1) as f32
                    })
            };
            let (instance, playback, overrides) = models
                .get(binding.model)
                .map_err(|_| "model instance is not ready")?;
            let view = instance
                .sample_camera_current(
                    binding.camera,
                    playback,
                    aspect,
                    overrides,
                    &mut scratch.pose,
                )
                .map_err(|_| "invalid playback selection")?
                .ok_or("camera feature is missing or invalid")?;
            let mut lens = view
                .perspective_projection(binding.fov_multiplier)
                .ok_or("invalid authored lens")?;
            lens.aspect_ratio = aspect;
            let hierarchy = transforms.p0();
            let root = current_world(
                instance.coordinate_entity(),
                &hierarchy,
                &mut scratch.chain,
                false,
            )?;
            let world = view
                .world_transform(root)
                .ok_or("degenerate authored camera view")?;
            let local = match parent {
                Some(parent) => local_view(
                    world,
                    current_world(parent.parent(), &hierarchy, &mut scratch.chain, false)?,
                )?,
                None => world,
            };
            Ok((local, lens))
        })();
        match result {
            Ok((local, lens)) => {
                scratch.warnings.remove(&entity);
                scratch.updates.push((entity, local, lens));
            }
            Err(reason) => {
                if scratch.warnings.insert(entity, reason) != Some(reason) {
                    warn!("Universal camera {entity:?} cannot play feature {:?}: {reason}; retaining previous view",binding.camera);
                }
            }
        }
    }
    let CameraScratch {
        warnings, present, ..
    } = &mut *scratch;
    warnings.retain(|entity, _| present.contains(entity));
    let mut outputs = transforms.p1();
    for (entity, transform, lens) in scratch.updates.drain(..) {
        let Ok((mut output, mut projection)) = outputs.get_mut(entity) else {
            continue;
        };
        if *output != transform {
            *output = transform;
        }
        let Projection::Perspective(current) = &mut *projection else {
            continue;
        };
        if current.fov != lens.fov
            || current.near != lens.near
            || current.far != lens.far
            || current.near_clip_plane != lens.near_clip_plane
        {
            current.fov = lens.fov;
            current.near = lens.near;
            current.far = lens.far;
            current.near_clip_plane = lens.near_clip_plane;
        }
    }
}

#[cfg(test)]
#[path = "camera_tests.rs"]
mod tests;
