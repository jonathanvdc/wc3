//! Conversion requirements and preserved semantic or native failures.
use crate::model::error::Location;
use crate::model::Error as SemanticError;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;

/// An unrepresentable native requirement or a preserved checked-operation failure.
///
/// Native conversion policy leaves remain descriptive. Existing semantic and
/// codec failures retain their concrete types and error sources.
#[derive(Clone, Debug)]
pub enum Error {
    /// A native-layout or explicitly selected conversion-policy requirement.
    Requirement {
        /// Actual field, owner and index components.
        location: Location,
        /// Requirement with no underlying error value.
        requirement: String,
    },
    /// A semantic operation failed with its domain reason and structured location.
    Semantic(SemanticError),
    /// A native codec or other checked conversion operation failed.
    Source {
        /// Conversion context surrounding the original cause.
        location: Location,
        /// Original concrete failure, shared when the diagnostic is cloned.
        source: Arc<dyn StdError + Send + Sync>,
    },
}
impl Error {
    /// Describes an authored native or policy constraint without an existing cause.
    pub fn requirement(location: Location, requirement: impl Into<String>) -> Self {
        Self::Requirement {
            location,
            requirement: requirement.into(),
        }
    }
    /// Preserves a checked operation's original error without formatting it.
    pub fn from_source(location: Location, source: impl StdError + Send + Sync + 'static) -> Self {
        Self::Source {
            location,
            source: Arc::new(source),
        }
    }
    /// Inspects conversion context without parsing a displayed path.
    pub fn location(&self) -> &Location {
        match self {
            Self::Requirement { location, .. } | Self::Source { location, .. } => location,
            Self::Semantic(source) => source.location(),
        }
    }
    /// Adds enclosing conversion context while preserving the original failure.
    pub fn with_context(self, context: Location) -> Self {
        match self {
            Self::Requirement {
                location,
                requirement,
            } => Self::Requirement {
                location: location.prefixed_by(context),
                requirement,
            },
            Self::Source { location, source } => Self::Source {
                location: location.prefixed_by(context),
                source,
            },
            Self::Semantic(source) => Self::Semantic(source.with_context(context)),
        }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Self::Semantic(source) = self {
            return write!(f, "conversion failed: {source}");
        }
        if self.location().segments().is_empty() {
            f.write_str("conversion failed: ")?;
        } else {
            write!(f, "conversion failed at {}: ", self.location())?;
        }
        match self {
            Self::Requirement { requirement, .. } => f.write_str(requirement),
            Self::Source { source, .. } => Display::fmt(source, f),
            Self::Semantic(_) => unreachable!(),
        }
    }
}
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Requirement { .. } => None,
            Self::Semantic(source) => Some(source),
            Self::Source { source, .. } => Some(source.as_ref()),
        }
    }
}
impl From<SemanticError> for Error {
    fn from(source: SemanticError) -> Self {
        Self::Semantic(source)
    }
}
