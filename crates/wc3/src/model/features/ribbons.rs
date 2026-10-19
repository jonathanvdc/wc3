//! Composed ribbon emission, geometry, appearance, and rendering programs.
use crate::model::profiles::m2;
use crate::model::{Axis, MaterialId, Property, Texture};
use crate::model::{Construct, PropertyValues};
use glam::Vec3;

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
    /// Family-specific activation, motion, and effect identity.
    pub behavior: C,
}
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
pub struct TrailAppearance<S, A = u32> {
    /// RGB tint.
    pub color: Property<Vec3>,
    /// Opacity multiplier.
    pub opacity: Property<f32>,
    /// Atlas rows and columns; export checks the destination's integer width.
    pub atlas: [A; 2],
    /// Atlas selector retaining its typed animated destination.
    pub texture_slot: S,
}
/// A trail rendered by one layered material.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MdlxRibbonRendering {
    /// Complete layered surface.
    pub material: Option<MaterialId>,
}
/// Ordered textures and shader surfaces supplying a trail.
#[derive(Clone, Debug)]
pub struct M2RibbonRendering {
    /// Ordered texture resources.
    pub textures: Vec<Texture>,
    /// Ordered surfaces.
    pub materials: Vec<MaterialId>,
    /// Signed pass-order plane.
    pub priority_plane: i16,
    /// External color-table selection.
    pub color_index: i8,
    /// Optional texture-coordinate transform.
    pub texture_transform: Option<m2::TextureTransform>,
}
/// Gravitational acceleration and fractional visibility.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct MdlxRibbonBehavior {
    /// Gravity acceleration.
    pub gravity: f32,
    /// Visibility multiplier.
    pub visibility: Property<f32>,
}
/// Emission direction, discrete activation, and effect identity.
#[derive(Clone, Debug)]
pub struct M2RibbonBehavior {
    /// Emission angle, independently of acceleration.
    pub angle: f32,
    /// Runtime effect identifier.
    pub id: u32,
    /// Discrete activation.
    pub enabled: Property<bool>,
}
/// Trail composition with integral section generation and fractional visibility.
pub type MdlxRibbonEmitter =
    TrailComposition<u32, Property<u32>, u32, MdlxRibbonRendering, MdlxRibbonBehavior>;
/// Trail composition with fractional edge generation and discrete activation.
pub type M2RibbonEmitter =
    TrailComposition<f32, Property<u16>, u16, M2RibbonRendering, M2RibbonBehavior>;

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
        self.behavior.visit_links(visitor);
        if !self.emission_rate.valid_rate() || !self.lifetime.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for MdlxRibbonRendering {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.material.visit_links(visitor);
    }
}
impl ValidateLinks for M2RibbonRendering {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.textures.visit_links(visitor);
        self.materials.visit_links(visitor);
        self.texture_transform.visit_links(visitor);
    }
}
impl ValidateLinks for MdlxRibbonBehavior {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.gravity.visit_links(visitor);
        self.visibility.visit_links(visitor);
    }
}
impl ValidateLinks for M2RibbonBehavior {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.angle.visit_links(visitor);
        self.enabled.visit_links(visitor);
    }
}
