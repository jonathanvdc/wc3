//! Checked composition of independently compiled models into one animated rig.
use crate::bounds::{rest_bounds, transform_bounds};
use crate::child_coordinates::ChildCoordinates;
use crate::compiled::{CompiledModel, RuntimeMount, RuntimeNode, RuntimeSkin};
use crate::compiler::node_order;
use crate::lod::LodDefinition;
use crate::mesh_attributes::{EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT};
use crate::pose::local_transform;
use crate::pose::{evaluate_pose, EvaluatedPose};
use crate::{ModelError, PartBinding::*};
use bevy::camera::primitives::Aabb;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use wc3::model::profiles::Universal;
use wc3::model::{
    AnimatedTransform, AnimationProgram, Billboard, ClipId, DrawId, Inheritance, InstanceId, Model,
    ModelMount, MountTarget, NodeId, Quaternion, Transform as ModelTransform,
};

/// Geometry choices independent of animation, materials, and automatic LOD.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawSelection {
    /// Explicit allowed draws. `None` retains every authored draw.
    pub draws: Option<HashSet<DrawId>>,
    /// Entire geometry instances suppressed by the application.
    pub hidden_instances: HashSet<InstanceId>,
}
impl DrawSelection {
    /// Whether an authored draw is selected.
    pub fn includes(&self, draw: DrawId) -> bool {
        self.draws
            .as_ref()
            .is_none_or(|draws| draws.contains(&draw))
    }
}

/// Explicit source-joint to destination-joint correspondence.
///
/// Scoped identities are checked during composition. Equal ordinal positions or
/// display names never imply a binding. Unmapped descendant joints remain local
/// equipment bones; a weighted joint without a mapped ancestor is rejected.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JointMap(BTreeMap<NodeId, NodeId>);
impl JointMap {
    /// Construct a mapping, rejecting conflicting bindings of a source joint.
    pub fn new(pairs: impl IntoIterator<Item = (NodeId, NodeId)>) -> Result<Self, ModelError> {
        let mut map = Self::default();
        for (source, destination) in pairs {
            map.insert(source, destination)?;
        }
        Ok(map)
    }
    /// Bind a source joint, rejecting a conflicting existing destination.
    pub fn insert(&mut self, source: NodeId, destination: NodeId) -> Result<(), ModelError> {
        if self
            .0
            .get(&source)
            .is_some_and(|value| *value != destination)
        {
            return Err(ModelError(
                "source joint has conflicting destinations".into(),
            ));
        }
        self.0.insert(source, destination);
        Ok(())
    }
    /// Ordered source/destination identities.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (NodeId, NodeId)> + '_ {
        self.0
            .iter()
            .map(|(&source, &destination)| (source, destination))
    }
}

/// Relationship of a component model to a character or other carrying rig.
#[derive(Clone, Debug, PartialEq)]
pub enum PartBinding {
    /// Instantiate an independent child rig at a semantic node or attachment point.
    Mounted(ModelMount),
    /// Reuse host joints for skinning and retain only extra equipment bones.
    Skinned {
        /// Checked source/destination correspondence.
        joints: JointMap,
        /// Part authored coordinates into host authored coordinates at rest.
        transform: Mat4,
    },
}

/// One independently compiled component, borrowed only during composition.
pub struct ModelPart<'a> {
    /// Compiled geometry, effects and optional extra skeleton.
    pub model: &'a CompiledModel,
    /// Rigid placement or shared skeleton correspondence.
    pub binding: PartBinding,
    /// Host clip to source clip; both run on the host's time in seconds.
    /// Unmapped clips leave source clip properties at their defaults; source
    /// global/shared animation still runs. No curve is silently stretched.
    pub clips: HashMap<ClipId, ClipId>,
}
impl<'a> ModelPart<'a> {
    /// Build a component with explicit binding and no clip synchronization.
    pub fn new(model: &'a CompiledModel, binding: PartBinding) -> Self {
        Self {
            model,
            binding,
            clips: HashMap::new(),
        }
    }
}

