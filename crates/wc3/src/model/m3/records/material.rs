//! Native material record revisions.
use super::dialect::*;
use super::{targets, AnimationValue, ReservedReferenceWords};
use crate::model::m3::{self, Record, Ref};
use core::fmt::Debug;
/// Fields of the native `MATM` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct MaterialMap {
    /// Selects the material family; 1 denotes standard materials and 2 denotes displacement materials.
    pub material_type: u32,
    /// Selects an entry in the material array chosen by `material_type`.
    pub material_index: u32,
}

impl Record for MaterialMap {
    const TAG: [u8; 4] = *b"MATM";
    const VERSION: u32 = 0;
}

/// Native field selections for independent `TextureLayer` section revisions.
pub trait TextureLayerDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `noise_amplitude`; `()` omits the field.
    type NoiseAmplitude: NativeField;
    /// Field representation for `noise_frequency`; `()` omits the field.
    type NoiseFrequency: NativeField;
    /// Field representation for `triplanar_offset`; `()` omits the field.
    type TriplanarOffset: NativeField;
    /// Field representation for `triplanar_scale`; `()` omits the field.
    type TriplanarScale: NativeField;
    /// Field representation for `fresnel_translation`; `()` omits the field.
    type FresnelTranslation: NativeField;
    /// Field representation for `fresnel_mask`; `()` omits the field.
    type FresnelMask: NativeField;
    /// Field representation for `fresnel_rotation`; `()` omits the field.
    type FresnelRotation: NativeField;
    /// Field representation for `uv_density`; `()` omits the field.
    type UvDensity: NativeField;
}

