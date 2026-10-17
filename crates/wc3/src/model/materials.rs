//! Resource identities, sampling, and composable complete material surfaces.
/// Diablo III shader and resource vocabulary.
pub mod diablo3;
/// Diablo IV external shader parameter vocabulary.
pub mod diablo4;
/// Model shader steps and texture inputs.
pub mod m2;
/// M3 shader and composition steps with animated inputs.
pub mod m3;
/// Ordered animated material steps.
pub mod mdlx;
/// Material programs and the universal shader-dialect step vocabulary.
pub mod program;
/// World of Warcraft world shader steps.
pub mod wmo;

/// Texture identity, coordinate, sampling, and contribution compositions.
pub mod bindings;
/// Surface response contributions.
pub mod contributions;
/// Ordered shader parameter entries and their typed domains.
pub mod parameters;
/// Composable draw state and rasterization controls.
pub mod state;

use crate::model::profiles::{
    m2::TextureControls as M2TextureControls, mdlx::TextureControls as MdlxTextureControls,
};
use crate::model::profiles::{Profile, SurfacePayload, Universal};
use crate::model::{Error, ImageResourceId, Link, TextureControls, ValidateLinks};
use std::borrow::Cow;

/// Texture color interpretation.
#[zerovec::make_ule(ColorSpaceULE)]
#[zerovec::skip_derive(Ord)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorSpace {
    /// No transfer function is declared; interpretation follows the material role.
    #[default]
    Unspecified = 0,
    /// Color values use the sRGB transfer function.
    Srgb = 1,
    /// Linear data or color.
    Linear = 2,
}
/// Texture address behavior.
#[zerovec::make_ule(AddressModeULE)]
#[zerovec::skip_derive(Ord)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AddressMode {
    /// Repeats the image.
    #[default]
    Repeat = 0,
    /// Clamps to the edge.
    Clamp = 1,
    /// Mirrors alternating repetitions.
    Mirror = 2,
}
/// Texture sampling choices independent of image ownership.
#[zerovec::make_ule(SamplerULE)]
#[zerovec::skip_derive(Ord)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sampler {
    /// Horizontal addressing.
    pub u: AddressMode,
    /// Vertical addressing.
    pub v: AddressMode,
}
/// A complete sampled image value owned by its material or effect input.
/// Image bytes and resource identities remain shared through `resource`.
#[zerovec::make_ule(TextureULE)]
#[zerovec::skip_derive(Ord)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Texture {
    /// Native sampling declarations with established owner-local semantics.
    pub controls: Option<TextureControls>,
    /// Shared image dependency.
    pub resource: ImageResourceId,
    /// Interpretation of the selected image values.
    pub color_space: ColorSpace,
    /// Complete address behavior for this input.
    pub sampler: Sampler,
}
impl Texture {
    /// Creates an independently editable input over a shared image dependency.
    pub fn new(resource: ImageResourceId) -> Self {
        Self {
            resource,
            controls: None,
            color_space: ColorSpace::Unspecified,
            sampler: Sampler::default(),
        }
    }
    /// Warcraft III sampling declarations, rejecting incompatible native behavior.
    pub(crate) fn mdlx_controls(self) -> Result<MdlxTextureControls, Error> {
        match self.controls {
            None => Ok(Default::default()),
            Some(TextureControls::Mdlx(value)) => Ok(value),
            _ => Err(Error::new(
                "texture.controls",
                "input carries sampling declarations outside the Warcraft III vocabulary",
            )),
        }
    }
    /// World of Warcraft sampling declarations, rejecting incompatible native behavior.
    pub(crate) fn m2_controls(self) -> Result<M2TextureControls, Error> {
        match self.controls {
            None => Ok(Default::default()),
            Some(TextureControls::M2(value)) => Ok(value),
            _ => Err(Error::new(
                "texture.controls",
                "input carries sampling declarations outside the M2 vocabulary",
            )),
        }
    }
    /// Attaches native sampling declarations.
    pub fn with_controls(mut self, controls: impl Into<TextureControls>) -> Self {
        self.controls = Some(controls.into());
        self
    }
}
impl ValidateLinks for Texture {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::SampledImage(*self));
        self.resource.visit_links(visitor);
        self.controls.visit_links(visitor);
    }
}
impl ValidateLinks for Sampler {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}

pub use program::MaterialSurface;
/// A named, complete logical surface description.
#[derive(Clone, Debug)]
pub struct Material<'a, P: Profile = Universal> {
    /// Display name.
    pub name: Cow<'a, str>,
    /// Complete surface description.
    pub surface: P::Surface<'a>,
}
impl<'a, P: Profile> Material<'a, P> {
    /// Names this material.
    pub fn named(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
}

impl<P: Profile> Material<'_, P> {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        self.surface.validate()
    }
}
