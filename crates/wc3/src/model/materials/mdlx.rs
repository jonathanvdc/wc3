//! Warcraft III layered surfaces independent of MDX and MDL storage.
use super::bindings::{Coordinates, TextureBinding};
use super::state::{DepthState, Lighting, Operation, PassState, Rasterization};
use crate::model::materials::state::BlendOperations;
use crate::model::profiles::mdlx::properties::{LayerField, Target};
use crate::model::profiles::SurfacePayload;
use crate::model::profiles::Unavailable;
use crate::model::values::layered::{LayerFlags, MaterialFlags};
use crate::model::TextureTransform;
use crate::model::{
    Animated, Error, Link, PropertyIdentityKey, PropertyPayload, Texture, ValidateLinks,
};
use std::{borrow::Cow, fmt::Debug};

/// Blending performed by a Warcraft III material layer.
pub type Blend = BlendOperations<Unavailable, (), AlphaAdditiveEncoding, u32>;
/// Authored spelling of source-opacity-weighted RGB addition in Warcraft III.
///
/// Both selectors add source RGB multiplied by source alpha. Their distinct
/// native declarations are retained so material export preserves the wire value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlphaAdditiveEncoding {
    /// The native `Additive` selector, also available on atlas particles.
    Additive,
    /// The native `AddAlpha` material-layer selector.
    AddAlpha,
}
impl ValidateLinks for AlphaAdditiveEncoding {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
/// Additional sampling and unresolved controls of a layered pass.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayerOptions {
    /// Select sphere-environment sampling.
    pub sphere_environment: bool,
    /// Preserve controls with unresolved meanings.
    pub unclassified: LayerFlags,
}
/// A layered pass selects explicit depth operations, face selection, and common lighting.
pub type Shading = Rasterization<DepthState<Operation, Operation>, bool, Lighting, LayerOptions>;

/// One independently animated texture slot in an extended shader.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureSlot {
    /// Semantic shader slot; undocumented selectors remain numeric.
    pub slot: u32,
    /// Texture sampled by this slot, or no texture.
    pub binding: TextureBinding<Animated<Option<Texture>>>,
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
/// A native-profile Warcraft III layer, preserving the complete input vocabulary.
/// Universal material programs interpret it into a shader-specific [`MaterialStep`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layer {
    /// Color composition and rasterization applied by this pass.
    pub state: PassState<Blend, Shading>,
    /// Primary image input and its referenced coordinate transform.
    pub binding: TextureBinding<Animated<Option<Texture>>, Coordinates<u32, TextureTransform>>,
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
/// An ordered-pass surface whose pass vocabulary is selected by its owner.
///
/// Profiles select a constrained layer type; custom vocabularies can reuse the
/// surface ordering and shader controls with another complete pass definition.
#[derive(Clone, Debug, PartialEq)]
pub struct LayeredSurface<'a, L = Layer> {
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
    pub layers: Vec<L>,
}

/// A complete pass vocabulary admitted by an ordered layered surface.
pub trait SurfaceLayer: Clone + Debug + Send + Sync + ValidateLinks {
    /// Whether this pass exposes the supplied structural destination at its ordinal.
    fn supports_property(&self, _key: &PropertyIdentityKey, _ordinal: usize) -> bool {
        false
    }
    /// Checks authored pass values before graph publication.
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}
impl SurfaceLayer for Layer {
    fn supports_property(&self, key: &PropertyIdentityKey, ordinal: usize) -> bool {
        match Target::from_identity(key) {
            Some(Target::Layer { index, field }) if index == ordinal => match field {
                LayerField::EmissiveGain => self.emissive_gain.is_some(),
                LayerField::FresnelColor
                | LayerField::FresnelOpacity
                | LayerField::FresnelTeamColor => self.fresnel.is_some(),
                _ => true,
            },
            Some(Target::LayerTexture { layer, slot }) if layer == ordinal => {
                self.textures.get(slot).is_some()
            }
            _ => false,
        }
    }
}
impl<L: SurfaceLayer> SurfacePayload for LayeredSurface<'_, L> {
    fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        self.layers
            .iter()
            .enumerate()
            .any(|(ordinal, layer)| layer.supports_property(key, ordinal))
    }
    fn validate(&self) -> Result<(), Error> {
        if self.has_unexecuted_declarations() {
            return Err(Error::new(
                "material.empty",
                "unused declarations on an empty native material belong to the native codec",
            ));
        }
        self.layers.iter().try_for_each(SurfaceLayer::validate)
    }
}
impl<L: ValidateLinks> ValidateLinks for LayeredSurface<'_, L> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.layers.visit_links(visitor);
    }
}
impl ValidateLinks for Layer {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.state.visit_links(visitor);
        self.binding.visit_links(visitor);
        self.opacity.visit_links(visitor);
        self.emissive_gain.visit_links(visitor);
        if let Some(fresnel) = &self.fresnel {
            fresnel.color.visit_links(visitor);
            fresnel.opacity.visit_links(visitor);
            fresnel.team_color.visit_links(visitor);
        }
        for slot in &self.textures {
            slot.binding.visit_links(visitor);
        }
    }
}