fn finite_frame(matrix: Mat4) -> Result<(), ModelError> {
    if !matrix.is_finite()
        || matrix.determinant().abs() < 1e-10
        || !matrix.row(3).abs_diff_eq(Vec4::W, 1e-6)
    {
        return Err(ModelError(
            "composition requires a finite invertible affine frame".into(),
        ));
    }
    Ok(())
}
fn transform(matrix: Mat4) -> Result<ModelTransform, ModelError> {
    finite_frame(matrix)?;
    let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
    if !Mat4::from_scale_rotation_translation(scale, rotation, translation)
        .abs_diff_eq(matrix, 1e-4)
    {
        return Err(ModelError(
            "composition frame contains shear that a hierarchy node cannot represent".into(),
        ));
    }
    Ok(ModelTransform {
        translation: translation.to_array(),
        rotation: Quaternion(rotation.to_array()),
        scale: scale.to_array(),
        ..Default::default()
    })
}
fn rest_poses(model: &CompiledModel) -> Vec<Mat4> {
    let mut poses = vec![EvaluatedPose::default(); model.nodes.len()];
    for &i in model.order.iter() {
        let node = &model.nodes[i];
        poses[i] = evaluate_pose(
            node,
            node.rest,
            node.parent
                .map_or_else(EvaluatedPose::default, |p| poses[p]),
            None,
        );
    }
    poses.into_iter().map(|pose| pose.matrix).collect()
}
fn used_joints(model: &CompiledModel) -> HashSet<usize> {
    let mut used = HashSet::new();
    for draw in &model.draws {
        let Some(skin) = draw.skin.map(|i| &model.skins[i]) else {
            continue;
        };
        for (indices, weights) in [
            (Mesh::ATTRIBUTE_JOINT_INDEX, Mesh::ATTRIBUTE_JOINT_WEIGHT),
            (EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT),
        ] {
            if let (
                Some(VertexAttributeValues::Uint16x4(indices)),
                Some(VertexAttributeValues::Float32x4(weights)),
            ) = (draw.mesh.attribute(indices), draw.mesh.attribute(weights))
            {
                for (indices, weights) in indices.iter().zip(weights) {
                    for (&joint, &weight) in indices.iter().zip(weights) {
                        if weight > 0. {
                            used.insert(skin.joints[usize::from(joint)]);
                        }
                    }
                }
            }
        }
    }
    used
}

