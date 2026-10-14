//! StarCraft shader declarations whose external lookup domains remain explicit.
use crate::model::semantic::profiles::m3::values::{
    AdditionalMaterialFlags, BlendMode, ColorChannel, DisplacementFlags, FresnelMode, LayerBlendOp,
    LayerFlags, MaterialClass, MaterialFlags, MaterialKind, ReflectionFlags, RttChannels,
    ShaderFresnelFlags, ShaderType, SpecularMode, UvMappingMode, VolumeCameraMode, VolumeFalloff,
    VolumeFlags, VolumeNoiseFlags,
};
use crate::model::semantic::profiles::m3::AnimationControl;
use crate::model::semantic::{Error, Link, TextureId, ValidateLinks};
use std::borrow::Cow;
/// Property values and controls selecting an external StarCraft animation identifier.
#[derive(Clone, Debug, Default)]
pub struct ExternalAnimation<T> {
    /// Value before the external program overrides this input.
    pub initial: T,
    /// Value used when the input resets.
    pub reset: T,
    /// External animation selection and evaluation controls.
    pub control: AnimationControl,
}
/// Authored StarCraft texture layer inputs.
#[derive(Clone, Debug, Default)]
pub struct TextureLayer {
    /// Identifies this texture layer within the material setup.
    pub id: u32,
    /// Shared texture identity; layer flags retain the authored sampling overrides.
    pub texture: Option<TextureId>,
    /// Controls color tint through its animation value.
    pub color: ExternalAnimation<[u8; 4]>,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: LayerFlags,
    /// Selects the behavior used for uv mapping source.
    pub uv_mapping: UvMappingMode,
    /// Selects the behavior used for channel selection.
    pub color_type: ColorChannel,
    /// Sets RGB multiply factor.
    pub rgb_multiply: ExternalAnimation<f32>,
    /// Sets RGB additive factor.
    pub rgb_add: ExternalAnimation<f32>,
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
    pub avi_play: ExternalAnimation<u32>,
    /// Sets AVI restart control.
    pub avi_restart: ExternalAnimation<u32>,
    /// Sets flipbook grid rows.
    pub flipbook_rows: u32,
    /// Sets flipbook grid columns.
    pub flipbook_columns: u32,
    /// Controls flipbook frame index through its animation value.
    pub current_frame: ExternalAnimation<u16>,
    /// Controls UV offset through its animation value.
    pub uv_offset: ExternalAnimation<[f32; 2]>,
    /// Controls UV rotation angles through its animation value.
    pub uv_angle: ExternalAnimation<[f32; 3]>,
    /// Controls UV tiling through its animation value.
    pub uv_tiling: ExternalAnimation<[f32; 2]>,
    /// Controls w offset (3D textures) through its animation value.
    pub w_offset: ExternalAnimation<f32>,
    /// Controls w tiling (3D textures) through its animation value.
    pub w_tiling: ExternalAnimation<f32>,
    /// Controls the texture contribution’s alpha through an animation value.
    pub map_alpha: ExternalAnimation<f32>,
    /// Sets tri-planar UV offset.
    pub triplanar_offset: ExternalAnimation<[f32; 3]>,
    /// Sets tri-planar UV scale.
    pub triplanar_scale: ExternalAnimation<[f32; 3]>,
    /// Selects the layer supplying shared UV settings; all bits set selects this layer’s own settings.
    pub uv_source_related: u32,
    /// Selects the behavior used for fresnel effect mode.
    pub fresnel_mode: FresnelMode,
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
/// Authored StarCraft standard inputs.
#[derive(Clone, Debug, Default)]
pub struct Standard<'a> {
    /// Names the text identifying the material.
    pub name: Cow<'a, str>,
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
    pub parallax_height: ExternalAnimation<f32>,
    /// Controls motion blur amount through its animation value.
    pub motion_blur_amount: ExternalAnimation<f32>,
    /// Weights the contributions of the additional blended normal maps.
    pub normal_blend_factors: Vec<ExternalAnimation<f32>>,
}
/// Authored StarCraft displacement inputs.
#[derive(Clone, Debug, Default)]
pub struct Displacement<'a> {
    /// Names the text identifying the material.
    pub name: Cow<'a, str>,
    /// The purpose of this field has not been established.
    pub unknown: u32,
    /// Controls displacement strength through its animation value.
    pub strength: ExternalAnimation<f32>,
    /// Supplies the map defining the displacement direction.
    pub normal_map: Option<TextureLayer>,
    /// Supplies the mask scaling displacement strength.
    pub strength_map: Option<TextureLayer>,
    /// Retains the native material control bits; unresolved bits remain intact.
    pub flags: DisplacementFlags,
    /// Sets render priority (lower = earlier).
    pub priority: u32,
}
/// Authored StarCraft composite section inputs.
#[derive(Clone, Debug, Default)]
pub struct CompositeSection {
    /// Material selector in the external StarCraft material lookup table.
    pub material_index: u32,
    /// Controls blend weight through its animation value.
    pub map_multiplier: ExternalAnimation<f32>,
}
/// Authored StarCraft composite inputs.
#[derive(Clone, Debug, Default)]
pub struct Composite<'a> {
    /// Names the text identifying the material.
    pub name: Cow<'a, str>,
    /// Sets render priority (lower = earlier).
    pub priority: u32,
    /// Sets sub-material sections (CMS_).
    pub sections: Vec<CompositeSection>,
}
/// Authored StarCraft terrain inputs.
#[derive(Clone, Debug, Default)]
pub struct Terrain<'a> {
    /// Names the text identifying the material.
    pub name: Cow<'a, str>,
    /// Supplies the texture layer used for the terrain contribution.
    pub terrain_map: Option<TextureLayer>,
    /// The purpose of this field has not been established.
    pub unknown: u32,
}
/// Authored StarCraft volume inputs.
#[derive(Clone, Debug, Default)]
pub struct Volume<'a> {
    /// Names the text identifying the material.
    pub name: Cow<'a, str>,
    /// Selects the behavior used for alpha blend mode.
    pub blend_mode: BlendMode,
    /// Selects the behavior used for density falloff type.
    pub falloff_type: VolumeFalloff,
    /// Controls density through its animation value.
    pub density: ExternalAnimation<f32>,
    /// Supplies the map supplying color.
    pub color_map: Option<TextureLayer>,
    /// Supplies the first noise texture.
    pub noise_map1: Option<TextureLayer>,
    /// Supplies the second noise texture.
    pub noise_map2: Option<TextureLayer>,
    /// Sets alpha test threshold.
    pub alpha_threshold: u32,
    /// Retains the native material control bits; unresolved bits remain intact.
    pub flags: VolumeFlags,
}
/// Authored StarCraft volume noise inputs.
#[derive(Clone, Debug, Default)]
pub struct VolumeNoise<'a> {
    /// Names the text identifying the material.
    pub name: Cow<'a, str>,
    /// Selects the behavior used for density falloff type.
    pub falloff_type: VolumeFalloff,
    /// Selects the transparency behavior for a camera inside or outside the volume.
    pub draw_transparency: VolumeCameraMode,
    /// Controls density through its animation value.
    pub density: ExternalAnimation<f32>,
    /// Controls near-plane clip through its animation value.
    pub near_plane: ExternalAnimation<f32>,
    /// Controls falloff distance through its animation value.
    pub falloff: ExternalAnimation<f32>,
    /// Supplies the map supplying color.
    pub color_map: Option<TextureLayer>,
    /// Supplies the first noise texture.
    pub noise_map1: Option<TextureLayer>,
    /// Supplies the second noise texture.
    pub noise_map2: Option<TextureLayer>,
    /// Controls noise scroll rate through its animation value.
    pub scroll_rate: ExternalAnimation<[f32; 3]>,
    /// Controls volume position through its animation value.
    pub position: ExternalAnimation<[f32; 3]>,
    /// Controls volume scale through its animation value.
    pub scale: ExternalAnimation<[f32; 3]>,
    /// Controls volume rotation through its animation value.
    pub rotation: ExternalAnimation<[f32; 3]>,
    /// Sets alpha test threshold.
    pub alpha_threshold: u32,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: VolumeNoiseFlags,
}
/// Authored StarCraft creep inputs.
#[derive(Clone, Debug, Default)]
pub struct Creep<'a> {
    /// Names the text identifying the material.
    pub name: Cow<'a, str>,
    /// Supplies the mask delimiting the creep effect.
    pub mask_map: Option<TextureLayer>,
    /// Controls the lower creep-mask parameter; its numerical interpretation is not established.
    pub creep_low: u32,
}
/// Authored StarCraft stb inputs.
#[derive(Clone, Debug, Default)]
pub struct Stb<'a> {
    /// Names the text identifying the material.
    pub name: Cow<'a, str>,
    /// Supplies the surface diffuse-color map.
    pub diffuse_map: Option<TextureLayer>,
    /// Supplies the map defining the displacement direction.
    pub normal_map: Option<TextureLayer>,
    /// Supplies the map controlling specular response.
    pub specular_map: Option<TextureLayer>,
}
/// Authored StarCraft reflection inputs.
#[derive(Clone, Debug, Default)]
pub struct Reflection<'a> {
    /// Names the text identifying the material.
    pub name: Cow<'a, str>,
    /// The purpose of this field has not been established.
    pub unknown: u32,
    /// Controls reflection strength through its animation value.
    pub reflection_strength: ExternalAnimation<f32>,
    /// Controls displacement strength through its animation value.
    pub displacement_strength: ExternalAnimation<f32>,
    /// Controls reflection offset through its animation value.
    pub reflection_offset: ExternalAnimation<f32>,
    /// Controls blur angle through its animation value.
    pub blur_angle: ExternalAnimation<f32>,
    /// Controls max blur distance through its animation value.
    pub blur_distance_max: ExternalAnimation<f32>,
    /// Supplies the map supplying reflected color.
    pub reflection_map: Option<TextureLayer>,
    /// Supplies the map defining displacement.
    pub displacement_map: Option<TextureLayer>,
    /// Supplies the map controlling blur.
    pub blur_map: Option<TextureLayer>,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: ReflectionFlags,
    /// The meaning of the unknown2 storage has not been established; its encoded bits are preserved.
    pub unknown2: u32,
}
/// Authored StarCraft sub flare inputs.
#[derive(Clone, Debug, Default)]
pub struct SubFlare {
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
    pub color_alpha: [u8; 4],
    /// Controls whether the element points toward the flare’s center.
    pub face_center: u32,
    /// Sets offset from flare center.
    pub offset: [f32; 2],
}
/// Authored StarCraft lens flare inputs.
#[derive(Clone, Debug, Default)]
pub struct LensFlare<'a> {
    /// Names the text identifying the material.
    pub name: Cow<'a, str>,
    /// Sets flare texture atlas.
    pub flare_map: Option<TextureLayer>,
    /// Supplies the mask delimiting the creep effect.
    pub mask_map: Option<TextureLayer>,
    /// Sets sub-flare elements (LFSB).
    pub sub_flares: Vec<SubFlare>,
    /// Sets flipbook grid columns.
    pub columns: u32,
    /// Sets flipbook grid rows.
    pub rows: u32,
    /// Sets distance fade start.
    pub distance_fade: f32,
    /// Names the text naming the flare library.
    pub lib_name: Cow<'a, str>,
    /// Controls intensity through its animation value.
    pub intensity: ExternalAnimation<f32>,
    /// Controls color tint through its animation value.
    pub color: ExternalAnimation<[u8; 4]>,
    /// Controls HDR multiplier through its animation value.
    pub hdr: ExternalAnimation<f32>,
    /// Controls size through its animation value.
    pub size: ExternalAnimation<f32>,
}
/// Authored StarCraft data driven inputs.
#[derive(Clone, Debug, Default)]
pub struct DataDriven<'a> {
    /// Names the text identifying the material.
    pub material_name: Cow<'a, str>,
    /// Supplies U32_ CRC32 identifiers for shader fragments in their linking order.
    pub fragment_hashes: Vec<u32>,
    /// Supplies additional U32_ shader hashes available from revision 2.
    pub extra_hashes: Vec<u32>,
    /// Supplies the CHAR-encoded dictionary of shader properties.
    pub property_blob: Vec<u8>,
    /// Names external texture resources selected by texture properties in the dictionary.
    pub texture_paths: Vec<Cow<'a, str>>,
    /// Unknown-purpose scalar reported as 1.0, 1.5, or 2.0 by the producer.
    pub unknown108: f32,
    /// Unknown-purpose scalar reported as 1.0 by the producer.
    pub unknown112: f32,
    /// The meaning of the unknown116 storage has not been established; its encoded bits are preserved.
    pub unknown116: f32,
    /// Stores the shader-permutation name’s CRC32; zero requests calculation during loading.
    pub effect_name_hash: u32,
    /// The meaning of the unknown124 storage has not been established; its encoded bits are preserved.
    pub unknown124: u32,
    /// Uninterpreted storage reported as zero by the producer.
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
    pub alpha_fresnel_flags: ShaderFresnelFlags,
    /// Selects the shader-family prefix used to form the permutation name.
    pub shader_type: ShaderType,
    /// The meaning of the unknown151 storage has not been established; its encoded bits are preserved.
    pub unknown151: u8,
    /// Stores another permutation identifier; all bits set indicates no second permutation (revision 3 onward).
    pub effect_name_hash2: u32,
    /// Stores a third permutation identifier; all bits set indicates no third permutation (revision 3 onward).
    pub effect_name_hash3: u32,
}
/// Authored StarCraft data driven property inputs.
#[derive(Clone, Debug, Default)]
pub struct DataDrivenProperty<'a> {
    /// Identifies a shader property by its source name hash.
    pub name_hash: u32,
    /// Names the text identifying the material.
    pub name: Cow<'a, str>,
    /// Stores the property’s encoded value bytes.
    pub data: Vec<u8>,
}
/// Authored StarCraft data driven group inputs.
#[derive(Clone, Debug, Default)]
pub struct DataDrivenGroup<'a> {
    /// Identifies a shader property by its source name hash.
    pub name_hash: u32,
    /// Names the text identifying the material.
    pub name: Cow<'a, str>,
    /// Stores the shader-property entries inline.
    pub properties: Vec<DataDrivenProperty<'a>>,
}
/// Authored StarCraft data driven properties inputs.
#[derive(Clone, Debug, Default)]
pub struct DataDrivenProperties<'a> {
    /// Stores the source material’s grouping metadata inline.
    pub groups: Vec<DataDrivenGroup<'a>>,
}
/// Actual authored StarCraft shader body.
#[derive(Clone, Debug)]
pub enum Body<'a> {
    /// Standard shader inputs.
    Standard(Box<Standard<'a>>),
    /// Displacement shader inputs.
    Displacement(Box<Displacement<'a>>),
    /// Composite shader inputs.
    Composite(Box<Composite<'a>>),
    /// Terrain shader inputs.
    Terrain(Box<Terrain<'a>>),
    /// Volume shader inputs.
    Volume(Box<Volume<'a>>),
    /// Creep shader inputs.
    Creep(Box<Creep<'a>>),
    /// VolumeNoise shader inputs.
    VolumeNoise(Box<VolumeNoise<'a>>),
    /// Stb shader inputs.
    Stb(Box<Stb<'a>>),
    /// Reflection shader inputs.
    Reflection(Box<Reflection<'a>>),
    /// LensFlare shader inputs.
    LensFlare(Box<LensFlare<'a>>),
    /// DataDriven shader inputs.
    DataDriven(Box<DataDriven<'a>>),
}
/// Complete native StarCraft shader declaration, independent of its common WEM representation.
#[derive(Clone, Debug)]
pub struct StarcraftMaterial<'a> {
    /// Separately declared source shader family.
    pub declared_family: MaterialKind,
    /// Declares data-driven authoring independent of the active body.
    pub authored_data_driven: bool,
    /// Active shader inputs and operations.
    pub body: Body<'a>,
    /// Independently authored named shader properties.
    pub data_driven_properties: Option<DataDrivenProperties<'a>>,
    /// Independently retained standard shader interpretation.
    pub restored_standard: Option<Standard<'a>>,
}
impl TextureLayer {
    fn is_valid(&self) -> bool {
        ((self.rgb_multiply.initial.is_finite()) && (self.rgb_multiply.reset.is_finite()))
            && ((self.rgb_add.initial.is_finite()) && (self.rgb_add.reset.is_finite()))
            && (self.noise_amplitude.is_finite())
            && (self.noise_frequency.is_finite())
            && ((self.uv_offset.initial.iter().all(|value| value.is_finite()))
                && (self.uv_offset.reset.iter().all(|value| value.is_finite())))
            && ((self.uv_angle.initial.iter().all(|value| value.is_finite()))
                && (self.uv_angle.reset.iter().all(|value| value.is_finite())))
            && ((self.uv_tiling.initial.iter().all(|value| value.is_finite()))
                && (self.uv_tiling.reset.iter().all(|value| value.is_finite())))
            && ((self.w_offset.initial.is_finite()) && (self.w_offset.reset.is_finite()))
            && ((self.w_tiling.initial.is_finite()) && (self.w_tiling.reset.is_finite()))
            && ((self.map_alpha.initial.is_finite()) && (self.map_alpha.reset.is_finite()))
            && ((self
                .triplanar_offset
                .initial
                .iter()
                .all(|value| value.is_finite()))
                && (self
                    .triplanar_offset
                    .reset
                    .iter()
                    .all(|value| value.is_finite())))
            && ((self
                .triplanar_scale
                .initial
                .iter()
                .all(|value| value.is_finite()))
                && (self
                    .triplanar_scale
                    .reset
                    .iter()
                    .all(|value| value.is_finite())))
            && (self.fresnel_exponent.is_finite())
            && (self.fresnel_min.is_finite())
            && (self.fresnel_max.is_finite())
            && (self
                .fresnel_translation
                .iter()
                .all(|value| value.is_finite()))
            && (self.fresnel_mask.iter().all(|value| value.is_finite()))
            && (self.fresnel_rotation.iter().all(|value| value.is_finite()))
    }
}
impl Standard<'_> {
    fn is_valid(&self) -> bool {
        (self.specular_exponent.is_finite())
            && (self.depth_blend_falloff.is_finite())
            && (self.hdr_specular_multiplier.is_finite())
            && (self.hdr_emissive_multiplier.is_finite())
            && (self.hdr_environment_constant.is_finite())
            && (self.hdr_environment_diffuse.is_finite())
            && (self.hdr_environment_specular.is_finite())
            && (self
                .diffuse_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .decal_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .specular_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .gloss_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .emissive_layer1
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .emissive_layer2
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .environment_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .environment_mask_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .alpha_layer1
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .alpha_layer2
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .normal_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .height_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .light_map_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .ambient_occlusion_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .normal_blend1_mask_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .normal_blend2_mask_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .normal_blend1_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .normal_blend2_layer
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && ((self.parallax_height.initial.is_finite())
                && (self.parallax_height.reset.is_finite()))
            && ((self.motion_blur_amount.initial.is_finite())
                && (self.motion_blur_amount.reset.is_finite()))
            && (self
                .normal_blend_factors
                .iter()
                .all(|value| (value.initial.is_finite()) && (value.reset.is_finite())))
    }
}
impl Displacement<'_> {
    fn is_valid(&self) -> bool {
        ((self.strength.initial.is_finite()) && (self.strength.reset.is_finite()))
            && (self
                .normal_map
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .strength_map
                .as_ref()
                .is_none_or(|value| value.is_valid()))
    }
}
impl CompositeSection {
    fn is_valid(&self) -> bool {
        (self.map_multiplier.initial.is_finite()) && (self.map_multiplier.reset.is_finite())
    }
}
impl Composite<'_> {
    fn is_valid(&self) -> bool {
        self.sections.iter().all(|value| value.is_valid())
    }
}
impl Terrain<'_> {
    fn is_valid(&self) -> bool {
        self.terrain_map
            .as_ref()
            .is_none_or(|value| value.is_valid())
    }
}
impl Volume<'_> {
    fn is_valid(&self) -> bool {
        ((self.density.initial.is_finite()) && (self.density.reset.is_finite()))
            && (self.color_map.as_ref().is_none_or(|value| value.is_valid()))
            && (self
                .noise_map1
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .noise_map2
                .as_ref()
                .is_none_or(|value| value.is_valid()))
    }
}
impl VolumeNoise<'_> {
    fn is_valid(&self) -> bool {
        ((self.density.initial.is_finite()) && (self.density.reset.is_finite()))
            && ((self.near_plane.initial.is_finite()) && (self.near_plane.reset.is_finite()))
            && ((self.falloff.initial.is_finite()) && (self.falloff.reset.is_finite()))
            && (self.color_map.as_ref().is_none_or(|value| value.is_valid()))
            && (self
                .noise_map1
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .noise_map2
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && ((self
                .scroll_rate
                .initial
                .iter()
                .all(|value| value.is_finite()))
                && (self.scroll_rate.reset.iter().all(|value| value.is_finite())))
            && ((self.position.initial.iter().all(|value| value.is_finite()))
                && (self.position.reset.iter().all(|value| value.is_finite())))
            && ((self.scale.initial.iter().all(|value| value.is_finite()))
                && (self.scale.reset.iter().all(|value| value.is_finite())))
            && ((self.rotation.initial.iter().all(|value| value.is_finite()))
                && (self.rotation.reset.iter().all(|value| value.is_finite())))
    }
}
impl Creep<'_> {
    fn is_valid(&self) -> bool {
        self.mask_map.as_ref().is_none_or(|value| value.is_valid())
    }
}
impl Stb<'_> {
    fn is_valid(&self) -> bool {
        (self
            .diffuse_map
            .as_ref()
            .is_none_or(|value| value.is_valid()))
            && (self
                .normal_map
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .specular_map
                .as_ref()
                .is_none_or(|value| value.is_valid()))
    }
}
impl Reflection<'_> {
    fn is_valid(&self) -> bool {
        ((self.reflection_strength.initial.is_finite())
            && (self.reflection_strength.reset.is_finite()))
            && ((self.displacement_strength.initial.is_finite())
                && (self.displacement_strength.reset.is_finite()))
            && ((self.reflection_offset.initial.is_finite())
                && (self.reflection_offset.reset.is_finite()))
            && ((self.blur_angle.initial.is_finite()) && (self.blur_angle.reset.is_finite()))
            && ((self.blur_distance_max.initial.is_finite())
                && (self.blur_distance_max.reset.is_finite()))
            && (self
                .reflection_map
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self
                .displacement_map
                .as_ref()
                .is_none_or(|value| value.is_valid()))
            && (self.blur_map.as_ref().is_none_or(|value| value.is_valid()))
    }
}
impl SubFlare {
    fn is_valid(&self) -> bool {
        (self.position.is_finite())
            && (self.size_xy.iter().all(|value| value.is_finite()))
            && (self.scale_xy.iter().all(|value| value.is_finite()))
            && (self.fade_in.iter().all(|value| value.is_finite()))
            && (self.fade_out.iter().all(|value| value.is_finite()))
            && (self.offset.iter().all(|value| value.is_finite()))
    }
}
impl LensFlare<'_> {
    fn is_valid(&self) -> bool {
        (self.flare_map.as_ref().is_none_or(|value| value.is_valid()))
            && (self.mask_map.as_ref().is_none_or(|value| value.is_valid()))
            && (self.sub_flares.iter().all(|value| value.is_valid()))
            && (self.distance_fade.is_finite())
            && ((self.intensity.initial.is_finite()) && (self.intensity.reset.is_finite()))
            && ((self.hdr.initial.is_finite()) && (self.hdr.reset.is_finite()))
            && ((self.size.initial.is_finite()) && (self.size.reset.is_finite()))
    }
}
impl DataDriven<'_> {
    fn is_valid(&self) -> bool {
        (self.unknown108.is_finite())
            && (self.unknown112.is_finite())
            && (self.unknown116.is_finite())
    }
}
impl DataDrivenProperty<'_> {
    fn is_valid(&self) -> bool {
        true
    }
}
impl DataDrivenGroup<'_> {
    fn is_valid(&self) -> bool {
        self.properties.iter().all(|value| value.is_valid())
    }
}
impl DataDrivenProperties<'_> {
    fn is_valid(&self) -> bool {
        self.groups.iter().all(|value| value.is_valid())
    }
}
impl Body<'_> {
    fn is_valid(&self) -> bool {
        match self {
            Self::Standard(value) => value.is_valid(),
            Self::Displacement(value) => value.is_valid(),
            Self::Composite(value) => value.is_valid(),
            Self::Terrain(value) => value.is_valid(),
            Self::Volume(value) => value.is_valid(),
            Self::Creep(value) => value.is_valid(),
            Self::VolumeNoise(value) => value.is_valid(),
            Self::Stb(value) => value.is_valid(),
            Self::Reflection(value) => value.is_valid(),
            Self::LensFlare(value) => value.is_valid(),
            Self::DataDriven(value) => value.is_valid(),
        }
    }
}
impl ValidateLinks for StarcraftMaterial<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.body.visit_links(visitor);
        self.restored_standard.visit_links(visitor);
        if !self.body.is_valid()
            || self
                .data_driven_properties
                .as_ref()
                .is_some_and(|value| !value.is_valid())
            || self
                .restored_standard
                .as_ref()
                .is_some_and(|value| !value.is_valid())
        {
            visitor(Link::InvalidValue);
        }
    }
}
impl StarcraftMaterial<'_> {
    /// Checks shader coefficients while leaving external lookup resolution to the caller.
    pub fn validate(&self) -> Result<(), Error> {
        let mut valid = true;
        self.visit_links(&mut |link| {
            if matches!(link, Link::InvalidValue) {
                valid = false
            }
        });
        if valid {
            Ok(())
        } else {
            Err(Error::new(
                "wem.material.native.starcraft",
                "nonfinite shader coefficient",
            ))
        }
    }
}

