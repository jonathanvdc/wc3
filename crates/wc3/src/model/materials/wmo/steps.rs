//! WMO shader equations with named inputs and explicit native projection.
use super::{Blend, MaterialRasterization, ShaderOverrides};
use crate::model::materials::bindings::TextureBinding;
use crate::model::materials::state::PassState;
use crate::model::profiles::SurfacePayload;
use crate::model::{Error, Link, PropertyIdentityKey, TextureId, ValidateLinks};
use std::borrow::Cow;

/// Material-wide draw, lighting, and retained native declarations.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaterialDeclaration<'a> {
    /// Optional authoring name.
    pub name: Cow<'a, str>,
    /// Color composition and shared rasterization controls.
    pub state: PassState<Blend, MaterialRasterization>,
    /// Self-illumination color in RGBA order.
    pub self_illumination: [u8; 4],
    /// Serialized runtime self-illumination color in RGBA order.
    pub runtime_self_illumination: [u8; 4],
    /// Diffuse color, also used for interior liquid shading.
    pub diffuse: [u8; 4],
    /// External terrain-type identifier.
    pub ground_type: u32,
    /// UV scroll velocity for the first two texture slots.
    pub uv_scroll: [[f32; 2]; 2],
    /// Optional extended shader overrides.
    pub shader_overrides: Option<ShaderOverrides>,
    /// Historical additional material color in RGBA order.
    pub legacy_color: Option<[u8; 4]>,
    /// Historical additional material controls with unresolved meanings.
    pub legacy_controls: Option<u32>,
    /// Historical runtime control values with unresolved meanings.
    pub legacy_runtime: Option<[u32; 4]>,
}
/// Inputs for the WMO Diffuse equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiffuseMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// Inputs for the WMO DiffuseWithoutTextureAlpha equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiffuseWithoutTextureAlphaMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// Inputs for the WMO ReflectionEmission equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReflectionEmissionMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// The reflection shader input.
    pub reflection: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// Inputs for the WMO DiffuseWeightedReflectionEmission equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiffuseWeightedReflectionEmissionMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// The reflection shader input.
    pub reflection: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// Inputs for the WMO LayeredDiffuse equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayeredDiffuseMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// The secondary diffuse shader input.
    pub secondary_diffuse: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// Inputs for the WMO MaskedEmission equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaskedEmissionMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// The emission shader input.
    pub emission: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// Inputs for the WMO MaskedDiffuseModulationReflection equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaskedDiffuseModulationReflectionMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// The reflection shader input.
    pub reflection: TextureBinding<Option<TextureId>>,
    /// The secondary diffuse shader input.
    pub secondary_diffuse: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// Inputs for the WMO DiffuseMultipleEmission equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiffuseMultipleEmissionMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// The reflection shader input.
    pub reflection: TextureBinding<Option<TextureId>>,
    /// The emission shader input.
    pub emission: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// Inputs for the WMO OpaqueLayeredDiffuse equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OpaqueLayeredDiffuseMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// The secondary diffuse shader input.
    pub secondary_diffuse: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// Inputs for the WMO LayeredDiffuseEmission equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayeredDiffuseEmissionMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// The secondary diffuse shader input.
    pub secondary_diffuse: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// Inputs for the WMO MaskedDiffuseModulation equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaskedDiffuseModulationMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// The reflection shader input.
    pub reflection: TextureBinding<Option<TextureId>>,
    /// The modulation shader input.
    pub modulation: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// Inputs for the WMO LayeredDiffuseModulation equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayeredDiffuseModulationMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// The secondary diffuse shader input.
    pub secondary_diffuse: TextureBinding<Option<TextureId>>,
    /// The modulation shader input.
    pub modulation: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// Inputs for the WMO DiffuseModulation equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiffuseModulationMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// The modulation shader input.
    pub modulation: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// Inputs for the WMO TextureMaskedLayeredModulation equation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureMaskedLayeredModulationMaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// The diffuse shader input.
    pub diffuse: TextureBinding<Option<TextureId>>,
    /// The secondary diffuse shader input.
    pub secondary_diffuse: TextureBinding<Option<TextureId>>,
    /// The modulation mask shader input.
    pub modulation_mask: TextureBinding<Option<TextureId>>,
    /// Authored trailing bindings not sampled by this equation.
    pub unused_inputs: Vec<TextureBinding<Option<TextureId>>>,
}
/// A shader whose complete input contract is not interpreted.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UnresolvedMaterialStep<'a> {
    /// Material-wide declarations.
    pub declaration: MaterialDeclaration<'a>,
    /// Raw native shader selection.
    pub shader: Option<u32>,
    /// Complete ordered input bindings.
    pub textures: Vec<TextureBinding<Option<TextureId>>>,
}
/// One WMO shader equation and its named inputs.
#[derive(Clone, Debug, PartialEq)]
pub enum MaterialStep<'a> {
    /// The Diffuse equation.
    Diffuse(DiffuseMaterialStep<'a>),
    /// The DiffuseWithoutTextureAlpha equation.
    DiffuseWithoutTextureAlpha(DiffuseWithoutTextureAlphaMaterialStep<'a>),
    /// The ReflectionEmission equation.
    ReflectionEmission(ReflectionEmissionMaterialStep<'a>),
    /// The DiffuseWeightedReflectionEmission equation.
    DiffuseWeightedReflectionEmission(DiffuseWeightedReflectionEmissionMaterialStep<'a>),
    /// The LayeredDiffuse equation.
    LayeredDiffuse(LayeredDiffuseMaterialStep<'a>),
    /// The MaskedEmission equation.
    MaskedEmission(MaskedEmissionMaterialStep<'a>),
    /// The MaskedDiffuseModulationReflection equation.
    MaskedDiffuseModulationReflection(MaskedDiffuseModulationReflectionMaterialStep<'a>),
    /// The DiffuseMultipleEmission equation.
    DiffuseMultipleEmission(DiffuseMultipleEmissionMaterialStep<'a>),
    /// The OpaqueLayeredDiffuse equation.
    OpaqueLayeredDiffuse(OpaqueLayeredDiffuseMaterialStep<'a>),
    /// The LayeredDiffuseEmission equation.
    LayeredDiffuseEmission(LayeredDiffuseEmissionMaterialStep<'a>),
    /// The MaskedDiffuseModulation equation.
    MaskedDiffuseModulation(MaskedDiffuseModulationMaterialStep<'a>),
    /// The LayeredDiffuseModulation equation.
    LayeredDiffuseModulation(LayeredDiffuseModulationMaterialStep<'a>),
    /// The DiffuseModulation equation.
    DiffuseModulation(DiffuseModulationMaterialStep<'a>),
    /// The TextureMaskedLayeredModulation equation.
    TextureMaskedLayeredModulation(TextureMaskedLayeredModulationMaterialStep<'a>),
    /// Uninterpreted or specialized shader inputs.
    Unresolved(UnresolvedMaterialStep<'a>),
}
impl Default for MaterialStep<'_> {
    fn default() -> Self {
        Self::Unresolved(UnresolvedMaterialStep::default())
    }
}
impl<'a> MaterialStep<'a> {
    /// Interprets native selection and texture roles without retaining known IDs.
    pub fn from_native(
        shader: Option<u32>,
        declaration: MaterialDeclaration<'a>,
        textures: Vec<TextureBinding<Option<TextureId>>>,
    ) -> Self {
        let mut inputs = textures.into_iter();
        match shader {
            Some(0) => {
                let diffuse = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::Diffuse(DiffuseMaterialStep {
                    declaration,
                    diffuse,
                    unused_inputs,
                })
            }
            Some(4) => {
                let diffuse = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::DiffuseWithoutTextureAlpha(DiffuseWithoutTextureAlphaMaterialStep {
                    declaration,
                    diffuse,
                    unused_inputs,
                })
            }
            Some(3) => {
                let diffuse = inputs.next().unwrap_or_default();
                let reflection = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::ReflectionEmission(ReflectionEmissionMaterialStep {
                    declaration,
                    diffuse,
                    reflection,
                    unused_inputs,
                })
            }
            Some(5) => {
                let diffuse = inputs.next().unwrap_or_default();
                let reflection = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::DiffuseWeightedReflectionEmission(
                    DiffuseWeightedReflectionEmissionMaterialStep {
                        declaration,
                        diffuse,
                        reflection,
                        unused_inputs,
                    },
                )
            }
            Some(6) => {
                let diffuse = inputs.next().unwrap_or_default();
                let secondary_diffuse = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::LayeredDiffuse(LayeredDiffuseMaterialStep {
                    declaration,
                    diffuse,
                    secondary_diffuse,
                    unused_inputs,
                })
            }
            Some(9) => {
                let diffuse = inputs.next().unwrap_or_default();
                let emission = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::MaskedEmission(MaskedEmissionMaterialStep {
                    declaration,
                    diffuse,
                    emission,
                    unused_inputs,
                })
            }
            Some(11) => {
                let diffuse = inputs.next().unwrap_or_default();
                let reflection = inputs.next().unwrap_or_default();
                let secondary_diffuse = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::MaskedDiffuseModulationReflection(
                    MaskedDiffuseModulationReflectionMaterialStep {
                        declaration,
                        diffuse,
                        reflection,
                        secondary_diffuse,
                        unused_inputs,
                    },
                )
            }
            Some(12) => {
                let diffuse = inputs.next().unwrap_or_default();
                let reflection = inputs.next().unwrap_or_default();
                let emission = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::DiffuseMultipleEmission(DiffuseMultipleEmissionMaterialStep {
                    declaration,
                    diffuse,
                    reflection,
                    emission,
                    unused_inputs,
                })
            }
            Some(13) => {
                let diffuse = inputs.next().unwrap_or_default();
                let secondary_diffuse = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::OpaqueLayeredDiffuse(OpaqueLayeredDiffuseMaterialStep {
                    declaration,
                    diffuse,
                    secondary_diffuse,
                    unused_inputs,
                })
            }
            Some(15) => {
                let diffuse = inputs.next().unwrap_or_default();
                let secondary_diffuse = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::LayeredDiffuseEmission(LayeredDiffuseEmissionMaterialStep {
                    declaration,
                    diffuse,
                    secondary_diffuse,
                    unused_inputs,
                })
            }
            Some(17) => {
                let diffuse = inputs.next().unwrap_or_default();
                let reflection = inputs.next().unwrap_or_default();
                let modulation = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::MaskedDiffuseModulation(MaskedDiffuseModulationMaterialStep {
                    declaration,
                    diffuse,
                    reflection,
                    modulation,
                    unused_inputs,
                })
            }
            Some(18) => {
                let diffuse = inputs.next().unwrap_or_default();
                let secondary_diffuse = inputs.next().unwrap_or_default();
                let modulation = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::LayeredDiffuseModulation(LayeredDiffuseModulationMaterialStep {
                    declaration,
                    diffuse,
                    secondary_diffuse,
                    modulation,
                    unused_inputs,
                })
            }
            Some(19) => {
                let diffuse = inputs.next().unwrap_or_default();
                let modulation = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::DiffuseModulation(DiffuseModulationMaterialStep {
                    declaration,
                    diffuse,
                    modulation,
                    unused_inputs,
                })
            }
            Some(20) => {
                let diffuse = inputs.next().unwrap_or_default();
                let secondary_diffuse = inputs.next().unwrap_or_default();
                let modulation_mask = inputs.next().unwrap_or_default();
                let mut unused_inputs: Vec<_> = inputs.collect();
                while unused_inputs
                    .last()
                    .is_some_and(|input| input == &TextureBinding::default())
                {
                    unused_inputs.pop();
                }
                Self::TextureMaskedLayeredModulation(TextureMaskedLayeredModulationMaterialStep {
                    declaration,
                    diffuse,
                    secondary_diffuse,
                    modulation_mask,
                    unused_inputs,
                })
            }
            _ => Self::Unresolved(UnresolvedMaterialStep {
                declaration,
                shader,
                textures: inputs.collect(),
            }),
        }
    }
}
impl<'a> MaterialStep<'a> {
    /// Material-wide rendering declarations.
    pub fn declaration(&self) -> &MaterialDeclaration<'a> {
        match self {
            Self::Diffuse(v) => &v.declaration,
            Self::DiffuseWithoutTextureAlpha(v) => &v.declaration,
            Self::ReflectionEmission(v) => &v.declaration,
            Self::DiffuseWeightedReflectionEmission(v) => &v.declaration,
            Self::LayeredDiffuse(v) => &v.declaration,
            Self::MaskedEmission(v) => &v.declaration,
            Self::MaskedDiffuseModulationReflection(v) => &v.declaration,
            Self::DiffuseMultipleEmission(v) => &v.declaration,
            Self::OpaqueLayeredDiffuse(v) => &v.declaration,
            Self::LayeredDiffuseEmission(v) => &v.declaration,
            Self::MaskedDiffuseModulation(v) => &v.declaration,
            Self::LayeredDiffuseModulation(v) => &v.declaration,
            Self::DiffuseModulation(v) => &v.declaration,
            Self::TextureMaskedLayeredModulation(v) => &v.declaration,
            Self::Unresolved(v) => &v.declaration,
        }
    }
    /// Mutable material-wide rendering declarations.
    pub fn declaration_mut(&mut self) -> &mut MaterialDeclaration<'a> {
        match self {
            Self::Diffuse(v) => &mut v.declaration,
            Self::DiffuseWithoutTextureAlpha(v) => &mut v.declaration,
            Self::ReflectionEmission(v) => &mut v.declaration,
            Self::DiffuseWeightedReflectionEmission(v) => &mut v.declaration,
            Self::LayeredDiffuse(v) => &mut v.declaration,
            Self::MaskedEmission(v) => &mut v.declaration,
            Self::MaskedDiffuseModulationReflection(v) => &mut v.declaration,
            Self::DiffuseMultipleEmission(v) => &mut v.declaration,
            Self::OpaqueLayeredDiffuse(v) => &mut v.declaration,
            Self::LayeredDiffuseEmission(v) => &mut v.declaration,
            Self::MaskedDiffuseModulation(v) => &mut v.declaration,
            Self::LayeredDiffuseModulation(v) => &mut v.declaration,
            Self::DiffuseModulation(v) => &mut v.declaration,
            Self::TextureMaskedLayeredModulation(v) => &mut v.declaration,
            Self::Unresolved(v) => &mut v.declaration,
        }
    }
    /// Texture bindings in stable shader-slot order, including unused authored bindings.
    pub fn inputs(&self) -> Vec<&TextureBinding<Option<TextureId>>> {
        match self {
            Self::Diffuse(v) => {
                let mut inputs = vec![&v.diffuse];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::DiffuseWithoutTextureAlpha(v) => {
                let mut inputs = vec![&v.diffuse];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::ReflectionEmission(v) => {
                let mut inputs = vec![&v.diffuse, &v.reflection];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::DiffuseWeightedReflectionEmission(v) => {
                let mut inputs = vec![&v.diffuse, &v.reflection];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::LayeredDiffuse(v) => {
                let mut inputs = vec![&v.diffuse, &v.secondary_diffuse];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::MaskedEmission(v) => {
                let mut inputs = vec![&v.diffuse, &v.emission];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::MaskedDiffuseModulationReflection(v) => {
                let mut inputs = vec![&v.diffuse, &v.reflection, &v.secondary_diffuse];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::DiffuseMultipleEmission(v) => {
                let mut inputs = vec![&v.diffuse, &v.reflection, &v.emission];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::OpaqueLayeredDiffuse(v) => {
                let mut inputs = vec![&v.diffuse, &v.secondary_diffuse];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::LayeredDiffuseEmission(v) => {
                let mut inputs = vec![&v.diffuse, &v.secondary_diffuse];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::MaskedDiffuseModulation(v) => {
                let mut inputs = vec![&v.diffuse, &v.reflection, &v.modulation];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::LayeredDiffuseModulation(v) => {
                let mut inputs = vec![&v.diffuse, &v.secondary_diffuse, &v.modulation];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::DiffuseModulation(v) => {
                let mut inputs = vec![&v.diffuse, &v.modulation];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::TextureMaskedLayeredModulation(v) => {
                let mut inputs = vec![&v.diffuse, &v.secondary_diffuse, &v.modulation_mask];
                inputs.extend(&v.unused_inputs);
                inputs
            }
            Self::Unresolved(v) => v.textures.iter().collect(),
        }
    }
    /// Derives the native selector from the interpreted equation.
    pub fn native_shader(&self) -> Option<u32> {
        match self {
            Self::Diffuse(_) => Some(0),
            Self::DiffuseWithoutTextureAlpha(_) => Some(4),
            Self::ReflectionEmission(_) => Some(3),
            Self::DiffuseWeightedReflectionEmission(_) => Some(5),
            Self::LayeredDiffuse(_) => Some(6),
            Self::MaskedEmission(_) => Some(9),
            Self::MaskedDiffuseModulationReflection(_) => Some(11),
            Self::DiffuseMultipleEmission(_) => Some(12),
            Self::OpaqueLayeredDiffuse(_) => Some(13),
            Self::LayeredDiffuseEmission(_) => Some(15),
            Self::MaskedDiffuseModulation(_) => Some(17),
            Self::LayeredDiffuseModulation(_) => Some(18),
            Self::DiffuseModulation(_) => Some(19),
            Self::TextureMaskedLayeredModulation(_) => Some(20),
            Self::Unresolved(v) => v.shader,
        }
    }
    /// Whether this is one of the explicitly partial terrain or LOD paths.
    pub fn is_partial(&self) -> bool {
        matches!(self,Self::Unresolved(v) if matches!(v.shader,Some(16 | 21)))
    }
    /// Checks draw declarations independently of graph references.
    pub fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}
impl ValidateLinks for MaterialStep<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.declaration().state.visit_links(visitor);
        for input in self.inputs() {
            input.visit_links(visitor);
        }
        if self
            .declaration()
            .uv_scroll
            .iter()
            .flatten()
            .any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}
impl SurfacePayload for MaterialStep<'_> {
    fn validate(&self) -> Result<(), Error> {
        self.validate()
    }
    fn supports_property(&self, _key: &PropertyIdentityKey) -> bool {
        false
    }
}
