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

use bindings::{Coordinates, TextureBinding};
use contributions::SurfaceContribution;
/// Texture identity, coordinate, sampling, and contribution compositions.
pub mod bindings;
/// Surface response contributions.
pub mod contributions;
/// Ordered shader parameter entries and their typed domains.
pub mod parameters;
/// Composable draw state and rasterization controls.
pub mod state;

use crate::model::semantic::profiles::{Profile, SurfacePayload, Unavailable, Universal};
use crate::model::semantic::{
    Animated, Error, Link, PropertyIdentityKey, TextureId, ValidateLinks,
};
use state::{DepthState, Lighting, Operation, PassState, Rasterization};
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
/// Meaning of a common texture input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureRole {
    /// Surface color.
    BaseColor,
    /// Tangent-space normal.
    Normal,
    /// Occlusion, roughness, and metallic channels.
    Surface,
    /// Emissive color.
    Emissive,
    /// Team or player color.
    TeamColor,
    /// Environment reflection.
    Environment,
    /// Shader-family-defined input.
    Custom(u32),
}
/// A resource bound to a surface input.
#[derive(Clone, Debug)]
pub struct TextureInput<
    Role = TextureRole,
    Binding = TextureBinding<Animated<Option<TextureId>>, Coordinates<u32>, Option<Sampler>>,
> {
    /// Semantic surface input selected by this shader vocabulary.
    pub role: Role,
    /// Independently chosen image, coordinates, sampling, and contribution.
    pub binding: Binding,
}
impl<Role: ValidateLinks, Binding: ValidateLinks> ValidateLinks for TextureInput<Role, Binding> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.role.visit_links(visitor);
        self.binding.visit_links(visitor);
    }
}
impl ValidateLinks for Sampler {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}

