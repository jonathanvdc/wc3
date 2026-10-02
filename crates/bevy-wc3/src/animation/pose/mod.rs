//! Shared CPU node evaluation for visible poses and effect birth times.
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub(crate) use self::emitter::{sample_emitter_transform, EmitterNodes};
pub(crate) use self::evaluation::{resolve_pose, PoseInput};
pub(crate) use self::node::AnimatedNode;
use crate::animation::Wc3Animation;
mod emitter;
mod evaluation;
mod node;

/// Selects the camera driving camera-dependent node flags on a model root.
/// Attached models inherit the nearest ancestor selection unless overridden.
#[derive(Component, Clone, Copy, Debug)]
pub struct Wc3NodeCamera(pub Entity);

/// Marks the default active 3D camera for WC3 node flags.
/// Without a marker, the sole active window camera (or sole active 3D camera)
/// is selected. Multiple matching cameras require an explicit selection.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Wc3DefaultNodeCamera;

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

type NodeCameras<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Camera,
        Option<&'static RenderTarget>,
        Has<Wc3DefaultNodeCamera>,
    ),
    With<Camera3d>,
>;

pub(crate) fn animate_nodes(
    animations: Query<&Wc3Animation>,
    animated_entities: Query<Entity, With<AnimatedNode>>,
    cameras: NodeCameras,
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
#[path = "tests.rs"]
mod tests;
