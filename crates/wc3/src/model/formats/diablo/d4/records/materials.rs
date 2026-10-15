//! Native materials records.
use super::*;
/// Native `TexAnimTranslationCurve` (208 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TexAnimTranslationCurve {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `flUScale` at byte 16.
    #[d4(value)]
    pub u_scale: f32,
    /// Native `flVScale` at byte 20.
    #[d4(value)]
    pub v_scale: f32,
    /// Native `pathU` at byte 24.
    pub path_u: InterpolationPathFloat,
    /// Native `pathV` at byte 80.
    pub path_v: InterpolationPathFloat,
    /// Native `dwAnimFlags` at byte 136.
    #[d4(value)]
    pub anim_flags: u32,
    /// Native `tUScaleScalar` at byte 140.
    pub u_scale_scalar: InterpolationScalar,
    /// Native `tVScaleScalar` at byte 172.
    pub v_scale_scalar: InterpolationScalar,
    /// Unclassified native bytes at offset 204.
    #[d4(value)]
    pub reserved_204: [u8; 4],
}

/// Native `UberMaterial` (56 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct UberMaterial<'data> {
    /// Native `snoShaderMap` at byte 0.
    #[d4(value)]
    pub shader_map: i32,
    /// Native `snoMaterialValueSetOverride` at byte 4.
    #[d4(value)]
    pub material_value_set_override: i32,
    /// Native `mat` at byte 8.
    pub mat: Material,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `ptMatTexList` at byte 16.
    #[d4(array)]
    pub mat_tex_list: Vec<MaterialTextureEntry<'data>>,
    /// Native `ptRunTimeMaterialValues` at byte 32.
    #[d4(array)]
    pub run_time_material_values: Vec<RunTimeMaterialValues>,
    /// Native `nTexAnimStateCount` at byte 48.
    #[d4(value)]
    pub tex_anim_state_count: i32,
    /// Native `fHasGraph` at byte 52.
    #[d4(value)]
    pub has_graph: i32,
}

/// Native `MaterialTextureEntry` (48 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct MaterialTextureEntry<'data> {
    /// Native `eShaderTex` at byte 0.
    #[d4(value)]
    pub shader_tex: i32,
    /// Unclassified native bytes at offset 4.
    #[d4(value)]
    pub reserved_4: [u8; 4],
    /// Native `tMatTex` at byte 8.
    pub mat_tex: MaterialTexture<'data>,
}

/// Native `ParametricClutterIndex` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ParametricClutterIndex {
    /// Native `eBaseClutterIndex` at byte 0.
    #[d4(value)]
    pub base_clutter_index: i32,
    /// Unclassified native bytes at offset 4.
    #[d4(value)]
    pub reserved_4: [u8; 4],
    /// Native `arParametricClutterIndexLayers` at byte 8.
    #[d4(array)]
    pub parametric_clutter_index_layers: Vec<ParametricClutterIndexLayer>,
}

/// Native `TexAnim` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TexAnim {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `flUScale` at byte 16.
    #[d4(value)]
    pub u_scale: f32,
    /// Native `flVScale` at byte 20.
    #[d4(value)]
    pub v_scale: f32,
}

/// Native `TexAnimFrameByFrame` (64 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TexAnimFrameByFrame {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `flUScale` at byte 16.
    #[d4(value)]
    pub u_scale: f32,
    /// Native `flVScale` at byte 20.
    #[d4(value)]
    pub v_scale: f32,
    /// Native `tFrameAnim` at byte 24.
    pub frame_anim: FrameAnim,
    /// Native `fScaleUVsToFrame` at byte 60.
    #[d4(value)]
    pub scale_uvs_to_frame: i32,
}

/// Native `RunTimeMaterialValues` (64 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct RunTimeMaterialValues {
    /// Native `arMaterialScalarValues` at byte 0.
    #[d4(array)]
    pub material_scalar_values: Vec<RunTimeMaterialScalarValueEntry>,
    /// Native `arMaterialScalarGraphs` at byte 16.
    #[d4(array)]
    pub material_scalar_graphs: Vec<RunTimeMaterialScalarGraphEntry>,
    /// Native `arMaterialVectorValues` at byte 32.
    #[d4(array)]
    pub material_vector_values: Vec<RunTimeMaterialVectorValueEntry>,
    /// Native `arMaterialVectorGraphs` at byte 48.
    #[d4(array)]
    pub material_vector_graphs: Vec<RunTimeMaterialVectorGraphEntry>,
}

/// Native `Material` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct Material {
    /// Native `nSortPri` at byte 0.
    #[d4(value)]
    pub sort_pri: i32,
}

/// Native `RunTimeMaterialScalarGraphEntry` (56 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct RunTimeMaterialScalarGraphEntry {
    /// Native `pathValGraph` at byte 0.
    pub path_val_graph: InterpolationPathFloat,
}

/// Native `TextureOverride` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TextureOverride {
    /// Native `eShaderTexOverride` at byte 0.
    #[d4(value)]
    pub shader_tex_override: i32,
    /// Native `snoTexture` at byte 4.
    #[d4(value)]
    pub texture: i32,
}