impl TextureLayerDialect for V0 {
    const VERSION: u32 = 0;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V1 {
    const VERSION: u32 = 1;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V2 {
    const VERSION: u32 = 2;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V3 {
    const VERSION: u32 = 3;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V4 {
    const VERSION: u32 = 4;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V5 {
    const VERSION: u32 = 5;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V6 {
    const VERSION: u32 = 6;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V7 {
    const VERSION: u32 = 7;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V8 {
    const VERSION: u32 = 8;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V9 {
    const VERSION: u32 = 9;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V10 {
    const VERSION: u32 = 10;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V11 {
    const VERSION: u32 = 11;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V12 {
    const VERSION: u32 = 12;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V13 {
    const VERSION: u32 = 13;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V14 {
    const VERSION: u32 = 14;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V15 {
    const VERSION: u32 = 15;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V16 {
    const VERSION: u32 = 16;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V17 {
    const VERSION: u32 = 17;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V18 {
    const VERSION: u32 = 18;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V19 {
    const VERSION: u32 = 19;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V20 {
    const VERSION: u32 = 20;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V21 {
    const VERSION: u32 = 21;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V22 {
    const VERSION: u32 = 22;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = ();
    type TriplanarScale = ();
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V23 {
    const VERSION: u32 = 23;
    type NoiseAmplitude = ();
    type NoiseFrequency = ();
    type TriplanarOffset = AnimationValue<[f32; 3]>;
    type TriplanarScale = AnimationValue<[f32; 3]>;
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V24 {
    const VERSION: u32 = 24;
    type NoiseAmplitude = f32;
    type NoiseFrequency = f32;
    type TriplanarOffset = AnimationValue<[f32; 3]>;
    type TriplanarScale = AnimationValue<[f32; 3]>;
    type FresnelTranslation = ();
    type FresnelMask = ();
    type FresnelRotation = ();
    type UvDensity = u32;
}

impl TextureLayerDialect for V25 {
    const VERSION: u32 = 25;
    type NoiseAmplitude = f32;
    type NoiseFrequency = f32;
    type TriplanarOffset = AnimationValue<[f32; 3]>;
    type TriplanarScale = AnimationValue<[f32; 3]>;
    type FresnelTranslation = [f32; 3];
    type FresnelMask = [f32; 3];
    type FresnelRotation = [f32; 2];
    type UvDensity = u32;
}

impl TextureLayerDialect for V26 {
    const VERSION: u32 = 26;
    type NoiseAmplitude = f32;
    type NoiseFrequency = f32;
    type TriplanarOffset = AnimationValue<[f32; 3]>;
    type TriplanarScale = AnimationValue<[f32; 3]>;
    type FresnelTranslation = [f32; 3];
    type FresnelMask = [f32; 3];
    type FresnelRotation = [f32; 2];
    type UvDensity = ();
}

/// Fields of the native `LAYR` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct TextureLayer<D: TextureLayerDialect> {
    /// Identifies this texture layer within the material setup.
    pub id: u32,
    /// Links to the CHAR bytes containing the texture filename.
    pub texture_path: Ref<targets::Character>,
    /// Controls color tint through its animation value.
    pub color: AnimationValue<[u8; 4]>,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: u32,
    /// Selects the behavior used for uv mapping source.
    pub uv_mapping: u32,
    /// Selects the behavior used for channel selection.
    pub color_type: u32,
    /// Sets RGB multiply factor.
    pub rgb_multiply: AnimationValue<f32>,
    /// Sets RGB additive factor.
    pub rgb_add: AnimationValue<f32>,
    /// Stores the POC texture selector; its target namespace is not established.
    pub poc_texture: u32,
    /// Sets noise amplitude. This field is present in revision 24 onward.
    pub noise_amplitude: D::NoiseAmplitude,
    /// Sets noise frequency. This field is present in revision 24 onward.
    pub noise_frequency: D::NoiseFrequency,
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
    pub avi_play: AnimationValue<u32>,
    /// Sets AVI restart control.
    pub avi_restart: AnimationValue<u32>,
    /// Sets flipbook grid rows.
    pub flipbook_rows: u32,
    /// Sets flipbook grid columns.
    pub flipbook_columns: u32,
    /// Controls flipbook frame index through its animation value.
    pub current_frame: AnimationValue<u16>,
    /// Controls UV offset through its animation value.
    pub uv_offset: AnimationValue<[f32; 2]>,
    /// Controls UV rotation angles through its animation value.
    pub uv_angle: AnimationValue<[f32; 3]>,
    /// Controls UV tiling through its animation value.
    pub uv_tiling: AnimationValue<[f32; 2]>,
    /// Controls w offset (3D textures) through its animation value.
    pub w_offset: AnimationValue<f32>,
    /// Controls w tiling (3D textures) through its animation value.
    pub w_tiling: AnimationValue<f32>,
    /// Controls the texture contribution’s alpha through an animation value.
    pub map_alpha: AnimationValue<f32>,
    /// Sets tri-planar UV offset. This field is present in revision 23 onward.
    pub triplanar_offset: D::TriplanarOffset,
    /// Sets tri-planar UV scale. This field is present in revision 23 onward.
    pub triplanar_scale: D::TriplanarScale,
    /// Selects the layer supplying shared UV settings; all bits set selects this layer’s own settings.
    pub uv_source_related: u32,
    /// Selects the behavior used for fresnel effect mode.
    pub fresnel_mode: u32,
    /// Sets fresnel exponent (edge sharpness).
    pub fresnel_exponent: f32,
    /// Sets fresnel minimum intensity.
    pub fresnel_min: f32,
    /// Sets fresnel maximum intensity.
    pub fresnel_max: f32,
    /// Sets fresnel UV translation. This field is present in revision 25 onward.
    pub fresnel_translation: D::FresnelTranslation,
    /// Sets fresnel mask vector. This field is present in revision 25 onward.
    pub fresnel_mask: D::FresnelMask,
    /// Sets fresnel UV rotation. This field is present in revision 25 onward.
    pub fresnel_rotation: D::FresnelRotation,
    /// Sets UV density hint. This field is present in revision 0–revision 25, absent in revision 26.
    pub uv_density: D::UvDensity,
}

impl<D: TextureLayerDialect> Record for TextureLayer<D> {
    const TAG: [u8; 4] = *b"LAYR";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `StandardMaterial` section revisions.
pub trait StandardMaterialDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `hdr_environment_constant`; `()` omits the field.
    type HdrEnvironmentConstant: NativeField;
    /// Field representation for `hdr_environment_diffuse`; `()` omits the field.
    type HdrEnvironmentDiffuse: NativeField;
    /// Field representation for `hdr_environment_specular`; `()` omits the field.
    type HdrEnvironmentSpecular: NativeField;
    /// Field representation for `gloss_layer`; `()` omits the field.
    type GlossLayer: NativeField;
    /// Field representation for `normal_blend1_mask_layer`; `()` omits the field.
    type NormalBlend1MaskLayer: NativeField;
    /// Field representation for `normal_blend2_mask_layer`; `()` omits the field.
    type NormalBlend2MaskLayer: NativeField;
    /// Field representation for `normal_blend1_layer`; `()` omits the field.
    type NormalBlend1Layer: NativeField;
    /// Field representation for `normal_blend2_layer`; `()` omits the field.
    type NormalBlend2Layer: NativeField;
    /// Field representation for `normal_blend_factors`; `()` omits the field.
    type NormalBlendFactors: NativeField;
}

impl StandardMaterialDialect for V0 {
    const VERSION: u32 = 0;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V1 {
    const VERSION: u32 = 1;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V2 {
    const VERSION: u32 = 2;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V3 {
    const VERSION: u32 = 3;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V4 {
    const VERSION: u32 = 4;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V5 {
    const VERSION: u32 = 5;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V6 {
    const VERSION: u32 = 6;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V7 {
    const VERSION: u32 = 7;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V8 {
    const VERSION: u32 = 8;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V9 {
    const VERSION: u32 = 9;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V10 {
    const VERSION: u32 = 10;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V11 {
    const VERSION: u32 = 11;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V12 {
    const VERSION: u32 = 12;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V13 {
    const VERSION: u32 = 13;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V14 {
    const VERSION: u32 = 14;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V15 {
    const VERSION: u32 = 15;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = ();
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V16 {
    const VERSION: u32 = 16;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = Ref<targets::TextureLayer>;
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V17 {
    const VERSION: u32 = 17;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = Ref<targets::TextureLayer>;
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V18 {
    const VERSION: u32 = 18;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = Ref<targets::TextureLayer>;
    type NormalBlend1MaskLayer = ();
    type NormalBlend2MaskLayer = ();
    type NormalBlend1Layer = ();
    type NormalBlend2Layer = ();
    type NormalBlendFactors = ();
}

impl StandardMaterialDialect for V19 {
    const VERSION: u32 = 19;
    type HdrEnvironmentConstant = ();
    type HdrEnvironmentDiffuse = ();
    type HdrEnvironmentSpecular = ();
    type GlossLayer = Ref<targets::TextureLayer>;
    type NormalBlend1MaskLayer = Ref<targets::TextureLayer>;
    type NormalBlend2MaskLayer = Ref<targets::TextureLayer>;
    type NormalBlend1Layer = Ref<targets::TextureLayer>;
    type NormalBlend2Layer = Ref<targets::TextureLayer>;
    type NormalBlendFactors = Ref<targets::AnimationValueReal>;
}

impl StandardMaterialDialect for V20 {
    const VERSION: u32 = 20;
    type HdrEnvironmentConstant = f32;
    type HdrEnvironmentDiffuse = f32;
    type HdrEnvironmentSpecular = f32;
    type GlossLayer = Ref<targets::TextureLayer>;
    type NormalBlend1MaskLayer = Ref<targets::TextureLayer>;
    type NormalBlend2MaskLayer = Ref<targets::TextureLayer>;
    type NormalBlend1Layer = Ref<targets::TextureLayer>;
    type NormalBlend2Layer = Ref<targets::TextureLayer>;
    type NormalBlendFactors = Ref<targets::AnimationValueReal>;
}

/// Fields of the native `MAT_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct StandardMaterial<D: StandardMaterialDialect> {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Stores the bit mask governing additional.
    pub additional_flags: u32,
    /// Stores the bit mask governing material rendering.
    pub flags: u32,
    /// Selects the behavior used for alpha blend mode.
    pub blend_mode: u32,
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
    /// Sets HDR environment constant. This field is present in revision 20.
    pub hdr_environment_constant: D::HdrEnvironmentConstant,
    /// Sets HDR environment diffuse. This field is present in revision 20.
    pub hdr_environment_diffuse: D::HdrEnvironmentDiffuse,
    /// Sets HDR environment specular. This field is present in revision 20.
    pub hdr_environment_specular: D::HdrEnvironmentSpecular,
    /// Links to the surface diffuse-color layer.
    pub diffuse_layer: Ref<targets::TextureLayer>,
    /// Links to the decal composited over the base material.
    pub decal_layer: Ref<targets::TextureLayer>,
    /// Links to the map controlling specular response.
    pub specular_layer: Ref<targets::TextureLayer>,
    /// Links to the map controlling the surface gloss. This field is present in revision 16 onward.
    pub gloss_layer: D::GlossLayer,
    /// Links to the first self-illumination layer.
    pub emissive_layer1: Ref<targets::TextureLayer>,
    /// Links to the second self-illumination layer.
    pub emissive_layer2: Ref<targets::TextureLayer>,
    /// Links to the map supplying environment reflections.
    pub environment_layer: Ref<targets::TextureLayer>,
    /// Sets environment mask.
    pub environment_mask_layer: Ref<targets::TextureLayer>,
    /// Links to the first alpha-mask layer controlling opacity.
    pub alpha_layer1: Ref<targets::TextureLayer>,
    /// Links to the second alpha-mask layer controlling opacity.
    pub alpha_layer2: Ref<targets::TextureLayer>,
    /// Links to the map defining perturbed surface normals.
    pub normal_layer: Ref<targets::TextureLayer>,
    /// Links to the height map used for parallax displacement.
    pub height_layer: Ref<targets::TextureLayer>,
    /// Links to the map supplying precomputed lighting.
    pub light_map_layer: Ref<targets::TextureLayer>,
    /// Links to the map modulating illumination with ambient occlusion.
    pub ambient_occlusion_layer: Ref<targets::TextureLayer>,
    /// Links to the mask controlling additional normal map 1’s contribution. This field is present in revision 19 onward.
    pub normal_blend1_mask_layer: D::NormalBlend1MaskLayer,
    /// Links to the mask controlling additional normal map 2’s contribution. This field is present in revision 19 onward.
    pub normal_blend2_mask_layer: D::NormalBlend2MaskLayer,
    /// Links to additional normal map 1 for blending with the base normal map. This field is present in revision 19 onward.
    pub normal_blend1_layer: D::NormalBlend1Layer,
    /// Links to additional normal map 2 for blending with the base normal map. This field is present in revision 19 onward.
    pub normal_blend2_layer: D::NormalBlend2Layer,
    /// Sets material class (unit, building, etc.).
    pub material_class: u32,
    /// Sets layer blend operation.
    pub layer_blend_mode: u32,
    /// Selects the compositing operation applied to emissive layer 1.
    pub emissive_blend_mode1: u32,
    /// Selects the compositing operation applied to emissive layer 2.
    pub emissive_blend_mode2: u32,
    /// Selects the behavior used for specular computation mode.
    pub specular_mode: u32,
    /// Controls parallax height through its animation value.
    pub parallax_height: AnimationValue<f32>,
    /// Controls motion blur amount through its animation value.
    pub motion_blur_amount: AnimationValue<f32>,
    /// Weights the contributions of the additional blended normal maps. This field is present in revision 19 onward.
    pub normal_blend_factors: D::NormalBlendFactors,
}

impl<D: StandardMaterialDialect> Record for StandardMaterial<D> {
    const TAG: [u8; 4] = *b"MAT_";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `DisplacementMaterial` section revisions.
pub trait DisplacementMaterialDialect:
    Revision + Copy + Clone + Debug + Default + PartialEq
{
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `name`; `()` omits the field.
    type Name: NativeField;
}

impl DisplacementMaterialDialect for V0 {
    const VERSION: u32 = 0;
    type Name = Ref<targets::Character>;
}

impl DisplacementMaterialDialect for V1 {
    const VERSION: u32 = 1;
    type Name = Ref<targets::Character>;
}

impl DisplacementMaterialDialect for V2 {
    const VERSION: u32 = 2;
    type Name = Ref<targets::Character>;
}

impl DisplacementMaterialDialect for V3 {
    const VERSION: u32 = 3;
    type Name = Ref<targets::Character>;
}

impl DisplacementMaterialDialect for V4 {
    const VERSION: u32 = 4;
    type Name = Ref<targets::Character>;
}

/// Fields of the native `DIS_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct DisplacementMaterial<D: DisplacementMaterialDialect> {
    /// Links to the CHAR bytes identifying the material.
    pub name: D::Name,
    /// The purpose of this field has not been established.
    pub unknown: u32,
    /// Controls displacement strength through its animation value.
    pub strength: AnimationValue<f32>,
    /// Links to the map defining the displacement direction.
    pub normal_map: Ref<targets::TextureLayer>,
    /// Links to the mask scaling displacement strength.
    pub strength_map: Ref<targets::TextureLayer>,
    /// Stores the bit mask configuring the displacement material.
    pub flags: u32,
    /// Sets render priority.
    pub priority: u32,
}

impl<D: DisplacementMaterialDialect> Record for DisplacementMaterial<D> {
    const TAG: [u8; 4] = *b"DIS_";
    const VERSION: u32 = D::VERSION;
}

/// Fields of the native `CMS_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct CompositeSection {
    /// Selects a material-map entry in the MATM section.
    pub material_index: u32,
    /// Controls blend weight through its animation value.
    pub map_multiplier: AnimationValue<f32>,
}

impl Record for CompositeSection {
    const TAG: [u8; 4] = *b"CMS_";
    const VERSION: u32 = 0;
}

/// Native field selections for independent `CompositeMaterial` section revisions.
pub trait CompositeMaterialDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `name`; `()` omits the field.
    type Name: NativeField;
}

impl CompositeMaterialDialect for V0 {
    const VERSION: u32 = 0;
    type Name = Ref<targets::Character>;
}

impl CompositeMaterialDialect for V1 {
    const VERSION: u32 = 1;
    type Name = Ref<targets::Character>;
}

impl CompositeMaterialDialect for V2 {
    const VERSION: u32 = 2;
    type Name = Ref<targets::Character>;
}

/// Fields of the native `CMP_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct CompositeMaterial<D: CompositeMaterialDialect> {
    /// Links to the CHAR bytes identifying the material.
    pub name: D::Name,
    /// Sets render priority.
    pub priority: u32,
    /// Sets sub-material sections (CMS_).
    pub sections: Ref<targets::CompositeSection>,
}

impl<D: CompositeMaterialDialect> Record for CompositeMaterial<D> {
    const TAG: [u8; 4] = *b"CMP_";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `TerrainMaterial` section revisions.
pub trait TerrainMaterialDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `unknown`; `()` omits the field.
    type Unknown: NativeField;
}

impl TerrainMaterialDialect for V0 {
    const VERSION: u32 = 0;
    type Unknown = ();
}

impl TerrainMaterialDialect for V1 {
    const VERSION: u32 = 1;
    type Unknown = u32;
}

/// Fields of the native `TER_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct TerrainMaterial<D: TerrainMaterialDialect> {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Links to the texture layer used for the terrain contribution.
    pub terrain_map: Ref<targets::TextureLayer>,
    /// The purpose of this field has not been established.
    pub unknown: D::Unknown,
}

impl<D: TerrainMaterialDialect> Record for TerrainMaterial<D> {
    const TAG: [u8; 4] = *b"TER_";
    const VERSION: u32 = D::VERSION;
}

/// Fields of the native `VOL_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct VolumeMaterial {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Selects the behavior used for blend mode.
    pub blend_mode: u32,
    /// Selects the behavior used for density falloff type.
    pub falloff_type: u32,
    /// Controls density through its animation value.
    pub density: AnimationValue<f32>,
    /// Links to the map supplying color.
    pub color_map: Ref<targets::TextureLayer>,
    /// Links to the first noise texture.
    pub noise_map1: Ref<targets::TextureLayer>,
    /// Links to the second noise texture.
    pub noise_map2: Ref<targets::TextureLayer>,
    /// Sets alpha test threshold.
    pub alpha_threshold: u32,
    /// Stores the bit mask configuring the volume material.
    pub flags: u32,
}

impl Record for VolumeMaterial {
    const TAG: [u8; 4] = *b"VOL_";
    const VERSION: u32 = 0;
}

/// Fields of the native `HAI_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct HairMaterial {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Links to the base diffuse-color layer.
    pub layer_base: Ref<targets::TextureLayer>,
    /// Links to the map shifting the anisotropic specular response.
    pub layer_spec_shift: Ref<targets::TextureLayer>,
    /// Links to the noise map varying the specular response.
    pub layer_spec_noise: Ref<targets::TextureLayer>,
    /// Links to the map modulating illumination with ambient occlusion.
    pub layer_a_o: Ref<targets::TextureLayer>,
    /// Sets primary specular shift.
    pub shift_primary: f32,
    /// Sets secondary specular shift.
    pub shift_secondary: f32,
    /// Controls diffuse tint through its animation value.
    pub color_diffuse: AnimationValue<[u8; 4]>,
    /// Controls specular tint through its animation value.
    pub color_spec: AnimationValue<[u8; 4]>,
    /// Sets primary specular exponent.
    pub spec_exponent0: f32,
    /// Sets secondary specular exponent.
    pub spec_exponent1: f32,
}

impl Record for HairMaterial {
    const TAG: [u8; 4] = *b"HAI_";
    const VERSION: u32 = 0;
}

/// Fields of the native `VON_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct VolumeNoiseMaterial {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Selects the behavior used for density falloff type.
    pub falloff_type: u32,
    /// Selects the transparency behavior for a camera inside or outside the volume.
    pub draw_transparency: u32,
    /// Controls density through its animation value.
    pub density: AnimationValue<f32>,
    /// Controls near-plane clip through its animation value.
    pub near_plane: AnimationValue<f32>,
    /// Controls falloff distance through its animation value.
    pub falloff: AnimationValue<f32>,
    /// Links to the map supplying color.
    pub color_map: Ref<targets::TextureLayer>,
    /// Links to the first noise texture.
    pub noise_map1: Ref<targets::TextureLayer>,
    /// Links to the second noise texture.
    pub noise_map2: Ref<targets::TextureLayer>,
    /// Controls noise scroll rate through its animation value.
    pub scroll_rate: AnimationValue<[f32; 3]>,
    /// Controls volume position through its animation value.
    pub position: AnimationValue<[f32; 3]>,
    /// Controls volume scale through its animation value.
    pub scale: AnimationValue<[f32; 3]>,
    /// Controls volume rotation through its animation value.
    pub rotation: AnimationValue<[f32; 3]>,
    /// Sets alpha test threshold.
    pub alpha_threshold: u32,
    /// Stores the bit mask governing volume noise material.
    pub flags: u32,
}

impl Record for VolumeNoiseMaterial {
    const TAG: [u8; 4] = *b"VON_";
    const VERSION: u32 = 0;
}

/// Native field selections for independent `CreepMaterial` section revisions.
pub trait CreepMaterialDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `creep_low`; `()` omits the field.
    type CreepLow: NativeField;
}

impl CreepMaterialDialect for V0 {
    const VERSION: u32 = 0;
    type CreepLow = ();
}

impl CreepMaterialDialect for V1 {
    const VERSION: u32 = 1;
    type CreepLow = u32;
}

/// Fields of the native `CREP` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct CreepMaterial<D: CreepMaterialDialect> {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Links to the mask delimiting the creep effect.
    pub mask_map: Ref<targets::TextureLayer>,
    /// Controls the lower creep-mask parameter; its numerical interpretation is not established.
    pub creep_low: D::CreepLow,
}

impl<D: CreepMaterialDialect> Record for CreepMaterial<D> {
    const TAG: [u8; 4] = *b"CREP";
    const VERSION: u32 = D::VERSION;
}

/// Fields of the native `STBM` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct STBMaterial {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// Links to the surface diffuse-color map.
    pub diffuse_map: Ref<targets::TextureLayer>,
    /// Links to the surface normal map.
    pub normal_map: Ref<targets::TextureLayer>,
    /// Links to the map controlling specular response.
    pub specular_map: Ref<targets::TextureLayer>,
}

impl Record for STBMaterial {
    const TAG: [u8; 4] = *b"STBM";
    const VERSION: u32 = 0;
}

/// Native field selections for independent `ReflectionMaterial` section revisions.
pub trait ReflectionMaterialDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `reflection_offset`; `()` omits the field.
    type ReflectionOffset: NativeField;
    /// Field representation for `blur_angle`; `()` omits the field.
    type BlurAngle: NativeField;
    /// Field representation for `blur_distance_max`; `()` omits the field.
    type BlurDistanceMax: NativeField;
    /// Field representation for `blur_map`; `()` omits the field.
    type BlurMap: NativeField;
    /// Field representation for `unknown2`; `()` omits the field.
    type Unknown2: NativeField;
}

impl ReflectionMaterialDialect for V0 {
    const VERSION: u32 = 0;
    type ReflectionOffset = ();
    type BlurAngle = ();
    type BlurDistanceMax = ();
    type BlurMap = ();
    type Unknown2 = ();
}

impl ReflectionMaterialDialect for V1 {
    const VERSION: u32 = 1;
    type ReflectionOffset = ();
    type BlurAngle = ();
    type BlurDistanceMax = ();
    type BlurMap = ();
    type Unknown2 = ();
}

impl ReflectionMaterialDialect for V2 {
    const VERSION: u32 = 2;
    type ReflectionOffset = AnimationValue<f32>;
    type BlurAngle = AnimationValue<f32>;
    type BlurDistanceMax = AnimationValue<f32>;
    type BlurMap = Ref<targets::TextureLayer>;
    type Unknown2 = ();
}

impl ReflectionMaterialDialect for V3 {
    const VERSION: u32 = 3;
    type ReflectionOffset = AnimationValue<f32>;
    type BlurAngle = AnimationValue<f32>;
    type BlurDistanceMax = AnimationValue<f32>;
    type BlurMap = Ref<targets::TextureLayer>;
    type Unknown2 = u32;
}

/// Fields of the native `REF_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct ReflectionMaterial<D: ReflectionMaterialDialect> {
    /// Links to the CHAR bytes identifying the material.
    pub name: Ref<targets::Character>,
    /// The purpose of this field has not been established.
    pub unknown: u32,
    /// Controls reflection strength through its animation value. This field is present in revision 2 onward.
    pub reflection_strength: AnimationValue<f32>,
    /// Controls displacement strength through its animation value. This field is present in revision 2 onward.
    pub displacement_strength: AnimationValue<f32>,
    /// Controls reflection offset through its animation value. This field is present in revision 2 onward.
    pub reflection_offset: D::ReflectionOffset,
    /// Controls blur angle through its animation value. This field is present in revision 2 onward.
    pub blur_angle: D::BlurAngle,
    /// Controls max blur distance through its animation value. This field is present in revision 2 onward.
    pub blur_distance_max: D::BlurDistanceMax,
    /// Links to the map supplying reflected color.
    pub reflection_map: Ref<targets::TextureLayer>,
    /// Links to the map defining displacement.
    pub displacement_map: Ref<targets::TextureLayer>,
    /// Links to the map controlling blur.
    pub blur_map: D::BlurMap,
    /// Stores the bit mask governing reflection. This field is present in revision 2 onward.
    pub flags: u32,
    /// The meaning of the unknown2 storage has not been established; its encoded bits are preserved.
    pub unknown2: D::Unknown2,
}

impl<D: ReflectionMaterialDialect> Record for ReflectionMaterial<D> {
    const TAG: [u8; 4] = *b"REF_";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `SubFlare` section revisions.
pub trait SubFlareDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `index`; `()` omits the field.
    type Index: NativeField;
}

impl SubFlareDialect for V0 {
    const VERSION: u32 = 0;
    type Index = u32;
}

impl SubFlareDialect for V1 {
    const VERSION: u32 = 1;
    type Index = u32;
}

impl SubFlareDialect for V2 {
    const VERSION: u32 = 2;
    type Index = u32;
}

/// Fields of the native `LFSB` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct SubFlare<D: SubFlareDialect> {
    /// Sets flare element index.
    pub index: D::Index,
    /// Places the flare element along its axis using a normalized coordinate from zero to one.
    pub position: f32,
    /// Sets base size (width, height).
    pub size_x_y: [f32; 2],
    /// Sets scale multiplier (width, height).
    pub scale_x_y: [f32; 2],
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

impl<D: SubFlareDialect> Record for SubFlare<D> {
    const TAG: [u8; 4] = *b"LFSB";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `LensFlare` section revisions.
pub trait LensFlareDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `lib_name`; `()` omits the field.
    type LibName: NativeField;
    /// Field representation for `color`; `()` omits the field.
    type Color: NativeField;
    /// Field representation for `hdr`; `()` omits the field.
    type Hdr: NativeField;
    /// Field representation for `size`; `()` omits the field.
    type Size: NativeField;
}

impl LensFlareDialect for V0 {
    const VERSION: u32 = 0;
    type LibName = ();
    type Color = ();
    type Hdr = ();
    type Size = ();
}

impl LensFlareDialect for V1 {
    const VERSION: u32 = 1;
    type LibName = ();
    type Color = ();
    type Hdr = ();
    type Size = ();
}

impl LensFlareDialect for V2 {
    const VERSION: u32 = 2;
    type LibName = ();
    type Color = ();
    type Hdr = ();
    type Size = ();
}

impl LensFlareDialect for V3 {
    const VERSION: u32 = 3;
    type LibName = Ref<targets::Character>;
    type Color = AnimationValue<FixedBytes<4>>;
    type Hdr = AnimationValue<f32>;
    type Size = AnimationValue<f32>;
}

/// Fields of the native `LFLR` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct LensFlare<D: LensFlareDialect> {
    /// Links to the CHAR bytes identifying this lens flare.
    pub name: Ref<targets::Character>,
    /// Sets flare texture atlas.
    pub flare_map: Ref<targets::TextureLayer>,
    /// Links to the mask shaping the lens flare.
    pub mask_map: Ref<targets::TextureLayer>,
    /// Sets sub-flare elements (LFSB).
    pub sub_flares: Ref<targets::SubFlare>,
    /// Sets flipbook grid columns.
    pub columns: u32,
    /// Sets flipbook grid rows.
    pub rows: u32,
    /// Sets distance fade start.
    pub distance_fade: f32,
    /// Links to the CHAR bytes naming the flare library.
    pub lib_name: D::LibName,
    /// Controls intensity through its animation value.
    pub intensity: AnimationValue<f32>,
    /// Controls color through its animation value.
    pub color: D::Color,
    /// Controls HDR multiplier through its animation value.
    pub hdr: D::Hdr,
    /// Controls size through its animation value.
    pub size: D::Size,
}

impl<D: LensFlareDialect> Record for LensFlare<D> {
    const TAG: [u8; 4] = *b"LFLR";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `DataDrivenMaterial` section revisions.
pub trait DataDrivenMaterialDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `extra_hashes`; `()` omits the field.
    type ExtraHashes: NativeField;
    /// Field representation for `effect_name_hash2`; `()` omits the field.
    type EffectNameHash2: NativeField;
    /// Field representation for `effect_name_hash3`; `()` omits the field.
    type EffectNameHash3: NativeField;
}

impl DataDrivenMaterialDialect for V0 {
    const VERSION: u32 = 0;
    type ExtraHashes = ();
    type EffectNameHash2 = ();
    type EffectNameHash3 = ();
}

impl DataDrivenMaterialDialect for V1 {
    const VERSION: u32 = 1;
    type ExtraHashes = ();
    type EffectNameHash2 = ();
    type EffectNameHash3 = ();
}

impl DataDrivenMaterialDialect for V2 {
    const VERSION: u32 = 2;
    type ExtraHashes = Ref<targets::U32>;
    type EffectNameHash2 = ();
    type EffectNameHash3 = ();
}

impl DataDrivenMaterialDialect for V3 {
    const VERSION: u32 = 3;
    type ExtraHashes = Ref<targets::U32>;
    type EffectNameHash2 = u32;
    type EffectNameHash3 = u32;
}

/// Fields of the native `MADD` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct DataDrivenMaterial<D: DataDrivenMaterialDialect> {
    /// Links to the CHAR bytes identifying the material.
    pub material_name: Ref<targets::Character>,
    /// Links to U32_ CRC32 identifiers for shader fragments in their linking order.
    pub fragment_hashes: Ref<targets::U32>,
    /// Links to additional U32_ shader hashes available from revision 2.
    pub extra_hashes: D::ExtraHashes,
    /// Links to the CHAR-encoded dictionary of shader properties.
    pub property_blob: Ref<targets::Character>,
    /// Links to SCHR texture filenames selected by texture properties in the dictionary.
    pub texture_paths: Ref<targets::StringReference>,
    /// Keeps reserved words with a reference-shaped encoding; they do not identify a section.
    pub reserved_0: ReservedReferenceWords,
    /// Keeps reserved words with a reference-shaped encoding; they do not identify a section.
    pub reserved_1: ReservedReferenceWords,
    /// Keeps reserved words with a reference-shaped encoding; they do not identify a section.
    pub reserved_2: ReservedReferenceWords,
    /// Keeps reserved words with a reference-shaped encoding; they do not identify a section.
    pub reserved_3: ReservedReferenceWords,
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
    pub shader_type: u8,
    /// The meaning of the unknown151 storage has not been established; its encoded bits are preserved.
    pub unknown151: u8,
    /// Stores another permutation identifier; all bits set indicates no second permutation (revision 3 onward).
    pub effect_name_hash2: D::EffectNameHash2,
    /// Stores a third permutation identifier; all bits set indicates no third permutation (revision 3 onward).
    pub effect_name_hash3: D::EffectNameHash3,
}

impl<D: DataDrivenMaterialDialect> Record for DataDrivenMaterial<D> {
    const TAG: [u8; 4] = *b"MADD";
    const VERSION: u32 = D::VERSION;
}
