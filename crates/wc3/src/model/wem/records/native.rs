//! Typed native records for indexed WEM payloads and inline values.
use super::*;
use crate::model::wem::{self, Bool, InlineList, InlineOption, NativeAbi, Record, Ref};
/// Typed m3 anim ref payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3AnimRef<T> {
    /// Selects the interpolation used by this animated property.
    pub interp_type: u16,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: u16,
    /// Identifies the animation channel driving this property.
    pub anim_id: u32,
    /// Supplies the value before an animation channel overrides it.
    pub init_value: T,
    /// Supplies the reset value of this property.
    pub null_value: T,
    /// Stores uninterpreted animation-reference metadata.
    pub unused: i32,
}

/// Typed m3 color bgra payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3ColorBGRA {
    /// Stores the blue color component as an encoded byte.
    pub b: u8,
    /// Stores the green color component as an encoded byte.
    pub g: u8,
    /// Stores the red color component as an encoded byte.
    pub r: u8,
    /// Stores opacity as an encoded byte.
    pub a: u8,
}

/// Typed m3 texture layer payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3TextureLayer {
    /// Identifies this texture layer within the material setup.
    pub id: u32,
    /// Links to the CHAR bytes containing the texture filename.
    pub texture_path: Ref<targets::Character>,
    /// Controls color tint through its animation value.
    pub color: M3AnimRef<M3ColorBGRA>,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: M3TextureLayerFlag,
    /// Selects the behavior used for uv mapping source.
    pub uv_mapping: M3UVMappingMode,
    /// Selects the behavior used for channel selection.
    pub color_type: M3ColorChannelSelect,
    /// Sets RGB multiply factor.
    pub rgb_multiply: M3AnimRef<f32>,
    /// Sets RGB additive factor.
    pub rgb_add: M3AnimRef<f32>,
    /// Stores the POC texture selector; its target namespace is not established.
    pub poc_texture: u32,
    /// Sets noise amplitude.
    pub noise_amplitude: f32,
    /// Sets noise frequency.
    pub noise_frequency: f32,
    /// Sets texture source override.
    pub texture_source: u32,
    /// Sets AVI playback frame rate.
    pub avi_frame_rate: u32,
    /// Sets AVI start frame.
    pub avi_start: u32,
    /// Sets AVI stop frame.
    pub avi_stop: u32,
    /// Selects the behavior used for avi loop mode.
    pub avi_loop: u32,
    /// Selects the behavior used for avi sync mode.
    pub avi_sync: u32,
    /// Sets AVI play control.
    pub avi_play: M3AnimRef<u32>,
    /// Sets AVI restart control.
    pub avi_restart: M3AnimRef<u32>,
    /// Sets flipbook grid rows.
    pub flipbook_rows: u32,
    /// Sets flipbook grid columns.
    pub flipbook_columns: u32,
    /// Controls flipbook frame index through its animation value.
    pub current_frame: M3AnimRef<u16>,
    /// Controls UV offset through its animation value.
    pub uv_offset: M3AnimRef<[f32; 2]>,
    /// Controls UV rotation angles through its animation value.
    pub uv_angle: M3AnimRef<[f32; 3]>,
    /// Controls UV tiling through its animation value.
    pub uv_tiling: M3AnimRef<[f32; 2]>,
    /// Controls w offset (3D textures) through its animation value.
    pub w_offset: M3AnimRef<f32>,
    /// Controls w tiling (3D textures) through its animation value.
    pub w_tiling: M3AnimRef<f32>,
    /// Controls the texture contribution’s alpha through an animation value.
    pub map_alpha: M3AnimRef<f32>,
    /// Sets tri-planar UV offset.
    pub triplanar_offset: M3AnimRef<[f32; 3]>,
    /// Sets tri-planar UV scale.
    pub triplanar_scale: M3AnimRef<[f32; 3]>,
    /// Selects the layer supplying shared UV settings; all bits set selects this layer’s own settings.
    pub uv_source_related: u32,
    /// Selects the behavior used for fresnel effect mode.
    pub fresnel_mode: M3FresnelMode,
    /// Sets fresnel exponent (edge sharpness).
    pub fresnel_exponent: f32,
    /// Sets fresnel minimum intensity.
    pub fresnel_min: f32,
    /// Sets fresnel maximum intensity.
    pub fresnel_max: f32,
    /// Sets fresnel UV translation.
    pub fresnel_translation: [f32; 3],
    /// Sets fresnel mask vector.
    pub fresnel_mask: [f32; 3],
    /// Sets fresnel UV rotation.
    pub fresnel_rotation: [f32; 2],
    /// Sets UV density hint.
    pub uv_density: u32,
}

