//! Warcraft III authoring semantics, independent of a particular MDX dialect.
/// Typed animated destinations for Warcraft III features and material layers.
mod links;
mod scene;
mod validation;
pub use crate::model::materials::mdlx::*;
use crate::model::materials::program::MaterialProgram;
/// Warcraft III selects an ordered program of typed shader passes.
pub type Material<'a> = MaterialProgram<MaterialStep, MaterialDeclaration<'a>>;
pub use scene::*;

mod vocabulary;
pub use vocabulary::Mdlx;

mod geometry;
pub use geometry::*;

/// Image dependency declarations admitted by this profile.
pub use crate::model::resources::MdlxImageResource as ImageResource;

mod events;
pub use events::EventSource;
mod properties;
pub use properties::*;
