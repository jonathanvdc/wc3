//! Typed material records for indexed WEM payloads and inline values.
use super::*;
use crate::model::wem::{self, Bool, InlineList, Record, Ref};
/// Typed look payload in its serialized field order.
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
pub struct Look {
    /// Links to the CHAR bytes naming this record.
    pub name: Ref<targets::Character>,
    /// Sets the relative selection weight for this look.
    pub weight: i32,
}
impl Record for Look {
    const TAG: [u8; 4] = *b"LOOK";
    const VERSION: u32 = 1;
}

/// Typed look table payload in its serialized field order.
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
pub struct LookTable {
    /// Links to the available appearance variants.
    pub looks: Ref<targets::Look>,
}

/// Typed fresnel feature payload in its serialized field order.
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
pub struct FresnelFeature {
    /// Specifies the color used by the Fresnel contribution.
    pub color: [f32; 3],
    /// Controls the angular sharpness of the Fresnel response.
    pub exponent: f32,
    /// Sets the lower output of the Fresnel response.
    pub out_min: f32,
    /// Sets the upper output of the Fresnel response.
    pub out_max: f32,
    /// Weights the team-color contribution.
    pub team_color: f32,
}
impl Record for FresnelFeature {
    const TAG: [u8; 4] = *b"FFRS";
    const VERSION: u32 = 1;
}

/// Typed uv animation feature payload in its serialized field order.
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
pub struct UvAnimationFeature {
    /// Specifies the change in UV translation over time.
    pub scroll_rate: [f32; 2],
    /// Specifies the change in UV rotation over time.
    pub rotate_rate: f32,
    /// Specifies the change in UV scale over time.
    pub scale_rate: [f32; 2],
}
impl Record for UvAnimationFeature {
    const TAG: [u8; 4] = *b"FUVA";
    const VERSION: u32 = 1;
}

/// Typed layer shading feature payload in its serialized field order.
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
pub struct LayerShadingFeature {
    /// Disables illumination of this layer when nonzero.
    pub unlit: Bool,
    /// Allows the layer to draw both sides of a surface when nonzero.
    pub two_sided: Bool,
    /// Excludes this layer from fog when nonzero.
    pub unfogged: Bool,
    /// Disables depth comparisons for this layer when nonzero.
    pub no_depth_test: Bool,
    /// Disables depth-buffer updates for this layer when nonzero.
    pub no_depth_write: Bool,
}
impl Record for LayerShadingFeature {
    const TAG: [u8; 4] = *b"FLSH";
    const VERSION: u32 = 1;
}

/// Typed material feature payload in its serialized field order.
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
pub struct MaterialFeature {
    /// Identifies the material feature within its owning material.
    pub id: u32,
    /// Selects the material layer to which this feature applies.
    pub layer: u32,
    /// Carries the feature-kind discriminator and its typed section link.
    pub payload: FeaturePayload,
}
impl Record for MaterialFeature {
    const TAG: [u8; 4] = *b"MFEA";
    const VERSION: u32 = 1;
}

/// Typed material record payload in its serialized field order.
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
pub struct MaterialRecord {
    /// Links to the CHAR bytes naming this record.
    pub name: Ref<targets::Character>,
    /// Contains the material state and body shared across game profiles.
    pub common: CommonMaterial,
    /// Records whether the common and native material representations agree or which representation was edited.
    pub sync: NativeSync,
    /// Carries the native-game discriminator and its optional section link.
    pub native: NativeMaterial,
}
impl Record for MaterialRecord {
    const TAG: [u8; 4] = *b"MATL";
    const VERSION: u32 = 1;
}

/// Typed depth state payload in its serialized field order.
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
pub struct DepthState {
    /// Enables comparison against the depth buffer when nonzero.
    pub test: Bool,
    /// Enables updates to the depth buffer when nonzero.
    pub write: Bool,
    /// Offsets the depth used for surface comparisons.
    pub bias: f32,
}

/// Typed matrix3x2f payload in its serialized field order.
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
pub struct Matrix3x2f {
    /// Stores the UV transformation matrix.
    pub m: [[f32; 3]; 2],
}

