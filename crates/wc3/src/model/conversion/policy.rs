use super::super::semantic::LookId;

/// Handling of data without a destination representation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LossPolicy {
    /// Reject semantic loss.
    #[default]
    Reject,
    /// Permit and report discarded data.
    Drop,
}
/// Handling of source blocks whose semantics are unknown.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UnknownPolicy {
    /// Reject unknown blocks.
    #[default]
    Reject,
    /// Discard and report unknown blocks.
    Drop,
}
/// Whether explicitly identified approximations are allowed.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ApproximationPolicy {
    /// Require exact representability.
    #[default]
    Reject,
    /// Allow approximations within the stated absolute tolerance.
    Allow {
        /// Maximum absolute component error.
        tolerance: f32,
    },
}
/// Native import policies.
#[derive(Clone, Debug, Default)]
pub struct ImportOptions {
    /// Unsupported semantic feature policy.
    pub loss_policy: LossPolicy,
    /// Unknown source payload policy.
    pub unknown: UnknownPolicy,
    /// Numeric approximation policy.
    pub approximation: ApproximationPolicy,
}
/// Explicit logical model, authored look, and native output selection.
#[derive(Clone, Debug, Default)]
pub struct ExportOptions {
    /// Model index in the imported logical asset.
    pub model: u32,
    /// Authored material look; absent selects the model default.
    pub look: Option<LookId>,
    /// Unsupported semantic feature policy.
    pub loss_policy: LossPolicy,
    /// Numeric approximation policy.
    pub approximation: ApproximationPolicy,
}