/// Typed m3 standard payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3Standard {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Stores the bit mask governing additional.
    pub additional_flags: M3MaterialAdditionalFlag,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: M3MaterialFlag,
    /// Selects the behavior used for alpha blend mode.
    pub blend_mode: M3BlendMode,
    /// Sets render priority (lower = earlier).
    pub priority: i32,
    /// Selects render-to-texture channels through a bit mask.
    pub rtt_channels: u32,
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
    pub diffuse_layer: InlineOption<M3TextureLayer>,
    /// Supplies the decal composited over the base material.
    pub decal_layer: InlineOption<M3TextureLayer>,
    /// Supplies the map controlling specular response.
    pub specular_layer: InlineOption<M3TextureLayer>,
    /// Supplies the map controlling the surface gloss.
    pub gloss_layer: InlineOption<M3TextureLayer>,
    /// Supplies the first self-illumination layer.
    pub emissive_layer1: InlineOption<M3TextureLayer>,
    /// Supplies the second self-illumination layer.
    pub emissive_layer2: InlineOption<M3TextureLayer>,
    /// Supplies the map supplying environment reflections.
    pub environment_layer: InlineOption<M3TextureLayer>,
    /// Sets environment mask.
    pub environment_mask_layer: InlineOption<M3TextureLayer>,
    /// Supplies the first alpha-mask layer controlling opacity.
    pub alpha_layer1: InlineOption<M3TextureLayer>,
    /// Supplies the second alpha-mask layer controlling opacity.
    pub alpha_layer2: InlineOption<M3TextureLayer>,
    /// Supplies the map defining perturbed surface normals.
    pub normal_layer: InlineOption<M3TextureLayer>,
    /// Supplies the height map used for parallax displacement.
    pub height_layer: InlineOption<M3TextureLayer>,
    /// Supplies the map supplying precomputed lighting.
    pub light_map_layer: InlineOption<M3TextureLayer>,
    /// Supplies the map modulating illumination with ambient occlusion.
    pub ambient_occlusion_layer: InlineOption<M3TextureLayer>,
    /// Supplies the mask controlling additional normal map 1’s contribution.
    pub normal_blend1_mask_layer: InlineOption<M3TextureLayer>,
    /// Supplies the mask controlling additional normal map 2’s contribution.
    pub normal_blend2_mask_layer: InlineOption<M3TextureLayer>,
    /// Supplies additional normal map 1 for blending with the base normal map.
    pub normal_blend1_layer: InlineOption<M3TextureLayer>,
    /// Supplies additional normal map 2 for blending with the base normal map.
    pub normal_blend2_layer: InlineOption<M3TextureLayer>,
    /// Sets material class (unit, building, etc.).
    pub material_class: M3MaterialClass,
    /// Sets layer blend operation.
    pub layer_blend_mode: M3LayerBlendOp,
    /// Selects the compositing operation applied to emissive layer 1.
    pub emissive_blend_mode1: M3LayerBlendOp,
    /// Selects the compositing operation applied to emissive layer 2.
    pub emissive_blend_mode2: M3LayerBlendOp,
    /// Selects the behavior used for specular computation mode.
    pub specular_mode: M3SpecularMode,
    /// Controls parallax height through its animation value.
    pub parallax_height: M3AnimRef<f32>,
    /// Controls motion blur amount through its animation value.
    pub motion_blur_amount: M3AnimRef<f32>,
    /// Weights the contributions of the additional blended normal maps.
    pub normal_blend_factors: InlineList<M3AnimRef<f32>>,
}

/// Typed m3 displacement payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3Displacement {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// The purpose of this field has not been established.
    pub unknown: u32,
    /// Controls displacement strength through its animation value.
    pub strength: M3AnimRef<f32>,
    /// Supplies the map defining the displacement direction.
    pub normal_map: InlineOption<M3TextureLayer>,
    /// Supplies the mask scaling displacement strength.
    pub strength_map: InlineOption<M3TextureLayer>,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: u32,
    /// Sets render priority (lower = earlier).
    pub priority: u32,
}

