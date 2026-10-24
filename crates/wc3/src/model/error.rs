//! Typed failures and locations for checked model construction, editing and evaluation.
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;
mod edit;
pub use edit::EditError;
mod location;
pub use location::{Location, Owner, Segment};
mod reference;
pub use reference::{ConstantValue, ReferenceError, ResourceVocabulary};
mod geometry;
pub use geometry::*;
mod animation;
pub use animation::AnimationError;
mod transform;
pub use transform::TransformError;
mod physics;
pub use physics::PhysicsError;
mod profile;
pub use profile::*;

/// The semantic domain and actual violated constraint.
#[derive(Clone, Debug)]
pub enum ErrorKind {
    /// A checked reference constraint.
    Reference(ReferenceError),
    /// A checked geometry constraint.
    Geometry(GeometryError),
    /// A checked animation constraint.
    Animation(AnimationError),
    /// A checked transform constraint.
    Transform(TransformError),
    /// A checked physics constraint.
    Physics(PhysicsError),
    /// A checked profile constraint.
    Profile(ProfileError),
    /// A custom semantic validator's original typed failure.
    Extension(Arc<dyn StdError + Send + Sync>),
    #[doc(hidden)]
    Migration(String),
}
impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reference(cause) => Display::fmt(cause, f),
            Self::Geometry(cause) => Display::fmt(cause, f),
            Self::Animation(cause) => Display::fmt(cause, f),
            Self::Transform(cause) => Display::fmt(cause, f),
            Self::Physics(cause) => Display::fmt(cause, f),
            Self::Profile(cause) => Display::fmt(cause, f),
            Self::Extension(cause) => Display::fmt(cause, f),
            Self::Migration(message) => f.write_str(message),
        }
    }
}
/// A semantic failure with structured context independent of its display text.
///
/// Cloning shares custom extension causes; it does not clone their arbitrary
/// application state. Built-in domain causes retain their owned diagnostic data.
#[derive(Clone, Debug)]
pub struct Error {
    location: Location,
    kind: Box<ErrorKind>,
}
impl Error {
    /// Associates an actual semantic failure with a structured location.
    pub fn at(location: Location, kind: impl Into<ErrorKind>) -> Self {
        Self {
            location,
            kind: Box::new(kind.into()),
        }
    }
    /// Retains a custom semantic validator's original typed error and source chain.
    pub fn extension(location: Location, cause: impl StdError + Send + Sync + 'static) -> Self {
        Self::at(location, ErrorKind::Extension(Arc::new(cause)))
    }
    /// Inspects the semantic cause without parsing prose.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
    /// Inspects actual owner, field, index and attribute components.
    pub fn location(&self) -> &Location {
        &self.location
    }
    /// Adds an enclosing semantic owner without replacing local evidence.
    pub fn with_owner(mut self, owner: impl Into<Owner>) -> Self {
        self.location = self.location.with_owner(owner);
        self
    }
    /// Adds enclosing structured context without replacing the cause or its source.
    pub fn with_context(mut self, context: Location) -> Self {
        self.location = self.location.prefixed_by(context);
        self
    }
    #[doc(hidden)]
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::at(
            Location::new().field(path.into()),
            ErrorKind::Migration(message.into()),
        )
    }
    /// Formats the location for presentation; use `location()` for inspection.
    pub fn path(&self) -> String {
        self.location.to_string()
    }
    /// Formats the cause for presentation; use `kind()` for inspection.
    pub fn message(&self) -> String {
        self.kind.to_string()
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.location.segments().is_empty() {
            Display::fmt(&self.kind, f)
        } else {
            write!(f, "{}: {}", self.location, self.kind)
        }
    }
}
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self.kind.as_ref() {
            ErrorKind::Reference(cause) => Some(cause),
            ErrorKind::Geometry(cause) => Some(cause),
            ErrorKind::Animation(cause) => Some(cause),
            ErrorKind::Transform(cause) => Some(cause),
            ErrorKind::Physics(cause) => Some(cause),
            ErrorKind::Profile(cause) => Some(cause),
            ErrorKind::Extension(cause) => Some(cause.as_ref()),
            ErrorKind::Migration(_) => None,
        }
    }
}
macro_rules! domain_errors {
    ($($variant:ident($ty:ty)),* $(,)?) => {$ (
        impl From<$ty> for ErrorKind { fn from(value: $ty) -> Self { Self::$variant(value) } }
        impl From<$ty> for Error { fn from(value: $ty) -> Self { Self::at(Location::new(), value) } }
    )*};
}
domain_errors! {
    Reference(ReferenceError),
    Geometry(GeometryError),
    Animation(AnimationError),
    Transform(TransformError),
    Physics(PhysicsError),
    Profile(ProfileError),
}
