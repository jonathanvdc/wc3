//! Shared CPU node evaluation for visible poses and effect birth times.
use bevy::camera::RenderTarget;
use bevy::math::Affine3A;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::f32::consts::FRAC_PI_2;
use wc3::model::scene::NodeFlags;

use crate::animation::{AnimatedNode, Wc3Animation};

/// Selects the camera driving camera-dependent node flags on a model root.
/// Attached models inherit the nearest ancestor selection unless overridden.
#[derive(Component, Clone, Copy, Debug)]
pub struct Wc3NodeCamera(pub Entity);

/// Marks the default active 3D camera for WC3 node flags.
/// Without a marker, the sole active window camera (or sole active 3D camera)
/// is selected. Multiple matching cameras require an explicit selection.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Wc3DefaultNodeCamera;

/// Ordering point for applications that move cameras or model roots in PostUpdate.
/// Schedule those systems before Evaluate; node poses resolve before Bevy propagation.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Wc3NodePoseSystems {
    Evaluate,
}

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

type NodeInputs<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        Option<&'static AnimatedNode>,
        Option<&'static ChildOf>,
        Option<&'static Wc3NodeCamera>,
    ),
>;

pub(crate) fn animate_nodes(
    animations: Query<&Wc3Animation>,
    animated_entities: Query<Entity, With<AnimatedNode>>,
    cameras: Query<
        (
            Entity,
            &Camera,
            Option<&RenderTarget>,
            Has<Wc3DefaultNodeCamera>,
        ),
        With<Camera3d>,
    >,
    mut nodes: ParamSet<(NodeInputs, Query<(&mut AnimatedNode, &mut Transform)>)>,
    mut last_warnings: Local<HashMap<Entity, Option<Entity>>>,
) {
    let active: Vec<_> = cameras
        .iter()
        .filter(|(_, camera, _, _)| camera.is_active)
        .collect();
    let marked: Vec<_> = active
        .iter()
        .filter(|(_, _, _, marked)| *marked)
        .map(|(entity, _, _, _)| *entity)
        .collect();
    let windows: Vec<_> = active
        .iter()
        .filter(|(_, _, target, _)| {
            target.is_none_or(|target| matches!(target, RenderTarget::Window(_)))
        })
        .map(|(entity, _, _, _)| *entity)
        .collect();
    let default_camera = if !marked.is_empty() {
        (marked.len() == 1).then(|| marked[0])
    } else if !windows.is_empty() {
        (windows.len() == 1).then(|| windows[0])
    } else {
        (active.len() == 1).then(|| active[0].0)
    };
    let inputs = nodes.p0();
    let mut selections = HashMap::new();
    let mut camera_roots = HashSet::new();
    for entity in &animated_entities {
        let Ok((_, _, Some(node), _, _)) = inputs.get(entity) else {
            continue;
        };
        if node.flags.bits() & 0xf8 != 0 {
            camera_roots.insert(node.root);
        }
        selections.entry(node.root).or_insert_with(|| {
            let mut ancestor = Some(node.root);
            let mut visited = HashSet::new();
            while let Some(entity) = ancestor {
                if !visited.insert(entity) {
                    break;
                }
                let Ok((_, _, _, parent, selection)) = inputs.get(entity) else {
                    break;
                };
                if let Some(selection) = selection {
                    return Some(selection.0);
                }
                ancestor = parent.map(ChildOf::parent);
            }
            default_camera
        });
    }
    // Camera transforms are read from current local transforms, avoiding a frame
    // of lag when the application moves its camera in Update.
    let camera_lookup = |entity| {
        let (_, transform, _, parent, _) = inputs.get(entity).ok()?;
        Some(PoseInput {
            local: *transform,
            parent: parent.map(ChildOf::parent),
            node: None,
            world: None,
            anchor: None,
        })
    };
    let mut camera_cache = HashMap::new();
    let mut camera_poses = HashMap::new();
    for (&root, &selection) in &selections {
        let pose = selection
            .filter(|entity| active.iter().any(|(id, _, _, _)| id == entity))
            .and_then(|entity| {
                resolve_pose(
                    entity,
                    &camera_lookup,
                    &mut camera_cache,
                    &mut HashSet::new(),
                )
            })
            .map(|pose| Transform {
                translation: pose.affine.translation.into(),
                rotation: pose.rotation,
                scale: pose.scale,
            });
        camera_poses.insert(root, pose);
        if pose.is_some() {
            last_warnings.remove(&root);
        } else if camera_roots.contains(&root) && last_warnings.get(&root) != Some(&selection) {
            warn!("WC3 model {root:?} has no unambiguous active node camera ({selection:?}); camera-dependent flags are skipped. Set Wc3NodeCamera or mark one Wc3DefaultNodeCamera.");
            last_warnings.insert(root, selection);
        }
    }
    last_warnings.retain(|root, _| selections.contains_key(root));
    let lookup = |entity| {
        let (_, transform, node, parent, _) = inputs.get(entity).ok()?;
        let local = node
            .and_then(|node| {
                animations
                    .get(node.root)
                    .ok()
                    .map(|animation| node.sample_transform(animation))
            })
            .unwrap_or(*transform);
        Some(PoseInput {
            local,
            parent: parent.map(ChildOf::parent),
            world: None,
            anchor: None,
            node: node
                .map(|node| node.pose_sample(camera_poses.get(&node.root).copied().flatten())),
        })
    };
    let mut cache = HashMap::new();
    let resolved: Vec<_> = animated_entities
        .iter()
        .filter_map(|entity| {
            let (_, _, node, _, _) = inputs.get(entity).ok()?;
            let node = node?;
            let pose = resolve_pose(entity, &lookup, &mut cache, &mut HashSet::new())?;
            Some((
                entity,
                pose.local,
                camera_poses.get(&node.root).copied().flatten(),
            ))
        })
        .collect();
    let mut outputs = nodes.p1();
    for (entity, local, camera) in resolved {
        if let Ok((mut node, mut transform)) = outputs.get_mut(entity) {
            node.camera = camera;
            *transform = local;
        }
    }
}

#[cfg(test)]
#[path = "node_pose_tests.rs"]
mod tests;