/// Typed m3 composite section payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3CompositeSection {
    /// Selects an entry in the material array chosen by `material_type`.
    pub material_index: u32,
    /// Controls blend weight through its animation value.
    pub map_multiplier: M3AnimRef<f32>,
}

/// Typed m3 composite payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3Composite {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Sets render priority (lower = earlier).
    pub priority: u32,
    /// Sets sub-material sections (CMS_).
    pub sections: InlineList<M3CompositeSection>,
}

/// Typed m3 terrain payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3Terrain {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Supplies the texture layer used for the terrain contribution.
    pub terrain_map: InlineOption<M3TextureLayer>,
    /// The purpose of this field has not been established.
    pub unknown: u32,
}

/// Typed m3 volume payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3Volume {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Selects the behavior used for alpha blend mode.
    pub blend_mode: u32,
    /// Selects the behavior used for density falloff type.
    pub falloff_type: M3VolumeFalloffType,
    /// Controls density through its animation value.
    pub density: M3AnimRef<f32>,
    /// Supplies the map supplying color.
    pub color_map: InlineOption<M3TextureLayer>,
    /// Supplies the first noise texture.
    pub noise_map1: InlineOption<M3TextureLayer>,
    /// Supplies the second noise texture.
    pub noise_map2: InlineOption<M3TextureLayer>,
    /// Sets alpha test threshold.
    pub alpha_threshold: u32,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: u32,
}

/// Typed m3 volume noise payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3VolumeNoise {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Selects the behavior used for density falloff type.
    pub falloff_type: M3VolumeFalloffType,
    /// Selects the transparency behavior for a camera inside or outside the volume.
    pub draw_transparency: M3VolumeNoiseCameraMode,
    /// Controls density through its animation value.
    pub density: M3AnimRef<f32>,
    /// Controls near-plane clip through its animation value.
    pub near_plane: M3AnimRef<f32>,
    /// Controls falloff distance through its animation value.
    pub falloff: M3AnimRef<f32>,
    /// Supplies the map supplying color.
    pub color_map: InlineOption<M3TextureLayer>,
    /// Supplies the first noise texture.
    pub noise_map1: InlineOption<M3TextureLayer>,
    /// Supplies the second noise texture.
    pub noise_map2: InlineOption<M3TextureLayer>,
    /// Controls noise scroll rate through its animation value.
    pub scroll_rate: M3AnimRef<[f32; 3]>,
    /// Controls volume position through its animation value.
    pub position: M3AnimRef<[f32; 3]>,
    /// Controls volume scale through its animation value.
    pub scale: M3AnimRef<[f32; 3]>,
    /// Controls volume rotation through its animation value.
    pub rotation: M3AnimRef<[f32; 3]>,
    /// Sets alpha test threshold.
    pub alpha_threshold: u32,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: M3VolumeNoiseMaterialFlag,
}

/// Typed m3 creep payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3Creep {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Supplies the mask delimiting the creep effect.
    pub mask_map: InlineOption<M3TextureLayer>,
    /// Controls the lower creep-mask parameter; its numerical interpretation is not established.
    pub creep_low: u32,
}

/// Typed m3 stb payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3Stb {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Supplies the surface diffuse-color map.
    pub diffuse_map: InlineOption<M3TextureLayer>,
    /// Supplies the map defining the displacement direction.
    pub normal_map: InlineOption<M3TextureLayer>,
    /// Supplies the map controlling specular response.
    pub specular_map: InlineOption<M3TextureLayer>,
}

/// Typed m3 reflection payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3Reflection {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// The purpose of this field has not been established.
    pub unknown: u32,
    /// Controls reflection strength through its animation value.
    pub reflection_strength: M3AnimRef<f32>,
    /// Controls displacement strength through its animation value.
    pub displacement_strength: M3AnimRef<f32>,
    /// Controls reflection offset through its animation value.
    pub reflection_offset: M3AnimRef<f32>,
    /// Controls blur angle through its animation value.
    pub blur_angle: M3AnimRef<f32>,
    /// Controls max blur distance through its animation value.
    pub blur_distance_max: M3AnimRef<f32>,
    /// Supplies the map supplying reflected color.
    pub reflection_map: InlineOption<M3TextureLayer>,
    /// Supplies the map defining displacement.
    pub displacement_map: InlineOption<M3TextureLayer>,
    /// Supplies the map controlling blur.
    pub blur_map: InlineOption<M3TextureLayer>,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: M3ReflectionMaterialFlag,
    /// The meaning of the unknown2 storage has not been established; its encoded bits are preserved.
    pub unknown2: u32,
}

