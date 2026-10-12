//! Reusable live-pose buffers, with prepared traversal and shared flag composition.
use super::evaluation::{compose_pose, NodeSample, Pose};
use super::AnimatedNode;
use super::{default_node_camera, resolve_pose, NodeCameras, NodeInputs, PoseInput};
use crate::animation::Wc3Animation;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Component)]
pub(crate) struct PoseOrder(pub(crate) Vec<Entity>);

struct RootFrame {
    entity: Entity,
    start: usize,
    end: usize,
    camera: Option<Transform>,
    camera_required: bool,
    correction_required: bool,
}
struct NodeFrameSample {
    entity: Entity,
    root: usize,
    parent: Option<Entity>,
    local: Transform,
    node: NodeSample,
}
impl NodeFrameSample {
    fn input(&self, roots: &[RootFrame]) -> PoseInput {
        let mut node = self.node;
        node.camera = roots[self.root].camera;
        PoseInput {
            local: self.local,
            parent: self.parent,
            node: Some(node),
            world: None,
            anchor: None,
        }
    }
}
#[derive(Default)]
pub(crate) struct PoseScratch {
    roots: Vec<RootFrame>,
    root_indices: HashMap<Entity, usize>,
    samples: Vec<NodeFrameSample>,
    sample_indices: HashMap<Entity, usize>,
    resolved: Vec<Option<Transform>>,
    active_cameras: HashSet<Entity>,
    camera_cache: HashMap<Entity, Pose>,
    pose_cache: HashMap<Entity, Pose>,
    visiting: HashSet<Entity>,
}

