//! Logical M3 standard values.
use super::super::values::{
    AdditionalMaterialFlags, BlendMode, LayerBlendOp, MaterialClass, MaterialFlags, RttChannels,
    SpecularMode,
};
use super::{Property, TextureLayer};

/// Editable M3 Standard values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Standard {
    /// Stores the bit mask governing additional.
    pub additional_flags: AdditionalMaterialFlags,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: MaterialFlags,
    /// Selects the behavior used for alpha blend mode.
    pub blend_mode: BlendMode,
    /// Sets render priority (lower = earlier).
    pub priority: i32,
    /// Selects render-to-texture channels through a bit mask.
    pub rtt_channels: RttChannels,
    /// Sets specular highlight exponent.
    pub specular_exponent: f32,
    /// Sets depth blend falloff distance.
    pub depth_blend_falloff: f32,
    /// Sets alpha test cut-off value.
    pub alpha_test_threshold: u32,
    /// Sets HDR specular multiplier.
    pub hdr_specular_multiplier: f32,
    /// Sets HDR emissive multiplier.
    pub hdr_emissive_multiplier: f32,
    /// Sets HDR environment constant.
    pub hdr_environment_constant: f32,
    /// Sets HDR environment diffuse.
    pub hdr_environment_diffuse: f32,
    /// Sets HDR environment specular.
    pub hdr_environment_specular: f32,
    /// Supplies the surface diffuse-color layer.
    pub diffuse_layer: Option<TextureLayer>,
    /// Supplies the decal composited over the base material.
    pub decal_layer: Option<TextureLayer>,
    /// Supplies the map controlling specular response.
    pub specular_layer: Option<TextureLayer>,
    /// Supplies the map controlling the surface gloss.
    pub gloss_layer: Option<TextureLayer>,
    /// Supplies the first self-illumination layer.
    pub emissive_layer1: Option<TextureLayer>,
    /// Supplies the second self-illumination layer.
    pub emissive_layer2: Option<TextureLayer>,
    /// Supplies the map supplying environment reflections.
    pub environment_layer: Option<TextureLayer>,
    /// Sets environment mask.
    pub environment_mask_layer: Option<TextureLayer>,
    /// Supplies the first alpha-mask layer controlling opacity.
    pub alpha_layer1: Option<TextureLayer>,
    /// Supplies the second alpha-mask layer controlling opacity.
    pub alpha_layer2: Option<TextureLayer>,
    /// Supplies the map defining perturbed surface normals.
    pub normal_layer: Option<TextureLayer>,
    /// Supplies the height map used for parallax displacement.
    pub height_layer: Option<TextureLayer>,
    /// Supplies the map supplying precomputed lighting.
    pub light_map_layer: Option<TextureLayer>,
    /// Supplies the map modulating illumination with ambient occlusion.
    pub ambient_occlusion_layer: Option<TextureLayer>,
    /// Supplies the mask controlling additional normal map 1’s contribution.
    pub normal_blend1_mask_layer: Option<TextureLayer>,
    /// Supplies the mask controlling additional normal map 2’s contribution.
    pub normal_blend2_mask_layer: Option<TextureLayer>,
    /// Supplies additional normal map 1 for blending with the base normal map.
    pub normal_blend1_layer: Option<TextureLayer>,
    /// Supplies additional normal map 2 for blending with the base normal map.
    pub normal_blend2_layer: Option<TextureLayer>,
    /// Sets material class (unit, building, etc.).
    pub material_class: MaterialClass,
    /// Sets layer blend operation.
    pub layer_blend_mode: LayerBlendOp,
    /// Selects the compositing operation applied to emissive layer 1.
    pub emissive_blend_mode1: LayerBlendOp,
    /// Selects the compositing operation applied to emissive layer 2.
    pub emissive_blend_mode2: LayerBlendOp,
    /// Selects the behavior used for specular computation mode.
    pub specular_mode: SpecularMode,
    /// Controls parallax height through its animation value.
    pub parallax_height: Property<f32>,
    /// Controls motion blur amount through its animation value.
    pub motion_blur_amount: Property<f32>,
    /// Weights the contributions of the additional blended normal maps.
    pub normal_blend_factors: Vec<Property<f32>>,
}
