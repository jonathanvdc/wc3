//! Owned, compact projections over the same immutable numeric evaluator.
use super::{ImmutableParticleView, ImmutableRecipe};
use crate::model::scene::popcorn::age::{AgePlan, Expression, Node, NodeId, Output};
use crate::model::scene::popcorn::{EffectFieldId, Value};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::Arc;

/// Preparation or sampling failure. Output buffers remain unchanged on error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImmutableProjectionError {
    /// The requested field is not a virtual output of this recipe.
    UnknownOutput(#[doc = "Requested virtual output field."] EffectFieldId),
    /// The supplied recipe is neither this projection's source nor its own
    /// projected recipe, even if its fields agree.
    RecipeMismatch,
    /// Exactly one output element is required per requested field.
    OutputLength {
        /// Number of requested results.
        expected: usize,
        /// Supplied output length.
        actual: usize,
    },
    /// Reuse a larger scratch buffer or allocate one for this projection.
    ScratchTooSmall {
        /// Required dependency values.
        needed_nodes: usize,
        /// Available dependency values.
        available_nodes: usize,
        /// Required capture values.
        needed_captures: usize,
        /// Available capture values.
        available_captures: usize,
    },
    /// The shared checked evaluator rejected the observation.
    Evaluation {
        /// Original instruction byte offset, when available.
        offset: Option<usize>,
        /// Checked evaluator diagnostic.
        message: &'static str,
    },
}
impl Display for ImmutableProjectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::UnknownOutput(field) => write!(f, "field {} is not an immutable output", field.0),
            Self::RecipeMismatch => f.write_str("immutable projection source or projected recipe does not match"),
            Self::OutputLength { expected, actual } => write!(f, "immutable projection needs {expected} outputs, received {actual}"),
            Self::ScratchTooSmall { needed_nodes, available_nodes, needed_captures, available_captures } => write!(f, "immutable projection needs {needed_nodes} nodes/{needed_captures} captures, scratch has {available_nodes}/{available_captures}"),
            Self::Evaluation { offset: Some(offset), message } => write!(f, "{message} at byte {offset}"),
            Self::Evaluation { offset: None, message } => f.write_str(message),
        }
    }
}
impl Error for ImmutableProjectionError {}

/// Reusable initialized evaluation storage. Preparation allocates; sampling does not.
#[derive(Debug)]
pub struct ImmutableProjectionScratch {
    nodes: Vec<Value>,
    captures: Vec<Value>,
}
impl ImmutableProjectionScratch {
    /// Number of node values available for this or a smaller projection.
    pub fn node_capacity(&self) -> usize {
        self.nodes.len()
    }
    /// Number of capture values available for this or a smaller projection.
    pub fn capture_capacity(&self) -> usize {
        self.captures.len()
    }
}

