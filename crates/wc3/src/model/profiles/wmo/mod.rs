//! World-model scene semantics independent of WMO chunk revisions.
pub use crate::model::values::world::*;
pub use crate::model::TriangleProperties;
mod region_lights;
pub use region_lights::*;
mod links;
mod validation;
mod world;
pub use crate::model::materials::wmo::*;
pub use world::*;

mod vocabulary;
pub use vocabulary::Wmo;

mod geometry;
pub use geometry::*;

/// Image dependency declarations admitted by this profile.
pub use crate::model::resources::WorldImageResource as ImageResource;

/// Complete shader-specific material definition selected by this profile.
pub type Material<'a> = MaterialStep<'a>;
mod properties;
pub use properties::*;
pub mod draw;
pub use draw::*;
