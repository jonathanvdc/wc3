//! Resource identities, sampling, and composable complete material surfaces.
/// Diablo III shader and resource vocabulary.
pub mod diablo3;
/// Diablo IV external shader parameter vocabulary.
pub mod diablo4;
/// Ordered animated material passes.
pub mod layered;
/// Model shader passes and texture inputs.
pub mod model_shader;
/// Programmed surface families and animated shader properties.
pub mod programmed;
/// Static-world surface shaders.
pub mod world;

/// Texture identity, coordinate, sampling, and contribution compositions.
pub mod bindings;
/// Surface response contributions.
pub mod contributions;
/// Ordered shader parameter entries and their typed domains.
pub mod parameters;
/// Composable draw state and rasterization controls.
pub mod state;

use crate::model::semantic::profiles::{Profile, SurfacePayload, Universal};
use crate::model::semantic::{Error, Link, PropertyIdentityKey, ValidateLinks};
use std::borrow::Cow;

/// Game namespace of an external Diablo resource identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SnoNamespace {
    /// The producer supplied numeric keys without a game namespace.
    #[default]
    Unspecified,
    /// Diablo III resource groups and identifiers.
    Diablo3,
    /// Diablo IV resource groups and identifiers.
    Diablo4,
}
/// Resource identity interpreted by an application resolver, not by a file layout.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Resource<'a> {
    /// No primary resource identity is declared.
    Unspecified,
    /// Unresolved caller-provided asset root.
    Root,
    /// Resource resolved relative to another logical asset identity.
    Relative {
        /// Identity supplying the filename stem.
        owner: Box<Resource<'a>>,
        /// Suffix appended to that stem.
        suffix: Cow<'a, str>,
    },
    /// Native path spelling, including non-UTF-8 paths.
    Path(Cow<'a, [u8]>),
    /// World of Warcraft resource identity.
    FileDataId(u32),
    /// Diablo resource identity and resource group.
    Sno {
        /// Game whose resource group and identifier are addressed.
        namespace: SnoNamespace,
        /// Resource group.
        group: i32,
        /// Resource identity.
        id: u32,
    },
    /// Application-provided replacement texture category.
    Replaceable(u32),
    /// Image bytes supplied with the authoring data, with no established encoding interpretation.
    EmbeddedImage {
        /// Authored image bytes, borrowed when available.
        bytes: Cow<'a, [u8]>,
    },
    /// A logical resource with an explicit namespace.
    Named {
        /// Provider namespace.
        namespace: Cow<'a, str>,
        /// Provider identity.
        name: Cow<'a, str>,
    },
}
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
    pub resource: Resource<'a>,
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

    /// Creates a filename texture without copying a borrowed UTF-8 filename.
    pub fn file(path: &'a str) -> Self {
        Self {
            resource: Resource::Path(Cow::Borrowed(path.as_bytes())),
            controls: Default::default(),
            color_space: ColorSpace::Unspecified,
            sampler: Sampler::default(),
        }
    }
}
impl ValidateLinks for Sampler {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}

/// A profile-selected set of complete surface families.
///
/// Each parameter is a semantic surface vocabulary. An unavailable family can
/// use `Unavailable`; one-family profiles can select their surface directly.
#[derive(Clone, Debug)]
pub enum SurfaceSet<D3, D4, Layered, ModelShader, Programmed, World> {
    /// Shader resources and render passes with Diablo III semantics.
    D3(Box<D3>),
    /// External material selections and shader parameters.
    D4(Box<D4>),
    /// Ordered animated layered shader passes.
    Layered(Box<Layered>),
    /// Model shader pass with independently animated inputs.
    ModelShader(Box<ModelShader>),
    /// Programmed surface family.
    Programmed(Box<Programmed>),
    /// Static-world surface shader.
    World(Box<World>),
}
/// The broad built-in selection of complete semantic surface families.
pub type MaterialSurface<'a> = SurfaceSet<
    diablo3::Material<'a>,
    diablo4::Material,
    layered::LayeredSurface<'a>,
    model_shader::ShaderPass<'a>,
    programmed::Material<'a>,
    world::Material<'a>,
>;
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

impl Default for Resource<'_> {
    fn default() -> Self {
        Self::Path(Cow::Borrowed(&[]))
    }
}

impl<
        D3: ValidateLinks,
        D4: ValidateLinks,
        Layered: ValidateLinks,
        ModelShader: ValidateLinks,
        Programmed: ValidateLinks,
        World: ValidateLinks,
    > ValidateLinks for SurfaceSet<D3, D4, Layered, ModelShader, Programmed, World>
{
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::D4(value) => value.visit_links(visitor),
            Self::D3(value) => value.visit_links(visitor),
            Self::Layered(value) => value.visit_links(visitor),
            Self::ModelShader(value) => value.visit_links(visitor),
            Self::Programmed(value) => value.visit_links(visitor),
            Self::World(value) => value.visit_links(visitor),
        }
    }
}
impl<
        D3: SurfacePayload,
        D4: SurfacePayload,
        Layered: SurfacePayload,
        ModelShader: SurfacePayload,
        Programmed: SurfacePayload,
        World: SurfacePayload,
    > SurfacePayload for SurfaceSet<D3, D4, Layered, ModelShader, Programmed, World>
{
    fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        match self {
            Self::Layered(v) => v.supports_property(key),
            Self::ModelShader(v) => v.supports_property(key),
            Self::Programmed(v) => v.supports_property(key),

            Self::D3(v) => v.supports_property(key),
            Self::D4(v) => v.supports_property(key),
            Self::World(v) => v.supports_property(key),
        }
    }

    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::D4(value) => value.validate(),
            Self::D3(value) => value.validate(),

            Self::Layered(value) => value.validate(),
            Self::ModelShader(value) => value.validate(),
            Self::Programmed(value) => value.validate(),
            Self::World(value) => value.validate(),
        }
    }
}
