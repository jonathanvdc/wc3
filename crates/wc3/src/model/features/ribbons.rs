//! Composed ribbon emission, geometry, appearance, and rendering programs.
use crate::model::profiles::{m2, mdlx};
use crate::model::{Axis, MaterialId, Property, Texture};
use crate::model::{Construct, PropertyValues};

/// A persistent trail assembled from successive cross sections.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct TrailComposition<R, S, A, B, C> {
    /// Authored generation rate and its semantic unit.
    pub emission_rate: R,
    /// Persistence of a generated section or edge in seconds.
    pub lifetime: f32,
    /// Cross-section extents.
    pub geometry: TrailGeometry,
    /// Tint, opacity, and atlas selection.
    pub appearance: TrailAppearance<S, A>,
    /// Material and texture bindings.
    pub rendering: B,
    /// Independent physical and activation controls.
    pub controls: C,
}
/// Broad trail composition selected by the Universal profile.
pub type TrailEmitter = TrailComposition<f64, TrailAtlasSlot, u32, TrailRendering, TrailControls>;
/// Animated upper and lower extent of a generated trail.
#[derive(Clone, Debug, PartialEq)]
pub struct TrailGeometry {
    /// Signed emitter-local direction of positive cross-section extent.
    /// This is independent of the model's up axis and world gravity.
    pub axis: Axis,
    /// Extent above the center line.
    pub height_above: Property<f32>,
    /// Extent below the center line.
    pub height_below: Property<f32>,
}
impl Construct for TrailGeometry {
    fn construct(values: &mut PropertyValues) -> Self {
        Self {
            axis: Axis::PositiveY,
            height_above: values.add(0.),
            height_below: values.add(0.),
        }
    }
}
/// Animated trail appearance and atlas layout.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct TrailAppearance<S = TrailAtlasSlot, A = u32> {
    /// RGB tint.
    pub color: Property<[f32; 3]>,
    /// Opacity multiplier.
    pub opacity: Property<f32>,
    /// Atlas rows and columns; export checks the destination's integer width.
    pub atlas: [A; 2],
    /// Atlas selector retaining its typed animated destination.
    pub texture_slot: S,
}
/// Typed atlas selectors retain animation identity and value domain.
#[derive(Clone, Debug)]
pub enum TrailAtlasSlot {
    /// Full-width cell selection.
    Wide(Property<u32>),
    /// Compact cell selection.
    Compact(Property<u16>),
}
/// Logical rendering alternatives rather than source-file layouts.
#[derive(Clone, Debug)]
pub enum TrailRendering {
    /// A layered material supplies the complete surface program.
    Layered(LayeredTrailRendering),
    /// Ordered texture and shader-surface bindings.
    Surfaces(SurfaceTrailRendering),
}
/// Physical and application controls with explicit meanings.
#[derive(Clone, Debug)]
#[allow(
    clippy::large_enum_variant,
    reason = "Authored controls remain directly editable without an allocation."
)]
pub enum TrailControls {
    /// Gravitational acceleration and fractional visibility.
    Gravity(GravityTrailControls),
    /// Emission direction and discrete activation with texture controls.
    EmissionAngle(AngleTrailControls),
}
/// A trail rendered by one layered material.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayeredTrailRendering {
    /// Complete layered surface.
    pub material: Option<MaterialId>,
}
/// Ordered textures and shader surfaces supplying a trail.
#[derive(Clone, Debug)]
pub struct SurfaceTrailRendering {
    /// Ordered texture resources.
    pub textures: Vec<Texture>,
    /// Ordered surfaces.
    pub materials: Vec<MaterialId>,
}
/// Gravitational acceleration and fractional visibility.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct GravityTrailControls {
    /// Gravity acceleration.
    pub gravity: f32,
    /// Visibility multiplier.
    pub visibility: Property<f32>,
}
/// Emission direction, discrete activation, and texture controls.
#[derive(Clone, Debug)]
pub struct AngleTrailControls {
    /// Emission angle, independently of acceleration.
    pub angle: f32,
    /// Runtime effect identifier.
    pub id: u32,
    /// Discrete activation.
    pub enabled: Property<bool>,
    /// Signed pass-order plane.
    pub priority_plane: i16,
    /// External color-table selection.
    pub color_index: i8,
    /// Optional texture-coordinate transform.
    pub texture_transform: Option<m2::TextureTransform>,
}
/// Trail composition with integral section generation and fractional visibility.
pub type GravityTrail =
    TrailComposition<u32, Property<u32>, u32, LayeredTrailRendering, GravityTrailControls>;
/// Trail composition with fractional edge generation and discrete activation.
pub type AngleTrail =
    TrailComposition<f32, Property<u16>, u16, SurfaceTrailRendering, AngleTrailControls>;
