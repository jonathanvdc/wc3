//! Read-only prepared age expressions and generation-scoped birth captures.
use super::age::{
    AgePlan, AgeSample, Expression, NodeId as AgeNodeId, MAX_CAPTURES, MAX_NODES, MAX_OUTPUTS,
};
use super::graph::{EffectFieldId, EffectSampler};
use super::runtime::FieldPlanes;
use super::{BinaryOperation, Value, ValueType};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::ptr::eq;

/// One topologically ordered expression in an immutable recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImmutableNodeId(usize);
impl ImmutableNodeId {
    /// Zero-based index into the owning recipe's nodes.
    pub fn index(self) -> usize {
        self.0
    }
}
/// A closed numeric expression, independent of native bytecode and VM registers.
#[derive(Clone, Copy, Debug)]
pub enum ImmutableExpression {
    /// A typed constant.
    Constant(Value),
    /// An index in the owning recipe's birth captures.
    Capture(usize),
    /// Graphics appearance life, distinct from authoritative accumulated lifetime.
    Life,
    /// Authorized effect-local time, converted to f32.
    Time,
    /// Preserve component words with another checked layout.
    Words {
        /// Earlier input node.
        input: ImmutableNodeId,
        /// Result layout.
        kind: ValueType,
    },
    /// Componentwise addition with scalar broadcast.
    Add {
        /// Earlier left input.
        left: ImmutableNodeId,
        /// Earlier right input.
        right: ImmutableNodeId,
    },
    /// Componentwise subtraction with scalar broadcast.
    Subtract {
        /// Earlier left input.
        left: ImmutableNodeId,
        /// Earlier right input.
        right: ImmutableNodeId,
    },
    /// Componentwise multiplication with scalar broadcast.
    Multiply {
        /// Earlier left input.
        left: ImmutableNodeId,
        /// Earlier right input.
        right: ImmutableNodeId,
    },
    /// Immutable checked curve sampling.
    Curve {
        /// Curve identifier resolved within the owning recipe.
        sampler: usize,
        /// Earlier scalar coordinate.
        input: ImmutableNodeId,
    },
}
/// A typed node with its original native instruction location.
#[derive(Clone, Copy, Debug)]
pub struct ImmutableNode {
    /// Numeric operation and dependencies.
    pub expression: ImmutableExpression,
    /// Output layout.
    pub kind: ValueType,
    /// Original instruction byte location.
    pub offset: usize,
}
/// A canonical initialized field required by a recipe.
#[derive(Clone, Copy, Debug)]
pub struct ImmutableCapture {
    /// Initialized canonical field.
    pub field: EffectFieldId,
    /// Checked value layout.
    pub kind: ValueType,
}
/// One virtual canonical field written by a recipe.
#[derive(Clone, Copy, Debug)]
pub struct ImmutableOutput {
    /// Canonical output field.
    pub field: EffectFieldId,
    /// Final expression node.
    pub value: ImmutableNodeId,
}
/// Shared immutable curve data; tangent pairs retain their authored convention.
#[derive(Clone, Copy, Debug)]
pub struct ImmutableCurve<'a> {
    /// Sampled output layout.
    pub kind: ValueType,
    /// Strictly increasing key times.
    pub times: &'a [f32],
    /// Four words per key; inactive components are ignored.
    pub values: &'a [[f32; 4]],
    /// Empty for linear curves; otherwise incoming/outgoing pairs per key.
    pub tangents: &'a [[f32; 4]],
}
/// Borrowed prepared expressions; clones of one programme share recipe identity.
#[derive(Clone, Copy)]
pub struct ImmutableRecipe<'a> {
    pub(super) plan: &'a AgePlan,
    pub(super) samplers: &'a [EffectSampler],
}
impl ImmutableRecipe<'_> {
    /// Whether two views refer to the exact same immutable prepared recipe.
    pub fn same_recipe(&self, other: &Self) -> bool {
        eq(self.plan, other.plan)
    }
    /// Number of certified constant-rate phase recurrences.
    pub fn phase_count(&self) -> usize {
        self.plan.phases
    }
    /// Number of bounded topologically ordered nodes.
    pub fn node_count(&self) -> usize {
        self.plan.nodes.len()
    }
    /// Inspect one checked node without decoding native selectors.
    pub fn node(&self, index: usize) -> Option<ImmutableNode> {
        let node = self.plan.nodes.get(index)?;
        let id = |value: AgeNodeId| ImmutableNodeId(value.0);
        let expression = match node.expression {
            Expression::Constant(value) => ImmutableExpression::Constant(value),
            Expression::Capture(index) => ImmutableExpression::Capture(index),
            Expression::Life => ImmutableExpression::Life,
            Expression::Time => ImmutableExpression::Time,
            Expression::Words { input, kind } => ImmutableExpression::Words {
                input: id(input),
                kind,
            },
            Expression::Binary {
                operation,
                left,
                right,
                ..
            } => match operation {
                BinaryOperation::Add => ImmutableExpression::Add {
                    left: id(left),
                    right: id(right),
                },
                BinaryOperation::Subtract => ImmutableExpression::Subtract {
                    left: id(left),
                    right: id(right),
                },
                BinaryOperation::Multiply => ImmutableExpression::Multiply {
                    left: id(left),
                    right: id(right),
                },
                _ => unreachable!("checked age arithmetic"),
            },
            Expression::Curve { sampler, input } => ImmutableExpression::Curve {
                sampler,
                input: id(input),
            },
        };
        Some(ImmutableNode {
            expression,
            kind: node.kind,
            offset: node.offset,
        })
    }
    /// Number of initialized canonical field captures.
    pub fn capture_count(&self) -> usize {
        self.plan.captures.len()
    }
    /// Inspect one capture's canonical field and layout.
    pub fn capture(&self, index: usize) -> Option<ImmutableCapture> {
        self.plan.captures.get(index).map(|c| ImmutableCapture {
            field: EffectFieldId(c.field as u32),
            kind: c.kind,
        })
    }
    /// Number of virtual canonical outputs.
    pub fn output_count(&self) -> usize {
        self.plan.outputs.len()
    }
    /// Inspect one virtual output binding.
    pub fn output(&self, index: usize) -> Option<ImmutableOutput> {
        self.plan.outputs.get(index).map(|o| ImmutableOutput {
            field: EffectFieldId(o.field as u32),
            value: ImmutableNodeId(o.value.0),
        })
    }
    /// Inspect an immutable curve referenced by a Curve node.
    pub fn curve(&self, index: usize) -> Option<ImmutableCurve<'_>> {
        let EffectSampler::Curve {
            kind,
            times,
            values,
            tangents,
        } = self.samplers.get(index)?
        else {
            return None;
        };
        Some(ImmutableCurve {
            kind: *kind,
            times,
            values,
            tangents,
        })
    }
}
/// Captured lifecycle anchor for Graphics-only appearance expressions.
#[derive(Clone, Copy, Debug, Default)]
pub struct ImmutableBirth {
    /// Effect-local birth anchor converted to f32.
    pub time: f32,
    /// Initialized life ratio; authoritative CPU lifetime remains independent.
    pub life: f32,
    /// Initialized finite inverse lifetime.
    pub inverse_life: f32,
}
impl ImmutableBirth {
    pub(super) fn sample(self, time: f64) -> ImmutableSample {
        ImmutableSample {
            effect_time: time,
            life: (self.life + (time as f32 - self.time) * self.inverse_life).min(1.),
        }
    }
}
/// Last authorized appearance observation; denied work never advances it.
#[derive(Clone, Copy, Debug, Default)]
pub struct ImmutableSample {
    /// Authorized effect-local seconds.
    pub effect_time: f64,
    /// Analytic Graphics appearance life; reading canonical Life uses CPU lifetime.
    pub life: f32,
}
impl From<ImmutableSample> for AgeSample {
    fn from(value: ImmutableSample) -> Self {
        Self {
            effect_time: value.effect_time,
            life: value.life,
        }
    }
}
/// Borrowed birth captures and authorized sample for one living slot incarnation.
#[derive(Clone, Copy)]
pub struct ImmutableParticleView<'a> {
    pub(super) recipe: ImmutableRecipe<'a>,
    pub(super) fields: &'a FieldPlanes,
    pub(super) slot: usize,
    pub(super) birth: ImmutableBirth,
    pub(super) sample: ImmutableSample,
}
impl<'a> ImmutableParticleView<'a> {
    /// Prepared recipe shared by this view and the fixed instance plan.
    pub fn recipe(&self) -> ImmutableRecipe<'a> {
        self.recipe
    }
    /// Read a raw initialized capture, without evaluating any appearance nodes.
    pub fn capture(&self, index: usize) -> Option<Value> {
        let capture = self.recipe.plan.captures.get(index)?;
        Some(Value::from_words(
            capture.kind,
            self.fields.read(capture.field, self.slot),
        ))
    }
    /// Initialized appearance-life anchor.
    pub fn birth(&self) -> ImmutableBirth {
        self.birth
    }
    /// Last authorized observation, including frozen observations after denied work.
    pub fn sample(&self) -> ImmutableSample {
        self.sample
    }
    #[inline(never)]
    pub(super) fn evaluate(&self, outputs: &mut [Value]) {
        let zero = Value::from(0f32);
        let mut captures = [zero; MAX_CAPTURES];
        for (index, capture) in captures
            .iter_mut()
            .take(self.recipe.capture_count())
            .enumerate()
        {
            *capture = self.capture(index).expect("prepared immutable capture");
        }
        let mut nodes = [zero; MAX_NODES];
        self.recipe
            .plan
            .evaluate(
                &captures[..self.recipe.capture_count()],
                self.recipe.samplers,
                self.sample.into(),
                &mut nodes[..self.recipe.node_count()],
                outputs,
            )
            .unwrap_or_else(|error| panic!("authorized immutable recipe must be total: {error}"));
    }
    #[inline(never)]
    pub(super) fn field(&self, field: EffectFieldId) -> Option<Value> {
        let index = self
            .recipe
            .plan
            .outputs
            .iter()
            .position(|o| o.field == field.0 as usize)?;
        let mut outputs = [Value::from(0f32); MAX_OUTPUTS];
        self.evaluate(&mut outputs[..self.recipe.output_count()]);
        Some(outputs[index])
    }
}

impl Debug for ImmutableRecipe<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ImmutableRecipe")
            .field("nodes", &self.node_count())
            .field("captures", &self.capture_count())
            .field("outputs", &self.output_count())
            .finish_non_exhaustive()
    }
}
impl Debug for ImmutableParticleView<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ImmutableParticleView")
            .field("slot", &self.slot)
            .field("birth", &self.birth)
            .field("sample", &self.sample)
            .finish_non_exhaustive()
    }
}