/// Typed m3 sub flare payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3SubFlare {
    /// Sets flare element index.
    pub index: u32,
    /// Controls volume position through its animation value.
    pub position: f32,
    /// Sets the flare element’s width and height.
    pub size_xy: [f32; 2],
    /// Scales the flare element along its width and height.
    pub scale_xy: [f32; 2],
    /// Sets fade-in range (start, end).
    pub fade_in: [f32; 2],
    /// Sets fade-out range (start, end).
    pub fade_out: [f32; 2],
    /// Sets flare color and alpha.
    pub color_alpha: M3ColorBGRA,
    /// Controls whether the element points toward the flare’s center.
    pub face_center: u32,
    /// Sets offset from flare center.
    pub offset: [f32; 2],
}

/// Typed m3 lens flare payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3LensFlare {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Sets flare texture atlas.
    pub flare_map: InlineOption<M3TextureLayer>,
    /// Supplies the mask delimiting the creep effect.
    pub mask_map: InlineOption<M3TextureLayer>,
    /// Sets sub-flare elements (LFSB).
    pub sub_flares: InlineList<M3SubFlare>,
    /// Sets flipbook grid columns.
    pub columns: u32,
    /// Sets flipbook grid rows.
    pub rows: u32,
    /// Sets distance fade start.
    pub distance_fade: f32,
    /// Links to the CHAR bytes naming the flare library.
    pub lib_name: Ref<targets::Character>,
    /// Controls intensity through its animation value.
    pub intensity: M3AnimRef<f32>,
    /// Controls color tint through its animation value.
    pub color: M3AnimRef<M3ColorBGRA>,
    /// Controls HDR multiplier through its animation value.
    pub hdr: M3AnimRef<f32>,
    /// Controls size through its animation value.
    pub size: M3AnimRef<f32>,
}

/// Typed m3 data driven payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3DataDriven {
    /// Links to the CHAR bytes identifying the material.
    pub material_name: Ref<targets::Character>,
    /// Supplies U32_ CRC32 identifiers for shader fragments in their linking order.
    pub fragment_hashes: InlineList<u32>,
    /// Supplies additional U32_ shader hashes available from revision 2.
    pub extra_hashes: InlineList<u32>,
    /// Supplies the CHAR-encoded dictionary of shader properties.
    pub property_blob: InlineList<u8>,
    /// Links to SCHR texture filenames selected by texture properties in the dictionary.
    pub texture_paths: InlineList<Ref<targets::Character>>,
    /// Unknown-purpose scalar reported as 1.0, 1.5, or 2.0 in the reference samples.
    pub unknown108: f32,
    /// Unknown-purpose scalar reported as 1.0 in the reference samples.
    pub unknown112: f32,
    /// The meaning of the unknown116 storage has not been established; its encoded bits are preserved.
    pub unknown116: f32,
    /// Stores the shader-permutation name’s CRC32; zero requests calculation during loading.
    pub effect_name_hash: u32,
    /// The meaning of the unknown124 storage has not been established; its encoded bits are preserved.
    pub unknown124: u32,
    /// Uninterpreted storage reported as zero in the reference samples.
    pub padding128: u32,
    /// The meaning of the unknown132 storage has not been established; its encoded bits are preserved.
    pub unknown132: i32,
    /// Contains packed bits whose individual meanings are not established.
    pub unknown136: u32,
    /// The meaning of the unknown140 storage has not been established; its encoded bits are preserved.
    pub unknown140: u32,
    /// The meaning of the unknown144 storage has not been established; its encoded bits are preserved.
    pub unknown144: u32,
    /// The meaning of the unknown148 storage has not been established; its encoded bits are preserved.
    pub unknown148: u8,
    /// Caches alpha and Fresnel settings derived from the property dictionary by the loader.
    pub alpha_fresnel_flags: u8,
    /// Selects the shader-family prefix used to form the permutation name.
    pub shader_type: M3MaterialShaderType,
    /// The meaning of the unknown151 storage has not been established; its encoded bits are preserved.
    pub unknown151: u8,
    /// Stores another permutation identifier; all bits set indicates no second permutation (revision 3 onward).
    pub effect_name_hash2: u32,
    /// Stores a third permutation identifier; all bits set indicates no third permutation (revision 3 onward).
    pub effect_name_hash3: u32,
}

