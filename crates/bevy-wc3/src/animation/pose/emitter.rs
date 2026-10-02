use super::evaluation::{camera_anchor, resolve_pose, world_input, PoseInput};
use super::{AnimatedNode, NodeFrame};
use crate::animation::Wc3Animation;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// Local and propagated node inputs used to reconstruct effect birth poses.
pub(crate) type EmitterNodes<'w, 's> = Query<
    'w,
    's,
    (
        &'static GlobalTransform,
        Option<&'static Transform>,
        Option<&'static AnimatedNode>,
        Option<&'static ChildOf>,
    ),
>;

pub(crate) fn sample_emitter_transform(
    entity: Entity,
    root: Entity,
    animation: &Wc3Animation,
    nodes: &EmitterNodes,
) -> Option<GlobalTransform> {
    sample_emitter_transform_at_frame(entity, root, animation, nodes, None)
}

pub(crate) fn sample_emitter_transform_at_frame(
    entity: Entity,
    root: Entity,
    animation: &Wc3Animation,
    nodes: &EmitterNodes,
    frame: Option<NodeFrame>,
) -> Option<GlobalTransform> {
    let lookup = |entity| {
        let (global, transform, node, parent) = nodes.get(entity).ok()?;
        match node.filter(|node| node.root == root) {
            Some(node) => Some(PoseInput {
                world: None,
                anchor: None,
                local: node.sample_transform_at_frame(animation, frame),
                parent: parent.map(ChildOf::parent),
                node: Some(node.pose_sample(node.camera)),
            }),
            None if transform.is_some() => Some(PoseInput {
                local: *transform?,
                parent: parent.map(ChildOf::parent),
                node: None,
                world: None,
                anchor: node.and_then(AnimatedNode::camera_anchor),
            }),
            None => {
                let anchor = camera_anchor(entity, |entity| {
                    let (_, _, node, parent) = nodes.get(entity).ok()?;
                    Some((
                        node.and_then(AnimatedNode::camera_anchor),
                        parent.map(ChildOf::parent),
                    ))
                });
                Some(world_input(global, anchor))
            }
        }
    };
    resolve_pose(entity, &lookup, &mut HashMap::new(), &mut HashSet::new())
        .map(|pose| GlobalTransform::from(Mat4::from(pose.affine)))
}