impl CompiledModel {
    /// Inspect authored hierarchy identities without exposing compiler storage.
    pub fn nodes(&self) -> impl ExactSizeIterator<Item = (NodeId, &str)> {
        self.nodes.iter().map(|node| (node.id, node.name.as_str()))
    }
    /// Compose independently scoped component assets into one reusable character definition.
    ///
    /// Source geometry stays shared. Skin palettes receive a rest-space correction
    /// so different bind poses remain distinct. Mapped joints are never duplicated
    /// or driven by equipment animation. Local extra bones, scene features, effects
    /// and physics are retained. Preparation and spawning use the ordinary model API.
    /// No input changes when validation fails.
    pub fn compose<'a>(
        &self,
        parts: impl IntoIterator<Item = ModelPart<'a>>,
    ) -> Result<Self, ModelError> {
        let mut output = self.clone();
        let mut identities = Model::<Universal>::builder("composition frames");
        for part in parts {
            let source = part.model;
            let source_indices: HashMap<_, _> = source
                .nodes
                .iter()
                .enumerate()
                .map(|(i, node)| (node.id, i))
                .collect();
            let host_indices: HashMap<_, _> = output
                .nodes
                .iter()
                .enumerate()
                .map(|(i, node)| (node.id, i))
                .collect();
            let rigid = match &part.binding {
                Mounted(placement) => {
                    let (parent, point) = match placement.target {
                        MountTarget::Node(parent) => (parent, None),
                        MountTarget::Socket(point) => {
                            let parent = output
                                .features
                                .attachment_node(point)
                                .ok_or_else(|| ModelError("attachment point is absent".into()))?;
                            (parent, Some(point))
                        }
                    };
                    Some((parent, point, *placement))
                }
                Skinned { .. } => None,
            };
            if let Some((parent, point, placement)) = rigid {
                let parent = *host_indices
                    .get(&parent)
                    .ok_or_else(|| ModelError("rigid mount node is absent".into()))?;
                placement
                    .validate_transform()
                    .map_err(|error| ModelError(error.to_string()))?;
                let conversion = ChildCoordinates {
                    coordinates: placement.coordinates,
                    parent_basis: output.basis,
                }
                .basis(source.basis)
                .to_matrix();
                let placement =
                    transform(local_transform(placement.transform).to_matrix() * conversion)?;
                for (&host, &child) in &part.clips {
                    if !output.clips.iter().any(|(id, _)| *id == host)
                        || !source.clips.iter().any(|(id, _)| *id == child)
                    {
                        return Err(ModelError(
                            "rigid clip mapping references an absent clip".into(),
                        ));
                    }
                }
                output.mounts.push(RuntimeMount {
                    parent,
                    socket: point,
                    transform: local_transform(placement),
                    model: Arc::new(source.clone()),
                    clips: part.clips,
                });
                let mut diagnostics = output.diagnostics.to_vec();
                diagnostics.extend(source.diagnostics.iter().cloned());
                output.diagnostics = diagnostics.into();
                continue;
            }
            if source_indices
                .keys()
                .any(|id| host_indices.contains_key(id))
            {
                return Err(ModelError(
                    "composition requires independently scoped model nodes".into(),
                ));
            }
            if !source.world.groups.is_empty() || !source.world.environments.is_empty() {
                return Err(ModelError(
                    "static-world parts require a rigid mount to retain their regional selection"
                        .into(),
                ));
            }
            let source_rest = rest_poses(source);
            let host_rest = rest_poses(&output);
            let mut mapped = HashMap::new();
            let (frame, parent) = match &part.binding {
                Mounted(_) => unreachable!("rigid parts were handled above"),
                Skinned { joints, transform } => {
                    if joints.0.is_empty() {
                        return Err(ModelError(
                            "skinned part requires explicit joint bindings".into(),
                        ));
                    }
                    for (source, destination) in joints.iter() {
                        let source = *source_indices
                            .get(&source)
                            .ok_or_else(|| ModelError("source binding joint is absent".into()))?;
                        let destination = *host_indices
                            .get(&destination)
                            .ok_or_else(|| ModelError("host binding joint is absent".into()))?;
                        mapped.insert(source, destination);
                    }
                    for joint in used_joints(source) {
                        let mut ancestor = Some(joint);
                        while let Some(index) = ancestor {
                            if mapped.contains_key(&index) {
                                break;
                            }
                            ancestor = source.nodes[index].parent;
                        }
                        if ancestor.is_none() {
                            return Err(ModelError(format!(
                                "weighted equipment joint {:?} has no host binding or bound ancestor",
                                source.nodes[joint].id
                            )));
                        }
                    }
                    (*transform, None)
                }
            };
            finite_frame(frame)?;
            let mut nodes = output.nodes.to_vec();
            let frame_index = nodes.len();
            let rest = transform(frame)?;
            let definition = identities
                .new_node("part coordinate frame")
                .with_transform(rest, identities.values_mut())
                .map_err(|e| ModelError(e.to_string()))?;
            let slots = definition.transform;
            let visibility = definition.visible;
            let frame_id = identities
                .add_node(definition)
                .map_err(|e| ModelError(e.to_string()))?;
            nodes.push(RuntimeNode {
                id: frame_id,
                name: "part coordinate frame".into(),
                parent,
                rest,
                transform: slots,
                visibility,
                channels: Default::default(),
                inheritance: Inheritance::default(),
                billboard: Billboard::None,
                billboard_convention: Default::default(),
                visible: true,
                camera_anchored: false,
            });
            let mut node_map = vec![usize::MAX; source.nodes.len()];
            for (index, node) in source.nodes.iter().enumerate() {
                if let Some(&host) = mapped.get(&index) {
                    node_map[index] = host;
                } else {
                    node_map[index] = nodes.len();
                    nodes.push(node.clone());
                }
            }
            // Only frames used by extra bones, geometry, or features are created.
            // Their identities remain source-local; the actual skin uses host joints.
            let mut anchors = HashMap::new();
            let mut anchor =
                |index: usize, nodes: &mut Vec<RuntimeNode>| -> Result<usize, ModelError> {
                    if let Some(&existing) = anchors.get(&index) {
                        return Ok(existing);
                    }
                    let host = mapped[&index];
                    finite_frame(host_rest[host])?;
                    let mut node = source.nodes[index].clone();
                    node.parent = Some(host);
                    node.rest = transform(host_rest[host].inverse() * frame * source_rest[index])?;
                    let pivot = Vec3::from_array(source.nodes[index].rest.pivot);
                    let rotation = Quat::from_array(node.rest.rotation.0);
                    let scale = Vec3::from_array(node.rest.scale);
                    node.rest.translation = (Vec3::from_array(node.rest.translation) - pivot
                        + rotation * (scale * pivot))
                        .to_array();
                    node.rest.pivot = pivot.to_array();
                    node.transform = AnimatedTransform::new(node.rest, identities.values_mut());
                    node.visibility = identities.add_value(node.visible);
                    node.channels = Default::default();
                    node.billboard = Billboard::None;
                    node.camera_anchored = false;
                    node.inheritance = Inheritance::default();
                    let output = nodes.len();
                    nodes.push(node);
                    anchors.insert(index, output);
                    Ok(output)
                };
            for (index, node) in source.nodes.iter().enumerate() {
                if mapped.contains_key(&index) {
                    continue;
                }
                let parent = match node.parent {
                    Some(parent) if mapped.contains_key(&parent) => anchor(parent, &mut nodes)?,
                    Some(parent) => node_map[parent],
                    None => frame_index,
                };
                nodes[node_map[index]].parent = Some(parent);
            }
            let mut frame_map = node_map.clone();
            // Definition interpreters retain source node identities, including
            // mapped feature anchors with a different rest-frame offset.
            let required: HashSet<_> = source
                .features
                .hooks()
                .iter()
                .map(|(feature, _)| feature.node)
                .chain(
                    source
                        .draws
                        .iter()
                        .filter(|draw| draw.skin.is_none())
                        .map(|draw| source.nodes[draw.node].id),
                )
                .chain(
                    source
                        .events
                        .definitions
                        .iter()
                        .filter_map(|event| event.node),
                )
                .chain(
                    source
                        .mounts
                        .iter()
                        .map(|mount| source.nodes[mount.parent].id),
                )
                .collect();
            for id in required {
                let index = source_indices[&id];
                if mapped.contains_key(&index) {
                    frame_map[index] = anchor(index, &mut nodes)?;
                }
            }
            let id_map: HashMap<_, _> = source
                .nodes
                .iter()
                .enumerate()
                .map(|(i, node)| (node.id, nodes[frame_map[i]].id))
                .collect();
            let bound_ids: HashSet<_> = mapped.keys().map(|&i| source.nodes[i].id).collect();
            let bound_slots: HashSet<_> = source
                .nodes
                .iter()
                .filter(|node| bound_ids.contains(&node.id))
                .flat_map(|node| {
                    [
                        node.transform.translation.slot(),
                        node.transform.rotation.slot(),
                        node.transform.scale.slot(),
                        node.visibility.slot(),
                    ]
                })
                .collect();
            let clip_pairs: Vec<_> = part
                .clips
                .iter()
                .map(|(&host, &part)| (host, part))
                .collect();
            output.animation = Arc::new(
                output
                    .animation
                    .compose(&source.animation, &clip_pairs, |channel| {
                        !bound_slots.contains(&channel.target())
                    })
                    .map_err(|e| ModelError(e.to_string()))?,
            );
            for mount in &source.mounts {
                output.mounts.push(RuntimeMount {
                    parent: frame_map[mount.parent],
                    socket: mount.socket,
                    transform: mount.transform,
                    model: mount.model.clone(),
                    clips: part
                        .clips
                        .iter()
                        .filter_map(|(&host, &source)| {
                            mount.clips.get(&source).map(|&child| (host, child))
                        })
                        .collect(),
                });
            }
            let mut timing = output.timing.as_ref().clone();
            for (&host, &part_clip) in &part.clips {
                if let Some(source_timing) = source
                    .timing
                    .get(&part_clip)
                    .filter(|timing| !timing.controllers.is_empty())
                {
                    let host_timing = timing.get_mut(&host).ok_or_else(|| {
                        ModelError(
                            "controller-driven component needs an authored host timeline".into(),
                        )
                    })?;
                    if host_timing.origin != source_timing.origin {
                        return Err(ModelError(
                            "controller-driven components need matching authored timeline origins"
                                .into(),
                        ));
                    }
                    let mut controllers = host_timing.controllers.to_vec();
                    for &controller in source_timing.controllers.iter() {
                        if !controllers.contains(&controller) {
                            controllers.push(controller);
                        }
                    }
                    host_timing.controllers = controllers.into();
                }
            }
            output.timing = Arc::new(timing);
            let skin_offset = output.skins.len();
            for skin in &source.skins {
                let mut binds = skin.binds.to_vec();
                for (slot, &joint) in skin.joints.iter().enumerate() {
                    if let Some(&host) = mapped.get(&joint) {
                        finite_frame(host_rest[host])?;
                        binds[slot] =
                            host_rest[host].inverse() * frame * source_rest[joint] * binds[slot];
                    }
                }
                output.skins.push(RuntimeSkin {
                    joints: skin.joints.iter().map(|&i| node_map[i]).collect(),
                    binds: binds.into(),
                });
            }
            for draw in &source.draws {
                let mut draw = draw.clone();
                draw.node = frame_map[draw.node];
                draw.skin = draw.skin.map(|i| i + skin_offset);
                output.draws.push(draw);
            }
            for liquid in &source.liquids {
                let mut liquid = liquid.clone();
                liquid.node = frame_map[liquid.node];
                output.liquids.push(liquid);
            }
            Arc::make_mut(&mut output.features).append_remapped(&source.features, &id_map);
            Arc::make_mut(&mut output.particles).append_remapped(
                &source.particles,
                &frame_map,
                &id_map,
                &part.clips,
            );
            Arc::make_mut(&mut output.events).append_remapped(
                &source.events,
                &frame_map,
                &id_map,
                &part.clips,
            );
            Arc::make_mut(&mut output.physics).append_remapped(&source.physics, &node_map, &nodes);
            let mut diagnostics = output.diagnostics.to_vec();
            diagnostics.extend(source.diagnostics.iter().cloned());
            output.diagnostics = diagnostics.into();
            output.has_animation |= source.has_animation;
            output.pose_animated |= source.pose_animated;
            output.camera_sensitive |= source.camera_sensitive;
            output.order = node_order(&nodes)?.into();
            output.nodes = nodes.into();
        }
        output.lod = Arc::new(LodDefinition::compile(
            &output.draws,
            output.world.lod_levels(),
        ));
        output.rest_bounds = rest_bounds(&output, None);
        let poses = rest_poses(&output);
        for mount in &output.mounts {
            if let Some(child) = mount.model.rest_bounds {
                let matrix = output.basis.to_matrix()
                    * poses[mount.parent]
                    * mount.transform.to_matrix()
                    * mount.model.basis.to_matrix().inverse();
                let bounds = transform_bounds(child, matrix);
                {
                    output.rest_bounds = Some(match output.rest_bounds {
                        Some(current) => Aabb::from_min_max(
                            current.min().min(bounds.min()).into(),
                            current.max().max(bounds.max()).into(),
                        ),
                        None => bounds,
                    });
                }
            }
        }
        let frames = identities.finish().map_err(|e| ModelError(e.to_string()))?;
        output.animation = Arc::new(
            output
                .animation
                .compose(&AnimationProgram::compile(&frames), &[], |_| true)
                .map_err(|e| ModelError(e.to_string()))?,
        );
        Ok(output)
    }
}

#[cfg(test)]
#[path = "composition_tests.rs"]
mod tests;
