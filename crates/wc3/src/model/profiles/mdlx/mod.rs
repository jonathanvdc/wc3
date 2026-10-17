//! Warcraft III authoring semantics, independent of a particular MDX dialect.
/// Typed animated destinations for Warcraft III features and material layers.
pub mod properties;
pub use crate::model::values::layered::*;
mod links;
mod scene;
mod validation;
pub use crate::model::controls::layered::*;
pub use crate::model::materials::mdlx::*;
/// Warcraft III selects animated layered shader passes.
pub type Material<'a> = LayeredSurface<'a, Layer>;
pub use scene::*;

mod vocabulary;
pub use vocabulary::Mdlx;

pub use crate::model::geometry::layered::*;

/// Image dependency declarations admitted by this profile.
pub use crate::model::resources::MdlxImageResource as ImageResource;