/// A compact multi-output recipe retaining its exact prepared source identity.
/// Clones share immutable preparation. Requested order and duplicates are preserved.
#[derive(Clone, Debug)]
pub struct ImmutableProjection {
    source: Arc<AgePlan>,
    plan: Arc<AgePlan>,
}
impl ImmutableRecipe<'_> {
    /// Prepare only dependencies of these virtual outputs. Empty requests are valid.
    /// Raw canonical fields can instead be read with [`super::super::ParticleView::sample_fields`].
    pub fn project(
        &self,
        fields: &[EffectFieldId],
    ) -> Result<ImmutableProjection, ImmutableProjectionError> {
        let mut requested = Vec::with_capacity(fields.len());
        let mut live = vec![false; self.plan.nodes.len()];
        for &field in fields {
            let output = self
                .plan
                .outputs
                .iter()
                .find(|output| output.field == field.0 as usize)
                .ok_or(ImmutableProjectionError::UnknownOutput(field))?;
            requested.push(*output);
            live[output.value.0] = true;
        }
        // All dependencies precede their users in the checked recipe.
        for index in (0..self.plan.nodes.len()).rev() {
            if !live[index] {
                continue;
            }
            match self.plan.nodes[index].expression {
                Expression::Words { input, .. } | Expression::Curve { input, .. } => {
                    live[input.0] = true
                }
                Expression::Binary { left, right, .. } => {
                    live[left.0] = true;
                    live[right.0] = true;
                }
                _ => {}
            }
        }
        let mut nodes = Vec::new();
        let mut captures = Vec::new();
        let mut node_ids: Vec<Option<NodeId>> = vec![None; self.plan.nodes.len()];
        let mut capture_ids = vec![None; self.plan.captures.len()];
        for (index, &node) in self.plan.nodes.iter().enumerate() {
            if !live[index] {
                continue;
            }
            let remap = |id: NodeId| node_ids[id.0].expect("topological projection dependency");
            let expression = match node.expression {
                Expression::Capture(index) => {
                    let capture = *capture_ids[index].get_or_insert_with(|| {
                        let id = captures.len();
                        captures.push(self.plan.captures[index]);
                        id
                    });
                    Expression::Capture(capture)
                }
                Expression::Words { input, kind } => Expression::Words {
                    input: remap(input),
                    kind,
                },
                Expression::Binary {
                    operation,
                    left,
                    right,
                } => Expression::Binary {
                    operation,
                    left: remap(left),
                    right: remap(right),
                },
                Expression::Curve { sampler, input } => Expression::Curve {
                    sampler,
                    input: remap(input),
                },
                expression => expression,
            };
            node_ids[index] = Some(NodeId(nodes.len()));
            nodes.push(Node { expression, ..node });
        }
        let outputs = requested
            .into_iter()
            .map(|output| Output {
                field: output.field,
                value: node_ids[output.value.0].expect("selected projection output"),
            })
            .collect();
        let integrated_delta = nodes
            .iter()
            .any(|node| matches!(node.expression, Expression::IntegratedDelta));
        Ok(ImmutableProjection {
            source: Arc::clone(self.plan),
            plan: Arc::new(AgePlan {
                nodes: nodes.into_boxed_slice(),
                captures: captures.into_boxed_slice(),
                outputs,
                phases: self.plan.phases,
                integrated_delta,
            }),
        })
    }
}
impl ImmutableProjection {
    /// Borrow the compact recipe, retaining curve resources from the exact
    /// prepared source. The source and this projection must both outlive it.
    pub fn recipe<'a>(
        &'a self,
        source: ImmutableRecipe<'a>,
    ) -> Result<ImmutableRecipe<'a>, ImmutableProjectionError> {
        if !Arc::ptr_eq(&self.source, source.plan) {
            return Err(ImmutableProjectionError::RecipeMismatch);
        }
        Ok(ImmutableRecipe {
            plan: &self.plan,
            samplers: source.samplers,
        })
    }
    /// Borrow this projection's compact captures and recipe for an authorized
    /// source particle. Slot, birth anchor and observation remain unchanged.
    pub fn view<'a>(
        &'a self,
        source: ImmutableParticleView<'a>,
    ) -> Result<ImmutableParticleView<'a>, ImmutableProjectionError> {
        Ok(ImmutableParticleView {
            recipe: self.recipe(source.recipe)?,
            ..source
        })
    }
    /// Number of retained dependency nodes.
    pub fn node_count(&self) -> usize {
        self.plan.nodes.len()
    }
    /// Number of distinct initialized captures required.
    pub fn capture_count(&self) -> usize {
        self.plan.captures.len()
    }
    /// Number of requested outputs, including duplicates.
    pub fn output_count(&self) -> usize {
        self.plan.outputs.len()
    }
    /// Requested field IDs in caller order, including duplicates.
    pub fn fields(&self) -> impl ExactSizeIterator<Item = EffectFieldId> + '_ {
        self.plan
            .outputs
            .iter()
            .map(|output| EffectFieldId(output.field as u32))
    }
    /// Allocate once; scratch may also serve another projection that fits it.
    pub fn new_scratch(&self) -> ImmutableProjectionScratch {
        ImmutableProjectionScratch {
            nodes: vec![Value::from(0f32); self.node_count()],
            captures: vec![Value::from(0f32); self.capture_count()],
        }
    }
    /// Evaluate each retained dependency once, without allocation or partial outputs.
    /// Accepts the original source view or a view made by this projection.
    pub fn evaluate(
        &self,
        particle: ImmutableParticleView<'_>,
        scratch: &mut ImmutableProjectionScratch,
        outputs: &mut [Value],
    ) -> Result<(), ImmutableProjectionError> {
        if !Arc::ptr_eq(&self.source, particle.recipe.plan)
            && !Arc::ptr_eq(&self.plan, particle.recipe.plan)
        {
            return Err(ImmutableProjectionError::RecipeMismatch);
        }
        if outputs.len() != self.output_count() {
            return Err(ImmutableProjectionError::OutputLength {
                expected: self.output_count(),
                actual: outputs.len(),
            });
        }
        if scratch.nodes.len() < self.node_count() || scratch.captures.len() < self.capture_count()
        {
            return Err(ImmutableProjectionError::ScratchTooSmall {
                needed_nodes: self.node_count(),
                available_nodes: scratch.nodes.len(),
                needed_captures: self.capture_count(),
                available_captures: scratch.captures.len(),
            });
        }
        particle
            .evaluate_plan(
                &self.plan,
                &mut scratch.captures[..self.capture_count()],
                &mut scratch.nodes[..self.node_count()],
                outputs,
            )
            .map_err(|error| ImmutableProjectionError::Evaluation {
                offset: error.offset,
                message: error.message,
            })
    }
}

#[cfg(test)]
#[path = "projection_tests.rs"]
mod tests;
