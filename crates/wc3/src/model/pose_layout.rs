//! Reusable hierarchy indexing and allocation-free pose output.
use crate::model::{
    AnimatedTransform, Billboard, Error, EvaluatedPose, Inheritance, Model, NodeId, PoseBehavior,
    Property, Skin, Transform as ModelTransform, TransformSemantics, ValueSample,
};
use glam::{Affine3A, Mat4, Vec3};
use std::collections::{HashMap, HashSet};
/// Authored node properties with a channel in any playback programme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeChannels {
    /// Whether any programme animates translation.
    pub translation: bool,
    /// Whether any programme animates rotation.
    pub rotation: bool,
    /// Whether any programme animates scale.
    pub scale: bool,
    /// Whether any programme animates visibility.
    pub visibility: bool,
}

/// Prepared hierarchy node with stable identity and resolved parent index.
#[derive(Clone, Debug)]
pub struct PoseNode {
    /// Source graph identity.
    pub id: NodeId,
    /// Owned source name.
    pub name: String,
    /// Parent index in the prepared node array.
    pub parent: Option<usize>,
    /// Authored local rest transform.
    pub rest: ModelTransform,
    /// Slot identities read by this hierarchy node.
    pub transform: AnimatedTransform,
    /// Visibility slot.
    pub visibility: Property<bool>,
    /// Presence of channels across all authored programmes.
    pub channels: NodeChannels,
    /// Components inherited from the parent.
    pub inheritance: Inheritance,
    /// Authored camera-facing behavior.
    pub billboard: Billboard,
    /// Transform composition and selective inheritance rules.
    pub transform_semantics: TransformSemantics,
    /// Authored local visibility.
    pub visible: bool,
    /// Whether the origin follows the driving camera.
    pub camera_anchored: bool,
    /// Parent-axis affine correction about the freshly animated joint origin.
    /// Applied after animation and before hierarchy accumulation; bind poses remain unchanged.
    pub correction: Option<Mat4>,
}

impl PoseNode {
    /// Camera and inheritance semantics of this node.
    #[inline]
    pub fn behavior(&self) -> PoseBehavior {
        PoseBehavior {
            inheritance: self.inheritance,
            billboard: self.billboard,
            transform_semantics: self.transform_semantics,
            camera_anchored: self.camera_anchored,
        }
    }
    /// Evaluates animation and a selected affine correction without decomposing shear.
    /// Corrected hierarchies require ordinary affine inheritance, checked by `PoseLayout::new`.
    pub fn evaluate(
        &self,
        pose: ModelTransform,
        parent: EvaluatedPose,
        camera: Option<Affine3A>,
    ) -> EvaluatedPose {
        let mut evaluated = self.behavior().evaluate(pose, parent, camera);
        if let Some(correction) = self.correction {
            let local = pose.to_matrix();
            let origin = local.transform_point3(Vec3::from_array(pose.pivot));
            evaluated.matrix = parent.matrix
                * Mat4::from_translation(origin)
                * correction
                * Mat4::from_translation(-origin)
                * local;
        }
        evaluated
    }
    /// Samples local animation without hierarchy composition or application overrides.
    #[inline]
    pub fn sample(&self, sample: &impl ValueSample) -> ModelTransform {
        ModelTransform {
            translation: sample.animated(&self.transform.translation),
            rotation: sample.animated(&self.transform.rotation),
            scale: sample.animated(&self.transform.scale),
            pivot: self.rest.pivot,
        }
    }
    /// Samples this node's local visibility; parent visibility is evaluated separately.
    #[inline]
    pub fn sample_visibility(&self, sample: &impl ValueSample) -> bool {
        sample.animated(&self.visibility)
    }
}
/// Immutable resolved hierarchy, reusable for independently sampled model instances.
/// Rebuild after changing node definitions or animation channel targets. Samples must
/// use the source model identities. No animation clocks or output buffers are retained.
#[derive(Clone, Debug)]
pub struct PoseLayout {
    /// Stable graph order; parent relationships index this array.
    nodes: Vec<PoseNode>,
    /// Parent-before-child traversal indexing `nodes`.
    order: Vec<usize>,
}
/// Reusable per-instance output. Matrices remain in authored model coordinates.
#[derive(Clone, Debug, Default)]
pub struct ModelPose {
    /// Evaluated matrices and signed factors in prepared node order.
    pub nodes: Vec<EvaluatedPose>,
    /// Visibility including the complete ancestor chain.
    pub visible: Vec<bool>,
}
/// Explicit external inputs to hierarchy evaluation.
#[derive(Clone, Copy, Default)]
pub struct PoseInputs<'a> {
    /// Driving camera in authored model coordinates.
    pub camera: Option<Affine3A>,
    /// Complete local replacements, applied after authored animation.
    pub overrides: Option<&'a HashMap<NodeId, ModelTransform>>,
}
impl PoseLayout {
    /// Prepared nodes in stable identity order.
    pub fn nodes(&self) -> &[PoseNode] {
        &self.nodes
    }
    /// Parent-before-child indices into `nodes`.
    pub fn order(&self) -> &[usize] {
        &self.order
    }
    /// Transfers prepared definitions to a renderer without rebuilding them.
    pub fn into_parts(self) -> (Vec<PoseNode>, Vec<usize>) {
        (self.nodes, self.order)
    }
    /// Finds a stable node identity in prepared order.
    pub fn index(&self, id: NodeId) -> Option<usize> {
        self.nodes.iter().position(|node| node.id == id)
    }

