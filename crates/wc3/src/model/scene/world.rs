//! Group visibility, environmental volumes, placements, and liquid grids.
use crate::model::common::AsULE;
use crate::model::controls::world::{DoodadSetId, GroupId, PortalId};
use crate::model::values::world::{
    AmbientFlags, FogFlags, GroupFlags, GroupFlags2, LightFlags, PortalFlags, WorldFlags,
};
use crate::model::{Animated, FeatureId, MaterialId, Resource, TextureId, ZeroVec};
use crate::model::{ChildModelDefinition, WorldPlacementAppearance};
use std::borrow::Cow;

/// Root world rendering options.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldOptions {
    /// Prefer the authored maximum detail-level selection.
    pub default_max_lod: bool,
    /// Disable distance attenuation of vertex colors.
    pub no_vertex_attenuation: bool,
    /// Authored vertex RGB excludes the root ambient colour.
    pub skip_ambient_in_vertex_color: bool,
    /// Interpret liquid selectors as database identifiers.
    pub database_liquids: bool,
    /// Preserve authored vertex RGB while fixing non-transition lighting alpha.
    pub vertex_color_alpha_only: bool,
    /// Root controls with unresolved meanings.
    pub unclassified: WorldFlags,
}
/// Visibility and lighting behavior of one world group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GroupOptions {
    /// Group is outdoors.
    pub exterior: bool,
    /// Group is indoors.
    pub interior: bool,
    /// Use exterior lighting within this group.
    pub exterior_lit: bool,
    /// Group cannot be reached through normal portal queries.
    pub unreachable: bool,
    /// Show the sky from this group.
    pub sky: bool,
    /// Uses authored baked lighting for this group.
    pub baked_lighting: bool,
    /// Group controls with unresolved meanings.
    pub unclassified: GroupFlags,
    /// Supplemental group controls with unresolved meanings.
    pub supplemental: GroupFlags2,
}
/// Portal boundary and authored culling plane.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Portal<'a> {
    /// Boundary polygon in world coordinates.
    pub vertices: ZeroVec<'a, [f32; 3]>,
    /// Plane equation (normal XYZ, distance).
    pub plane: [f32; 4],
    /// Supplemental culling-plane adjustment, whose individual controls are unresolved.
    pub adjustment: Option<[u32; 3]>,
}
/// Group connectivity through a portal.
#[derive(Clone, Debug, PartialEq)]
pub struct PortalConnection {
    /// Portal being traversed.
    pub portal: PortalId,
    /// Neighboring group on the other side.
    pub neighbor: GroupId,
    /// Signed side of the portal plane.
    pub side: i16,
    /// Skip viewer queries for this connection.
    pub skip_viewer: bool,
    /// Apply the portal's supplemental plane adjustment.
    pub adjusted_plane: bool,
    /// Connection controls with unresolved meanings.
    pub unclassified: PortalFlags,
}
/// Named set with explicit placement membership.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DoodadSet<'a> {
    /// Set name.
    pub name: Cow<'a, str>,
    /// Doodad features belonging to this set.
    pub members: Vec<FeatureId>,
    /// Unresolved authored set control.
    pub control: u32,
}
/// External resource placement and authored tint.
pub type Doodad<'a> = ChildModelDefinition<Resource<'a>, (), WorldPlacementAppearance>;
/// Color alpha and light selection are distinct authored tint behaviors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoodadTint {
    /// Explicit RGBA tint.
    Color([u8; 4]),
    /// RGB tint illuminated by a referenced world light.
    Lit {
        /// Authored RGB tint.
        color: [u8; 3],
        /// Light supplying illumination.
        light: FeatureId,
    },
}

