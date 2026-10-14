//! Independent Diablo shader state and external SNO dependencies.
#[path = "values.rs"]
mod values;
use crate::model::semantic::{Error, Link, ValidateLinks};
use std::borrow::Cow;
pub use values::*;
/// Authored Diablo asset ref inputs.
#[derive(Clone, Debug, Default)]
pub struct AssetRef {
    /// Identifies the source SNO resource within its group.
    pub id: i32,
    /// Identifies the source SNO resource group.
    pub group: Group,
}
/// Authored Diablo material colors inputs.
#[derive(Clone, Debug, Default)]
pub struct MaterialColors {
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
    pub material_flags: MaterialFlags,
}
/// Authored Diablo tex anim inputs.
#[derive(Clone, Debug, Default)]
pub struct TexAnim {
    /// Sets the magnitude of this source texture animation.
    pub amount: f32,
    /// Stores the first rate parameter of the source texture animation.
    pub rate0: f32,
    /// Stores the second rate parameter of the source texture animation.
    pub rate1: f32,
}
/// Authored Diablo texture entry inputs.
#[derive(Clone, Debug, Default)]
pub struct TextureEntry {
    /// Selects the source shader texture role.
    pub texture_type: TextureType,
    /// The purpose of this source field is unresolved; its authored value is retained.
    pub unclassified_control: i32,
    /// Identifies the source texture resource by SNO group and ID.
    pub texture: AssetRef,
    /// Selects the source UV transformation convention.
    pub uv_transform_mode: UvTransformMode,
    /// Stores one row of the source UV transformation matrix.
    pub uv_row0: [f32; 4],
    /// Stores one row of the source UV transformation matrix.
    pub uv_row1: [f32; 4],
    /// Stores one row of the source UV transformation matrix.
    pub uv_row2: [f32; 4],
    /// Stores one row of the source UV transformation matrix.
    pub uv_row3: [f32; 4],
    /// Stores the U-coordinate animation parameters.
    pub anim_u: TexAnim,
    /// Stores the V-coordinate animation parameters.
    pub anim_v: TexAnim,
    /// Stores the texture rotation animation parameters.
    pub anim_rotate: TexAnim,
    /// The purpose of this source field is unresolved; its authored value is retained.
    pub anim3: TexAnim,
    /// The purpose of this source field is unresolved; its authored value is retained.
    pub anim4: TexAnim,
    /// The purpose of this source field is unresolved; its authored value is retained.
    pub anim5: TexAnim,
    /// Stores source UV-mapping options.
    pub uv_flags: UvFlags,
    /// The purpose of this source field is unresolved; its authored value is retained.
    pub unclassified_control2: i32,
}
/// Authored Diablo uber material inputs.
#[derive(Clone, Debug, Default)]
pub struct UberMaterial {
    /// Identifies the source shader-map resource.
    pub shader_map: AssetRef,
    /// Contains the source material colors.
    pub colors: MaterialColors,
    /// Stores the source material’s texture inputs in authored order.
    pub textures: Vec<TextureEntry>,
}
/// Authored Diablo tag value inputs.
#[derive(Clone, Debug, Default)]
pub struct TagValue {
    /// Selects the encoded type of this source shader parameter.
    pub value_type: ParameterType,
    /// Identifies the source shader parameter.
    pub tag_id: u32,
    /// Stores the authored source shader-tag value.
    pub value: u32,
}
/// Authored Diablo material asset inputs.
#[derive(Clone, Debug, Default)]
pub struct MaterialAsset {
    /// Identifies the source SNO resource.
    pub sno_id: i32,
    /// The purpose of this source field is unresolved; its authored value is retained.
    pub unclassified_control: i32,
    /// Stores source shader-parameter entries in authored order.
    pub shader_params: Vec<TagValue>,
    /// Contains the source shader’s additional material state.
    pub uber: UberMaterial,
}
/// Authored Diablo render params inputs.
#[derive(Clone, Debug, Default)]
pub struct RenderParams {
    /// Selects the source surface-culling mode.
    pub cull_mode: CullMode,
    /// Controls updates to the depth buffer.
    pub z_write_enable: Switch,
    /// Selects the source depth comparison.
    pub z_func: Comparison,
    /// Offsets the source depth comparison.
    pub depth_bias: f32,
    /// The purpose of this source field is unresolved; its authored value is retained.
    pub unclassified_control: f32,
    /// Controls use of the stencil buffer.
    pub stencil_enable: Switch,
    /// Selects the stencil comparison.
    pub stencil_func: Comparison,
    /// Supplies the value used for stencil comparisons.
    pub stencil_ref: i32,
    /// Selects the stencil operation after successful stencil and depth tests.
    pub stencil_pass: StencilOperation,
    /// Selects the stencil operation after a failed stencil test.
    pub stencil_fail: StencilOperation,
    /// Selects the stencil operation after a failed depth test.
    pub stencil_z_fail: StencilOperation,
    /// Controls alpha testing.
    pub alpha_test_enable: Switch,
    /// Selects the alpha comparison.
    pub alpha_func: Comparison,
    /// Supplies the encoded byte used as the alpha-test reference.
    pub alpha_ref: u8,
    /// Controls conversion of alpha to sample coverage.
    pub alpha_to_coverage: Switch,
    /// Controls application of fog.
    pub fog_enable: Switch,
    /// Selects the rasterization fill mode.
    pub fill_mode: FillMode,
    /// Controls writes to color components.
    pub color_write_enable: Switch,
    /// Controls writes to opacity.
    pub alpha_write_enable: Switch,
    /// Controls alpha blending.
    pub alpha_blend_enable: Switch,
    /// Selects the operation combining source and destination colors.
    pub blend_op: BlendOperation,
    /// Selects the source blend factor.
    pub src_blend: BlendFactor,
    /// Selects the destination blend factor.
    pub dest_blend: BlendFactor,
    /// Supplies the constant color used by blending.
    pub constant_color: i32,
}
/// Authored Diablo texture stage inputs.
#[derive(Clone, Debug, Default)]
pub struct TextureStage {
    /// Identifies the source content stage.
    pub content_stage: i32,
    /// Selects texture addressing along U.
    pub address_u: AddressMode,
    /// Selects texture addressing along V.
    pub address_v: AddressMode,
    /// Selects texture addressing along W.
    pub address_w: AddressMode,
    /// Selects source texture filtering.
    pub filter: TextureFilter,
    /// Offsets selection of the texture mipmap level.
    pub mip_map_lod_bias: f32,
}
/// Authored Diablo shader tag value inputs.
#[derive(Clone, Debug, Default)]
pub struct ShaderTagValue {
    /// Selects the encoded type of this source shader parameter.
    pub value_type: ParameterType,
    /// Identifies the source shader parameter.
    pub tag_id: u32,
    /// Stores the authored source shader-tag value.
    pub value: u32,
}
/// Authored Diablo render state inputs.
#[derive(Clone, Debug, Default)]
pub struct RenderState<'a> {
    /// The purpose of this source field is unresolved; its authored value is retained.
    pub unclassified_control: i32,
    /// The purpose of this source field is unresolved; its authored value is retained.
    pub unclassified_control2: i32,
    /// Contains the source draw-state settings.
    pub render_params: RenderParams,
    /// The purpose of this source field is unresolved; its authored value is retained.
    pub unclassified_control3: i32,
    /// Records the number of active source texture stages.
    pub texture_stage_count: i32,
    /// Contains the source texture-stage states.
    pub texture_stages: Vec<TextureStage>,
    /// Stores the source material pass options.
    pub pass_flags: PassFlags,
    /// Names the name of the source effect file.
    pub effect_file: Cow<'a, str>,
    /// Names the source vertex-shader entry name.
    pub vertex_shader_entry: Cow<'a, str>,
    /// Names the source pixel-shader entry name.
    pub pixel_shader_entry: Cow<'a, str>,
    /// Stores source shader-parameter entries in authored order.
    pub shader_params: Vec<ShaderTagValue>,
}
/// Diablo III shader declaration with external dependencies and ordered render passes.
#[derive(Clone, Debug, Default)]
pub struct Material<'a> {
    /// Independently authored subobject appearance qualifier.
    pub appearance_control: i32,
    /// Shader resource, color response, and animated texture inputs.
    pub uber: UberMaterial,
    /// External base-material dependency.
    pub base_material: AssetRef,
    /// External cloth-material dependency.
    pub cloth: AssetRef,
    /// Ordered shader property entries.
    pub shader_params: Vec<TagValue>,
    /// Ordered opaque rasterization passes.
    pub opaque_passes: Vec<RenderState<'a>>,
    /// Ordered translucent rasterization passes.
    pub translucent_passes: Vec<RenderState<'a>>,
}
impl AssetRef {
    fn is_valid(&self) -> bool {
        true
    }
}
impl MaterialColors {
    fn is_valid(&self) -> bool {
        (self.diffuse.iter().all(|value| value.is_finite()))
            && (self.specular.iter().all(|value| value.is_finite()))
            && (self.emissive.iter().all(|value| value.is_finite()))
            && (self.ambient.iter().all(|value| value.is_finite()))
            && (self.shininess.is_finite())
    }
}
impl TexAnim {
    fn is_valid(&self) -> bool {
        (self.amount.is_finite()) && (self.rate0.is_finite()) && (self.rate1.is_finite())
    }
}
impl TextureEntry {
    fn is_valid(&self) -> bool {
        (self.texture.is_valid())
            && (self.uv_row0.iter().all(|value| value.is_finite()))
            && (self.uv_row1.iter().all(|value| value.is_finite()))
            && (self.uv_row2.iter().all(|value| value.is_finite()))
            && (self.uv_row3.iter().all(|value| value.is_finite()))
            && (self.anim_u.is_valid())
            && (self.anim_v.is_valid())
            && (self.anim_rotate.is_valid())
            && (self.anim3.is_valid())
            && (self.anim4.is_valid())
            && (self.anim5.is_valid())
    }
}
impl UberMaterial {
    fn is_valid(&self) -> bool {
        (self.shader_map.is_valid())
            && (self.colors.is_valid())
            && (self.textures.iter().all(|value| value.is_valid()))
    }
}
impl TagValue {
    fn is_valid(&self) -> bool {
        true
    }
}
impl MaterialAsset {
    /// Checks finite independently authored material and texture-animation coefficients.
    pub fn validate(&self) -> Result<(), Error> {
        if self.is_valid() {
            Ok(())
        } else {
            Err(Error::new(
                "d3.material_asset",
                "invalid material or texture-animation coefficient",
            ))
        }
    }
    fn is_valid(&self) -> bool {
        (self.shader_params.iter().all(|value| value.is_valid())) && (self.uber.is_valid())
    }
}
impl RenderParams {
    fn is_valid(&self) -> bool {
        (self.depth_bias.is_finite()) && (self.unclassified_control.is_finite())
    }
}
impl TextureStage {
    fn is_valid(&self) -> bool {
        self.mip_map_lod_bias.is_finite()
    }
}
impl ShaderTagValue {
    fn is_valid(&self) -> bool {
        true
    }
}
impl RenderState<'_> {
    fn is_valid(&self) -> bool {
        (self.render_params.is_valid())
            && (self.texture_stages.iter().all(|value| value.is_valid()))
            && (self.shader_params.iter().all(|value| value.is_valid()))
            && (self.texture_stage_count >= 0)
            && (self.texture_stage_count as usize <= self.texture_stages.len())
    }
}
impl ValidateLinks for Material<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if !self.uber.is_valid()
            || self
                .opaque_passes
                .iter()
                .chain(&self.translucent_passes)
                .any(|pass| !pass.is_valid())
        {
            visitor(Link::InvalidValue);
        }
    }
}
impl Material<'_> {
    /// Checks coefficients and active stage windows before graph publication.
    pub fn validate(&self) -> Result<(), Error> {
        let mut valid = true;
        self.visit_links(&mut |_| valid = false);
        if valid {
            Ok(())
        } else {
            Err(Error::new(
                "d3.material",
                "invalid shader coefficient or active texture stage window",
            ))
        }
    }
}