/// Native `ParametricClutterIndexLayer` (132 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ParametricClutterIndexLayer {
    /// Native `eClutterIndexLayer` at byte 0.
    #[d4(value)]
    pub clutter_index_layer: i32,
    /// Native `tSettings` at byte 4.
    pub settings: ParametricSettings,
}

/// Native `RunTimeMaterialVectorGraphEntry` (56 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct RunTimeMaterialVectorGraphEntry {
    /// Native `pathValGraph` at byte 0.
    pub path_val_graph: InterpolationPathBcVec4,
}

/// Native `FoleyMaterialTag` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct FoleyMaterialTag {
    /// Native `szTag` at byte 0.
    #[d4(value)]
    pub tag: u32,
}

/// Native `RunTimeMaterialScalarValueEntry` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct RunTimeMaterialScalarValueEntry {
    /// Native `tValue` at byte 0.
    pub value: RunTimeMaterialValueFloat,
}

/// Native `RunTimeMaterialVectorValueEntry` (28 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct RunTimeMaterialVectorValueEntry {
    /// Native `tValue` at byte 0.
    pub value: RunTimeMaterialValueBcVec4,
}

/// Native `RunTimeMaterialValue_float` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct RunTimeMaterialValueFloat {
    /// Native `snoMaterialValue` at byte 0.
    #[d4(value)]
    pub material_value: i32,
    /// Native `value` at byte 4.
    #[d4(value)]
    pub value: f32,
    /// Native `nPathValGraphIndex` at byte 8.
    #[d4(value)]
    pub path_val_graph_index: i32,
    /// Native `eLightGroup` at byte 12.
    #[d4(value)]
    pub light_group: i32,
}

/// Native `TexAnimFrameByFrameCurve` (136 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TexAnimFrameByFrameCurve {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `flUScale` at byte 16.
    #[d4(value)]
    pub u_scale: f32,
    /// Native `flVScale` at byte 20.
    #[d4(value)]
    pub v_scale: f32,
    /// Native `tFrameAnim` at byte 24.
    pub frame_anim: FrameAnim,
    /// Unclassified native bytes at offset 60.
    #[d4(value)]
    pub reserved_60: [u8; 4],
    /// Native `pathFramesOverTime` at byte 64.
    pub path_frames_over_time: InterpolationPathInt32,
    /// Native `tDuration` at byte 120.
    #[d4(value)]
    pub duration: [f32; 2],
    /// Native `fScaleUVsToFrame` at byte 128.
    #[d4(value)]
    pub scale_uvs_to_frame: i32,
    /// Unclassified native bytes at offset 132.
    #[d4(value)]
    pub reserved_132: [u8; 4],
}