    /// Validates resolved parent indices and computes a reusable traversal.
    pub fn new(nodes: Vec<PoseNode>) -> Result<Self, Error> {
        let mut order = Vec::with_capacity(nodes.len());
        let mut children = vec![Vec::new(); nodes.len()];
        for (index, node) in nodes.iter().enumerate() {
            if let Some(parent) = node.parent {
                let target = children
                    .get_mut(parent)
                    .ok_or_else(|| Error::new("pose.parent", "parent index is absent"))?;
                target.push(index);
            } else {
                order.push(index);
            }
        }
        let mut cursor = 0;
        while cursor < order.len() {
            order.extend(&children[order[cursor]]);
            cursor += 1;
        }
        if order.len() != nodes.len() {
            return Err(Error::new("pose", "model hierarchy contains a cycle"));
        }
        let mut corrected = vec![false; nodes.len()];
        for &index in &order {
            let node = &nodes[index];
            corrected[index] =
                node.correction.is_some() || node.parent.is_some_and(|parent| corrected[parent]);
            if corrected[index]
                && (!node.behavior().hierarchical()
                    || node.transform_semantics != TransformSemantics::Affine)
            {
                return Err(Error::new("pose.correction", "corrected hierarchies require ordinary affine inheritance without billboards or camera anchoring"));
            }
            if node.correction.is_some_and(|matrix| {
                !matrix.is_finite()
                    || matrix.x_axis.w != 0.
                    || matrix.y_axis.w != 0.
                    || matrix.z_axis.w != 0.
                    || matrix.w_axis.w != 1.
            }) {
                return Err(Error::new(
                    "pose.correction",
                    "correction must be a finite affine matrix",
                ));
            }
        }
        Ok(Self { nodes, order })
    }
    /// Samples a hierarchy into a new output. Use `evaluate_into` for repeated playback.
    pub fn evaluate(
        &self,
        sample: &impl ValueSample,
        inputs: PoseInputs<'_>,
    ) -> Result<ModelPose, Error> {
        let mut output = ModelPose::default();
        self.evaluate_into(sample, inputs, &mut output)?;
        Ok(output)
    }
    /// Reuses output capacity; evaluation allocates nothing once buffers are sized.
    /// Missing driving cameras leave camera-dependent adjustments unapplied.
    pub fn evaluate_into(
        &self,
        sample: &impl ValueSample,
        inputs: PoseInputs<'_>,
        output: &mut ModelPose,
    ) -> Result<(), Error> {
        if inputs.camera.is_some_and(|camera| {
            !camera.is_finite() || camera.matrix3.determinant().abs() <= 1e-12
        }) {
            return Err(Error::new(
                "pose.camera",
                "camera must be finite and nonsingular",
            ));
        }
        output
            .nodes
            .resize(self.nodes.len(), EvaluatedPose::default());
        output.visible.resize(self.nodes.len(), true);
        for &index in &self.order {
            let node = &self.nodes[index];
            let local = inputs
                .overrides
                .and_then(|v| v.get(&node.id))
                .copied()
                .unwrap_or_else(|| node.sample(sample));
            let parent = node
                .parent
                .map_or_else(EvaluatedPose::default, |p| output.nodes[p]);
            output.nodes[index] = node.evaluate(local, parent, inputs.camera);
            output.visible[index] =
                node.sample_visibility(sample) && node.parent.is_none_or(|p| output.visible[p]);
        }
        Ok(())
    }
}
impl Model<'_> {
    /// Prepares hierarchy relationships and channel-presence masks once for repeated evaluation.
    pub fn prepare_pose(&self) -> Result<PoseLayout, Error> {
        let targets: HashSet<_> = self.tracks().map(|channel| channel.target()).collect();
        let indices: HashMap<_, _> = self
            .nodes()
            .iter()
            .enumerate()
            .map(|(index, (id, _))| (id, index))
            .collect();
        PoseLayout::new(
            self.nodes()
                .iter()
                .map(|(id, node)| PoseNode {
                    id,
                    name: node.name.to_string(),
                    parent: node.parent.map(|id| indices[&id]),
                    rest: node.transform.defaults(self.values()),
                    transform: node.transform,
                    visibility: node.visible,
                    channels: NodeChannels {
                        translation: targets.contains(&node.transform.translation.slot()),
                        rotation: targets.contains(&node.transform.rotation.slot()),
                        scale: targets.contains(&node.transform.scale.slot()),
                        visibility: targets.contains(&node.visible.slot()),
                    },
                    inheritance: node.inheritance,
                    billboard: node.billboard,
                    transform_semantics: node.transform_semantics,
                    visible: *self.values().get(node.visible).expect("validated slot"),
                    camera_anchored: node.camera_anchored,
                    correction: None,
                })
                .collect(),
        )
    }
}