impl ValidateLinks for MaterialAsset {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if !self.is_valid() {
            visitor(Link::InvalidValue);
        }
    }
}

/// Independently authored shader resource with ordered native render passes.
#[derive(Clone, Debug, Default)]
pub struct ShaderAsset<'a> {
    /// Shader SNO identity.
    pub sno_id: i32,
    /// Shader-wide options, including unresolved bits.
    pub flags: i32,
    /// Unclassified shader-wide scalar.
    pub unclassified_control: i32,
    /// Authored shader name.
    pub name: Cow<'a, str>,
    /// Ordered render passes; classification is not inferred from blending flags.
    pub passes: Vec<RenderState<'a>>,
}
impl ShaderAsset<'_> {
    /// Checks finite rasterization coefficients and active texture-stage windows.
    pub fn validate(&self) -> Result<(), Error> {
        if self.passes.iter().all(RenderState::is_valid) {
            Ok(())
        } else {
            Err(Error::new(
                "d3.shader",
                "invalid pass coefficient or active texture stage window",
            ))
        }
    }
}
/// One independently authored shader-map parameter mapping.
#[derive(Clone, Debug, Default)]
pub struct ShaderMapEntry {
    /// Encoded shader parameter category, including unresolved values.
    pub value_type: ParameterType,
    /// Authored shader selection tag.
    pub tag_id: u32,
    /// Explicit external shader dependency.
    pub shader: AssetRef,
}
/// Independently authored shader-map resource.
#[derive(Clone, Debug, Default)]
pub struct ShaderMapAsset {
    /// Shader-map SNO identity.
    pub sno_id: i32,
    /// Shader-map options, including unresolved bits.
    pub flags: i32,
    /// Parameter mappings in authored order.
    pub entries: Vec<ShaderMapEntry>,
}
