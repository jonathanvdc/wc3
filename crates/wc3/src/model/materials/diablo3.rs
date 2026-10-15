//! Independent Diablo shader state and external SNO dependencies.

use super::bindings::{Coordinates, TextureBinding};
use super::parameters::{OpaqueWord, OrderedParameters, ParameterEntry, ShaderTag};
use super::state::{BlendEquation, DepthState, PassState, Rasterization};
pub use crate::model::values::diablo3::*;
use crate::model::{Error, Link, ValidateLinks};
use std::borrow::Cow;
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
/// Matrix and independently authored parametric coordinate animations.
#[derive(Clone, Debug, Default)]
pub struct TextureTransform {
    /// Four authored matrix rows, preserved without transposition.
    pub matrix: [[f32; 4]; 4],
    /// U, V, rotation, and three unresolved animation channels, in source order.
    pub animations: [TexAnim; 6],
    /// Coordinate-mapping options, including unresolved bits.
    pub flags: UvFlags,
}
/// Shader role and additional declarations of one texture contribution.
#[derive(Clone, Debug, Default)]
pub struct TextureContribution {
    /// Producer-defined shader texture role.
    pub role: TextureType,
    /// Independently authored unresolved controls in source order.
    pub unclassified_controls: [i32; 2],
}
/// A Diablo III external image with its selected coordinate program and shader role.
pub type TextureEntry = TextureBinding<
    AssetRef,
    Coordinates<UvTransformMode, TextureTransform>,
    (),
    TextureContribution,
>;

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
pub type TagValue = ParameterEntry<ShaderTag, OpaqueWord, ParameterType>;
/// Authored Diablo material asset inputs.
#[derive(Clone, Debug, Default)]
pub struct MaterialAsset {
    /// Identifies the source SNO resource.
    pub sno_id: i32,
    /// The purpose of this source field is unresolved; its authored value is retained.
    pub unclassified_control: i32,
    /// Stores source shader-parameter entries in authored order.
    pub shader_params: OrderedParameters<ShaderTag, OpaqueWord, ParameterType>,
    /// Contains the source shader’s additional material state.
    pub uber: UberMaterial,
}
/// Additional draw controls with explicit stencil, alpha-test, and write policies.
#[derive(Clone, Debug, Default)]
pub struct RenderControls {
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
    /// Selects the rasterization fill mode.
    pub fill_mode: FillMode,
    /// Controls writes to color components.
    pub color_write_enable: Switch,
    /// Controls writes to opacity.
    pub alpha_write_enable: Switch,
}
/// Complete draw state selecting the authored equation and rasterization domains.
pub type RenderParams = PassState<
    BlendEquation<Switch, BlendOperation, BlendFactor, i32>,
    Rasterization<DepthState<Comparison, Switch, f32>, CullMode, Switch, RenderControls>,
>;
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
pub type ShaderTagValue = ParameterEntry<ShaderTag, OpaqueWord, ParameterType>;
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
    pub shader_params: OrderedParameters<ShaderTag, OpaqueWord, ParameterType>,
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
    pub shader_params: OrderedParameters<ShaderTag, OpaqueWord, ParameterType>,
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
impl TextureTransform {
    fn is_valid(&self) -> bool {
        self.matrix.iter().flatten().all(|value| value.is_finite())
            && self.animations.iter().all(TexAnim::is_valid)
    }
}
impl ValidateLinks for TextureTransform {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if !self.is_valid() {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for TextureContribution {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for AssetRef {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl TextureEntry {
    fn is_valid(&self) -> bool {
        self.image.is_valid() && self.coordinates.transform.is_valid()
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
        (self.rasterization.depth.bias.is_finite())
            && (self.rasterization.controls.unclassified_control.is_finite())
    }
}
impl TextureStage {
    fn is_valid(&self) -> bool {
        self.mip_map_lod_bias.is_finite()
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
pub type ShaderMapEntry = ParameterEntry<ShaderTag, AssetRef, ParameterType>;
/// Independently authored shader-map resource.
#[derive(Clone, Debug, Default)]
pub struct ShaderMapAsset {
    /// Shader-map SNO identity.
    pub sno_id: i32,
    /// Shader-map options, including unresolved bits.
    pub flags: i32,
    /// Parameter mappings in authored order.
    pub entries: OrderedParameters<ShaderTag, AssetRef, ParameterType>,
}

impl ValidateLinks for UvTransformMode {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}

impl ValidateLinks for ParameterType {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}

impl ValidateLinks for RenderControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.unclassified_control.visit_links(visitor);
    }
}
macro_rules! state_selectors {
    ($($selector:ty),* $(,)?) => { $(
        impl ValidateLinks for $selector {
            fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
        }
    )* };
}
state_selectors!(Switch, BlendOperation, BlendFactor, Comparison, CullMode);
