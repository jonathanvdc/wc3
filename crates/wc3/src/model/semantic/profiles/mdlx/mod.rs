//! Warcraft III authoring semantics, independent of a particular MDX dialect.
/// Typed animated destinations for Warcraft III features and material layers.
pub mod properties;
pub use crate::model::semantic::values::layered::*;
mod links;
mod scene;
mod validation;
pub use crate::model::semantic::controls::layered::*;
pub use crate::model::semantic::materials::layered::*;
/// Warcraft III selects animated layered shader passes.
pub type Material<'a> = LayeredSurface<'a, Layer>;
pub use scene::*;

mod vocabulary;
pub use vocabulary::Mdlx;

pub use crate::model::semantic::geometry::layered::*;
