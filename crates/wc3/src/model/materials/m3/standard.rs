//! Logical M3 standard values.
use super::{Property, TextureLayer};
use crate::model::materials::contributions::{
    NormalContributions, PairedInputs, SpecularInputs, SurfaceContribution,
};
use crate::model::materials::state::PassState;
use crate::model::values::programmed::{
    AdditionalMaterialFlags, BlendMode, LayerBlendOp, MaterialClass, MaterialFlags, RttChannels,
    SpecularMode,
};
use crate::model::Construct;
use crate::model::{Link, ValidateLinks};

/// Editable M3 StandardMaterialStep values.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct StandardMaterialStep {
    /// Draw composition, rasterization controls, and authored priority.
    pub state: PassState<BlendMode, StandardRasterization, i32>,
    /// Sets HDR environment constant.
    pub hdr_environment_constant: f32,
    /// Sets HDR environment diffuse.
    pub hdr_environment_diffuse: f32,
    /// Sets HDR environment specular.
    pub hdr_environment_specular: f32,
    /// Supplies the decal composited over the base material.
    pub decal_layer: Option<TextureLayer>,
    /// Supplies the map supplying environment reflections.
    pub environment_layer: Option<TextureLayer>,
    /// Sets environment mask.
    pub environment_mask_layer: Option<TextureLayer>,
    /// Supplies the height map used for parallax displacement.
    pub height_layer: Option<TextureLayer>,
    /// Supplies the map supplying precomputed lighting.
    pub light_map_layer: Option<TextureLayer>,
    /// Supplies the map modulating illumination with ambient occlusion.
    pub ambient_occlusion_layer: Option<TextureLayer>,
    /// Sets material class (unit, building, etc.).
    pub material_class: MaterialClass,
    /// Controls parallax height through its animation value.
    pub parallax_height: Property<f32>,
    /// Controls motion blur amount through its animation value.
    pub motion_blur_amount: Property<f32>,
    /// Base diffuse input.
    pub diffuse: Option<TextureLayer>,
    /// Shared composition operation for the decal and environment layers.
    pub layer_blend: LayerBlendOp,
    /// Specular and gloss inputs with their shader response model.
    pub specular: SurfaceContribution<SpecularInputs<Option<TextureLayer>>, SpecularResponse>,
    /// Independently composited emission slots and their shared HDR scale.
    pub emissive: SurfaceContribution<
        PairedInputs<SurfaceContribution<Option<TextureLayer>, LayerBlendOp>>,
        f32,
    >,
    /// Two independent authored opacity slots.
    pub opacity: PairedInputs<Option<TextureLayer>>,
    /// Base and masked additional normal inputs with authored weights.
    pub normals: NormalContributions<Option<TextureLayer>, Vec<Property<f32>>>,
}

/// Specular response parameters of the programmed standard shader.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpecularResponse {
    /// Highlight exponent.
    pub exponent: f32,
    /// HDR scale applied to the specular contribution.
    pub hdr_multiplier: f32,
    /// Authored specular computation model.
    pub mode: SpecularMode,
}

impl ValidateLinks for SpecularResponse {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.exponent.visit_links(visitor);
        self.hdr_multiplier.visit_links(visitor);
    }
}
impl ValidateLinks for LayerBlendOp {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}

/// Shader-specific rasterization controls whose values remain in their authored domains.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StandardRasterization {
    /// Additional shader behavior flags.
    pub additional_flags: AdditionalMaterialFlags,
    /// MaterialStep behavior flags, including unresolved bits.
    pub flags: MaterialFlags,
    /// Render-to-texture channel selection.
    pub rtt_channels: RttChannels,
    /// Depth blending falloff distance.
    pub depth_blend_falloff: f32,
    /// Authored alpha-test cut-off word.
    pub alpha_test_threshold: u32,
}

impl ValidateLinks for StandardRasterization {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.depth_blend_falloff.visit_links(visitor);
    }
}