/// Typed m3 data driven property payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3DataDrivenProperty {
    /// Identifies a shader property by its source name hash.
    pub name_hash: u32,
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Stores the property’s encoded value bytes.
    pub data: InlineList<u8>,
}

/// Typed m3 data driven group payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3DataDrivenGroup {
    /// Identifies a shader property by its source name hash.
    pub name_hash: u32,
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Stores the shader-property entries inline.
    pub properties: InlineList<M3DataDrivenProperty>,
}

/// Typed m3 data driven properties payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3DataDrivenProperties {
    /// Stores the source material’s grouping metadata inline.
    pub groups: InlineList<M3DataDrivenGroup>,
}

/// Typed mdx sub texture payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct MdxSubTexture {
    /// Selects the source texture entry.
    pub texture_id: u32,
    /// Selects the native texture-input slot.
    pub slot: MdxSlotType,
}

/// Typed mdx layer payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct MdxLayer {
    /// Selects the native layer’s filtering or blending mode.
    pub filter_mode: MdxFilterMode,
    /// Stores the native layer’s shading options as bits.
    pub shading_flags: MdxShadingFlag,
    /// Selects the source texture entry.
    pub texture_id: u32,
    /// Selects the source texture-transform animation.
    pub texture_animation_id: u32,
    /// Selects the texture-coordinate channel.
    pub coord_id: u32,
    /// Scales the native layer’s opacity.
    pub alpha: f32,
    /// Scales self-illumination in the native layer.
    pub emissive_gain: f32,
    /// Supplies the color of the native Fresnel contribution.
    pub fresnel_color: [f32; 3],
    /// Scales opacity in the native Fresnel contribution.
    pub fresnel_opacity: f32,
    /// Weights team color in the native Fresnel contribution.
    pub fresnel_team_color: f32,
    /// Selects the shader-family prefix used to form the permutation name.
    pub shader_type: MdxShaderType,
    /// Marks a layer using the high-definition material representation.
    pub is_hd: Bool,
    /// Stores the additional texture inputs of a high-definition layer.
    pub sub_textures: InlineList<MdxSubTexture>,
}

/// Typed mdx material payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct MdxMaterial {
    /// Records the source format version that supplied this native material.
    pub source_version: u32,
    /// Sets the source material’s draw-order priority plane.
    pub priority_plane: i32,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: MdxMaterialFlag,
    /// Links to the native shader name.
    pub shader: Ref<targets::Character>,
    /// Stores the native material’s texture layers inline.
    pub layers: InlineList<MdxLayer>,
}
impl Record for MdxMaterial {
    const TAG: [u8; 4] = *b"NMDX";
    const VERSION: u32 = 1;
}

/// Typed m2 texture unit payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M2TextureUnit {
    /// Selects the source texture entry.
    pub texture: u16,
    /// Selects the source texture-coordinate channel.
    pub uv_set: u16,
    /// Contains the source texture-coordinate transform.
    pub transform: u16,
    /// Scales the texture input’s contribution.
    pub weight: u16,
    /// Selects the native texture’s source or replacement mechanism.
    pub texture_type: u8,
    /// Identifies a source texture supplied by a replaceable-texture mechanism.
    pub replaceable_id: u8,
}

/// Typed m2 material payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M2Material {
    /// Records the source format version that supplied this native material.
    pub source_version: u32,
    /// Stores the source material’s rendering options.
    pub render_flags: u16,
    /// Selects the native material’s compositing mode.
    pub blending_mode: u16,
    /// Selects the source shader.
    pub shader_id: u16,
    /// Sets the source material’s draw-order priority plane.
    pub priority_plane: i16,
    /// Identifies the source material layer.
    pub material_layer: u16,
    /// Selects the source animated color entry.
    pub color_index: i16,
    /// Stores the source batch’s drawing options.
    pub batch_flags: u8,
    /// Stores the texture units forming the source batch inline.
    pub units: InlineList<M2TextureUnit>,
}
impl Record for M2Material {
    const TAG: [u8; 4] = *b"NM2_";
    const VERSION: u32 = 1;
}

/// Typed d3 asset ref payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct D3AssetRef {
    /// Identifies the source SNO resource within its group.
    pub id: i32,
    /// Identifies the source SNO resource group.
    pub group: D3Group,
}

