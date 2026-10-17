//! Diablo IV authoring meanings remain independent of native format hashes.
pub use crate::model::controls::diablo4::*;
pub use crate::model::geometry::diablo4::*;
pub use crate::model::materials::diablo4::*;
use crate::model::materials::program::MaterialProgram;
mod vocabulary;
pub use vocabulary::D4;

pub use crate::model::appearance::diablo4::*;

/// A native external material has zero or one supplied shader definition.
pub type Material = MaterialProgram<MaterialStep>;
