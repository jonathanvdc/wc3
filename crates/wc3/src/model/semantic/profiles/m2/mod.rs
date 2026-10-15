//! World of Warcraft model authoring features independent of binary layout.
/// Typed animated destinations for World of Warcraft features.
pub mod properties;
mod texture;
pub use crate::model::semantic::controls::texture::{SupplementalTextureFlags, TextureControls};
pub use crate::model::semantic::materials::model_shader::*;
/// World of Warcraft selects independently animated model-shader inputs.
pub type Material<'a> = ShaderPass<'a, TextureUnit<TextureTransform>>;
mod actors;
pub use actors::*;
mod particles;
pub use crate::model::semantic::controls::model::*;
pub use crate::model::semantic::systems::model_physics::*;
/// Logical choices and behavior flags specific to this model family.
/// Logical option domains selected from the shared vocabulary.
pub use crate::model::semantic::values::model as values;
pub use particles::*;

pub use crate::model::semantic::systems::model::*;

mod vocabulary;
pub use vocabulary::M2;

pub use crate::model::semantic::geometry::weighted_model::*;