/// Typed texture input payload in its serialized field order.
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
pub struct TextureInput {
    /// Selects a document texture; the invalid-index sentinel denotes no texture.
    pub texture: u32,
    /// Supplies the input value when a texture contribution is not used.
    pub constant: [f32; 4],
    /// Selects the mesh UV channel for this input.
    pub uv_set: u32,
    /// Selects the mapping convention used to generate texture coordinates.
    pub mapping: UVMappingMode,
    /// Identifies how sampled color components are interpreted.
    pub color_space: ColorSpace,
    /// Selects addressing behavior outside the U coordinate range.
    pub wrap_u: WrapMode,
    /// Selects addressing behavior outside the V coordinate range.
    pub wrap_v: WrapMode,
    /// Transforms the input’s texture coordinates.
    pub uv_transform: Matrix3x2f,
    /// Sets the relative selection weight for this look.
    pub weight: f32,
}

/// Typed composite layer payload in its serialized field order.
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
pub struct CompositeLayer {
    /// Describes the sampled texture or constant contribution.
    pub input: TextureInput,
    /// Selects the destination contribution of this composite layer.
    pub target: SurfaceChannel,
    /// Selects the operation used to combine this layer.
    pub op: CompositeOp,
}
impl Record for CompositeLayer {
    const TAG: [u8; 4] = *b"MKCL";
    const VERSION: u32 = 1;
}

/// Static field selections for `CompositeBody` section or inline context revisions.
pub trait CompositeBodyDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `simulate_roughness`; `()` omits this slot.
    type SimulateRoughness: NativeField;
}

impl CompositeBodyDialect for V1 {
    const VERSION: u32 = 1;
    type SimulateRoughness = ();
}

impl CompositeBodyDialect for V2 {
    const VERSION: u32 = 2;
    type SimulateRoughness = Bool;
}

/// Typed composite body payload in its serialized field order.
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
pub struct CompositeBody<D: CompositeBodyDialect> {
    /// Links to the ordered composite layers.
    pub layers: Ref<targets::CompositeLayer>,
    /// Scales the diffuse-color contribution.
    pub diffuse_factor: [f32; 4],
    /// Scales the self-illumination contribution.
    pub emissive_factor: [f32; 4],
    /// Scales the reflected highlight contribution.
    pub specular_factor: [f32; 4],
    /// Controls the width of the specular highlight.
    pub specular_exponent: f32,
    /// Scales the environment-reflection contribution.
    pub environment_factor: f32,
    /// Requests roughness approximation in the common material representation.
    pub simulate_roughness: D::SimulateRoughness,
}
impl<D: CompositeBodyDialect> Record for CompositeBody<D> {
    const TAG: [u8; 4] = *b"MKCP";
    const VERSION: u32 = D::VERSION;
}

/// Typed combiner stage payload in its serialized field order.
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
pub struct CombinerStage {
    /// Describes the sampled texture or constant contribution.
    pub input: TextureInput,
    /// Describes the combiner operation applied to color components.
    pub rgb: CombinerOp,
    /// Describes the combiner operation applied to opacity.
    pub alpha: CombinerOp,
}
impl Record for CombinerStage {
    const TAG: [u8; 4] = *b"MKCS";
    const VERSION: u32 = 1;
}

/// Typed combiners body payload in its serialized field order.
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
pub struct CombinersBody {
    /// Links to the ordered texture-combiner stages.
    pub stages: Ref<targets::CombinerStage>,
    /// Scales the diffuse-color contribution.
    pub diffuse_factor: [f32; 4],
    /// Scales the self-illumination contribution.
    pub emissive_factor: [f32; 4],
}
impl Record for CombinersBody {
    const TAG: [u8; 4] = *b"MKCB";
    const VERSION: u32 = 1;
}

