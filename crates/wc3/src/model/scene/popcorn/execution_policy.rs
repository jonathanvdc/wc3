//! Preparation preferences and observable routes for immutable effect execution.
use super::graph::EffectLayerId;

/// Preferred execution strategy when preparing an effect programme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExecutionPreference {
    /// Prepare native execution when available; otherwise retain the Rust evaluator.
    #[default]
    Automatic,
    /// Use the checked Rust evaluator without preparing native executable code.
    /// Its existing eligible pure-batch optimization remains an implementation detail.
    Interpreter,
}

/// Numerical transformations permitted during execution preparation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NumericPolicy {
    /// Preserve the checked evaluator's operation ordering and numerical policies.
    /// A backend that cannot do so must retain interpreter execution.
    Strict,
    /// Permit graphics-oriented floating-point reassociation, contraction and
    /// equivalent analytic trajectories. These permissions do not require a
    /// backend to apply them and do not change typed values, capability ownership,
    /// ordered host effects, lifecycle, quotas or checked failure boundaries.
    #[default]
    Graphics,
}

/// Immutable preparation choices for new effect instances.
/// Population and simulation work allowances remain in `EffectLimits`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExecutionOptions {
    /// Native preparation preference, with interpreter fallback before execution.
    pub preference: ExecutionPreference,
    /// Numerical transformations permitted for this prepared execution plan.
    pub numeric_policy: NumericPolicy,
}

/// Selected lifecycle invocation within one compiled layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionStage {
    /// The layer's once-per-birth initializer.
    Initialization,
    /// The layer's selected evolution programme, not every native alternative.
    Evolution,
}

/// Prepared implementation for a layer invocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExecutionBackend {
    /// The checked Rust evaluator, including its eligible pure-batch fast path.
    Interpreter,
    /// Native scalar machine code prepared through the optional JIT compiler.
    ScalarJit,
}

/// Why automatic native preparation retained the interpreter.
/// This is a preparation result, not an execution fault or a deferred retry.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExecutionFallback {
    /// The crate was built without optional native execution support.
    FeatureDisabled,
    /// The current platform has no supported native execution implementation.
    UnsupportedPlatform,
    /// Native preparation failed before any particle invocation executed.
    CompilationFailed {
        /// Diagnostic from preparation; no native code is called for this stage.
        message: Box<str>,
    },
    /// Selected programme operations exceed the internal native preparation bound.
    BudgetExceeded {
        /// Operation count considered by the bounded preparation decision.
        operations: usize,
        /// Maximum operation count admitted for native preparation.
        limit: usize,
    },
}

/// Immutable route selected during preparation for one existing layer stage.
/// Reports describe prepared routes, not invocation counts or profiling results.
/// An explicitly requested interpreter has no fallback reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionReport {
    /// Layer in the owning effect programme.
    pub layer: EffectLayerId,
    /// Initializer or the selected evolution programme.
    pub stage: ExecutionStage,
    /// Implementation selected for this stage.
    pub backend: ExecutionBackend,
    /// Reason automatic native preparation retained interpreter execution.
    pub fallback: Option<ExecutionFallback>,
}

/// Actual generated-code entries, separate from semantic simulation statistics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExecutionStats {
    /// Scalar native invocations attempted, including invocations ending in a fault.
    pub scalar_jit_invocations: u64,
}