impl PoseLayout {
    /// Builds compact joint deformation matrices in an explicit output frame.
    /// `palette` selects indices in the skin's source joint list. Prepare and retain
    /// it through `ProjectedGeometry::skin_weights`; inverse-bind conventions are preserved.
    pub fn joint_matrices_into(
        &self,
        skin: &Skin<'_>,
        pose: &ModelPose,
        palette: &[usize],
        frame: Mat4,
        output: &mut Vec<Mat4>,
    ) -> Result<(), Error> {
        if pose.nodes.len() != self.nodes.len() || !frame.is_finite() {
            return Err(Error::new(
                "pose.skin",
                "inconsistent pose or nonfinite output frame",
            ));
        }
        output.clear();
        let binds = skin.inverse_bind();
        for &index in palette {
            let joint = skin
                .joints
                .get(index)
                .ok_or_else(|| Error::new("pose.skin", "joint palette index is absent"))?;
            let node = self
                .index(*joint)
                .ok_or_else(|| Error::new("pose.skin", "joint belongs to another hierarchy"))?;
            let bind = binds
                .get(index)
                .ok_or_else(|| Error::new("pose.skin", "inverse bind is absent"))?;
            output.push(frame * pose.nodes[node].matrix * Mat4::from_cols_array_2d(&bind));
        }
        Ok(())
    }
}
