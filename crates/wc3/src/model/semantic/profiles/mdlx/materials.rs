//! Warcraft III layered surfaces independent of MDX and MDL storage.
use super::{LayerFlags, MaterialFlags};
use crate::model::semantic::TextureTransformId;
use crate::model::semantic::{Animated, TextureId};
use std::borrow::Cow;

/// Blending performed by a Warcraft III material layer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Blend {
    /// Replace the destination color.
    #[default]
    Opaque,
    /// Discard transparent texels.
    AlphaKey,
    /// Blend using source opacity.
    Alpha,
    /// Add source color.
    Additive,
    /// Add opacity-weighted source color.
    AdditiveAlpha,
    /// Multiply destination color.
    Modulate,
    /// Multiply destination color and double the result.
    Modulate2x,
    /// A source operation whose meaning is not established.
    Unknown(u32),
}
/// Sampling and lighting controls for an individual pass.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shading {
    /// Bypass directional lighting.
    pub unlit: bool,
    /// Render both sides of the geometry.
    pub two_sided: bool,
    /// Bypass scene fog.
    pub unfogged: bool,
    /// Disable depth comparison.
    pub no_depth_test: bool,
    /// Disable depth writes.
    pub no_depth_write: bool,
    /// Keep sphere-environment sampling enabled.
    pub sphere_environment: bool,
    /// Wrap the U coordinate at layer scope.
    pub wrap_u: bool,
    /// Wrap the V coordinate at layer scope.
    pub wrap_v: bool,
    /// Preserve shading controls with unresolved meanings.
    pub unclassified: LayerFlags,
}
/// One independently animated texture slot in an extended shader.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureSlot {
    /// Semantic shader slot; undocumented selectors remain numeric.
    pub slot: u32,
    /// Texture sampled by this slot, or no texture.
    pub texture: Animated<Option<TextureId>>,
}
/// Fresnel controls for a material pass.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fresnel {
    /// Edge color in RGB order.
    pub color: Animated<[f32; 3]>,
    /// Edge opacity.
    pub opacity: Animated<f32>,
    /// Amount of team-color contribution.
    pub team_color: Animated<f32>,
}
/// One Warcraft III draw pass. Pass ordering is the order in the material.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layer {
    /// Color composition operation.
    pub blend: Blend,
    /// Lighting, depth, and face controls.
    pub shading: Shading,
    /// Animated primary texture selection.
    pub texture: Animated<Option<TextureId>>,
    /// Index of the mesh UV set sampled by this pass.
    pub uv_set: u32,
    /// Local texture-transform identity within the profile.
    pub texture_transform: Option<TextureTransformId>,
    /// Opacity multiplier.
    pub opacity: Animated<f32>,
    /// Emissive contribution, when supported by the shader.
    pub emissive_gain: Option<Animated<f32>>,
    /// Fresnel contribution, when present.
    pub fresnel: Option<Fresnel>,
    /// Shader-table selector, whose table is external to the file.
    pub shader: Option<u32>,
    /// Additional shader texture slots.
    pub textures: Vec<TextureSlot>,
}
/// Complete Warcraft III material, including every ordered draw pass.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Material<'a> {
    /// Rendering priority relative to other materials.
    pub priority_plane: i32,
    /// Shader name; empty selects the classic shader family.
    pub shader: Cow<'a, str>,
    /// Use constant-color material behavior.
    pub constant_color: bool,
    /// Render both sides at material scope.
    pub two_sided: bool,
    /// Sort primitives near to far along view Z.
    pub sort_near_z: bool,
    /// Sort primitives far to near along view Z.
    pub sort_far_z: bool,
    /// Request full-resolution texture sampling.
    pub full_resolution: bool,
    /// Preserve material controls with unresolved meanings.
    pub unclassified: MaterialFlags,
    /// Ordered rendering passes.
    pub layers: Vec<Layer>,
}