/// Typed d3 material colors payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct D3MaterialColors {
    /// Supplies the source material’s diffuse color.
    pub diffuse: [f32; 4],
    /// Supplies the source material’s specular color.
    pub specular: [f32; 4],
    /// Supplies the source material’s self-illumination color.
    pub emissive: [f32; 4],
    /// Supplies the source material’s ambient color.
    pub ambient: [f32; 4],
    /// Controls the concentration of the source specular highlight.
    pub shininess: f32,
    /// Stores the source material options.
    pub material_flags: i32,
}

/// Typed d3 tex anim payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct D3TexAnim {
    /// Sets the magnitude of this source texture animation.
    pub amount: f32,
    /// Stores the first rate parameter of the source texture animation.
    pub rate0: f32,
    /// Stores the second rate parameter of the source texture animation.
    pub rate1: f32,
}

/// Typed d3 texture entry payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct D3TextureEntry {
    /// Selects the source texture-animation operation.
    pub type_: i32,
    /// The purpose of this source field is unresolved; its encoded value is preserved.
    pub dw_unknown04: i32,
    /// Identifies the source texture resource by SNO group and ID.
    pub texture: D3AssetRef,
    /// Selects the source UV transformation convention.
    pub uv_transform_mode: i32,
    /// Stores one row of the source UV transformation matrix.
    pub uv_row0: [f32; 4],
    /// Stores one row of the source UV transformation matrix.
    pub uv_row1: [f32; 4],
    /// Stores one row of the source UV transformation matrix.
    pub uv_row2: [f32; 4],
    /// Stores one row of the source UV transformation matrix.
    pub uv_row3: [f32; 4],
    /// Stores the U-coordinate animation parameters.
    pub anim_u: D3TexAnim,
    /// Stores the V-coordinate animation parameters.
    pub anim_v: D3TexAnim,
    /// Stores the texture rotation animation parameters.
    pub anim_rotate: D3TexAnim,
    /// The purpose of this source field is unresolved; its encoded value is preserved.
    pub anim3: D3TexAnim,
    /// The purpose of this source field is unresolved; its encoded value is preserved.
    pub anim4: D3TexAnim,
    /// The purpose of this source field is unresolved; its encoded value is preserved.
    pub anim5: D3TexAnim,
    /// Stores source UV-mapping options.
    pub uv_flags: i32,
    /// The purpose of this source field is unresolved; its encoded value is preserved.
    pub dw_unknown9_c: i32,
}

/// Typed d3 uber material payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct D3UberMaterial {
    /// Identifies the source shader-map resource.
    pub shader_map: D3AssetRef,
    /// Contains the source material colors.
    pub colors: D3MaterialColors,
    /// Stores the source material’s texture inputs inline.
    pub textures: InlineList<D3TextureEntry>,
}

/// Typed d3 tag value payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct D3TagValue {
    /// Selects the encoded type of this source shader parameter.
    pub value_type: i32,
    /// Identifies the source shader parameter.
    pub tag_id: u32,
    /// Stores the encoded source shader-tag value.
    pub value: u32,
}

/// Typed d3 material asset payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct D3MaterialAsset {
    /// Identifies the source SNO resource.
    pub sno_id: i32,
    /// The purpose of this source field is unresolved; its encoded value is preserved.
    pub dw_unknown0_c: i32,
    /// Stores source shader-parameter entries inline.
    pub shader_params: InlineList<D3TagValue>,
    /// Contains the source shader’s additional material state.
    pub uber: D3UberMaterial,
}