impl ValidateLinks for TextureLayer {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.texture.visit_links(visitor);
    }
}
impl ValidateLinks for Standard<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.diffuse_layer.visit_links(visitor);
        self.decal_layer.visit_links(visitor);
        self.specular_layer.visit_links(visitor);
        self.gloss_layer.visit_links(visitor);
        self.emissive_layer1.visit_links(visitor);
        self.emissive_layer2.visit_links(visitor);
        self.environment_layer.visit_links(visitor);
        self.environment_mask_layer.visit_links(visitor);
        self.alpha_layer1.visit_links(visitor);
        self.alpha_layer2.visit_links(visitor);
        self.normal_layer.visit_links(visitor);
        self.height_layer.visit_links(visitor);
        self.light_map_layer.visit_links(visitor);
        self.ambient_occlusion_layer.visit_links(visitor);
        self.normal_blend1_mask_layer.visit_links(visitor);
        self.normal_blend2_mask_layer.visit_links(visitor);
        self.normal_blend1_layer.visit_links(visitor);
        self.normal_blend2_layer.visit_links(visitor);
    }
}
impl ValidateLinks for Displacement<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.normal_map.visit_links(visitor);
        self.strength_map.visit_links(visitor);
    }
}
impl ValidateLinks for Terrain<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.terrain_map.visit_links(visitor);
    }
}
impl ValidateLinks for Volume<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.color_map.visit_links(visitor);
        self.noise_map1.visit_links(visitor);
        self.noise_map2.visit_links(visitor);
    }
}
impl ValidateLinks for VolumeNoise<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.color_map.visit_links(visitor);
        self.noise_map1.visit_links(visitor);
        self.noise_map2.visit_links(visitor);
    }
}
impl ValidateLinks for Creep<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.mask_map.visit_links(visitor);
    }
}
impl ValidateLinks for Stb<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.diffuse_map.visit_links(visitor);
        self.normal_map.visit_links(visitor);
        self.specular_map.visit_links(visitor);
    }
}
impl ValidateLinks for Reflection<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.reflection_map.visit_links(visitor);
        self.displacement_map.visit_links(visitor);
        self.blur_map.visit_links(visitor);
    }
}
impl ValidateLinks for LensFlare<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.flare_map.visit_links(visitor);
        self.mask_map.visit_links(visitor);
    }
}
impl ValidateLinks for Body<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Standard(value) => value.visit_links(visitor),
            Self::Displacement(value) => value.visit_links(visitor),
            Self::Composite(_) => {}
            Self::Terrain(value) => value.visit_links(visitor),
            Self::Volume(value) => value.visit_links(visitor),
            Self::Creep(value) => value.visit_links(visitor),
            Self::VolumeNoise(value) => value.visit_links(visitor),
            Self::Stb(value) => value.visit_links(visitor),
            Self::Reflection(value) => value.visit_links(visitor),
            Self::LensFlare(value) => value.visit_links(visitor),
            Self::DataDriven(_) => {}
        }
    }
}
