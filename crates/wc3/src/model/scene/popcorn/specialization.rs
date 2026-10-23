//! Preparation eligibility, independent of selected backends and runtime work.
use super::{EffectFieldId, EffectLayerId, Instruction, ValueType};

/// Result of one semantic specialization analysis during preparation.
/// Eligibility does not promise SIMD invocation or GPU rendering at runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecializationStatus {
    /// The analyzer constructed a checked specialization plan.
    Accepted,
    /// This execution policy did not request the analysis.
    NotExamined,
    /// The layer has no selected evolution programme.
    NotApplicable,
    /// The selected programme is outside the current specialization subset.
    Rejected(SpecializationRejection),
}

/// First rejection in deterministic analysis order, not an execution fault.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpecializationRejection {
    /// Unsupported capability or preparation bound.
    pub reason: SpecializationReason,
    /// Original instruction byte offset, when an instruction caused rejection.
    pub offset: Option<usize>,
    /// Canonical field within the reported layer, when relevant.
    pub field: Option<EffectFieldId>,
}
impl SpecializationRejection {
    pub(super) fn new(reason: SpecializationReason) -> Self {
        Self {
            reason,
            offset: None,
            field: None,
        }
    }
    pub(super) fn at(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
    pub(super) fn field(mut self, field: usize) -> Self {
        self.field = u32::try_from(field).ok().map(EffectFieldId);
        self
    }
}

/// Semantic reason a particular specialization cannot prepare a programme.
/// Native feature/platform/code-generation failures use `ExecutionFallback`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SpecializationReason {
    /// Immutable expressions require the Graphics numeric policy.
    NumericPolicy,
    /// The layer spawns other particles and requires ordered CPU control.
    Spawner,
    /// The layer has no renderer consuming its outputs.
    NotRendered,
    /// The layer requests evolution on its death interval.
    EvolutionOnDeath,
    /// A checked value layout is outside this specialization subset.
    Layout(ValueType),
    /// A checked instruction is outside this specialization subset.
    Instruction(SpecializationInstruction),
    /// The specialization cannot directly bind this external declaration.
    External(u16),
    /// A source store modifies canonical lifecycle state.
    LifecycleWrite,
    /// A persistent register is read before being defined in this invocation.
    PersistentInput {
        /// Register scope within the checked programme.
        scope: usize,
        /// Register slot within that scope.
        slot: usize,
    },
    /// A written field has no certified closed recurrence.
    Feedback,
    /// A time reference is outside a certified clock recurrence.
    UncertifiedTime,
    /// A curve coordinate lacks the required finite-domain certificate.
    CurveCoordinate {
        /// Immutable sampler ordinal in the prepared effect.
        sampler: usize,
    },
    /// A curve resource is outside the supported immutable layout.
    CurveResource {
        /// Immutable sampler ordinal in the prepared effect.
        sampler: usize,
    },
    /// A checked binding is absent or inconsistent with its value layout.
    InvalidBinding,
    /// A bounded preparation resource was exceeded.
    Limit {
        /// Resource whose bound was reached.
        resource: SpecializationResource,
        /// Maximum admitted quantity.
        limit: usize,
    },
    /// No canonical output is written by the selected evolution.
    NoOutputs,
}

/// Checked instruction category used in specialization diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecializationInstruction {
    /// External read.
    Load,
    /// External write.
    Store,
    /// Unary or binary numerical operation.
    Arithmetic,
    /// Bit-preserving layout reinterpretation.
    Reinterpret,
    /// Numeric value conversion.
    Convert,
    /// Vector construction.
    Construct,
    /// Component swizzle.
    Swizzle,
    /// Conditional value selection.
    Select,
    /// Host callback invocation.
    Call,
}
impl From<&Instruction> for SpecializationInstruction {
    fn from(instruction: &Instruction) -> Self {
        match instruction {
            Instruction::Load { .. } => Self::Load,
            Instruction::Store { .. } => Self::Store,
            Instruction::Binary { .. } | Instruction::Unary { .. } => Self::Arithmetic,
            Instruction::Reinterpret { .. } => Self::Reinterpret,
            Instruction::Convert { .. } => Self::Convert,
            Instruction::Construct { .. } => Self::Construct,
            Instruction::Swizzle { .. } => Self::Swizzle,
            Instruction::Select { .. } => Self::Select,
            Instruction::Call { .. } => Self::Call,
        }
    }
}

/// Bounded resource used while constructing specialization plans.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecializationResource {
    /// Symbolic or closed expression nodes.
    Nodes,
    /// Initialized canonical field captures.
    Captures,
    /// Canonical output bindings.
    Outputs,
    /// Checked register address space.
    RegisterAddress,
}

/// Owned eligibility snapshot of one selected layer evolution.
/// Queries neither allocate nor repeat analysis. Runtime counters remain separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpecializationReport {
    /// Layer in the owning immutable effect graph.
    pub layer: EffectLayerId,
    /// Whole-program pure-packet eligibility, independent of tile code generation.
    pub pure_packet: SpecializationStatus,
    /// Closed-expression eligibility under this wrapper's preparation choices.
    pub immutable_expressions: SpecializationStatus,
}

#[cfg(test)]
#[path = "specialization_tests.rs"]
mod tests;
