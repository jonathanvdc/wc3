//! WEM shading representations are authored independently and select their own authority.
mod shader_values;
mod values;
mod warcraft;
mod wow;
use crate::model::semantic::profiles::SurfacePayload;
use crate::model::semantic::{
    ColorSpace, Error, Id, Link, Sampler, Scope, TextureId, ValidateLinks,
};
use bitfield::bitfield;
pub use shader_values::*;
pub use values::*;
pub use warcraft::*;
pub use wow::*;

bitfield! {
    /// Shading and visibility options retained independently of the selected shader body.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct MaterialFlags(u32);
    /// Complete authored option bits.
    pub bits, _:31,0;
    /// Excludes illumination.
    pub unlit,set_unlit:0;
    /// Excludes scene fog.
    pub unfogged,set_unfogged:1;
    /// Excludes shadow casting.
    pub no_shadow_cast,set_no_shadow_cast:2;
    /// Excludes shadow reception.
    pub no_shadow_receive,set_no_shadow_receive:3;
    /// Sorts using the nearest depth.
    pub sort_near_z,set_sort_near_z:4;
    /// Sorts using the farthest depth.
    pub sort_far_z,set_sort_far_z:5;
    /// Suppresses drawing.
    pub invisible,set_invisible:6;
}
impl MaterialFlags {
    /// Constructs options without discarding unnamed bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
/// Which independently authored shading representation supplies the selected meaning.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RepresentationAuthority {
    /// Both representations are declared synchronized.
    Synchronized,
    /// The common shader representation was edited.
    Common,
    /// The game-specific shader representation was edited.
    Specialized,
    /// No synchronization relationship is declared.
    #[default]
    Unspecified,
    /// Unrecognized synchronization declaration.
    Unknown(u8),
}
/// Complete authored WEM surface, without cached documents or hidden comparison baselines.
#[derive(Clone, Debug)]
pub struct Material<'a> {
    /// Common shader and draw state.
    pub common: CommonSurface,
    /// Independently authored game-specific shader, when supplied.
    pub specialized: Option<SpecializedSurface<'a>>,
    /// Declared edit authority; no implicit reconciliation is performed.
    pub authority: RepresentationAuthority,
}
/// Game-specific shading represented alongside the common WEM shader.
#[derive(Clone, Debug)]
pub enum SpecializedSurface<'a> {
    /// A declared native family whose material payload has not been supplied.
    Declared(ShaderFamily),
    /// Warcraft III layer semantics.
    Warcraft(WarcraftMaterial<'a>),
    /// World of Warcraft texture-unit semantics.
    Wow(WowMaterial),
    /// StarCraft II shader semantics.
    Starcraft(Box<starcraft::StarcraftMaterial<'a>>),
    /// Diablo shader resource declarations and render passes.
    Diablo(Box<diablo::DiabloMaterial<'a>>),
}
/// Common draw state and complete shader body.
#[derive(Clone, Debug)]
pub struct CommonSurface {
    /// Color compositing operation.
    pub blend: BlendMode,
    /// Orientation excluded from drawing.
    pub cull: CullMode,
    /// Depth comparison setting.
    pub depth_test: bool,
    /// Depth-buffer update setting.
    pub depth_write: bool,
    /// Comparison depth offset.
    pub depth_bias: f32,
    /// Normalized opacity threshold.
    pub alpha_test_threshold: f32,
    /// Drawing order priority.
    pub priority: i32,
    /// Lighting, fog, sorting, and visibility options.
    pub flags: MaterialFlags,
    /// Complete common shader inputs and operations.
    pub body: Body,
    /// Additional layer-local shading operations.
    pub features: Vec<LayerFeature>,
}
/// Sampled texture or constant contribution.
#[derive(Clone, Debug)]
pub struct Input {
    /// Referenced texture, if this contribution samples an image.
    pub texture: Option<TextureId>,
    /// Constant contribution when an image is absent.
    pub constant: [f32; 4],
    /// Texture-coordinate stream.
    pub uv_set: u32,
    /// Generated-coordinate convention.
    pub mapping: UVMappingMode,
    /// Transfer-function interpretation.
    pub color_space: ColorSpace,
    /// Texture addressing.
    pub sampler: Sampler,
    /// Affine UV transform.
    pub uv_transform: [[f32; 3]; 2],
    /// Relative contribution weight.
    pub weight: f32,
}
/// Ordered common-shader operations.
#[derive(Clone, Debug)]
pub enum Body {
    /// Layered compositing shader.
    Composite {
        /// Ordered composite contributions.
        layers: Vec<CompositeLayer>,
        /// RGBA diffuse-color multiplier.
        diffuse: [f32; 4],
        /// Self-illumination multiplier.
        emissive: [f32; 4],
        /// RGBA reflected-highlight multiplier.
        specular: [f32; 4],
        /// Concentration of the specular highlight.
        specular_exponent: f32,
        /// Environment-reflection multiplier.
        environment: f32,
        /// Requests roughness approximation for this shader.
        simulate_roughness: bool,
    },
    /// Independent color and opacity combiners.
    Combiners {
        /// Ordered color and opacity combiner stages.
        stages: Vec<CombinerStage>,
        /// RGBA diffuse-color multiplier.
        diffuse: [f32; 4],
        /// Self-illumination multiplier.
        emissive: [f32; 4],
    },
    /// Deferred legacy-lighting inputs.
    LegacyDeferred {
        /// Shader-role texture inputs in authored order.
        slots: Vec<SlotInput<LegacySlot>>,
        /// RGBA diffuse-color multiplier.
        diffuse: [f32; 4],
        /// Self-illumination multiplier.
        emissive: [f32; 4],
        /// RGBA reflected-highlight multiplier.
        specular: [f32; 4],
        /// Concentration of the specular highlight.
        specular_exponent: f32,
        /// Environment-reflection multiplier.
        environment: f32,
    },
    /// Deferred physically based inputs.
    PbrDeferred {
        /// Shader-role texture inputs in authored order.
        slots: Vec<SlotInput<PbrSlot>>,
        /// RGBA base-color multiplier.
        base_color: [f32; 4],
        /// Metallic contribution multiplier.
        metallic: f32,
        /// Perceptual roughness multiplier.
        roughness: f32,
        /// Self-illumination multiplier.
        emissive: [f32; 3],
    },
}
/// One composite contribution.
#[derive(Clone, Debug)]
pub struct CompositeLayer {
    id: ShaderLayerId,
    /// Texture or constant contribution.
    pub input: Input,
    /// Surface contribution receiving the layer.
    pub target: SurfaceChannel,
    /// Layer combination operation.
    pub operation: CompositeOp,
}
/// One color/opacity combiner contribution.
#[derive(Clone, Debug)]
pub struct CombinerStage {
    id: ShaderLayerId,
    /// Texture or constant contribution.
    pub input: Input,
    /// Color-component combination operation.
    pub rgb: CombinerOp,
    /// Opacity combination operation.
    pub alpha: CombinerOp,
}
/// Identity marker for one shader input, independent of its current list position.
pub enum ShaderLayerObject {}
/// Shader input identity, resolved within the surface owning it.
pub type ShaderLayerId = Id<ShaderLayerObject>;
/// An identified texture input assigned to a deferred shader role.
#[derive(Clone, Debug)]
pub struct SlotInput<Role> {
    id: ShaderLayerId,
    /// Shader role receiving this contribution.
    pub slot: Role,
    /// Sampled image and constant contribution.
    pub input: Input,
}
impl<Role> SlotInput<Role> {
    /// Creates an independently identified contribution to a shader role.
    pub fn new(slot: Role, input: Input) -> Self {
        Self {
            id: Scope::default().allocate(),
            slot,
            input,
        }
    }
    /// Identity retained when this contribution changes list position.
    pub fn id(&self) -> ShaderLayerId {
        self.id
    }
}
impl CompositeLayer {
    /// Creates an independently identified compositing contribution.
    pub fn new(input: Input, target: SurfaceChannel, operation: CompositeOp) -> Self {
        Self {
            id: Scope::default().allocate(),
            input,
            target,
            operation,
        }
    }
    /// Identity retained when this contribution changes list position.
    pub fn id(&self) -> ShaderLayerId {
        self.id
    }
}
impl CombinerStage {
    /// Creates an independently identified color and opacity combiner stage.
    pub fn new(input: Input, rgb: CombinerOp, alpha: CombinerOp) -> Self {
        Self {
            id: Scope::default().allocate(),
            input,
            rgb,
            alpha,
        }
    }
    /// Identity retained when this contribution changes list position.
    pub fn id(&self) -> ShaderLayerId {
        self.id
    }
}
impl Body {
    /// Iterates input identities in the shader's authored operation order.
    pub fn layer_ids(&self) -> impl ExactSizeIterator<Item = ShaderLayerId> + '_ {
        let count = match self {
            Self::Composite { layers, .. } => layers.len(),
            Self::Combiners { stages, .. } => stages.len(),
            Self::LegacyDeferred { slots, .. } => slots.len(),
            Self::PbrDeferred { slots, .. } => slots.len(),
        };
        (0..count).map(|index| match self {
            Self::Composite { layers, .. } => layers[index].id(),
            Self::Combiners { stages, .. } => stages[index].id(),
            Self::LegacyDeferred { slots, .. } => slots[index].id(),
            Self::PbrDeferred { slots, .. } => slots[index].id(),
        })
    }
    /// Resolves an input without depending on its current encoded layer position.
    pub fn input(&self, id: ShaderLayerId) -> Option<&Input> {
        match self {
            Self::Composite { layers, .. } => layers
                .iter()
                .find(|value| value.id() == id)
                .map(|value| &value.input),
            Self::Combiners { stages, .. } => stages
                .iter()
                .find(|value| value.id() == id)
                .map(|value| &value.input),
            Self::LegacyDeferred { slots, .. } => slots
                .iter()
                .find(|value| value.id() == id)
                .map(|value| &value.input),
            Self::PbrDeferred { slots, .. } => slots
                .iter()
                .find(|value| value.id() == id)
                .map(|value| &value.input),
        }
    }
    /// Edits an input while preserving the references selecting it.
    pub fn input_mut(&mut self, id: ShaderLayerId) -> Option<&mut Input> {
        match self {
            Self::Composite { layers, .. } => layers
                .iter_mut()
                .find(|value| value.id() == id)
                .map(|value| &mut value.input),
            Self::Combiners { stages, .. } => stages
                .iter_mut()
                .find(|value| value.id() == id)
                .map(|value| &mut value.input),
            Self::LegacyDeferred { slots, .. } => slots
                .iter_mut()
                .find(|value| value.id() == id)
                .map(|value| &mut value.input),
            Self::PbrDeferred { slots, .. } => slots
                .iter_mut()
                .find(|value| value.id() == id)
                .map(|value| &mut value.input),
        }
    }
}
/// An individually identified shader feature attached to one layer.
#[derive(Clone, Debug)]
pub struct LayerFeature {
    /// Authored feature identity within the surface.
    pub id: u32,
    /// Actual shader input receiving this feature.
    pub layer: ShaderLayerId,
    /// Layer combination operation.
    pub operation: Feature,
}
/// Additional angular, UV, and rasterization behavior.
#[derive(Clone, Debug)]
pub enum Feature {
    /// Angular reflectance and team-color response.
    Fresnel {
        /// RGB angular-response color.
        color: [f32; 3],
        /// Angular response exponent.
        exponent: f32,
        /// Lower and upper angular-response output.
        range: [f32; 2],
        /// Team-color contribution weight.
        team_color: f32,
    },
    /// Continuous texture-coordinate motion.
    UvAnimation {
        /// UV translation rate on both axes.
        scroll_rate: [f32; 2],
        /// UV rotation rate.
        rotate_rate: f32,
        /// UV scale rate on both axes.
        scale_rate: [f32; 2],
    },
    /// Layer-local rasterization and illumination overrides.
    Shading {
        /// Suppresses illumination of this layer.
        unlit: bool,
        /// Draws both surface orientations.
        two_sided: bool,
        /// Excludes this layer from fog.
        unfogged: bool,
        /// Disables depth comparison for this layer.
        no_depth_test: bool,
        /// Disables depth-buffer updates for this layer.
        no_depth_write: bool,
    },
}
impl ValidateLinks for Input {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.texture.visit_links(visitor);
        if self
            .constant
            .iter()
            .chain(self.uv_transform.iter().flatten())
            .chain([&self.weight])
            .any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for Material<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match &self.common.body {
            Body::Composite { layers, .. } => {
                for layer in layers {
                    layer.input.visit_links(visitor)
                }
            }
            Body::Combiners { stages, .. } => {
                for stage in stages {
                    stage.input.visit_links(visitor)
                }
            }
            Body::LegacyDeferred { slots, .. } => {
                for slot in slots {
                    slot.input.visit_links(visitor)
                }
            }
            Body::PbrDeferred { slots, .. } => {
                for slot in slots {
                    slot.input.visit_links(visitor)
                }
            }
        }
        match &self.specialized {
            Some(SpecializedSurface::Declared(_)) => {}
            Some(SpecializedSurface::Warcraft(v)) => v.visit_links(visitor),
            Some(SpecializedSurface::Wow(v)) => v.visit_links(visitor),
            Some(SpecializedSurface::Starcraft(v)) => v.visit_links(visitor),
            Some(SpecializedSurface::Diablo(v)) => v.visit_links(visitor),
            None => {}
        }
    }
}
impl Material<'_> {
    /// Checks shader values independently of graph relationships.
    pub fn validate(&self) -> Result<(), Error> {
        if !self.common.depth_bias.is_finite() || !self.common.alpha_test_threshold.is_finite() {
            return Err(Error::new("wem.material", "nonfinite rasterization state"));
        }
        let body_finite = match &self.common.body {
            Body::Composite {
                diffuse,
                emissive,
                specular,
                specular_exponent,
                environment,
                ..
            }
            | Body::LegacyDeferred {
                diffuse,
                emissive,
                specular,
                specular_exponent,
                environment,
                ..
            } => diffuse
                .iter()
                .chain(emissive)
                .chain(specular)
                .chain([specular_exponent, environment])
                .all(|value| value.is_finite()),
            Body::Combiners {
                diffuse, emissive, ..
            } => diffuse
                .iter()
                .chain(emissive)
                .all(|value| value.is_finite()),
            Body::PbrDeferred {
                base_color,
                metallic,
                roughness,
                emissive,
                ..
            } => base_color
                .iter()
                .chain(emissive)
                .chain([metallic, roughness])
                .all(|value| value.is_finite()),
        };
        if !body_finite {
            return Err(Error::new(
                "wem.material.body",
                "nonfinite shader coefficient",
            ));
        }
        let layer_ids: Vec<_> = self.common.body.layer_ids().collect();
        for (index, id) in layer_ids.iter().enumerate() {
            if layer_ids[..index].contains(id) {
                return Err(Error::new(
                    "wem.material.layers",
                    "shader layer identities must be distinct",
                ));
            }
        }
        for (index, feature) in self.common.features.iter().enumerate() {
            if !layer_ids.contains(&feature.layer) {
                return Err(Error::new(
                    "wem.material.features",
                    "shader feature selects an input owned by another surface",
                ));
            }
            if self.common.features[..index]
                .iter()
                .any(|other| other.id == feature.id)
            {
                return Err(Error::new(
                    "wem.material.features",
                    "duplicate shader feature identity",
                ));
            }
            let finite = match &feature.operation {
                Feature::Fresnel {
                    color,
                    exponent,
                    range,
                    team_color,
                } => color
                    .iter()
                    .chain(range)
                    .chain([exponent, team_color])
                    .all(|value| value.is_finite()),
                Feature::UvAnimation {
                    scroll_rate,
                    rotate_rate,
                    scale_rate,
                } => scroll_rate
                    .iter()
                    .chain(scale_rate)
                    .chain([rotate_rate])
                    .all(|value| value.is_finite()),
                Feature::Shading { .. } => true,
            };
            if !finite {
                return Err(Error::new(
                    "wem.material.features",
                    "nonfinite shader feature",
                ));
            }
        }
        match &self.specialized {
            Some(SpecializedSurface::Declared(_)) => {}
            Some(SpecializedSurface::Warcraft(value)) => value.validate()?,
            Some(SpecializedSurface::Wow(value)) => value.validate()?,
            Some(SpecializedSurface::Starcraft(value)) => value.validate()?,
            Some(SpecializedSurface::Diablo(value)) => value.validate()?,
            None => {}
        }
        Ok(())
    }
}

impl SurfacePayload for Material<'_> {
    fn validate(&self) -> Result<(), Error> {
        Material::validate(self)
    }
}

/// Independently authored StarCraft shader declarations and external lookup domains.
pub mod starcraft;
pub use starcraft::StarcraftMaterial;

/// Native-game shading family declared independently of payload availability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderFamily {
    /// Warcraft III shading.
    Warcraft,
    /// World of Warcraft shading.
    WarcraftOnline,
    /// StarCraft II and Heroes shading.
    StarCraft,
    /// Diablo III shading.
    Diablo,
}

/// Independently authored Diablo shader declarations and external resource dependencies.
pub mod diablo;
pub use diablo::DiabloMaterial;
