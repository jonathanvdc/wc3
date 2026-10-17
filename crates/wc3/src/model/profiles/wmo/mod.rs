//! World-model scene semantics independent of WMO chunk revisions.
pub use crate::model::values::world::*;
pub use crate::model::{SurfaceTriangleControls, TriangleFlags};
mod region_lights;
pub use region_lights::*;
mod links;
pub use crate::model::controls::world_draw::*;
mod validation;
mod world;
pub use crate::model::controls::world::*;
pub use crate::model::materials::wmo::*;
pub use world::*;

mod vocabulary;
pub use vocabulary::Wmo;

pub use crate::model::geometry::world::*;

/// Image dependency declarations admitted by this profile.
pub use crate::model::resources::WorldImageResource as ImageResource;
