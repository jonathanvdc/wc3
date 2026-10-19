//! World of Warcraft model authoring features independent of binary layout.
/// Typed animated destinations for World of Warcraft features.
mod texture;
pub use crate::model::materials::m2::*;
mod actors;
pub use actors::*;
mod particles;
pub use crate::model::systems::model_physics::*;
/// Logical choices and behavior flags specific to this model family.
/// Logical option domains selected from the shared vocabulary.
pub use crate::model::values::model as values;
pub use particles::*;
pub use values::AnimationId;

pub use crate::model::systems::model::*;

mod vocabulary;
pub use vocabulary::M2;

pub use crate::model::geometry::weighted_model::*;

/// Image dependency declarations admitted by this profile.
pub use crate::model::resources::M2ImageResource as ImageResource;
/// External companion addresses admitted by this profile.
pub use crate::model::resources::M2ResourceLocator as ResourceLocator;

use crate::model::MaterialProgram;

/// Ordered shader-specific material steps selected by this profile.
pub type Material<'a> = MaterialProgram<MaterialStep<'a>>;

mod events;
pub use events::EventSource;
mod properties;
pub use properties::*;
