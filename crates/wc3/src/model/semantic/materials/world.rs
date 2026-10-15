//! World-model surfaces and texture bindings independent of WMO chunk layout.
use crate::model::semantic::values::world::MaterialFlags;
use crate::model::semantic::TextureId;
use std::borrow::Cow;

/// World-model blend operations, including operations absent from other formats.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Blend {
    /// Replace destination color.
    #[default]
    Opaque,
    /// Discard transparent texels.
    AlphaKey,
    /// Blend using source opacity.
    Alpha,
    /// Add source color.
    Additive,
    /// Multiply destination color.
    Modulate,
    /// Multiply destination color and double the result.
    Modulate2x,
    /// Multiply and add according to the world shader operation.
    ModulateAdd,
    /// Add using inverse source opacity.
    InverseSourceAlphaAdd,
    /// A source operation with unresolved meaning.
    Unknown(u32),
}
/// Known world-model material options and unresolved controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// Disable scene lighting.
    pub unlit: bool,
    /// Disable fog application.
    pub unfogged: bool,
    /// Render both sides.
    pub two_sided: bool,
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
    /// Blend operation.
    pub blend: Blend,
    /// Lighting and sampling options.
    pub options: Options,
    /// Texture bindings in shader-slot order.
    pub textures: Vec<Option<TextureId>>,
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