/// Native `MaterialDefinition` (568 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct MaterialRecord<'data> {
    /// Resource SNO identity from the root prefix.
    #[d4(value)]
    pub sno_id: i32,
    /// Root record flags.
    #[d4(value)]
    pub record_flags: u32,
    /// Native `dwFlags` at byte 8.
    #[d4(value)]
    pub flags: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tUberMaterial` at byte 16.
    pub uber_material: UberMaterial<'data>,
    /// Native `arDecalLookVariantMap` at byte 72.
    #[d4(array)]
    pub decal_look_variant_map: Vec<DecalLookVariantMap>,
    /// Native `snoSurface` at byte 88.
    #[d4(value)]
    pub surface: i32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `arBiomeSettings` at byte 96.
    #[d4(inline_array)]
    pub biome_settings: [BiomeSetting; 4],
    /// Native `tParametricClutterIndex` at byte 352.
    pub parametric_clutter_index: ParametricClutterIndex,
    /// Native `unk_93ea20a` at byte 376.
    #[d4(value)]
    pub unk_93ea20a: f32,
    /// Unclassified native bytes at offset 380.
    #[d4(value)]
    pub reserved_380: [u8; 4],
    /// Native `pathPlaybackMultiplier` at byte 384.
    pub path_playback_multiplier: InterpolationPathFloat,
    /// Native `pathPlaybackController` at byte 440.
    pub path_playback_controller: InterpolationPathFloat,
    /// Native `unk_ce6fe3b` at byte 496.
    #[d4(value)]
    pub unk_ce6fe3b: u32,
    /// Native `unk_245935` at byte 500.
    #[d4(value)]
    pub unk_245935: i32,
    /// Native `unk_6d58226` at byte 504.
    #[d4(value)]
    pub unk_6d58226: i32,
    /// Native `flDuration` at byte 508.
    #[d4(value)]
    pub duration: f32,
    /// Native `tOutroDuration` at byte 512.
    #[d4(value)]
    pub outro_duration: f32,
    /// Native `unk_a08119e` at byte 516.
    #[d4(value)]
    pub unk_a08119e: f32,
    /// Native `unk_5b1ce37` at byte 520.
    #[d4(value)]
    pub unk_5b1ce37: f32,
    /// Native `flStiffness` at byte 524.
    #[d4(value)]
    pub stiffness: f32,
    /// Native `flGravityMagnitude` at byte 528.
    #[d4(value)]
    pub gravity_magnitude: f32,
    /// Native `tFxAnimatedInfluences` at byte 532.
    pub fx_animated_influences: FxAnimatedInfluences,
    /// Unclassified native bytes at offset 564.
    #[d4(value)]
    pub reserved_564: [u8; 4],
}

/// Native `RunTimeMaterialValue_bcVec4` (28 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct RunTimeMaterialValueBcVec4 {
    /// Native `snoMaterialValue` at byte 0.
    #[d4(value)]
    pub material_value: i32,
    /// Native `value` at byte 4.
    #[d4(value)]
    pub value: [f32; 4],
    /// Native `nPathValGraphIndex` at byte 20.
    #[d4(value)]
    pub path_val_graph_index: i32,
    /// Native `eLightGroup` at byte 24.
    #[d4(value)]
    pub light_group: i32,
}

/// Native `BiomeSetting` (64 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct BiomeSetting {
    /// Native `snoBiome` at byte 0.
    #[d4(value)]
    pub biome: i32,
    /// Native `unk_936122c` at byte 4.
    #[d4(value)]
    pub unk_936122c: i32,
    /// Native `tClutterFalloff` at byte 8.
    pub clutter_falloff: InterpolationPathFloat,
}

/// Native `TexAnimTranslation` (224 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TexAnimTranslation {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `flUScale` at byte 16.
    #[d4(value)]
    pub u_scale: f32,
    /// Native `flVScale` at byte 20.
    #[d4(value)]
    pub v_scale: f32,
    /// Native `flUTransInitial` at byte 24.
    #[d4(value)]
    pub u_trans_initial: f32,
    /// Native `unk_803e8a7` at byte 28.
    #[d4(value)]
    pub unk_803e8a7: [f32; 2],
    /// Native `tUTransVelScalar` at byte 36.
    pub u_trans_vel_scalar: InterpolationScalar,
    /// Native `flVTransInitial` at byte 68.
    #[d4(value)]
    pub v_trans_initial: f32,
    /// Native `unk_42b1708` at byte 72.
    #[d4(value)]
    pub unk_42b1708: [f32; 2],
    /// Native `tVTransVelScalar` at byte 80.
    pub v_trans_vel_scalar: InterpolationScalar,
    /// Native `ePinTranslation` at byte 112.
    #[d4(value)]
    pub pin_translation: i32,
    /// Native `eOrderOperation` at byte 116.
    #[d4(value)]
    pub order_operation: i32,
    /// Native `dwURandomSeed` at byte 120.
    #[d4(value)]
    pub u_random_seed: u32,
    /// Native `dwVRandomSeed` at byte 124.
    #[d4(value)]
    pub v_random_seed: u32,
    /// Native `dwAnimFlags` at byte 128.
    #[d4(value)]
    pub anim_flags: u32,
    /// Native `unk_e5b0c7f` at byte 132.
    #[d4(value)]
    pub unk_e5b0c7f: [f32; 2],
    /// Native `fXOffset` at byte 140.
    #[d4(value)]
    pub x_offset: f32,
    /// Native `fYOffset` at byte 144.
    #[d4(value)]
    pub y_offset: f32,
    /// Native `aRotationInitial` at byte 148.
    #[d4(value)]
    pub a_rotation_initial: f32,
    /// Native `dwRotationRandomSeed` at byte 152.
    #[d4(value)]
    pub rotation_random_seed: u32,
    /// Native `tUScaleScalar` at byte 156.
    pub u_scale_scalar: InterpolationScalar,
    /// Native `tVScaleScalar` at byte 188.
    pub v_scale_scalar: InterpolationScalar,
    /// Unclassified native bytes at offset 220.
    #[d4(value)]
    pub reserved_220: [u8; 4],
}

/// Native `MaterialTexture` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct MaterialTexture<'data> {
    /// Native `snoTex` at byte 0.
    #[d4(value)]
    pub tex: i32,
    /// Unclassified native bytes at offset 4.
    #[d4(value)]
    pub reserved_4: [u8; 4],
    /// Native `ptTexAnim` at byte 8.
    #[d4(polymorphic = 911604182)]
    pub tex_anim: Vec<Polymorphic<'data>>,
    /// Native `nTexAnimStateIndex` at byte 32.
    #[d4(value)]
    pub tex_anim_state_index: i32,
    /// Native `unk_e1a4010` at byte 36.
    #[d4(value)]
    pub unk_e1a4010: i32,
}

/// Native `AppearanceMaterial` (32 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AppearanceMaterial<'data> {
    /// Native `dwMaterialHash` at byte 0.
    #[d4(value)]
    pub material_hash: u32,
    /// Native `fPersonaMaterial` at byte 4.
    #[d4(value)]
    pub persona_material: i32,
    /// Native `dwPersona` at byte 8.
    #[d4(value)]
    pub persona: u8,
    /// Unclassified native bytes at offset 9.
    #[d4(value)]
    pub reserved_9: [u8; 7],
    /// Native `ptSOAs` at byte 16.
    #[d4(array)]
    pub so_as: Vec<SubObjectAppearance<'data>>,
}
