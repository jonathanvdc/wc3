//! Shared CPU node evaluation for visible poses and effect birth times.
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use std::collections::HashSet;

pub(crate) use self::emitter::{
    sample_emitter_transform, sample_emitter_transform_at_frame, EmitterNodes,
};
pub(crate) use self::evaluation::{resolve_pose, PoseInput};
pub(crate) use self::node::{AnimatedNode, NodeFrame};
mod emitter;
mod live;
pub(crate) use live::{animate_nodes, PoseOrder};
pub(crate) mod evaluation;
mod node;

/// Selects the camera driving camera-dependent node flags on a model root.
/// Attached models inherit the nearest ancestor selection unless overridden.
#[derive(Component, Clone, Copy, Debug)]
pub struct Wc3NodeCamera(
    /// Entity of the camera driving this instance's node poses.
    pub Entity,
);

/// Marks the default active 3D camera for WC3 node flags.
/// Without a marker, the sole active window camera (or sole active 3D camera)
/// is selected. Multiple matching cameras require an explicit selection.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Wc3DefaultNodeCamera;

/// Shared driving-camera precedence for node poses and LOD selection.
pub(crate) fn default_node_camera(
    cameras: impl IntoIterator<Item = (Entity, bool, bool)>,
) -> Option<Entity> {
    let mut active = (0, None);
    let mut marked = (0, None);
    let mut windows = (0, None);
    for (entity, window, marker) in cameras {
        active.0 += 1;
        active.1 = Some(entity);
        if marker {
            marked.0 += 1;
            marked.1 = Some(entity);
        }
        if window {
            windows.0 += 1;
            windows.1 = Some(entity);
        }
    }
    let (count, entity) = if marked.0 > 0 {
        marked
    } else if windows.0 > 0 {
        windows
    } else {
        active
    };
    (count == 1).then_some(entity).flatten()
}

pub(crate) fn inherited_node_camera(
    root: Entity,
    lookup: impl Fn(Entity) -> Option<(Option<Entity>, Option<Entity>)>,
) -> Option<Entity> {
    let mut ancestor = Some(root);
    let mut visited = HashSet::new();
    while let Some(entity) = ancestor {
        if !visited.insert(entity) {
            break;
        }
        let (parent, selection) = lookup(entity)?;
        if selection.is_some() {
            return selection;
        }
        ancestor = parent;
    }
    None
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