/// World light behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LightKind {
    /// Point illumination.
    #[default]
    Point,
    /// Spot illumination.
    Spot,
    /// Directional illumination.
    Directional,
    /// Ambient illumination.
    Ambient,
    /// Unresolved authored kind selector.
    Unknown(i32),
}
/// Complete world light parameters.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Light {
    /// Illumination behavior.
    pub kind: LightKind,
    /// Animated color in RGBA order.
    pub color: Animated<[f32; 4]>,
    /// Animated intensity.
    pub intensity: Animated<f32>,
    /// Enable distance attenuation.
    pub use_attenuation: bool,
    /// Attenuation start and end distances.
    pub attenuation: [f32; 2],
    /// Cone start and end angles.
    pub cone: [f32; 2],
    /// Light controls with unresolved meanings.
    pub controls: LightFlags,
    /// Optional modern light color, cookie, flicker, and shadow controls.
    pub modern: Option<ModernLight>,
    /// Optional directional extension terms and their unresolved scalars.
    pub directional_terms: Option<[([f32; 3], f32); 6]>,
}
/// Light-flicker behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Flicker {
    /// Disable flicker.
    #[default]
    Off,
    /// Continuous sine response.
    Sine,
    /// Continuous noise response.
    Noise,
    /// Stepped noise response.
    SteppedNoise,
    /// Selector with unresolved meaning.
    Unknown(i32),
}
/// Modern light authored properties independent of packed half-float storage.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModernLight {
    /// Authored Euler orientation; native axis order has not been established.
    pub orientation: [f32; 3],
    /// External authored light identifier.
    pub id: i32,
    /// Restrict the light to one doodad set.
    pub doodad_set: Option<DoodadSetId>,
    /// Enable inner-to-outer color blending.
    pub blend_colors: bool,
    /// Cast shadows.
    pub shadows: bool,
    /// Outer light color in RGBA order.
    pub outer_color: [u8; 4],
    /// Distance interval for inner-to-outer color blending.
    pub color_blend: [f32; 2],
    /// Flicker intensity and speed.
    pub flicker_parameters: [f32; 2],
    /// Flicker behavior.
    pub flicker: Flicker,
    /// Light projection cookie texture.
    pub cookie: Option<TextureId>,
    /// Authored falloff parameter.
    pub falloff: f32,
    /// Authored scale as a numerical value.
    pub scale: f32,
    /// Authored intensity multiplier; zero retains the authored client-default behavior.
    pub intensity_multiplier: f32,
    /// Unresolved authored vector control.
    pub vector_control: [f32; 3],
    /// Unresolved authored words surrounding known light properties.
    pub controls: [u32; 7],
    /// Trailing unresolved authored controls.
    pub trailing_controls: [u32; 11],
}
/// Fog color and distance response.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogParameters {
    /// Distance at full fog strength.
    pub end: f32,
    /// Fog start as a fraction of the end distance.
    pub start_fraction: f32,
    /// Fog color in RGBA order.
    pub color: [u8; 4],
}
/// Environmental volume boundary.
#[derive(Clone, Debug, PartialEq)]
pub enum Volume {
    /// Radial volume and its inner and outer blend radii.
    Sphere {
        /// World center.
        center: [f32; 3],
        /// Inner and outer radius.
        radii: [f32; 2],
    },
    /// Six bounding planes and boundary fade distance.
    Planes {
        /// Plane equations.
        planes: [[f32; 4]; 6],
        /// Boundary fade distance.
        fade: f32,
    },
}
/// Authored fog volume and activation controls.
#[derive(Clone, Debug, PartialEq)]
pub struct Fog {
    /// World-space volume boundary.
    pub volume: Volume,
    /// Above-water response.
    pub above_water: FogParameters,
    /// Underwater response.
    pub underwater: FogParameters,
    /// Restrict activation to one doodad set.
    pub doodad_set: Option<DoodadSetId>,
    /// Skip radial client queries as an infinite region.
    pub infinite: bool,
    /// Unresolved environmental controls.
    pub controls: FogFlags,
}
/// Ambient-light region and activation controls.
#[derive(Clone, Debug, PartialEq)]
pub struct Ambient {
    /// Apply this ambient region as part of global world illumination.
    pub global: bool,
    /// World-space volume boundary.
    pub volume: Volume,
    /// Ambient colors in RGBA order.
    pub colors: [[u8; 4]; 3],
    /// Use all three authored colors.
    pub distinct_colors: bool,
    /// Restrict activation to one doodad set.
    pub doodad_set: Option<DoodadSetId>,
    /// Unresolved environmental controls.
    pub controls: AmbientFlags,
}
/// Surface elevation and authored liquid-flow interpretation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LiquidVertex {
    /// Water-flow components and their authored mixture.
    Water {
        /// Surface elevation.
        height: f32,
        /// Two authored flow velocities.
        velocities: [u8; 2],
        /// Percentage assigned to the first component.
        first_percentage: u8,
        /// Supplemental unresolved control.
        control: u8,
    },
    /// Magma or slime texture coordinates.
    Magma {
        /// Surface elevation.
        height: f32,
        /// Signed authored texture coordinates.
        coordinates: [i16; 2],
    },
    /// Flow controls whose interpretation requires external liquid context.
    Unresolved {
        /// Surface elevation.
        height: f32,
        /// Authored controls retained numerically without choosing a flow behavior.
        controls: [u16; 2],
    },
}
impl LiquidVertex {
    /// Surface elevation independent of the flow interpretation.
    pub fn height(self) -> f32 {
        match self {
            Self::Water { height, .. }
            | Self::Magma { height, .. }
            | Self::Unresolved { height, .. } => height,
        }
    }
}
/// Liquid surface independent of its native grid serialization.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Liquid<'a> {
    /// Grid origin in world coordinates.
    pub position: [f32; 3],
    /// Number of vertices along X and Y.
    pub dimensions: [u32; 2],
    /// Surface grid vertices in row-major order.
    pub vertices: ZeroVec<'a, LiquidVertex>,
    /// Tile rendering and liquid selectors with unresolved bit-level meanings.
    pub tiles: ZeroVec<'a, u8>,
    /// Rendering material.
    pub material: Option<MaterialId>,
    /// External liquid-type database identifier.
    pub liquid_type: u32,
}

