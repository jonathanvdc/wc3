//! Complete M3 material families as editable logical values.
pub use crate::model::semantic::animation::programmed::{AnimationControl, Property};
mod layer;
pub use layer::*;
mod standard;
pub use standard::*;
mod composite;
pub use composite::*;
mod volume;
pub use volume::*;
mod surface;
pub use surface::*;
mod reflection;
pub use reflection::*;
mod flare;
pub use flare::*;
mod data_driven;
pub use data_driven::*;

mod hair;
pub use hair::*;

/// Profile-specific surface definition.
#[derive(Clone, Debug, PartialEq)]
pub enum Material<'a> {
    /// Editable Standard surface.
    Standard(Box<Standard>),
    /// Editable Displacement surface.
    Displacement(Box<Displacement>),
    /// Editable Composite surface.
    Composite(Box<Composite>),
    /// Editable Terrain surface.
    Terrain(Box<Terrain>),
    /// Editable Volume surface.
    Volume(Box<Volume>),
    /// Editable VolumeNoise surface.
    VolumeNoise(Box<VolumeNoise>),
    /// Anisotropic hair shading.
    Hair(Box<Hair>),
    /// Editable Creep surface.
    Creep(Box<Creep>),
    /// Editable SplatTerrainBake surface.
    SplatTerrainBake(Box<SplatTerrainBake>),
    /// Editable Reflection surface.
    Reflection(Box<Reflection>),
    /// Editable LensFlare surface.
    LensFlare(Box<LensFlare<'a>>),
    /// Editable DataDriven surface.
    DataDriven(Box<DataDriven<'a>>),
}
