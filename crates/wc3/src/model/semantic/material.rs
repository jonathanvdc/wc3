//! Logical texture resources, common render state, and complete profile-specific surfaces.
use crate::model::semantic::profiles;
use crate::model::semantic::profiles::{Profile, SurfacePayload, Universal};
use crate::model::semantic::{Animated, Error, TextureId};
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
    /// An image supplied with the authoring data rather than resolved externally.
    EmbeddedImage {
        /// Explicit interpretation of the image bytes.
        encoding: ImageEncoding,
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
/// Interpretation of an embedded image payload.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ImageEncoding {
    /// The original image representation has no established channel or compression interpretation.
    #[default]
    Unspecified,
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
pub struct TextureInput {
    /// Semantic surface input.
    pub role: TextureRole,
    /// Animated texture resource.
    pub texture: Animated<Option<TextureId>>,
    /// UV set used by this input.
    pub uv_set: u32,
    /// Per-input sampler override.
    pub sampler: Option<Sampler>,
}
/// Blending behavior independent of native numeric codes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Blend {
    /// Replaces destination color.
    #[default]
    Opaque,
    /// Discards fragments below an alpha threshold.
    AlphaTest(f32),
    /// Standard alpha compositing.
    Alpha,
    /// Adds source color.
    Add,
    /// Adds alpha-weighted source color.
    AddAlpha,
    /// Multiplies destination color.
    Modulate,
    /// Doubled modulation.
    Modulate2x,
    /// Combined blending and addition.
    BlendAdd,
    /// Alpha has already been applied to source color.
    Premultiplied,
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
/// Shared surface rendering state.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderState {
    /// Color blending.
    pub blend: Blend,
    /// Face culling.
    pub cull: Cull,
    /// Test depth.
    pub depth_test: bool,
    /// Write depth.
    pub depth_write: bool,
    /// Receive lighting.
    pub lit: bool,
    /// Receive scene fog.
    pub fog: bool,
    /// Cast shadows.
    pub cast_shadows: bool,
    /// Receive shadows.
    pub receive_shadows: bool,
    /// Authored ordering priority.
    pub priority: i32,
}
impl Default for RenderState {
    fn default() -> Self {
        Self {
            blend: Blend::Opaque,
            cull: Cull::Back,
            depth_test: true,
            depth_write: true,
            lit: true,
            fog: true,
            cast_shadows: true,
            receive_shadows: true,
            priority: 0,
        }
    }
}
/// A portable physically based surface.
#[derive(Clone, Debug)]
pub struct PbrSurface {
    /// Rasterization, blending, lighting, and shadow behavior.
    pub state: RenderState,
    /// Base color multiplier.
    pub base_color: [f32; 4],
    /// Metalness multiplier.
    pub metallic: f32,
    /// Perceptual roughness multiplier.
    pub roughness: f32,
    /// Emissive color multiplier.
    pub emissive: [f32; 3],
    /// Texture inputs.
    pub inputs: Vec<TextureInput>,
}
impl Default for PbrSurface {
    fn default() -> Self {
        Self {
            state: RenderState::default(),
            base_color: [1.; 4],
            metallic: 0.,
            roughness: 1.,
            emissive: [0.; 3],
            inputs: Vec::new(),
        }
    }
}
/// Complete logical surface definition, retaining distinct game shader semantics.
#[derive(Clone, Debug)]
pub enum MaterialSurface<'a> {
    /// Diablo III shader resources and render passes.
    D3(Box<profiles::diablo::d3::Material<'a>>),
    /// Diablo IV external material selections and shader definitions.
    D4(Box<profiles::diablo::d4::Material>),
    /// Portable surface.
    Generic(Box<PbrSurface>),
    /// Warcraft III layers and shader parameters.
    Mdlx(Box<profiles::mdlx::Material<'a>>),
    /// World of Warcraft model surface.
    M2(Box<profiles::m2::Material<'a>>),
    /// StarCraft II surface.
    M3(Box<profiles::m3::Material<'a>>),
    /// World-model surface.
    Wmo(Box<profiles::wmo::Material<'a>>),
    /// Editable-model common and native material authorities.
    Wem(Box<profiles::wem::Material<'a>>),
}
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
                texture: Animated::constant(Some(texture)),
                uv_set: 0,
                sampler: None,
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
            if matches!(p.state.blend,Blend::AlphaTest(v) if !v.is_finite() || !(0. ..=1.).contains(&v))
            {
                return Err(Error::new(
                    "material.alpha_test",
                    "threshold must be within zero and one",
                ));
            }
            if p.base_color
                .iter()
                .chain(&p.emissive)
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
