//! World-model surfaces and texture bindings independent of WMO chunk layout.
use super::bindings::TextureBinding;
use super::state::{Lighting, PassState, Rasterization};
use crate::model::materials::state::BlendOperations;
use crate::model::profiles::Unavailable;
use crate::model::values::world::MaterialFlags;
use crate::model::{Link, TextureId, ValidateLinks};
use std::borrow::Cow;

/// World-model blend operations, including operations absent from other formats.
pub type Blend = BlendOperations<WorldBlend, (), Unavailable, u32>;
/// World-shader options beyond common face, lighting, and fog controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SurfaceOptions {
    /// Use exterior lighting even for interior geometry.
    pub exterior_lit: bool,
    /// Clamp the U coordinate.
    pub clamp_u: bool,
    /// Clamp the V coordinate.
    pub clamp_v: bool,
    /// Enable the authored self-illumination color.
    pub self_illuminated: bool,
    /// Preserve controls with unresolved meanings.
    pub unclassified: MaterialFlags,
}
/// World rasterization selects no independent depth program.
pub type Options = Rasterization<(), bool, Lighting, SurfaceOptions>;
/// Additional shader color and texture override information.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Extension {
    /// Shader-23 override color in RGBA order.
    pub color: [u8; 4],
    /// External texture override-set identity.
    pub texture_override: u64,
    /// Unresolved shader control value.
    pub control: u32,
}
/// Complete world-model material and all shader texture slots.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Material<'a> {
    /// Optional authoring name.
    pub name: Cow<'a, str>,
    /// External client shader-table selector, absent for fixed-function surfaces.
    pub shader: Option<u32>,
    /// Color composition and shared rasterization controls.
    pub state: PassState<Blend, Options>,
    /// Texture bindings in shader-slot order.
    pub textures: Vec<TextureBinding<Option<TextureId>>>,
    /// Self-illumination color in RGBA order.
    pub self_illumination: [u8; 4],
    /// Serialized runtime self-illumination color in RGBA order.
    pub runtime_self_illumination: [u8; 4],
    /// Diffuse color, also used for interior liquid shading.
    pub diffuse: [u8; 4],
    /// External terrain-type identifier.
    pub ground_type: u32,
    /// UV scroll velocity for the first two texture slots.
    pub uv_scroll: [[f32; 2]; 2],
    /// Optional extended shader overrides.
    pub extension: Option<Extension>,
    /// Historical additional material color in RGBA order.
    pub legacy_color: Option<[u8; 4]>,
    /// Historical additional material controls with unresolved meanings.
    pub legacy_controls: Option<u32>,
    /// Historical runtime control values with unresolved meanings.
    pub legacy_runtime: Option<[u32; 4]>,
}

impl ValidateLinks for SurfaceOptions {
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