/// Blending behavior independent of native numeric codes.
pub type Blend = state::BlendOperations<PortableBlend, f32, (), Unavailable>;
/// Additional portable compositing operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PortableBlend {
    /// Combined blending and addition.
    BlendAdd,
    /// Alpha has already been applied to source color.
    Premultiplied,
}
impl ValidateLinks for PortableBlend {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
/// Which triangle orientations are culled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Cull {
    /// Cull back faces.
    #[default]
    Back,
    /// Cull front faces.
    Front,
    /// Draw both orientations.
    None,
}
/// Independent shadow participation for a surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shadows {
    /// Contribute to shadow maps.
    pub cast: bool,
    /// Apply shadow-map illumination.
    pub receive: bool,
}
impl Default for Shadows {
    fn default() -> Self {
        Self {
            cast: true,
            receive: true,
        }
    }
}
impl ValidateLinks for Shadows {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for Cull {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for TextureRole {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
/// Portable surfaces select explicit depth operations, lighting, culling, and shadows.
pub type RenderState =
    PassState<Blend, Rasterization<DepthState<Operation, Operation>, Cull, Lighting, Shadows>, i32>;

/// A portable physically based surface.
#[derive(Clone, Debug)]
pub struct PbrSurface {
    /// Rasterization, blending, lighting, and shadow behavior.
    pub state: RenderState,
    /// Base color multiplier.
    pub base_color: SurfaceContribution<(), [f32; 4]>,
    /// Metalness multiplier.
    pub metallic: f32,
    /// Perceptual roughness multiplier.
    pub roughness: f32,
    /// Emissive color multiplier.
    pub emissive: SurfaceContribution<(), [f32; 3]>,
    /// Texture inputs.
    pub inputs: Vec<TextureInput>,
}
impl Default for PbrSurface {
    fn default() -> Self {
        Self {
            state: RenderState::default(),
            base_color: SurfaceContribution {
                input: (),
                parameters: [1.; 4],
            },
            metallic: 0.,
            roughness: 1.,
            emissive: SurfaceContribution {
                input: (),
                parameters: [0.; 3],
            },
            inputs: Vec::new(),
        }
    }
}
/// A profile-selected set of complete surface families.
///
/// Each parameter is a semantic surface vocabulary. An unavailable family can
/// use `Unavailable`; one-family profiles can select their surface directly.
#[derive(Clone, Debug)]
pub enum SurfaceSet<D3, D4, Generic, Layered, ModelShader, Programmed, World> {
    /// Shader resources and render passes with Diablo III semantics.
    D3(Box<D3>),
    /// External material selections and shader parameters.
    D4(Box<D4>),
    /// Physically based surface.
    Generic(Box<Generic>),
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
    PbrSurface,
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
impl<'a> Material<'a> {
    /// Creates a portable PBR material.
    pub fn pbr() -> Self {
        Self {
            name: Cow::Borrowed(""),
            surface: MaterialSurface::Generic(Box::default()),
        }
    }
    /// Names this material.
    pub fn named(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Binds the base-color texture of a portable surface.
    pub fn base_color(mut self, texture: TextureId) -> Result<Self, Error> {
        if let MaterialSurface::Generic(surface) = &mut self.surface {
            surface.inputs.push(TextureInput {
                role: TextureRole::BaseColor,
                binding: TextureBinding {
                    image: Animated::constant(Some(texture)),
                    coordinates: Coordinates {
                        source: 0,
                        transform: (),
                    },
                    sampling: None,
                    contribution: (),
                },
            });
        } else {
            return Err(Error::new(
                "material.surface",
                "base color builder requires a portable surface",
            ));
        }
        Ok(self)
    }
    pub(crate) fn validate_portable(&self) -> Result<(), Error> {
        if let MaterialSurface::Generic(p) = &self.surface {
            if matches!(p.state.blend,Blend::AlphaKey(v) if !v.is_finite() || !(0. ..=1.).contains(&v))
            {
                return Err(Error::new(
                    "material.alpha_test",
                    "threshold must be within zero and one",
                ));
            }
            if p.base_color
                .parameters
                .iter()
                .chain(&p.emissive.parameters)
                .any(|x| !x.is_finite())
                || !p.metallic.is_finite()
                || !(0. ..=1.).contains(&p.metallic)
                || !p.roughness.is_finite()
                || !(0. ..=1.).contains(&p.roughness)
            {
                return Err(Error::new(
                    "material.surface",
                    "surface values must be finite and physical multipliers between zero and one",
                ));
            }
        }
        Ok(())
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
        Generic: ValidateLinks,
        Layered: ValidateLinks,
        ModelShader: ValidateLinks,
        Programmed: ValidateLinks,
        World: ValidateLinks,
    > ValidateLinks for SurfaceSet<D3, D4, Generic, Layered, ModelShader, Programmed, World>
{
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::D4(value) => value.visit_links(visitor),
            Self::D3(value) => value.visit_links(visitor),
            Self::Generic(value) => value.visit_links(visitor),
            Self::Layered(value) => value.visit_links(visitor),
            Self::ModelShader(value) => value.visit_links(visitor),
            Self::Programmed(value) => value.visit_links(visitor),
            Self::World(value) => value.visit_links(visitor),
        }
    }
}
impl ValidateLinks for PbrSurface {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.state.visit_links(visitor);
        self.base_color.visit_links(visitor);
        self.emissive.visit_links(visitor);
        for input in &self.inputs {
            input.visit_links(visitor);
        }
    }
}
impl SurfacePayload for PbrSurface {
    fn supports_property(&self, _key: &PropertyIdentityKey) -> bool {
        false
    }

    fn validate(&self) -> Result<(), Error> {
        Material {
            name: "".into(),
            surface: MaterialSurface::Generic(Box::new(self.clone())),
        }
        .validate_portable()
    }
}
impl<
        D3: SurfacePayload,
        D4: SurfacePayload,
        Generic: SurfacePayload,
        Layered: SurfacePayload,
        ModelShader: SurfacePayload,
        Programmed: SurfacePayload,
        World: SurfacePayload,
    > SurfacePayload for SurfaceSet<D3, D4, Generic, Layered, ModelShader, Programmed, World>
{
    fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        match self {
            Self::Layered(v) => v.supports_property(key),
            Self::ModelShader(v) => v.supports_property(key),
            Self::Programmed(v) => v.supports_property(key),

            Self::D3(v) => v.supports_property(key),
            Self::D4(v) => v.supports_property(key),
            Self::World(v) => v.supports_property(key),
            Self::Generic(value) => value.supports_property(key),
        }
    }

    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::D4(value) => value.validate(),
            Self::D3(value) => value.validate(),
            Self::Generic(value) => value.validate(),

            Self::Layered(value) => value.validate(),
            Self::ModelShader(value) => value.validate(),
            Self::Programmed(value) => value.validate(),
            Self::World(value) => value.validate(),
        }
    }
}