pub(crate) fn animate_nodes(
    mut animations: Query<(Entity, &mut Wc3Animation, Option<&PoseOrder>)>,
    animated_entities: Query<Entity, With<AnimatedNode>>,
    cameras: NodeCameras,
    mut nodes: ParamSet<(NodeInputs, Query<(&mut AnimatedNode, &mut Transform)>)>,
    mut last_warnings: Local<HashMap<Entity, Option<Entity>>>,
    mut scratch: Local<PoseScratch>,
) {
    let PoseScratch {
        roots,
        root_indices,
        samples,
        sample_indices,
        resolved,
        active_cameras,
        camera_cache,
        pose_cache,
        visiting,
    } = &mut *scratch;
    roots.clear();
    root_indices.clear();
    samples.clear();
    sample_indices.clear();
    resolved.clear();
    active_cameras.clear();
    camera_cache.clear();
    pose_cache.clear();
    visiting.clear();
    let default_camera = default_node_camera(
        cameras
            .iter()
            .filter(|(_, camera, _, _)| camera.is_active)
            .map(|(entity, _, target, marked)| {
                active_cameras.insert(entity);
                (
                    entity,
                    target.is_none_or(|target| matches!(target, RenderTarget::Window(_))),
                    marked,
                )
            }),
    );
    let inputs = nodes.p0();
    for (root, animation, order) in &animations {
        let root_index = roots.len();
        let start = samples.len();
        let mut append = |entity| {
            let Ok((_, _, Some(node), parent, _)) = inputs.get(entity) else {
                return;
            };
            if node.root != root {
                return;
            }
            sample_indices.insert(entity, samples.len());
            samples.push(NodeFrameSample {
                entity,
                root: root_index,
                parent: parent.map(ChildOf::parent),
                local: node.sample_transform(entity, animation),
                node: node.pose_sample(None),
            });
        };
        if let Some(order) = order {
            for &entity in &order.0 {
                append(entity);
            }
        } else {
            // Manually created animation roots have no prepared rig order.
            for entity in &animated_entities {
                append(entity);
            }
        }
        let end = samples.len();
        if start == end {
            continue;
        }
        root_indices.insert(root, root_index);
        roots.push(RootFrame {
            entity: root,
            start,
            end,
            camera: None,
            camera_required: samples[start..end]
                .iter()
                .any(|sample| sample.node.flags.bits() & 0xf8 != 0),
            correction_required: samples[start..end]
                .iter()
                .any(|sample| sample.node.flags.bits() & 0xff != 0),
        });
    }
    // Resolve current camera transforms once, independently of the render world's
    // previous frame. Selections inherit through attached model roots.
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
    for root in roots.iter_mut() {
        visiting.clear();
        let mut ancestor = Some(root.entity);
        let mut selection = None;
        while let Some(entity) = ancestor {
            if !visiting.insert(entity) {
                break;
            }
            let Ok((_, _, _, parent, camera)) = inputs.get(entity) else {
                break;
            };
            if let Some(camera) = camera {
                selection = Some(camera.0);
                break;
            }
            ancestor = parent.map(ChildOf::parent);
        }
        let selection = selection.or(default_camera);
        visiting.clear();
        root.camera = selection
            .filter(|entity| active_cameras.contains(entity))
            .and_then(|entity| resolve_pose(entity, &camera_lookup, camera_cache, visiting))
            .map(|pose| Transform {
                translation: pose.affine.translation.into(),
                rotation: pose.rotation,
                scale: pose.scale,
            });
        if root.camera.is_some() {
            last_warnings.remove(&root.entity);
        } else if root.camera_required && last_warnings.get(&root.entity) != Some(&selection) {
            warn!("WC3 model {:?} has no unambiguous active node camera ({selection:?}); camera-dependent flags are skipped. Set Wc3NodeCamera or mark one Wc3DefaultNodeCamera.", root.entity);
            last_warnings.insert(root.entity, selection);
        }
    }
    last_warnings.retain(|root, _| root_indices.contains_key(root));
    let lookup = |entity| {
        if let Some(&index) = sample_indices.get(&entity) {
            return Some(samples[index].input(roots));
        }
        let (_, transform, node, parent, _) = inputs.get(entity).ok()?;
        Some(PoseInput {
            local: *transform,
            parent: parent.map(ChildOf::parent),
            node: node.map(|node| {
                node.pose_sample(
                    root_indices
                        .get(&node.root)
                        .and_then(|&index| roots[index].camera),
                )
            }),
            world: None,
            anchor: None,
        })
    };
    resolved.extend(
        samples
            .iter()
            .map(|sample| (sample.node.flags.bits() & 0xff == 0).then_some(sample.local)),
    );
    for root in roots.iter().filter(|root| root.correction_required) {
        visiting.clear();
        let Some(instance) = resolve_pose(root.entity, &lookup, pose_cache, visiting) else {
            continue;
        };
        for index in root.start..root.end {
            let sample = &samples[index];
            let pose = if let Some(pose) = pose_cache.get(&sample.entity) {
                *pose
            } else {
                let parent = if let Some(parent) = sample.parent {
                    visiting.clear();
                    let Some(parent) = resolve_pose(parent, &lookup, pose_cache, visiting) else {
                        continue;
                    };
                    parent
                } else {
                    Pose::default()
                };
                let pose = compose_pose(sample.input(roots), parent, instance);
                pose_cache.insert(sample.entity, pose);
                pose
            };
            resolved[index] = Some(pose.local);
        }
    }
    let mut outputs = nodes.p1();
    for (sample, local) in samples.iter().zip(resolved.iter()) {
        let Some(local) = local else { continue };
        if let Ok((mut node, mut transform)) = outputs.get_mut(sample.entity) {
            let camera = roots[sample.root].camera;
            if node.camera != camera {
                node.camera = camera;
            }
            transform.set_if_neq(*local);
        }
    }
    for root in roots.iter() {
        if let Ok((_, mut animation, _)) = animations.get_mut(root.entity) {
            let current = &animation.pose_playback.current;
            let samples = &samples[root.start..root.end];
            if current.len() != samples.len()
                || samples
                    .iter()
                    .any(|sample| current.get(&sample.entity) != Some(&sample.local))
            {
                let current = Arc::make_mut(&mut animation.pose_playback.current);
                current.clear();
                current.extend(samples.iter().map(|sample| (sample.entity, sample.local)));
            }
        }
    }
}
