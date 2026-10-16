//! World of Warcraft model authoring features independent of binary layout.
/// Typed animated destinations for World of Warcraft features.
pub mod properties;
mod texture;
pub use crate::model::controls::texture::{SupplementalTextureFlags, TextureControls};
pub use crate::model::materials::model_shader::*;
/// World of Warcraft selects independently animated model-shader inputs.
pub type Material<'a> = ShaderPass<'a, TextureUnit<TextureTransform>>;
mod actors;
pub use actors::*;
mod particles;
pub use crate::model::controls::model::*;
pub use crate::model::systems::model_physics::*;
/// Logical choices and behavior flags specific to this model family.
/// Logical option domains selected from the shared vocabulary.
pub use crate::model::values::model as values;
pub use particles::*;

pub use crate::model::systems::model::*;

mod vocabulary;
pub use vocabulary::M2;

pub use crate::model::geometry::weighted_model::*;

/// Image dependency declarations admitted by this profile.
pub use crate::model::resources::M2ImageResource as ImageResource;
/// External companion addresses admitted by this profile.
pub use crate::model::resources::M2ResourceLocator as ResourceLocator;
