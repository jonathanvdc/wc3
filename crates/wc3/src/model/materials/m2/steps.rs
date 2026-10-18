//! Interpreted M2 shader definitions. Native selector encodings are confined to conversion.
use super::{GuildPalette, ModelRasterization, ShaderInput, TextureUnit};
use crate::model::materials::state::PassState;
use crate::model::profiles::m2::properties::Target;
use crate::model::profiles::SurfacePayload;
use crate::model::values::model::{BatchFlags, BlendMode};
use crate::model::{Construct, PropertyValues};
use crate::model::{Error, Link, Property, PropertyIdentityKey, PropertyPayload, ValidateLinks};
use std::borrow::Cow;

/// Shared M2 draw declarations and animated surface multipliers.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct MaterialDeclaration<'a> {
    /// Established batch behavior and retained extension bits.
    pub batch_flags: BatchFlags,
    /// Framebuffer composition, rasterization, and pass priority.
    pub state: PassState<BlendMode, ModelRasterization, i8>,
    /// Animated RGB multiplier.
    pub color: Property<[f32; 3]>,
    /// Animated opacity multiplier.
    pub opacity: Property<f32>,
    /// Optional authoring label, independent of shader selection.
    pub label: Cow<'a, str>,
}
/// Texture alpha contribution of a legacy additive equation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AdditiveOpacity {
    /// Texture masks do not contribute to surface opacity.
    #[default]
    Opaque,
    /// Use diffuse alpha.
    Diffuse,
    /// Sum diffuse and emission alpha.
    DiffuseAndEmission,
}
/// Inputs for the M2 Diffuse equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct DiffuseMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// Multiply surface opacity by diffuse texture alpha.
    pub use_texture_alpha: bool,
}
/// Inputs for the M2 Modulate equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct ModulateMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The modulation shader input.
    pub modulation: TextureUnit,
    /// Multiply surface opacity by diffuse texture alpha.
    pub use_diffuse_alpha: bool,
    /// Multiply surface opacity by modulation texture alpha.
    pub use_modulation_alpha: bool,
    /// Double the modulated RGB contribution.
    pub doubled: bool,
    /// Authored bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureUnit>,
}
/// Inputs for the M2 Additive equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct AdditiveMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The emission shader input.
    pub emission: TextureUnit,
    /// Select the texture alpha contribution to surface opacity.
    pub opacity: AdditiveOpacity,
    /// Authored bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureUnit>,
}
/// Inputs for the M2 MaskedDoubledModulation equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct MaskedDoubledModulationMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The modulation shader input.
    pub modulation: TextureUnit,
}
/// Inputs for the M2 OpaqueAdditiveAlpha equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct OpaqueAdditiveAlphaMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The emission shader input.
    pub emission: TextureUnit,
}
/// Inputs for the M2 InverseMaskedOpaqueAdditiveAlpha equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct InverseMaskedOpaqueAdditiveAlphaMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The emission shader input.
    pub emission: TextureUnit,
}
/// Inputs for the M2 MaskedDoubledModulationGlow equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct MaskedDoubledModulationGlowMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The modulation shader input.
    pub modulation: TextureUnit,
    /// The glow shader input.
    pub glow: TextureUnit,
}
/// Inputs for the M2 ModulatedAdditiveAlpha equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct ModulatedAdditiveAlphaMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The emission shader input.
    pub emission: TextureUnit,
}
/// Inputs for the M2 InverseMaskedModulatedAdditiveAlpha equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct InverseMaskedModulatedAdditiveAlphaMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The emission shader input.
    pub emission: TextureUnit,
}
/// Inputs for the M2 DoubleAlphaInterpolation equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct DoubleAlphaInterpolationMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The replacement shader input.
    pub replacement: TextureUnit,
}
/// Inputs for the M2 DoubledModulationInterpolation equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct DoubledModulationInterpolationMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The modulation shader input.
    pub modulation: TextureUnit,
    /// The replacement shader input.
    pub replacement: TextureUnit,
}
/// Inputs for the M2 WeightedOpaqueAdditiveAlpha equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct WeightedOpaqueAdditiveAlphaMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The emission shader input.
    pub emission: TextureUnit,
}
/// Inputs for the M2 ModulatedAdditiveInverseMask equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct ModulatedAdditiveInverseMaskMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The emission shader input.
    pub emission: TextureUnit,
}
/// Inputs for the M2 MaskedModulation equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct MaskedModulationMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The modulation shader input.
    pub modulation: TextureUnit,
}
/// Inputs for the M2 WeightedModulatedAdditiveAlpha equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct WeightedModulatedAdditiveAlphaMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The emission shader input.
    pub emission: TextureUnit,
}
/// Inputs for the M2 WeightedModulationAdditive equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct WeightedModulationAdditiveMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The modulation shader input.
    pub modulation: TextureUnit,
}
/// Inputs for the M2 MaskedDoubledModulationAlpha equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct MaskedDoubledModulationAlphaMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The modulation shader input.
    pub modulation: TextureUnit,
    /// The unshaded shader input.
    pub unshaded: TextureUnit,
}
/// Inputs for the M2 DualCrossfade equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct DualCrossfadeMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The base shader input.
    pub base: TextureUnit,
    /// The first shader input.
    pub first: TextureUnit,
    /// The second shader input.
    pub second: TextureUnit,
}
/// Inputs for the M2 FacingOpacity equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct FacingOpacityMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
}
/// Inputs for the M2 DoubledModulationDoubleInterpolation equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct DoubledModulationDoubleInterpolationMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The modulation shader input.
    pub modulation: TextureUnit,
    /// The replacement shader input.
    pub replacement: TextureUnit,
}
/// Inputs for the M2 ModulatedFacingOpacity equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct ModulatedFacingOpacityMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The modulation shader input.
    pub modulation: TextureUnit,
}
/// Inputs for the M2 MaskedDualCrossfade equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct MaskedDualCrossfadeMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The base shader input.
    pub base: TextureUnit,
    /// The first shader input.
    pub first: TextureUnit,
    /// The second shader input.
    pub second: TextureUnit,
    /// The alpha mask shader input.
    pub alpha_mask: TextureUnit,
}
/// Inputs for the M2 AlphaInterpolation equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct AlphaInterpolationMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The replacement shader input.
    pub replacement: TextureUnit,
}
/// Inputs for the M2 Guild equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct GuildMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The background shader input.
    pub background: TextureUnit,
    /// The emblem shader input.
    pub emblem: TextureUnit,
    /// The border shader input.
    pub border: TextureUnit,
    /// Application-supplied guild colours.
    pub palette: GuildPalette,
}
/// Inputs for the M2 GuildEmblem equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct GuildEmblemMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The background shader input.
    pub background: TextureUnit,
    /// The emblem shader input.
    pub emblem: TextureUnit,
    /// Application-supplied guild colours.
    pub palette: GuildPalette,
}
/// Inputs for the M2 OpaqueGuild equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct OpaqueGuildMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The background shader input.
    pub background: TextureUnit,
    /// The emblem shader input.
    pub emblem: TextureUnit,
    /// The border shader input.
    pub border: TextureUnit,
    /// Application-supplied guild colours.
    pub palette: GuildPalette,
}
/// Inputs for the M2 DiffuseWithoutTextureAlpha equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct DiffuseWithoutTextureAlphaMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
}
/// Inputs for the M2 DoubledModulatedFacingOpacity equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct DoubledModulatedFacingOpacityMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The modulation shader input.
    pub modulation: TextureUnit,
}
/// Inputs for the M2 DepthModulatedFacingOpacity equation.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct DepthModulatedFacingOpacityMaterialStep<'a> {
    /// Draw state and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureUnit,
    /// The modulation shader input.
    pub modulation: TextureUnit,
}
/// An uninterpreted selector or incompatible input contract.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct UnresolvedMaterialStep<'a> {
    /// Draw declarations and animated surface response.
    pub declaration: MaterialDeclaration<'a>,
    /// Uninterpreted native shader selector.
    pub shader: u16,
    /// Ordered uninterpreted inputs.
    pub textures: Vec<TextureUnit>,
    /// Retained application palette on an uninterpreted equation.
    pub guild_palette: GuildPalette,
}
/// One M2 shader equation with a fixed, named input contract.
#[derive(Clone, Debug, PartialEq)]
pub enum MaterialStep<'a> {
    /// The Diffuse equation.
    Diffuse(DiffuseMaterialStep<'a>),
    /// The Modulate equation.
    Modulate(ModulateMaterialStep<'a>),
    /// The Additive equation.
    Additive(AdditiveMaterialStep<'a>),
    /// The MaskedDoubledModulation equation.
    MaskedDoubledModulation(MaskedDoubledModulationMaterialStep<'a>),
    /// The OpaqueAdditiveAlpha equation.
    OpaqueAdditiveAlpha(OpaqueAdditiveAlphaMaterialStep<'a>),
    /// The InverseMaskedOpaqueAdditiveAlpha equation.
    InverseMaskedOpaqueAdditiveAlpha(InverseMaskedOpaqueAdditiveAlphaMaterialStep<'a>),
    /// The MaskedDoubledModulationGlow equation.
    MaskedDoubledModulationGlow(Box<MaskedDoubledModulationGlowMaterialStep<'a>>),
    /// The ModulatedAdditiveAlpha equation.
    ModulatedAdditiveAlpha(ModulatedAdditiveAlphaMaterialStep<'a>),
    /// The InverseMaskedModulatedAdditiveAlpha equation.
    InverseMaskedModulatedAdditiveAlpha(InverseMaskedModulatedAdditiveAlphaMaterialStep<'a>),
    /// The DoubleAlphaInterpolation equation.
    DoubleAlphaInterpolation(DoubleAlphaInterpolationMaterialStep<'a>),
    /// The DoubledModulationInterpolation equation.
    DoubledModulationInterpolation(Box<DoubledModulationInterpolationMaterialStep<'a>>),
    /// The WeightedOpaqueAdditiveAlpha equation.
    WeightedOpaqueAdditiveAlpha(WeightedOpaqueAdditiveAlphaMaterialStep<'a>),
    /// The ModulatedAdditiveInverseMask equation.
    ModulatedAdditiveInverseMask(ModulatedAdditiveInverseMaskMaterialStep<'a>),
    /// The MaskedModulation equation.
    MaskedModulation(MaskedModulationMaterialStep<'a>),
    /// The WeightedModulatedAdditiveAlpha equation.
    WeightedModulatedAdditiveAlpha(WeightedModulatedAdditiveAlphaMaterialStep<'a>),
    /// The WeightedModulationAdditive equation.
    WeightedModulationAdditive(WeightedModulationAdditiveMaterialStep<'a>),
    /// The MaskedDoubledModulationAlpha equation.
    MaskedDoubledModulationAlpha(Box<MaskedDoubledModulationAlphaMaterialStep<'a>>),
    /// The DualCrossfade equation.
    DualCrossfade(Box<DualCrossfadeMaterialStep<'a>>),
    /// The FacingOpacity equation.
    FacingOpacity(FacingOpacityMaterialStep<'a>),
    /// The DoubledModulationDoubleInterpolation equation.
    DoubledModulationDoubleInterpolation(Box<DoubledModulationDoubleInterpolationMaterialStep<'a>>),
    /// The ModulatedFacingOpacity equation.
    ModulatedFacingOpacity(ModulatedFacingOpacityMaterialStep<'a>),
    /// The MaskedDualCrossfade equation.
    MaskedDualCrossfade(Box<MaskedDualCrossfadeMaterialStep<'a>>),
    /// The AlphaInterpolation equation.
    AlphaInterpolation(AlphaInterpolationMaterialStep<'a>),
    /// The Guild equation.
    Guild(Box<GuildMaterialStep<'a>>),
    /// The GuildEmblem equation.
    GuildEmblem(GuildEmblemMaterialStep<'a>),
    /// The OpaqueGuild equation.
    OpaqueGuild(Box<OpaqueGuildMaterialStep<'a>>),
    /// The DiffuseWithoutTextureAlpha equation.
    DiffuseWithoutTextureAlpha(DiffuseWithoutTextureAlphaMaterialStep<'a>),
    /// The DoubledModulatedFacingOpacity equation.
    DoubledModulatedFacingOpacity(DoubledModulatedFacingOpacityMaterialStep<'a>),
    /// The DepthModulatedFacingOpacity equation.
    DepthModulatedFacingOpacity(DepthModulatedFacingOpacityMaterialStep<'a>),
    /// Retained uninterpreted shader inputs.
    Unresolved(UnresolvedMaterialStep<'a>),
}
impl Construct for MaterialStep<'_> {
    fn construct(values: &mut PropertyValues) -> Self {
        Self::Unresolved(Construct::construct(values))
    }
}
impl<'a> MaterialStep<'a> {
    /// Interprets native selection and input ordinals into a shader-specific schema.
    pub fn from_native(
        shader: u16,
        declaration: MaterialDeclaration<'a>,
        textures: Vec<TextureUnit>,
        guild_palette: GuildPalette,
    ) -> Self {
        let mut textures = textures;
        let count = textures.len();
        let legacy = shader & 0x8000 == 0;
        let table = shader & 0x7fff;
        let expected = match table {
            3 | 9 | 17 | 18 | 20 | 24 | 26 | 28 => 3,
            19 | 25 | 32 | 34 => 1,
            22 => 4,
            0..=28 | 33 | 35 => 2,
            _ => 0,
        };
        if (legacy
            && (shader & !0x40ff != 0 || count == 0 || (count == 1 && shader & 0x4080 == 0x4000)))
            || (!legacy && (expected == 0 || count != expected))
            || textures
                .iter()
                .take(if legacy { count.min(2) } else { expected })
                .any(|input| input.coordinates.source < -1)
            || ((legacy || !matches!(table, 26..=28)) && guild_palette != GuildPalette::default())
        {
            return Self::Unresolved(UnresolvedMaterialStep {
                declaration,
                shader,
                textures,
                guild_palette,
            });
        }
        let environment = if legacy {
            [shader & 0x80 != 0, shader & 8 != 0]
        } else {
            [
                table == 25,
                matches!(table,0..=4 | 7..=9 | 11..=13 | 16..=17 | 20 | 24),
            ]
        };
        for (input, environment) in textures.iter_mut().zip(environment) {
            if environment {
                input.coordinates.source = -1;
            }
        }
        let mut inputs = textures.into_iter();
        if legacy {
            let diffuse = inputs.next().expect("nonempty inputs");
            let use_diffuse_alpha = shader & 0x70 != 0;
            let doubled = matches!(shader & 7, 4 | 6);
            if count == 1 {
                return Self::Diffuse(DiffuseMaterialStep {
                    declaration,
                    diffuse,
                    use_texture_alpha: use_diffuse_alpha,
                });
            }
            let second = inputs.next().expect("two inputs");
            let unused_inputs = inputs.collect();
            return if matches!(shader & 7, 3 | 7) {
                let opacity = if !use_diffuse_alpha {
                    AdditiveOpacity::Opaque
                } else if shader & 7 == 3 {
                    AdditiveOpacity::DiffuseAndEmission
                } else {
                    AdditiveOpacity::Diffuse
                };
                Self::Additive(AdditiveMaterialStep {
                    declaration,
                    diffuse,
                    emission: second,
                    opacity,
                    unused_inputs,
                })
            } else {
                Self::Modulate(ModulateMaterialStep {
                    declaration,
                    diffuse,
                    modulation: second,
                    use_diffuse_alpha,
                    use_modulation_alpha: !matches!(shader & 7, 0 | 6),
                    doubled,
                    unused_inputs,
                })
            };
        }
        match table {
            0 => Self::MaskedDoubledModulation(MaskedDoubledModulationMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
                modulation: inputs.next().expect("validated input count"),
            }),
            1 | 5 => Self::OpaqueAdditiveAlpha(OpaqueAdditiveAlphaMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
                emission: inputs.next().expect("validated input count"),
            }),
            2 => Self::InverseMaskedOpaqueAdditiveAlpha(
                InverseMaskedOpaqueAdditiveAlphaMaterialStep {
                    declaration,
                    diffuse: inputs.next().expect("validated input count"),
                    emission: inputs.next().expect("validated input count"),
                },
            ),
            3 => Self::MaskedDoubledModulationGlow(Box::new(
                MaskedDoubledModulationGlowMaterialStep {
                    declaration,
                    diffuse: inputs.next().expect("validated input count"),
                    modulation: inputs.next().expect("validated input count"),
                    glow: inputs.next().expect("validated input count"),
                },
            )),
            4 | 6 => Self::ModulatedAdditiveAlpha(ModulatedAdditiveAlphaMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
                emission: inputs.next().expect("validated input count"),
            }),
            7 => Self::InverseMaskedModulatedAdditiveAlpha(
                InverseMaskedModulatedAdditiveAlphaMaterialStep {
                    declaration,
                    diffuse: inputs.next().expect("validated input count"),
                    emission: inputs.next().expect("validated input count"),
                },
            ),
            8 => Self::DoubleAlphaInterpolation(DoubleAlphaInterpolationMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
                replacement: inputs.next().expect("validated input count"),
            }),
            9 => Self::DoubledModulationInterpolation(Box::new(
                DoubledModulationInterpolationMaterialStep {
                    declaration,
                    diffuse: inputs.next().expect("validated input count"),
                    modulation: inputs.next().expect("validated input count"),
                    replacement: inputs.next().expect("validated input count"),
                },
            )),
            10 | 15 => Self::WeightedOpaqueAdditiveAlpha(WeightedOpaqueAdditiveAlphaMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
                emission: inputs.next().expect("validated input count"),
            }),
            11 => Self::ModulatedAdditiveInverseMask(ModulatedAdditiveInverseMaskMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
                emission: inputs.next().expect("validated input count"),
            }),
            12 => Self::MaskedModulation(MaskedModulationMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
                modulation: inputs.next().expect("validated input count"),
            }),
            13 | 14 => {
                Self::WeightedModulatedAdditiveAlpha(WeightedModulatedAdditiveAlphaMaterialStep {
                    declaration,
                    diffuse: inputs.next().expect("validated input count"),
                    emission: inputs.next().expect("validated input count"),
                })
            }
            16 => Self::WeightedModulationAdditive(WeightedModulationAdditiveMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
                modulation: inputs.next().expect("validated input count"),
            }),
            17 | 24 => Self::MaskedDoubledModulationAlpha(Box::new(
                MaskedDoubledModulationAlphaMaterialStep {
                    declaration,
                    diffuse: inputs.next().expect("validated input count"),
                    modulation: inputs.next().expect("validated input count"),
                    unshaded: inputs.next().expect("validated input count"),
                },
            )),
            18 => Self::DualCrossfade(Box::new(DualCrossfadeMaterialStep {
                declaration,
                base: inputs.next().expect("validated input count"),
                first: inputs.next().expect("validated input count"),
                second: inputs.next().expect("validated input count"),
            })),
            19 | 25 | 34 => Self::FacingOpacity(FacingOpacityMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
            }),
            20 => Self::DoubledModulationDoubleInterpolation(Box::new(
                DoubledModulationDoubleInterpolationMaterialStep {
                    declaration,
                    diffuse: inputs.next().expect("validated input count"),
                    modulation: inputs.next().expect("validated input count"),
                    replacement: inputs.next().expect("validated input count"),
                },
            )),
            21 => Self::ModulatedFacingOpacity(ModulatedFacingOpacityMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
                modulation: inputs.next().expect("validated input count"),
            }),
            22 => Self::MaskedDualCrossfade(Box::new(MaskedDualCrossfadeMaterialStep {
                declaration,
                base: inputs.next().expect("validated input count"),
                first: inputs.next().expect("validated input count"),
                second: inputs.next().expect("validated input count"),
                alpha_mask: inputs.next().expect("validated input count"),
            })),
            23 => Self::AlphaInterpolation(AlphaInterpolationMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
                replacement: inputs.next().expect("validated input count"),
            }),
            26 => Self::Guild(Box::new(GuildMaterialStep {
                declaration,
                background: inputs.next().expect("validated input count"),
                emblem: inputs.next().expect("validated input count"),
                border: inputs.next().expect("validated input count"),
                palette: guild_palette,
            })),
            27 => Self::GuildEmblem(GuildEmblemMaterialStep {
                declaration,
                background: inputs.next().expect("validated input count"),
                emblem: inputs.next().expect("validated input count"),
                palette: guild_palette,
            }),
            28 => Self::OpaqueGuild(Box::new(OpaqueGuildMaterialStep {
                declaration,
                background: inputs.next().expect("validated input count"),
                emblem: inputs.next().expect("validated input count"),
                border: inputs.next().expect("validated input count"),
                palette: guild_palette,
            })),
            32 => Self::DiffuseWithoutTextureAlpha(DiffuseWithoutTextureAlphaMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
            }),
            33 => Self::DoubledModulatedFacingOpacity(DoubledModulatedFacingOpacityMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
                modulation: inputs.next().expect("validated input count"),
            }),
            35 => Self::DepthModulatedFacingOpacity(DepthModulatedFacingOpacityMaterialStep {
                declaration,
                diffuse: inputs.next().expect("validated input count"),
                modulation: inputs.next().expect("validated input count"),
            }),
            _ => unreachable!("validated table entry"),
        }
    }
}
impl<'a> MaterialStep<'a> {
    /// Shared draw declarations, independent of native selector spelling.
    pub fn declaration(&self) -> &MaterialDeclaration<'a> {
        match self {
            Self::Diffuse(v) => &v.declaration,
            Self::Modulate(v) => &v.declaration,
            Self::Additive(v) => &v.declaration,
            Self::MaskedDoubledModulation(v) => &v.declaration,
            Self::OpaqueAdditiveAlpha(v) => &v.declaration,
            Self::InverseMaskedOpaqueAdditiveAlpha(v) => &v.declaration,
            Self::MaskedDoubledModulationGlow(v) => &v.declaration,
            Self::ModulatedAdditiveAlpha(v) => &v.declaration,
            Self::InverseMaskedModulatedAdditiveAlpha(v) => &v.declaration,
            Self::DoubleAlphaInterpolation(v) => &v.declaration,
            Self::DoubledModulationInterpolation(v) => &v.declaration,
            Self::WeightedOpaqueAdditiveAlpha(v) => &v.declaration,
            Self::ModulatedAdditiveInverseMask(v) => &v.declaration,
            Self::MaskedModulation(v) => &v.declaration,
            Self::WeightedModulatedAdditiveAlpha(v) => &v.declaration,
            Self::WeightedModulationAdditive(v) => &v.declaration,
            Self::MaskedDoubledModulationAlpha(v) => &v.declaration,
            Self::DualCrossfade(v) => &v.declaration,
            Self::FacingOpacity(v) => &v.declaration,
            Self::DoubledModulationDoubleInterpolation(v) => &v.declaration,
            Self::ModulatedFacingOpacity(v) => &v.declaration,
            Self::MaskedDualCrossfade(v) => &v.declaration,
            Self::AlphaInterpolation(v) => &v.declaration,
            Self::Guild(v) => &v.declaration,
            Self::GuildEmblem(v) => &v.declaration,
            Self::OpaqueGuild(v) => &v.declaration,
            Self::DiffuseWithoutTextureAlpha(v) => &v.declaration,
            Self::DoubledModulatedFacingOpacity(v) => &v.declaration,
            Self::DepthModulatedFacingOpacity(v) => &v.declaration,
            Self::Unresolved(v) => &v.declaration,
        }
    }
    /// Mutable shared draw declarations.
    pub fn declaration_mut(&mut self) -> &mut MaterialDeclaration<'a> {
        match self {
            Self::Diffuse(v) => &mut v.declaration,
            Self::Modulate(v) => &mut v.declaration,
            Self::Additive(v) => &mut v.declaration,
            Self::MaskedDoubledModulation(v) => &mut v.declaration,
            Self::OpaqueAdditiveAlpha(v) => &mut v.declaration,
            Self::InverseMaskedOpaqueAdditiveAlpha(v) => &mut v.declaration,
            Self::MaskedDoubledModulationGlow(v) => &mut v.declaration,
            Self::ModulatedAdditiveAlpha(v) => &mut v.declaration,
            Self::InverseMaskedModulatedAdditiveAlpha(v) => &mut v.declaration,
            Self::DoubleAlphaInterpolation(v) => &mut v.declaration,
            Self::DoubledModulationInterpolation(v) => &mut v.declaration,
            Self::WeightedOpaqueAdditiveAlpha(v) => &mut v.declaration,
            Self::ModulatedAdditiveInverseMask(v) => &mut v.declaration,
            Self::MaskedModulation(v) => &mut v.declaration,
            Self::WeightedModulatedAdditiveAlpha(v) => &mut v.declaration,
            Self::WeightedModulationAdditive(v) => &mut v.declaration,
            Self::MaskedDoubledModulationAlpha(v) => &mut v.declaration,
            Self::DualCrossfade(v) => &mut v.declaration,
            Self::FacingOpacity(v) => &mut v.declaration,
            Self::DoubledModulationDoubleInterpolation(v) => &mut v.declaration,
            Self::ModulatedFacingOpacity(v) => &mut v.declaration,
            Self::MaskedDualCrossfade(v) => &mut v.declaration,
            Self::AlphaInterpolation(v) => &mut v.declaration,
            Self::Guild(v) => &mut v.declaration,
            Self::GuildEmblem(v) => &mut v.declaration,
            Self::OpaqueGuild(v) => &mut v.declaration,
            Self::DiffuseWithoutTextureAlpha(v) => &mut v.declaration,
            Self::DoubledModulatedFacingOpacity(v) => &mut v.declaration,
            Self::DepthModulatedFacingOpacity(v) => &mut v.declaration,
            Self::Unresolved(v) => &mut v.declaration,
        }
    }
    /// Shader inputs in stable native animation ordinal order.
    pub fn inputs(&self) -> Vec<&TextureUnit> {
        match self {
            Self::Diffuse(v) => vec![&v.diffuse],
            Self::Modulate(v) => {
                let mut inputs = vec![&v.diffuse, &v.modulation];
                inputs.extend(v.unused_inputs.iter());
                inputs
            }
            Self::Additive(v) => {
                let mut inputs = vec![&v.diffuse, &v.emission];
                inputs.extend(v.unused_inputs.iter());
                inputs
            }
            Self::MaskedDoubledModulation(v) => vec![&v.diffuse, &v.modulation],
            Self::OpaqueAdditiveAlpha(v) => vec![&v.diffuse, &v.emission],
            Self::InverseMaskedOpaqueAdditiveAlpha(v) => vec![&v.diffuse, &v.emission],
            Self::MaskedDoubledModulationGlow(v) => vec![&v.diffuse, &v.modulation, &v.glow],
            Self::ModulatedAdditiveAlpha(v) => vec![&v.diffuse, &v.emission],
            Self::InverseMaskedModulatedAdditiveAlpha(v) => vec![&v.diffuse, &v.emission],
            Self::DoubleAlphaInterpolation(v) => vec![&v.diffuse, &v.replacement],
            Self::DoubledModulationInterpolation(v) => {
                vec![&v.diffuse, &v.modulation, &v.replacement]
            }
            Self::WeightedOpaqueAdditiveAlpha(v) => vec![&v.diffuse, &v.emission],
            Self::ModulatedAdditiveInverseMask(v) => vec![&v.diffuse, &v.emission],
            Self::MaskedModulation(v) => vec![&v.diffuse, &v.modulation],
            Self::WeightedModulatedAdditiveAlpha(v) => vec![&v.diffuse, &v.emission],
            Self::WeightedModulationAdditive(v) => vec![&v.diffuse, &v.modulation],
            Self::MaskedDoubledModulationAlpha(v) => vec![&v.diffuse, &v.modulation, &v.unshaded],
            Self::DualCrossfade(v) => vec![&v.base, &v.first, &v.second],
            Self::FacingOpacity(v) => vec![&v.diffuse],
            Self::DoubledModulationDoubleInterpolation(v) => {
                vec![&v.diffuse, &v.modulation, &v.replacement]
            }
            Self::ModulatedFacingOpacity(v) => vec![&v.diffuse, &v.modulation],
            Self::MaskedDualCrossfade(v) => vec![&v.base, &v.first, &v.second, &v.alpha_mask],
            Self::AlphaInterpolation(v) => vec![&v.diffuse, &v.replacement],
            Self::Guild(v) => vec![&v.background, &v.emblem, &v.border],
            Self::GuildEmblem(v) => vec![&v.background, &v.emblem],
            Self::OpaqueGuild(v) => vec![&v.background, &v.emblem, &v.border],
            Self::DiffuseWithoutTextureAlpha(v) => vec![&v.diffuse],
            Self::DoubledModulatedFacingOpacity(v) => vec![&v.diffuse, &v.modulation],
            Self::DepthModulatedFacingOpacity(v) => vec![&v.diffuse, &v.modulation],
            Self::Unresolved(v) => v.textures.iter().collect(),
        }
    }
    /// Shader inputs in stable native animation ordinal order.
    pub fn inputs_mut(&mut self) -> Vec<&mut TextureUnit> {
        match self {
            Self::Diffuse(v) => vec![&mut v.diffuse],
            Self::Modulate(v) => {
                let mut inputs = vec![&mut v.diffuse, &mut v.modulation];
                inputs.extend(v.unused_inputs.iter_mut());
                inputs
            }
            Self::Additive(v) => {
                let mut inputs = vec![&mut v.diffuse, &mut v.emission];
                inputs.extend(v.unused_inputs.iter_mut());
                inputs
            }
            Self::MaskedDoubledModulation(v) => vec![&mut v.diffuse, &mut v.modulation],
            Self::OpaqueAdditiveAlpha(v) => vec![&mut v.diffuse, &mut v.emission],
            Self::InverseMaskedOpaqueAdditiveAlpha(v) => vec![&mut v.diffuse, &mut v.emission],
            Self::MaskedDoubledModulationGlow(v) => {
                vec![&mut v.diffuse, &mut v.modulation, &mut v.glow]
            }
            Self::ModulatedAdditiveAlpha(v) => vec![&mut v.diffuse, &mut v.emission],
            Self::InverseMaskedModulatedAdditiveAlpha(v) => vec![&mut v.diffuse, &mut v.emission],
            Self::DoubleAlphaInterpolation(v) => vec![&mut v.diffuse, &mut v.replacement],
            Self::DoubledModulationInterpolation(v) => {
                vec![&mut v.diffuse, &mut v.modulation, &mut v.replacement]
            }
            Self::WeightedOpaqueAdditiveAlpha(v) => vec![&mut v.diffuse, &mut v.emission],
            Self::ModulatedAdditiveInverseMask(v) => vec![&mut v.diffuse, &mut v.emission],
            Self::MaskedModulation(v) => vec![&mut v.diffuse, &mut v.modulation],
            Self::WeightedModulatedAdditiveAlpha(v) => vec![&mut v.diffuse, &mut v.emission],
            Self::WeightedModulationAdditive(v) => vec![&mut v.diffuse, &mut v.modulation],
            Self::MaskedDoubledModulationAlpha(v) => {
                vec![&mut v.diffuse, &mut v.modulation, &mut v.unshaded]
            }
            Self::DualCrossfade(v) => vec![&mut v.base, &mut v.first, &mut v.second],
            Self::FacingOpacity(v) => vec![&mut v.diffuse],
            Self::DoubledModulationDoubleInterpolation(v) => {
                vec![&mut v.diffuse, &mut v.modulation, &mut v.replacement]
            }
            Self::ModulatedFacingOpacity(v) => vec![&mut v.diffuse, &mut v.modulation],
            Self::MaskedDualCrossfade(v) => {
                vec![&mut v.base, &mut v.first, &mut v.second, &mut v.alpha_mask]
            }
            Self::AlphaInterpolation(v) => vec![&mut v.diffuse, &mut v.replacement],
            Self::Guild(v) => vec![&mut v.background, &mut v.emblem, &mut v.border],
            Self::GuildEmblem(v) => vec![&mut v.background, &mut v.emblem],
            Self::OpaqueGuild(v) => vec![&mut v.background, &mut v.emblem, &mut v.border],
            Self::DiffuseWithoutTextureAlpha(v) => vec![&mut v.diffuse],
            Self::DoubledModulatedFacingOpacity(v) => vec![&mut v.diffuse, &mut v.modulation],
            Self::DepthModulatedFacingOpacity(v) => vec![&mut v.diffuse, &mut v.modulation],
            Self::Unresolved(v) => v.textures.iter_mut().collect(),
        }
    }
    /// External colours on guild equations or uninterpreted inputs.
    pub fn guild_palette(&self) -> Option<GuildPalette> {
        match self {
            Self::Guild(v) => Some(v.palette),
            Self::GuildEmblem(v) => Some(v.palette),
            Self::OpaqueGuild(v) => Some(v.palette),
            Self::Unresolved(v) => Some(v.guild_palette),
            _ => None,
        }
    }
    /// Whether this step exposes the structural M2 animation destination.
    pub fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        matches!(
            Target::from_identity(key),
            Some(Target::Color { .. } | Target::Opacity { .. })
        ) || self
            .inputs()
            .iter()
            .enumerate()
            .any(|(ordinal, input)| input.supports_property(key, ordinal))
    }
    /// Validates the equation's authored values.
    pub fn validate(&self) -> Result<(), Error> {
        let palette = match self {
            Self::Unresolved(v) => Some(v.guild_palette),
            _ => self.guild_palette(),
        };
        if palette.is_some_and(|v| !v.is_finite()) {
            return Err(Error::new(
                "m2.material.guild_palette",
                "guild colours must be finite",
            ));
        }
        self.inputs().iter().try_for_each(|input| input.validate())
    }
    /// Derives the native selector from the equation and its coordinate choices.
    pub fn native_shader(&self) -> u16 {
        let textures = self.inputs();
        let environment = |slot: usize| {
            textures
                .get(slot)
                .is_some_and(|v| v.coordinates.source == -1)
        };
        let uv1 = |slot: usize| {
            textures
                .get(slot)
                .is_some_and(|v| v.coordinates.source == 1)
        };
        let legacy_coordinates = (u16::from(environment(0)) * 0x80)
            | (u16::from(environment(1)) * 8)
            | (u16::from(uv1(usize::from(textures.len() > 1))) * 0x4000);
        match self {
            Self::Diffuse(v) => (u16::from(v.use_texture_alpha) * 0x10) | legacy_coordinates,
            Self::Modulate(v) => {
                let lower = match (v.use_modulation_alpha, v.doubled) {
                    (false, false) => 0,
                    (true, false) => 1,
                    (true, true) => 4,
                    (false, true) => 6,
                };
                lower | (u16::from(v.use_diffuse_alpha) * 0x10) | legacy_coordinates
            }
            Self::Additive(v) => {
                (match v.opacity {
                    AdditiveOpacity::Opaque => 3,
                    AdditiveOpacity::Diffuse => 0x17,
                    AdditiveOpacity::DiffuseAndEmission => 0x13,
                }) | legacy_coordinates
            }
            Self::MaskedDoubledModulation(_) => 0x8000,
            Self::OpaqueAdditiveAlpha(_) => 0x8000 | (if environment(1) { 1 } else { 5 }),
            Self::InverseMaskedOpaqueAdditiveAlpha(_) => 0x8000 | (2),
            Self::MaskedDoubledModulationGlow(_) => 0x8000 | (3),
            Self::ModulatedAdditiveAlpha(_) => 0x8000 | (if environment(1) { 4 } else { 6 }),
            Self::InverseMaskedModulatedAdditiveAlpha(_) => 0x8000 | (7),
            Self::DoubleAlphaInterpolation(_) => 0x8000 | (8),
            Self::DoubledModulationInterpolation(_) => 0x8000 | (9),
            Self::WeightedOpaqueAdditiveAlpha(_) => 0x8000 | (if uv1(1) { 15 } else { 10 }),
            Self::ModulatedAdditiveInverseMask(_) => 0x8000 | (11),
            Self::MaskedModulation(_) => 0x8000 | (12),
            Self::WeightedModulatedAdditiveAlpha(_) => {
                0x8000 | (if environment(1) { 13 } else { 14 })
            }
            Self::WeightedModulationAdditive(_) => 0x8000 | (16),
            Self::MaskedDoubledModulationAlpha(_) => 0x8000 | (if uv1(2) { 24 } else { 17 }),
            Self::DualCrossfade(_) => 0x8000 | (18),
            Self::FacingOpacity(_) => 0x8000 | (if environment(0) { 25 } else { 19 }),
            Self::DoubledModulationDoubleInterpolation(_) => 0x8000 | (20),
            Self::ModulatedFacingOpacity(_) => 0x8000 | (21),
            Self::MaskedDualCrossfade(_) => 0x8000 | (22),
            Self::AlphaInterpolation(_) => 0x8000 | (23),
            Self::Guild(_) => 0x8000 | (26),
            Self::GuildEmblem(_) => 0x8000 | (27),
            Self::OpaqueGuild(_) => 0x8000 | (28),
            Self::DiffuseWithoutTextureAlpha(_) => 0x8000 | (32),
            Self::DoubledModulatedFacingOpacity(_) => 0x8000 | (33),
            Self::DepthModulatedFacingOpacity(_) => 0x8000 | (35),
            Self::Unresolved(v) => v.shader,
        }
    }
}
impl ValidateLinks for MaterialStep<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.declaration().state.visit_links(visitor);
        self.declaration().color.visit_links(visitor);
        self.declaration().opacity.visit_links(visitor);
        for input in self.inputs() {
            input.visit_links(visitor);
        }
    }
}

impl SurfacePayload for MaterialStep<'_> {
    fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        self.supports_property(key)
    }
    fn validate(&self) -> Result<(), Error> {
        self.validate()
    }
}

impl<'a> UnresolvedMaterialStep<'a> {
    /// Extracts the established equation of a single-input UV1 programme.
    /// Its native second-transform ownership remains unresolved on this definition.
    pub fn partial_equation(&self) -> Option<MaterialStep<'a>> {
        if self.shader & !0x40ff != 0 || self.shader & 0x4080 != 0x4000 || self.textures.len() != 1
        {
            return None;
        }
        let equation = MaterialStep::from_native(
            self.shader & !0x4000,
            self.declaration.clone(),
            self.textures.clone(),
            self.guild_palette,
        );
        if matches!(equation, MaterialStep::Unresolved(_)) {
            None
        } else {
            Some(equation)
        }
    }
}
