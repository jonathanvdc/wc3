//! Logical M3 surfaces, effects, and source controls.
/// Typed animated destinations for StarCraft II features.
pub mod properties;
pub use crate::model::semantic::controls::unbound::{OrderedKeys, UnboundBlock, UnboundKeys};
pub use crate::model::semantic::materials::programmed::*;
/// Logical option domains selected from the shared vocabulary.
pub use crate::model::semantic::values::programmed as values;
mod effects;
pub use effects::*;
mod scene;
pub use crate::model::semantic::systems::programmed_physics::*;
pub use scene::*;
mod model;
pub use crate::model::semantic::controls::programmed::*;
pub use model::*;
mod features;
mod links;
mod validation;
pub use features::*;

pub use crate::model::semantic::controls::playback::*;
/// Programmed-scene controllers select native key blocks and typed unbound keys.
pub type ControllerControls<'a> = ControllerProgram<BlockControls, UnboundBlock<'a>>;

pub use crate::model::semantic::systems::programmed::*;

mod vocabulary;
pub use vocabulary::M3;

pub use crate::model::semantic::geometry::programmed::*;