impl From<mdlx::Ribbon> for TrailEmitter {
    fn from(v: mdlx::Ribbon) -> Self {
        Self {
            emission_rate: f64::from(v.emission_rate),
            lifetime: v.lifetime,
            geometry: v.geometry,
            appearance: TrailAppearance {
                color: v.appearance.color,
                opacity: v.appearance.opacity,
                atlas: v.appearance.atlas,
                texture_slot: TrailAtlasSlot::Wide(v.appearance.texture_slot),
            },
            rendering: TrailRendering::Layered(v.rendering),
            controls: TrailControls::Gravity(v.controls),
        }
    }
}
impl From<m2::Ribbon> for TrailEmitter {
    fn from(v: m2::Ribbon) -> Self {
        Self {
            emission_rate: f64::from(v.emission_rate),
            lifetime: v.lifetime,
            geometry: v.geometry,
            appearance: TrailAppearance {
                color: v.appearance.color,
                opacity: v.appearance.opacity,
                atlas: v.appearance.atlas.map(u32::from),
                texture_slot: TrailAtlasSlot::Compact(v.appearance.texture_slot),
            },
            rendering: TrailRendering::Surfaces(v.rendering),
            controls: TrailControls::EmissionAngle(v.controls),
        }
    }
}
impl TryFrom<TrailEmitter> for mdlx::Ribbon {
    type Error = Box<TrailEmitter>;
    fn try_from(v: TrailEmitter) -> Result<Self, Self::Error> {
        if !(v.emission_rate >= 0.0
            && v.emission_rate <= f64::from(u32::MAX)
            && v.emission_rate.fract() == 0.0
            && matches!(
                (&v.appearance.texture_slot, &v.rendering, &v.controls),
                (
                    TrailAtlasSlot::Wide(_),
                    TrailRendering::Layered(_),
                    TrailControls::Gravity(_)
                )
            ))
        {
            return Err(Box::new(v));
        }
        let emission_rate = v.emission_rate as u32;
        let TrailAtlasSlot::Wide(texture_slot) = v.appearance.texture_slot else {
            unreachable!()
        };
        let TrailRendering::Layered(rendering) = v.rendering else {
            unreachable!()
        };
        let TrailControls::Gravity(controls) = v.controls else {
            unreachable!()
        };
        Ok(Self {
            geometry: v.geometry,
            appearance: TrailAppearance {
                opacity: v.appearance.opacity,
                color: v.appearance.color,
                texture_slot,
                atlas: v.appearance.atlas,
            },
            lifetime: v.lifetime,
            emission_rate,
            rendering,
            controls,
        })
    }
}
impl TryFrom<TrailEmitter> for m2::Ribbon {
    type Error = Box<TrailEmitter>;
    fn try_from(v: TrailEmitter) -> Result<Self, Self::Error> {
        if f64::from(v.emission_rate as f32).to_bits() != v.emission_rate.to_bits()
            || v.appearance
                .atlas
                .iter()
                .any(|n| u16::try_from(*n).is_err())
            || !matches!(
                (&v.appearance.texture_slot, &v.rendering, &v.controls),
                (
                    TrailAtlasSlot::Compact(_),
                    TrailRendering::Surfaces(_),
                    TrailControls::EmissionAngle(_)
                )
            )
        {
            return Err(Box::new(v));
        }
        let edges_per_second = v.emission_rate as f32;
        let TrailAtlasSlot::Compact(texture_slot) = v.appearance.texture_slot else {
            unreachable!()
        };
        let TrailRendering::Surfaces(rendering) = v.rendering else {
            unreachable!()
        };
        let TrailControls::EmissionAngle(controls) = v.controls else {
            unreachable!()
        };
        Ok(Self {
            controls,
            rendering,
            appearance: TrailAppearance {
                color: v.appearance.color,
                opacity: v.appearance.opacity,
                atlas: v.appearance.atlas.map(|n| n as u16),
                texture_slot,
            },
            geometry: v.geometry,
            emission_rate: edges_per_second,
            lifetime: v.lifetime,
        })
    }
}

use crate::model::{Link, ValidateLinks};

/// Numeric domain for a trail generation rate.
pub trait TrailRate {
    /// Whether this value describes a finite, nonnegative generation rate.
    fn valid_rate(&self) -> bool;
}
impl TrailRate for u32 {
    fn valid_rate(&self) -> bool {
        true
    }
}
impl TrailRate for f32 {
    fn valid_rate(&self) -> bool {
        self.is_finite() && *self >= 0.0
    }
}
impl TrailRate for f64 {
    fn valid_rate(&self) -> bool {
        self.is_finite() && *self >= 0.0
    }
}
impl<R: TrailRate, S: ValidateLinks, A, B: ValidateLinks, C: ValidateLinks> ValidateLinks
    for TrailComposition<R, S, A, B, C>
{
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.geometry.height_above.visit_links(visitor);
        self.geometry.height_below.visit_links(visitor);
        self.appearance.color.visit_links(visitor);
        self.appearance.opacity.visit_links(visitor);
        self.appearance.texture_slot.visit_links(visitor);
        self.rendering.visit_links(visitor);
        self.controls.visit_links(visitor);
        if !self.emission_rate.valid_rate() || !self.lifetime.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for TrailAtlasSlot {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Wide(value) => value.visit_links(visitor),
            Self::Compact(value) => value.visit_links(visitor),
        }
    }
}
impl ValidateLinks for TrailRendering {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Layered(value) => value.visit_links(visitor),
            Self::Surfaces(value) => value.visit_links(visitor),
        }
    }
}
impl ValidateLinks for LayeredTrailRendering {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.material.visit_links(visitor);
    }
}
impl ValidateLinks for SurfaceTrailRendering {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.textures.visit_links(visitor);
        self.materials.visit_links(visitor);
    }
}
impl ValidateLinks for GravityTrailControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.gravity.visit_links(visitor);
        self.visibility.visit_links(visitor);
    }
}
impl ValidateLinks for AngleTrailControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.angle.visit_links(visitor);
        self.enabled.visit_links(visitor);
        self.texture_transform.visit_links(visitor);
    }
}
impl ValidateLinks for TrailControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Gravity(value) => value.visit_links(visitor),
            Self::EmissionAngle(value) => value.visit_links(visitor),
        }
    }
}