/// Typed d3 render params payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct D3RenderParams {
    /// Selects the source surface-culling mode.
    pub dw_cull_mode: i32,
    /// Controls updates to the depth buffer.
    pub dw_z_write_enable: i32,
    /// Selects the source depth comparison.
    pub dw_z_func: i32,
    /// Offsets the source depth comparison.
    pub fl_depth_bias: f32,
    /// The purpose of this source field is unresolved; its encoded value is preserved.
    pub fl_unknown10: f32,
    /// Controls use of the stencil buffer.
    pub dw_stencil_enable: i32,
    /// Selects the stencil comparison.
    pub dw_stencil_func: i32,
    /// Supplies the value used for stencil comparisons.
    pub dw_stencil_ref: i32,
    /// Selects the stencil operation after successful stencil and depth tests.
    pub dw_stencil_pass: i32,
    /// Selects the stencil operation after a failed stencil test.
    pub dw_stencil_fail: i32,
    /// Selects the stencil operation after a failed depth test.
    pub dw_stencil_z_fail: i32,
    /// Controls alpha testing.
    pub dw_alpha_test_enable: i32,
    /// Selects the alpha comparison.
    pub dw_alpha_func: i32,
    /// Supplies the encoded byte used as the alpha-test reference.
    pub b_alpha_ref: u8,
    /// Controls conversion of alpha to sample coverage.
    pub dw_alpha_to_coverage: i32,
    /// Controls application of fog.
    pub dw_fog_enable: i32,
    /// Selects the rasterization fill mode.
    pub dw_fill_mode: i32,
    /// Controls writes to color components.
    pub dw_color_write_enable: i32,
    /// Controls writes to opacity.
    pub dw_alpha_write_enable: i32,
    /// Controls alpha blending.
    pub dw_alpha_blend_enable: i32,
    /// Selects the operation combining source and destination colors.
    pub dw_blend_op: i32,
    /// Selects the source blend factor.
    pub dw_src_blend: i32,
    /// Selects the destination blend factor.
    pub dw_dest_blend: i32,
    /// Supplies the constant color used by blending.
    pub dw_constant_color: i32,
}

/// Typed d3 texture stage payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct D3TextureStage {
    /// Identifies the source content stage.
    pub content_stage: i32,
    /// Selects texture addressing along U.
    pub dw_address_u: i32,
    /// Selects texture addressing along V.
    pub dw_address_v: i32,
    /// Selects texture addressing along W.
    pub dw_address_w: i32,
    /// Selects source texture filtering.
    pub dw_filter: i32,
    /// Offsets selection of the texture mipmap level.
    pub fl_mip_map_lod_bias: f32,
}

/// Typed d3 shader tag value payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct D3ShaderTagValue {
    /// Selects the encoded type of this source shader parameter.
    pub value_type: i32,
    /// Identifies the source shader parameter.
    pub tag_id: u32,
    /// Stores the encoded source shader-tag value.
    pub value: u32,
}

/// Typed d3 render state payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct D3RenderState {
    /// The purpose of this source field is unresolved; its encoded value is preserved.
    pub dw_unknown00: i32,
    /// The purpose of this source field is unresolved; its encoded value is preserved.
    pub dw_unknown04: i32,
    /// Contains the source draw-state settings.
    pub render_params: D3RenderParams,
    /// The purpose of this source field is unresolved; its encoded value is preserved.
    pub dw_unknown68: i32,
    /// Records the number of active source texture stages.
    pub texture_stage_count: i32,
    /// Contains the source texture-stage states.
    pub texture_stages: InlineList<D3TextureStage>,
    /// Stores the source material pass options.
    pub pass_flags: i32,
    /// Links to the name of the source effect file.
    pub effect_file: Ref<targets::Character>,
    /// Links to the source vertex-shader entry name.
    pub vertex_shader_entry: Ref<targets::Character>,
    /// Links to the source pixel-shader entry name.
    pub pixel_shader_entry: Ref<targets::Character>,
    /// Stores source shader-parameter entries inline.
    pub shader_params: InlineList<D3ShaderTagValue>,
}

/// Typed m2 batch payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M2Batch {
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: u8,
    /// Sets the source material’s draw-order priority plane.
    pub priority_plane: i8,
    /// Selects the source shader.
    pub shader_id: u16,
    /// Selects the source skin section.
    pub skin_section_index: u16,
    /// Selects the source geoset.
    pub geoset_index: u16,
    /// Selects the source animated color entry.
    pub color_index: i16,
    /// Selects an entry in the material array chosen by `material_type`.
    pub material_index: u16,
    /// Identifies the source material layer.
    pub material_layer: u16,
    /// Records the number of source texture inputs.
    pub texture_count: u16,
    /// Selects the source texture combination.
    pub texture_combo_index: u16,
    /// Selects the source texture-coordinate combination.
    pub texture_coord_combo_index: u16,
    /// Selects the source texture-weight combination.
    pub texture_weight_combo_index: u16,
    /// Selects the source texture-transform combination.
    pub texture_transform_combo_index: u16,
}

/// Typed m2 texture payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M2Texture {
    /// Selects the source texture-animation operation.
    pub type_: u32,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: u32,
    /// Links to the source texture filename.
    pub filename: Ref<targets::Character>,
}

/// Typed m2 render record payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M2RenderRecord {
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: u16,
    /// Selects the native material’s compositing mode.
    pub blending_mode: u16,
}