#[zerovec::make_ule(LiquidKindULE)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[doc(hidden)]
pub enum LiquidKind {
    Water = 0,
    Magma = 1,
    Unresolved = 2,
}

impl AsULE for LiquidVertex {
    type ULE = <(LiquidKind, f32, [u16; 2]) as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let record: (LiquidKind, f32, [u16; 2]) = match self {
            Self::Water {
                height,
                velocities,
                first_percentage,
                control,
            } => (
                LiquidKind::Water,
                height,
                [
                    u16::from_le_bytes(velocities),
                    u16::from_le_bytes([first_percentage, control]),
                ],
            ),
            Self::Magma {
                height,
                coordinates,
            } => (LiquidKind::Magma, height, coordinates.map(|n| n as u16)),
            Self::Unresolved { height, controls } => (LiquidKind::Unresolved, height, controls),
        };
        record.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let (kind, height, controls) = <(LiquidKind, f32, [u16; 2])>::from_unaligned(value);
        match kind {
            LiquidKind::Water => {
                let [first_percentage, control] = controls[1].to_le_bytes();
                Self::Water {
                    height,
                    velocities: controls[0].to_le_bytes(),
                    first_percentage,
                    control,
                }
            }
            LiquidKind::Magma => Self::Magma {
                height,
                coordinates: controls.map(|n| n as i16),
            },
            LiquidKind::Unresolved => Self::Unresolved { height, controls },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LiquidVertex;
    use zerovec::ZeroVec;

    #[test]
    fn packed_liquid_vertices_preserve_variants_and_reject_unknown_tags() {
        let vertices = [
            LiquidVertex::Water {
                height: 1.,
                velocities: [2, 3],
                first_percentage: 4,
                control: 5,
            },
            LiquidVertex::Magma {
                height: 6.,
                coordinates: [-7, 8],
            },
            LiquidVertex::Unresolved {
                height: 9.,
                controls: [10, 11],
            },
        ];
        let packed = ZeroVec::alloc_from_slice(&vertices);
        assert_eq!(
            ZeroVec::<LiquidVertex>::parse_bytes(packed.as_bytes())
                .unwrap()
                .to_vec(),
            vertices
        );
        let mut bytes = packed.as_bytes().to_vec();
        bytes[0] = 3;
        assert!(ZeroVec::<LiquidVertex>::parse_bytes(&bytes).is_err());
    }
}
