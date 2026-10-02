use bevy::math::Affine3A;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::f32::consts::FRAC_PI_2;
use wc3::model::scene::NodeFlags;

#[derive(Clone, Copy)]
pub(crate) struct NodeSample {
    pub(crate) root: Entity,
    pub(crate) flags: NodeFlags,
    pub(crate) pivot_offset: Vec3,
    pub(crate) camera: Option<Transform>,
}

pub(crate) struct PoseInput {
    pub(crate) local: Transform,
    pub(crate) parent: Option<Entity>,
    pub(crate) node: Option<NodeSample>,
    pub(crate) world: Option<GlobalTransform>,
    pub(crate) anchor: Option<Vec3>,
}

#[derive(Clone, Copy)]
pub(crate) struct Pose {
    pub(crate) affine: Affine3A,
    pub(crate) rotation: Quat,
    pub(crate) scale: Vec3,
    // Accumulated authored translation, excluding rest-pivot offsets and root placement.
    motion: Vec3,
    anchor: Vec3,
    pub(crate) local: Transform,
}

impl Default for Pose {
    fn default() -> Self {
        Self {
            affine: Affine3A::IDENTITY,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            motion: Vec3::ZERO,
            anchor: Vec3::ZERO,
            local: Transform::IDENTITY,
        }
    }
}

pub(crate) fn world_input(global: &GlobalTransform, anchor: Vec3) -> PoseInput {
    PoseInput {
        local: global.compute_transform(),
        parent: None,
        node: None,
        world: Some(*global),
        anchor: Some(anchor),
    }
}

pub(crate) fn camera_anchor(
    mut entity: Entity,
    lookup: impl Fn(Entity) -> Option<(Option<Vec3>, Option<Entity>)>,
) -> Vec3 {
    let mut visited = HashSet::new();
    while visited.insert(entity) {
        let Some((anchor, parent)) = lookup(entity) else {
            break;
        };
        if let Some(anchor) = anchor {
            return anchor;
        }
        let Some(parent) = parent else {
            break;
        };
        entity = parent;
    }
    Vec3::ZERO
}

fn safe_ratio(numerator: Vec3, denominator: Vec3) -> Vec3 {
    let divide = |a: f32, b: f32| if b.abs() > 1e-8 { a / b } else { 0.0 };
    Vec3::new(
        divide(numerator.x, denominator.x),
        divide(numerator.y, denominator.y),
        divide(numerator.z, denominator.z),
    )
}

fn inverse_vector(parent: &Pose, vector: Vec3) -> Vec3 {
    if parent.affine.matrix3.determinant().abs() > 1e-12 {
        parent.affine.inverse().transform_vector3(vector)
    } else {
        // A collapsed parent cannot be compensated exactly. Keep the result finite.
        safe_ratio(parent.rotation.inverse() * vector, parent.scale)
    }
}

fn compose(mut local: Transform, node: Option<NodeSample>, parent: Pose, instance: Pose) -> Pose {
    let mut motion = parent.motion;
    let mut anchor = parent.anchor;
    if let Some(node) = node {
        let flags = node.flags;
        let authored_translation = local.translation - node.pivot_offset;
        if flags.dont_inherit_translation() {
            local.translation -= inverse_vector(&parent, motion);
            motion = Vec3::ZERO;
        }
        motion += parent.affine.transform_vector3(authored_translation);
        if flags.dont_inherit_scaling() {
            local.scale *= safe_ratio(instance.scale, parent.scale);
        }
        // Full billboarding already cancels parent rotation. For the other modes,
        // suppress inherited skeletal rotation while retaining model placement.
        if flags.dont_inherit_rotation() && !(flags.billboarded() && node.camera.is_some()) {
            local.rotation = parent.rotation.inverse() * instance.rotation * local.rotation;
        }
        if let Some(camera) = node.camera {
            if flags.billboarded() {
                // MDX billboard geometry faces +X; Bevy cameras face local -Z.
                let basis = Quat::from_rotation_y(-FRAC_PI_2) * Quat::from_rotation_x(-FRAC_PI_2);
                local.rotation =
                    parent.rotation.inverse() * camera.rotation * basis * local.rotation;
            } else if flags.billboard_lock_x()
                || flags.billboard_lock_y()
                || flags.billboard_lock_z()
            {
                let ray =
                    (parent.rotation * local.rotation).inverse() * (camera.rotation * Vec3::Z);
                let (axis, angle) = if flags.billboard_lock_x() {
                    local.scale.z *= -1.0;
                    (Vec3::X, ray.z.atan2(ray.y))
                } else if flags.billboard_lock_y() {
                    (Vec3::Y, (-ray.z).atan2(ray.x))
                } else {
                    (Vec3::Z, ray.y.atan2(ray.x))
                };
                if ray.cross(axis).length_squared() > 1e-12 {
                    local.rotation *= Quat::from_axis_angle(axis, angle);
                }
            }
            if flags.camera_anchored() {
                local.translation += inverse_vector(&parent, camera.translation - anchor);
                anchor = camera.translation;
            }
        }
    }
    Pose {
        affine: parent.affine * local.compute_affine(),
        rotation: (parent.rotation * local.rotation).normalize(),
        scale: parent.scale * local.scale,
        motion,
        anchor,
        local,
    }
}

/// Resolve parents first, keeping signed scale/rotation separate from the affine
/// matrix. Cycle detection also protects callers sampling malformed hierarchies.
pub(crate) fn resolve_pose(
    entity: Entity,
    lookup: &impl Fn(Entity) -> Option<PoseInput>,
    cache: &mut HashMap<Entity, Pose>,
    visiting: &mut HashSet<Entity>,
) -> Option<Pose> {
    if let Some(pose) = cache.get(&entity) {
        return Some(*pose);
    }
    if !visiting.insert(entity) {
        return None;
    }
    let input = lookup(entity)?;
    let mut parent = match input.parent {
        Some(parent) => resolve_pose(parent, lookup, cache, visiting)?,
        None => Pose::default(),
    };
    let instance = match input.node {
        Some(node) => resolve_pose(node.root, lookup, cache, visiting)?,
        None => Pose::default(),
    };
    if input
        .node
        .is_some_and(|node| input.parent == Some(node.root))
    {
        parent.motion = Vec3::ZERO;
    }
    let mut pose = compose(input.local, input.node, parent, instance);
    if let Some(world) = input.world {
        pose.affine = world.affine();
    }
    if let Some(anchor) = input.anchor {
        pose.anchor = anchor;
    }
    visiting.remove(&entity);
    cache.insert(entity, pose);
    Some(pose)
}
