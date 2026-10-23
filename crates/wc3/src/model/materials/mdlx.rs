//! Warcraft III layered surfaces independent of MDX and MDL storage.
use super::bindings::{Coordinates, TextureBinding};
use super::state::{DepthState, Lighting, Operation, PassState, Rasterization};
use crate::model::common::IntoOwned;
use crate::model::materials::state::BlendOperations;
use crate::model::profiles::Unavailable;
use crate::model::TextureTransform;
use crate::model::{Construct, PropertyValues};
use crate::model::{Link, Property, Texture, ValidateLinks};
use glam::Vec3;
use std::borrow::Cow;

/// Blending performed by a Warcraft III material layer.
pub type Blend = BlendOperations<Unavailable, (), AlphaAdditiveEncoding, u32>;
/// Authored spelling of source-opacity-weighted RGB addition in Warcraft III.
///
/// Both selectors add source RGB multiplied by source alpha. Their distinct
/// native declarations are retained so material export preserves the wire value.
#[derive(IntoOwned, Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
#[derive(IntoOwned, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayerOptions {
    /// Select sphere-environment sampling.
    pub sphere_environment: bool,
    /// Authored layer U-wrap control, independent of the texture sampler.
    pub wrap_u: bool,
    /// Authored layer V-wrap control, independent of the texture sampler.
    pub wrap_v: bool,
    /// Authored `Unlit` declaration, distinct from the common `Unshaded` control.
    pub unlit: bool,
    /// Authored back-face shadow participation.
    pub back_faces_for_shadows: bool,
    /// Authored ambient-occlusion participation.
    pub ambient_occlusion: bool,
}
/// A layered pass selects explicit depth operations, face selection, and common lighting.
pub type Shading = Rasterization<DepthState<Operation, Operation>, bool, Lighting, LayerOptions>;

/// One independently animated texture slot in an extended shader.
#[derive(IntoOwned, Clone, Debug, PartialEq)]
pub struct TextureSlot {
    /// Semantic shader slot; undocumented selectors remain numeric.
    pub slot: u32,
    /// Texture sampled by this slot, or no texture.
    pub binding: TextureBinding<Property<Option<Texture>>>,
}
/// Fresnel controls for a material pass.
#[derive(IntoOwned, Clone, Debug, Construct, PartialEq)]
pub struct Fresnel {
    /// Edge color in RGB order.
    pub color: Property<Vec3>,
    /// Edge opacity.
    pub opacity: Property<f32>,
    /// Amount of team-color contribution.
    pub team_color: Property<f32>,
}
/// Material-wide draw declarations shared by the steps of an MDLX material.
#[derive(IntoOwned, Clone, Debug, Default, PartialEq)]
pub struct MaterialDeclaration<'a> {
    /// Material-level named shader declaration, including an unresolved name.
    pub shader_name: Cow<'a, str>,
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
}

