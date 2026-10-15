//! Logical M3 surface values.
use super::{Property, TextureLayer};
use crate::model::materials::contributions::SurfaceContribution;
use crate::model::values::programmed::DisplacementFlags;

/// Editable M3 Displacement values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Displacement {
    /// The purpose of this field has not been established.
    pub unknown: u32,
    /// Supplies the mask scaling displacement strength.
    pub strength_map: Option<TextureLayer>,
    /// Retains the native material control bits; unresolved bits remain intact.
    pub flags: DisplacementFlags,
    /// Sets render priority (lower = earlier).
    pub priority: u32,
    /// Authored displacement input and response.
    pub displacement: SurfaceContribution<Option<TextureLayer>, Property<f32>>,
}

/// Editable M3 Terrain values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Terrain {
    /// Supplies the texture layer used for the terrain contribution.
    pub terrain_map: Option<TextureLayer>,
    /// The purpose of this field has not been established.
    pub unknown: u32,
}

/// Editable M3 Creep values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Creep {
    /// Supplies the mask delimiting the creep effect.
    pub mask_map: Option<TextureLayer>,
    /// Controls the lower creep-mask parameter; its numerical interpretation is not established.
    pub creep_low: u32,
}

/// Editable M3 SplatTerrainBake values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SplatTerrainBake {
    /// Supplies the surface diffuse-color map.
    pub diffuse_map: Option<TextureLayer>,
    /// Supplies the map defining the displacement direction.
    pub normal_map: Option<TextureLayer>,
    /// Supplies the map controlling specular response.
    pub specular_map: Option<TextureLayer>,
}