/// Typed legacy deferred body payload in its serialized field order.
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
pub struct LegacyDeferredBody {
    /// Stores the mappings from material slots to texture inputs inline.
    pub slots: InlineList<LegacySlotBinding>,
    /// Scales the diffuse-color contribution.
    pub diffuse_factor: [f32; 4],
    /// Scales the self-illumination contribution.
    pub emissive_factor: [f32; 4],
    /// Scales the reflected highlight contribution.
    pub specular_factor: [f32; 4],
    /// Controls the width of the specular highlight.
    pub specular_exponent: f32,
    /// Scales the environment-reflection contribution.
    pub environment_factor: f32,
}
impl Record for LegacyDeferredBody {
    const TAG: [u8; 4] = *b"MKLD";
    const VERSION: u32 = 1;
}

/// Typed pbr deferred body payload in its serialized field order.
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
pub struct PbrDeferredBody {
    /// Stores the mappings from material slots to texture inputs inline.
    pub slots: InlineList<PbrSlotBinding>,
    /// Scales the base surface color.
    pub base_color_factor: [f32; 4],
    /// Sets the metallic contribution of the PBR material.
    pub metallic_factor: f32,
    /// Sets the roughness contribution of the PBR material.
    pub roughness_factor: f32,
    /// Scales the self-illumination contribution.
    pub emissive_factor: [f32; 3],
}
impl Record for PbrDeferredBody {
    const TAG: [u8; 4] = *b"MKPB";
    const VERSION: u32 = 1;
}

/// Typed common material payload in its serialized field order.
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
pub struct CommonMaterial {
    /// Selects the compositing mode used to draw the material.
    pub blend: BlendMode,
    /// Selects which surface orientation is excluded from drawing.
    pub cull: CullMode,
    /// Contains depth-test, depth-write, and depth-bias settings.
    pub depth: DepthState,
    /// Sets the opacity cutoff on a normalized zero-to-one scale.
    pub alpha_test_threshold: f32,
    /// Sets the draw-order priority plane.
    pub priority_plane: i32,
    /// Stores common material options as individual bits.
    pub flags: MaterialFlags,
    /// Carries the shading-kind discriminator and its body-section link.
    pub body: MaterialBody,
    /// Links to the additional material feature records.
    pub features: Ref<targets::MaterialFeature>,
}

/// Typed texture path payload in its serialized field order.
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
pub struct TexturePath {
    /// Stores the typed value of this inline table entry.
    pub value: Ref<targets::Character>,
}

/// Typed texture file data id payload in its serialized field order.
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
pub struct TextureFileDataId {
    /// Stores the typed value of this inline table entry.
    pub value: u32,
}

/// Typed texture sno id payload in its serialized field order.
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
pub struct TextureSnoId {
    /// Identifies the source resource namespace or kind.
    pub group: u32,
    /// Identifies the material feature within its owning material.
    pub id: u32,
}

/// Typed texture record payload in its serialized field order.
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
pub struct TextureRecord {
    /// Contains the source resource identity and optional path.
    pub key: TextureKey,
    /// Links to the CHAR bytes used when the source resource is named by a path.
    pub path: Ref<targets::Character>,
    /// Stores common material options as individual bits.
    pub flags: u32,
    /// Identifies a texture supplied through the source format’s replaceable-texture mechanism.
    pub replaceable_id: u32,
    /// Selects the interpretation of this texture slot.
    pub slot_type: u8,
    /// Records the texture’s declared color space.
    pub declared_space: ColorSpace,
}
impl Record for TextureRecord {
    const TAG: [u8; 4] = *b"TEXR";
    const VERSION: u32 = 1;
}

/// Typed legacy slot binding payload in its serialized field order.
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
pub struct LegacySlotBinding {
    /// Selects the material input slot associated with this entry.
    pub slot: LegacySlot,
    /// Describes the sampled texture or constant contribution.
    pub input: TextureInput,
}

/// Typed pbr slot binding payload in its serialized field order.
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
pub struct PbrSlotBinding {
    /// Selects the material input slot associated with this entry.
    pub slot: PbrSlot,
    /// Describes the sampled texture or constant contribution.
    pub input: TextureInput,
}