impl<L> Default for LayeredSurface<'_, L> {
    fn default() -> Self {
        Self {
            priority_plane: 0,
            shader: Cow::Borrowed(""),
            constant_color: false,
            two_sided: false,
            sort_near_z: false,
            sort_far_z: false,
            full_resolution: false,
            unclassified: MaterialFlags::default(),
            layers: Vec::new(),
        }
    }
}

impl ValidateLinks for LayerOptions {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}

/// Material-wide draw declarations shared by the steps of an MDLX material.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaterialDeclaration {
    /// Rendering priority relative to other geometry.
    pub priority_plane: i32,
    /// Use constant-color behavior.
    pub constant_color: bool,
    /// Render both sides regardless of the layer's face setting.
    pub two_sided: bool,
    /// Sort primitives near to far along view Z.
    pub sort_near_z: bool,
    /// Sort primitives far to near along view Z.
    pub sort_far_z: bool,
    /// Request full-resolution sampling.
    pub full_resolution: bool,
    /// Shader/draw flags with unresolved meanings.
    pub unclassified: MaterialFlags,
}

/// Inputs and draw state shared by the classic and Reforged default shaders.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaterialInputs {
    /// Composition and rasterization for this step.
    pub state: PassState<Blend, Shading>,
    /// Primary image and referenced coordinates.
    pub binding: TextureBinding<Animated<Option<Texture>>, Coordinates<u32, TextureTransform>>,
    /// Opacity multiplier.
    pub opacity: Animated<f32>,
    /// Authored emissive multiplier.
    pub emissive_gain: Option<Animated<f32>>,
}
/// A classic Warcraft III texture-composition step.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClassicMaterialStep {
    /// Shader inputs and rendering state.
    pub inputs: MaterialInputs,
    /// Material-wide draw declarations.
    pub declaration: MaterialDeclaration,
}
/// A Reforged DefaultUnit shader step.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReforgedDefaultMaterialStep {
    /// Primary image, opacity, emissive gain, and rendering state.
    pub inputs: MaterialInputs,
    /// Material-wide draw declarations.
    pub declaration: MaterialDeclaration,
    /// Authored shader texture slots, retaining order and duplicate selectors.
    pub textures: Vec<TextureSlot>,
    /// Authored Fresnel response.
    pub fresnel: Option<Fresnel>,
}
/// An uninterpreted shader declaration, retaining all authored inputs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UnresolvedMaterialStep<'a> {
    /// Uninterpreted material shader name.
    pub shader_name: Cow<'a, str>,
    /// Uninterpreted layer inputs and selectors.
    pub layer: Layer,
    /// Material-wide draw declarations.
    pub declaration: MaterialDeclaration,
}
/// A Warcraft III material step with a shader-specific input vocabulary.
#[derive(Clone, Debug, PartialEq)]
pub enum MaterialStep<'a> {
    /// Classic texture composition.
    Classic(ClassicMaterialStep),
    /// Reforged's default unit shader.
    ReforgedDefault(ReforgedDefaultMaterialStep),
    /// A shader or input combination without an interpreted vocabulary.
    Unresolved(UnresolvedMaterialStep<'a>),
}
impl Default for MaterialStep<'_> {
    fn default() -> Self {
        Self::Classic(ClassicMaterialStep::default())
    }
}
impl<'a> From<Layer> for MaterialStep<'a> {
    fn from(layer: Layer) -> Self {
        Self::from_layer(layer, MaterialDeclaration::default(), Cow::Borrowed(""))
    }
}
impl<'a> MaterialStep<'a> {
    /// Interprets native shader selection once, retaining unresolved input combinations.
    pub fn from_layer(
        layer: Layer,
        declaration: MaterialDeclaration,
        shader_name: Cow<'a, str>,
    ) -> Self {
        let family = shader_family(layer.shader, &shader_name);
        if family == ShaderFamily::Unresolved
            || (family == ShaderFamily::Classic
                && (!layer.textures.is_empty() || layer.fresnel.is_some()))
        {
            return Self::Unresolved(UnresolvedMaterialStep {
                layer,
                declaration,
                shader_name,
            });
        }
        let Layer {
            state,
            binding,
            opacity,
            emissive_gain,
            shader: _,
            textures,
            fresnel,
        } = layer;
        let inputs = MaterialInputs {
            state,
            binding,
            opacity,
            emissive_gain,
        };
        if family == ShaderFamily::ReforgedDefault {
            Self::ReforgedDefault(ReforgedDefaultMaterialStep {
                inputs,
                declaration,
                textures,
                fresnel,
            })
        } else {
            Self::Classic(ClassicMaterialStep {
                inputs,
                declaration,
            })
        }
    }
    /// Material-wide draw declarations.
    pub fn declaration(&self) -> &MaterialDeclaration {
        match self {
            Self::Classic(v) => &v.declaration,
            Self::ReforgedDefault(v) => &v.declaration,
            Self::Unresolved(v) => &v.declaration,
        }
    }
    /// Edits the material-wide draw declarations.
    pub fn declaration_mut(&mut self) -> &mut MaterialDeclaration {
        match self {
            Self::Classic(v) => &mut v.declaration,
            Self::ReforgedDefault(v) => &mut v.declaration,
            Self::Unresolved(v) => &mut v.declaration,
        }
    }
    /// Projects inputs into native layer vocabulary, deriving known shader selectors.
    pub fn to_layer(&self) -> Layer {
        let (inputs, shader, textures, fresnel) = match self {
            Self::Classic(v) => (&v.inputs, None, Vec::new(), None),
            Self::ReforgedDefault(v) => (&v.inputs, Some(1), v.textures.clone(), v.fresnel.clone()),
            Self::Unresolved(v) => return v.layer.clone(),
        };
        Layer {
            state: inputs.state,
            binding: inputs.binding.clone(),
            opacity: inputs.opacity.clone(),
            emissive_gain: inputs.emissive_gain.clone(),
            shader,
            textures,
            fresnel,
        }
    }
    /// Whether material-wide draw declarations agree.
    pub(crate) fn same_declaration(&self, other: &Self) -> bool {
        self.declaration() == other.declaration()
    }
    /// Whether this step exposes the native structural animation destination.
    pub fn supports_property(&self, key: &PropertyIdentityKey, ordinal: usize) -> bool {
        let (inputs, fresnel, slots) = match self {
            Self::Classic(v) => (&v.inputs, false, 0),
            Self::ReforgedDefault(v) => (&v.inputs, v.fresnel.is_some(), v.textures.len()),
            Self::Unresolved(v) => return v.layer.supports_property(key, ordinal),
        };
        match Target::from_identity(key) {
            Some(Target::Layer { index, field }) if index == ordinal => match field {
                LayerField::EmissiveGain => inputs.emissive_gain.is_some(),
                LayerField::FresnelColor
                | LayerField::FresnelOpacity
                | LayerField::FresnelTeamColor => fresnel,
                _ => true,
            },
            Some(Target::LayerTexture { layer, slot }) => layer == ordinal && slot < slots,
            _ => false,
        }
    }
}
impl ValidateLinks for MaterialInputs {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.state.visit_links(visitor);
        self.binding.visit_links(visitor);
        self.opacity.visit_links(visitor);
        self.emissive_gain.visit_links(visitor);
    }
}
impl ValidateLinks for MaterialStep<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Classic(v) => v.inputs.visit_links(visitor),
            Self::ReforgedDefault(v) => {
                v.inputs.visit_links(visitor);
                for slot in &v.textures {
                    slot.binding.visit_links(visitor);
                }
                if let Some(fresnel) = &v.fresnel {
                    fresnel.color.visit_links(visitor);
                    fresnel.opacity.visit_links(visitor);
                    fresnel.team_color.visit_links(visitor);
                }
            }
            Self::Unresolved(v) => v.layer.visit_links(visitor),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum ShaderFamily {
    Classic,
    ReforgedDefault,
    Unresolved,
}
fn shader_family(selector: Option<u32>, name: &str) -> ShaderFamily {
    match selector {
        Some(1) => ShaderFamily::ReforgedDefault,
        Some(0 | 2) => ShaderFamily::Classic,
        Some(_) => ShaderFamily::Unresolved,
        None => match name {
            "Shader_HD_DefaultUnit" => ShaderFamily::ReforgedDefault,
            "" | "Shader_SD_FixedFunction" => ShaderFamily::Classic,
            _ => ShaderFamily::Unresolved,
        },
    }
}

impl<L> LayeredSurface<'_, L> {
    /// Unused native declarations cannot describe any shader execution.
    pub(crate) fn has_unexecuted_declarations(&self) -> bool {
        self.layers.is_empty()
            && (self.priority_plane != 0
                || !self.shader.is_empty()
                || self.constant_color
                || self.two_sided
                || self.sort_near_z
                || self.sort_far_z
                || self.full_resolution
                || self.unclassified.bits() != 0)
    }
}
