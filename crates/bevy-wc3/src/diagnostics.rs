//! Structured renderer coverage and resource diagnostics.
use wc3::model::{ClipId, FeatureId, InstanceId, MaterialId, NodeId, SystemId};

/// Why rendering differs from the authored model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RenderDiagnosticKind {
    /// An authored feature has no active renderer interpreter.
    Unsupported,
    /// Rendering uses an intentionally different or incomplete interpretation.
    Approximation,
    /// Behavior is implemented but fidelity has not been established.
    Unverified,
    /// A required or optional resource could not be used.
    Resource,
    /// An explicit population, storage, or work limit can omit authored work.
    Budget,
}

/// Default importance of a nonfatal renderer diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderDiagnosticSeverity {
    /// A known limit or validation boundary to consider when selecting quality.
    Info,
    /// Rendering omits or changes authored content.
    Warning,
}

/// Semantic identity responsible for a renderer diagnostic.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RenderDiagnosticSource {
    /// A model-wide behavior without a more specific semantic owner.
    Model,
    /// An authored hierarchy node.
    Node(NodeId),
    /// An animation clip.
    Clip(ClipId),
    /// An authored material.
    Material(MaterialId),
    /// An attached model feature.
    Feature(FeatureId),
    /// A geometry instance.
    Instance(InstanceId),
    /// An authored simulation or world system.
    System(SystemId),
    /// Accepted native import loss at the conversion report's source path.
    NativeImport {
        /// Native field path retained from the conversion report.
        path: String,
    },
}

/// A nonfatal rendering limitation with machine-readable policy and identity.
#[derive(Clone, Debug)]
pub struct RenderDiagnostic {
    /// Whether behavior is missing, approximate, unverified, resource-bound, or limited.
    pub kind: RenderDiagnosticKind,
    /// Default importance; applications can choose their own reporting policy.
    pub severity: RenderDiagnosticSeverity,
    /// Semantic owner or native import location.
    pub source: RenderDiagnosticSource,
    /// Human-readable explanation, not a stable identifier for filtering.
    pub message: String,
}

impl RenderDiagnostic {
    /// Create a diagnostic with the default severity for its kind.
    /// Unsupported features, approximations, and resource failures are warnings;
    /// fidelity boundaries and work budgets are informational.
    pub fn new(
        kind: RenderDiagnosticKind,
        source: RenderDiagnosticSource,
        message: impl Into<String>,
    ) -> Self {
        let severity = match kind {
            RenderDiagnosticKind::Unsupported
            | RenderDiagnosticKind::Approximation
            | RenderDiagnosticKind::Resource => RenderDiagnosticSeverity::Warning,
            RenderDiagnosticKind::Unverified | RenderDiagnosticKind::Budget => {
                RenderDiagnosticSeverity::Info
            }
        };
        Self {
            kind,
            severity,
            source,
            message: message.into(),
        }
    }
}

/// A compiler-local issue whose semantic owner is attached by its caller.
#[derive(Clone, Debug)]
pub(crate) struct RenderIssue {
    pub(crate) kind: RenderDiagnosticKind,
    pub(crate) message: String,
}
impl RenderIssue {
    pub(crate) fn new(kind: RenderDiagnosticKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
    pub(crate) fn at(self, source: RenderDiagnosticSource) -> RenderDiagnostic {
        RenderDiagnostic::new(self.kind, source, self.message)
    }
}
