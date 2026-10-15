//! Warcraft III layered surfaces independent of MDX and MDL storage.
use crate::model::semantic::profiles::mdlx::properties::{LayerField, Target};
use crate::model::semantic::profiles::SurfacePayload;
use crate::model::semantic::values::layered::{LayerFlags, MaterialFlags};
use crate::model::semantic::TextureTransformId;
use crate::model::semantic::{
    Animated, Error, Link, PropertyIdentityKey, PropertyPayload, TextureId, ValidateLinks,
};
use std::{borrow::Cow, fmt::Debug};

/// Blending performed by a Warcraft III material layer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Blend {
    /// Replace the destination color.
    #[default]
    Opaque,
    /// Discard transparent texels.
    AlphaKey,
    /// Blend using source opacity.
    Alpha,
    /// Add source color.
    Additive,
    /// Add opacity-weighted source color.
    AdditiveAlpha,
    /// Multiply destination color.
    Modulate,
    /// Multiply destination color and double the result.
    Modulate2x,
    /// A source operation whose meaning is not established.
    Unknown(u32),
}
/// Sampling and lighting controls for an individual pass.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shading {
    /// Bypass directional lighting.
    pub unlit: bool,
    /// Render both sides of the geometry.
    pub two_sided: bool,
    /// Bypass scene fog.
    pub unfogged: bool,
    /// Disable depth comparison.
    pub no_depth_test: bool,
    /// Disable depth writes.
    pub no_depth_write: bool,
    /// Keep sphere-environment sampling enabled.
    pub sphere_environment: bool,
    /// Wrap the U coordinate at layer scope.
    pub wrap_u: bool,
    /// Wrap the V coordinate at layer scope.
    pub wrap_v: bool,
    /// Preserve shading controls with unresolved meanings.
    pub unclassified: LayerFlags,
}
/// One independently animated texture slot in an extended shader.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureSlot {
    /// Semantic shader slot; undocumented selectors remain numeric.
    pub slot: u32,
    /// Texture sampled by this slot, or no texture.
    pub texture: Animated<Option<TextureId>>,
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
/// One Warcraft III draw pass. Pass ordering is the order in the material.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layer {
    /// Color composition operation.
    pub blend: Blend,
    /// Lighting, depth, and face controls.
    pub shading: Shading,
    /// Animated primary texture selection.
    pub texture: Animated<Option<TextureId>>,
    /// Index of the mesh UV set sampled by this pass.
    pub uv_set: u32,
    /// Local texture-transform identity within the profile.
    pub texture_transform: Option<TextureTransformId>,
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
        self.texture_transform.visit_links(visitor);
        self.texture.default.visit_links(visitor);
        self.texture.visit_links(visitor);
        self.opacity.visit_links(visitor);
        self.emissive_gain.visit_links(visitor);
        if let Some(fresnel) = &self.fresnel {
            fresnel.color.visit_links(visitor);
            fresnel.opacity.visit_links(visitor);
            fresnel.team_color.visit_links(visitor);
        }
        for slot in &self.textures {
            slot.texture.default.visit_links(visitor);
            slot.texture.visit_links(visitor);
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