/// Image and scalar inputs shared by the classic and default Reforged shaders.
#[derive(IntoOwned, Clone, Debug, Construct, PartialEq)]
pub struct MaterialInputs {
    /// Primary image and referenced coordinates.
    pub binding: TextureBinding<Property<Option<Texture>>, Coordinates<u32, TextureTransform>>,
    /// Opacity multiplier.
    pub opacity: Property<f32>,
    /// Authored emissive multiplier.
    pub emissive_gain: Option<Property<f32>>,
}
/// Shared role and scalar inputs to Reforged DefaultUnit and Crystal shaders.
#[derive(IntoOwned, Clone, Debug, Construct, PartialEq)]
pub struct ReforgedInputs {
    /// Primary image, opacity, and emissive gain.
    pub inputs: MaterialInputs,
    /// Additional texture slots, retaining order and duplicate selectors.
    pub textures: Vec<TextureSlot>,
    /// Authored Fresnel response.
    pub fresnel: Option<Fresnel>,
}
/// Inputs to a shader whose execution is not interpreted.
#[derive(IntoOwned, Clone, Debug, Construct, PartialEq)]
pub struct UnresolvedInputs {
    /// Uninterpreted per-pass selector, when authored.
    pub selector: Option<u32>,
    /// Primary image, opacity, and emissive gain.
    pub inputs: MaterialInputs,
    /// Additional texture slots, retaining order and duplicate selectors.
    pub textures: Vec<TextureSlot>,
    /// Authored Fresnel response.
    pub fresnel: Option<Fresnel>,
}
/// Shader-specific Warcraft III inputs, independent of pass state.
#[derive(IntoOwned, Clone, Debug, PartialEq)]
pub enum Shader {
    /// Classic texture composition.
    Classic(MaterialInputs),
    /// Reforged's default unit shader.
    ReforgedDefault(ReforgedInputs),
    /// Reforged Crystal surface composition, distinct from DefaultUnit.
    ReforgedCrystal(ReforgedInputs),
    /// An uninterpreted shader or input combination.
    Unresolved(UnresolvedInputs),
}
impl Construct for Shader {
    fn construct(values: &mut PropertyValues) -> Self {
        Self::Classic(Construct::construct(values))
    }
}
/// One Warcraft III pass, shared by the MDLX and Universal profiles.
#[derive(IntoOwned, Clone, Debug, Construct, PartialEq)]
pub struct MaterialStep {
    /// Color composition and rasterization for this pass.
    pub state: PassState<Blend, Shading>,
    /// Inputs admitted by the selected shader.
    pub shader: Shader,
}
impl Shader {
    /// Common image and scalar inputs.
    pub fn inputs(&self) -> &MaterialInputs {
        match self {
            Self::Classic(inputs) => inputs,
            Self::ReforgedDefault(value) | Self::ReforgedCrystal(value) => &value.inputs,
            Self::Unresolved(value) => &value.inputs,
        }
    }
    /// Edits the common image and scalar inputs.
    pub fn inputs_mut(&mut self) -> &mut MaterialInputs {
        match self {
            Self::Classic(inputs) => inputs,
            Self::ReforgedDefault(value) | Self::ReforgedCrystal(value) => &mut value.inputs,
            Self::Unresolved(value) => &mut value.inputs,
        }
    }
    /// Additional authored texture slots.
    pub fn textures(&self) -> &[TextureSlot] {
        match self {
            Self::Classic(_) => &[],
            Self::ReforgedDefault(value) | Self::ReforgedCrystal(value) => &value.textures,
            Self::Unresolved(value) => &value.textures,
        }
    }
    /// Edits additional authored texture slots without changing shader family.
    pub fn textures_mut(&mut self) -> &mut [TextureSlot] {
        match self {
            Self::Classic(_) => &mut [],
            Self::ReforgedDefault(value) | Self::ReforgedCrystal(value) => &mut value.textures,
            Self::Unresolved(value) => &mut value.textures,
        }
    }
    /// Authored Fresnel response.
    pub fn fresnel(&self) -> Option<&Fresnel> {
        match self {
            Self::Classic(_) => None,
            Self::ReforgedDefault(value) | Self::ReforgedCrystal(value) => value.fresnel.as_ref(),
            Self::Unresolved(value) => value.fresnel.as_ref(),
        }
    }
    /// Edits an existing Fresnel response.
    pub fn fresnel_mut(&mut self) -> Option<&mut Fresnel> {
        match self {
            Self::Classic(_) => None,
            Self::ReforgedDefault(value) | Self::ReforgedCrystal(value) => value.fresnel.as_mut(),
            Self::Unresolved(value) => value.fresnel.as_mut(),
        }
    }
}
impl ValidateLinks for LayerOptions {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for MaterialInputs {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.binding.visit_links(visitor);
        self.opacity.visit_links(visitor);
        self.emissive_gain.visit_links(visitor);
    }
}
impl ValidateLinks for Shader {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.inputs().visit_links(visitor);
        for slot in self.textures() {
            slot.binding.visit_links(visitor);
        }
        if let Some(fresnel) = self.fresnel() {
            fresnel.color.visit_links(visitor);
            fresnel.opacity.visit_links(visitor);
            fresnel.team_color.visit_links(visitor);
        }
    }
}

impl ValidateLinks for MaterialStep {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.state.visit_links(visitor);
        self.shader.visit_links(visitor);
    }
}

impl ValidateLinks for MaterialDeclaration<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
