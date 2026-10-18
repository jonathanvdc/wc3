//! M3 shader and material-composition steps as editable logical values.
pub use crate::model::animation::programmed::{AnimationControl, PropertyBinding};
mod layer;
pub(crate) mod properties;
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

/// One shader or material-composition operation in the M3 dialect.
#[derive(Clone, Debug, PartialEq)]
pub enum MaterialStep<'a> {
    /// M3 Standard shader inputs and rendering settings.
    Standard(Box<StandardMaterialStep>),
    /// M3 Displacement shader inputs and rendering settings.
    Displacement(Box<DisplacementMaterialStep>),
    /// Weighted composition of referenced material programs.
    Composite(Box<CompositeMaterialStep>),
    /// M3 Terrain shader inputs and rendering settings.
    Terrain(Box<TerrainMaterialStep>),
    /// M3 Volume shader inputs and rendering settings.
    Volume(Box<VolumeMaterialStep>),
    /// M3 VolumeNoise shader inputs and rendering settings.
    VolumeNoise(Box<VolumeNoiseMaterialStep>),
    /// Anisotropic hair shading.
    Hair(Box<HairMaterialStep>),
    /// M3 Creep shader inputs and rendering settings.
    Creep(Box<CreepMaterialStep>),
    /// M3 SplatTerrainBake shader inputs and rendering settings.
    SplatTerrainBake(Box<SplatTerrainBakeMaterialStep>),
    /// M3 Reflection shader inputs and rendering settings.
    Reflection(Box<ReflectionMaterialStep>),
    /// M3 LensFlare shader inputs and rendering settings.
    LensFlare(Box<LensFlareMaterialStep<'a>>),
    /// M3 DataDriven shader inputs and rendering settings.
    DataDriven(Box<DataDrivenMaterialStep<'a>>),
}
