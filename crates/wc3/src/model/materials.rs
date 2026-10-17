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

use crate::model::profiles::{Profile, SurfacePayload, Universal};
use crate::model::{Error, ImageResourceId, Link, ValidateLinks};
use std::borrow::Cow;

/// Texture color interpretation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorSpace {
    /// No transfer function is declared; interpretation follows the material role.
    #[default]
    Unspecified,
    /// Color values use the sRGB transfer function.
    Srgb,
    /// Linear data or color.
    Linear,
    /// Transfer-function selector whose meaning has not been established.
    Unknown(u8),
}
/// Texture address behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AddressMode {
    /// Repeats the image.
    #[default]
    Repeat,
    /// Clamps to the edge.
    Clamp,
    /// Mirrors alternating repetitions.
    Mirror,
    /// Address selector whose meaning has not been established.
    Unknown(u8),
}
/// Texture sampling choices independent of image ownership.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Sampler {
    /// Horizontal addressing.
    pub u: AddressMode,
    /// Vertical addressing.
    pub v: AddressMode,
}
/// A texture resource and its declared interpretation.
#[derive(Clone, Debug)]
pub struct Texture<'a, P: Profile = Universal> {
    /// Additional owner-local behavior permitted by the vocabulary.
    pub controls: P::Texture<'a>,
    /// Resource identity.
    pub resource: ImageResourceId,
    /// Color interpretation.
    pub color_space: ColorSpace,
    /// Default sampler.
    pub sampler: Sampler,
}
impl<'a, P: Profile> Texture<'a, P> {
    /// Supplies resource behavior from this graph's vocabulary.
    pub fn with_controls(mut self, controls: impl Into<P::Texture<'a>>) -> Self {
        self.controls = controls.into();
        self
    }

    /// Creates a texture referencing an image declared in the owning model.
    pub fn new(resource: ImageResourceId) -> Self {
        Self {
            resource,
            controls: Default::default(),
            color_space: ColorSpace::Unspecified,
            sampler: Sampler::default(),
        }
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
