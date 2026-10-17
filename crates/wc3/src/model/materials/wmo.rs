//! World-model surfaces and texture bindings independent of WMO chunk layout.
use super::state::{Lighting, Rasterization};
use crate::model::materials::state::BlendOperations;
use crate::model::profiles::Unavailable;
use crate::model::values::world::MaterialFlags;
use crate::model::{Link, ValidateLinks};

/// World-model blend operations, including operations absent from other formats.
pub type Blend = BlendOperations<WorldBlend, (), Unavailable, u32>;
/// World-shader options beyond common face, lighting, and fog controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RasterizationOptions {
    /// Use exterior lighting even for interior geometry.
    pub exterior_lit: bool,
    /// Enable the authored self-illumination color.
    pub self_illuminated: bool,
    /// Preserve controls with unresolved meanings.
    pub unclassified: MaterialFlags,
}
/// World rasterization selects no independent depth program.
pub type MaterialRasterization = Rasterization<(), bool, Lighting, RasterizationOptions>;
/// Additional shader color and texture override information.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShaderOverrides {
    /// Shader-23 override color in RGBA order.
    pub color: [u8; 4],
    /// External texture override-set identity.
    pub texture_override: u64,
    /// Unresolved shader control value.
    pub control: u32,
}
impl ValidateLinks for RasterizationOptions {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}

/// Additional world-shader color operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WorldBlend {
    /// Multiply and add according to the world shader operation.
    ModulateAdd,
    /// Add using inverse source opacity.
    InverseSourceAlphaAdd,
}
impl ValidateLinks for WorldBlend {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}

mod steps;
pub use steps::*;