/// Static field selections for `D3Material` section or inline context revisions.
pub trait D3MaterialDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `variant_flags`; `()` omits this slot.
    type VariantFlags: NativeField;
    /// Representation of `shader_params`; `()` omits this slot.
    type ShaderParams: NativeField;
    /// Representation of `opaque_passes`; `()` omits this slot.
    type OpaquePasses: NativeField;
    /// Representation of `translucent_passes`; `()` omits this slot.
    type TranslucentPasses: NativeField;
}

impl D3MaterialDialect for V1 {
    const VERSION: u32 = 1;
    type VariantFlags = ();
    type ShaderParams = InlineList<D3TagValue>;
    type OpaquePasses = InlineList<D3RenderState>;
    type TranslucentPasses = InlineList<D3RenderState>;
}

impl D3MaterialDialect for V2 {
    const VERSION: u32 = 2;
    type VariantFlags = i32;
    type ShaderParams = InlineList<D3TagValue>;
    type OpaquePasses = InlineList<D3RenderState>;
    type TranslucentPasses = InlineList<D3RenderState>;
}

/// Typed d3 material payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct D3Material<D: D3MaterialDialect> {
    /// Records the source format version that supplied this native material.
    pub source_version: u32,
    /// Stores the source material variant options.
    pub variant_flags: D::VariantFlags,
    /// Contains the source shader’s additional material state.
    pub uber: D3UberMaterial,
    /// Contains the source base material state.
    pub base_material: D3AssetRef,
    /// Contains the source cloth-material state.
    pub cloth: D3AssetRef,
    /// Stores source shader-parameter entries inline.
    pub shader_params: D::ShaderParams,
    /// Stores the source opaque drawing passes inline.
    pub opaque_passes: D::OpaquePasses,
    /// Stores the source translucent drawing passes inline.
    pub translucent_passes: D::TranslucentPasses,
}
impl<D: D3MaterialDialect> Record for D3Material<D> {
    const TAG: [u8; 4] = *b"ND3_";
    const VERSION: u32 = D::VERSION;
}

/// Static field selections for `M3Material` section or inline context revisions.
pub trait M3MaterialDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `source_version`; `()` omits this slot.
    type SourceVersion: NativeField;
}

impl M3MaterialDialect for V0 {
    const VERSION: u32 = 0;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V1 {
    const VERSION: u32 = 1;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V2 {
    const VERSION: u32 = 2;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V3 {
    const VERSION: u32 = 3;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V4 {
    const VERSION: u32 = 4;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V5 {
    const VERSION: u32 = 5;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V6 {
    const VERSION: u32 = 6;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V7 {
    const VERSION: u32 = 7;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V8 {
    const VERSION: u32 = 8;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V9 {
    const VERSION: u32 = 9;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V10 {
    const VERSION: u32 = 10;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V11 {
    const VERSION: u32 = 11;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V12 {
    const VERSION: u32 = 12;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V13 {
    const VERSION: u32 = 13;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V14 {
    const VERSION: u32 = 14;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V15 {
    const VERSION: u32 = 15;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V16 {
    const VERSION: u32 = 16;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V17 {
    const VERSION: u32 = 17;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V18 {
    const VERSION: u32 = 18;
    type SourceVersion = u32;
}

impl M3MaterialDialect for V19 {
    const VERSION: u32 = 19;
    type SourceVersion = u32;
}

/// Typed m3 material payload in its serialized field order.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct M3Material<D: M3MaterialDialect, A: NativeLayout> {
    /// Records the source format version that supplied this native material.
    pub source_version: D::SourceVersion,
    /// Identifies the native shader family.
    pub kind: M3MaterialKind,
    /// Marks a material authored using the data-driven shader representation.
    pub authored_data_driven: Bool,
    /// Carries the native material-body discriminator and its payload.
    pub body: NativeM3Body<A>,
    /// Contains the source data-driven property dictionary.
    pub data_driven_properties: InlineOption<M3DataDrivenProperties>,
    /// Contains a recovered standard-material representation when supplied.
    pub restored_standard: InlineOption<M3Standard>,
}
impl<D: M3MaterialDialect, A: NativeLayout> Record for M3Material<D, A> {
    const NATIVE_ABI: Option<NativeAbi> = Some(A::ABI);
    const TAG: [u8; 4] = *b"NM3_";
    const VERSION: u32 = D::VERSION;
}
