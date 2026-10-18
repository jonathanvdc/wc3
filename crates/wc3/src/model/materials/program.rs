//! Ordered material programs with explicit shader dialects.
use super::m2::MaterialStep as M2MaterialStep;
use super::mdlx::{LayeredSurface, MaterialDeclaration, MaterialStep as MdlxMaterialStep};
use super::wmo::MaterialStep as WmoMaterialStep;
use super::{diablo3, diablo4, m3};
use crate::model::profiles::SurfacePayload;
use crate::model::{Error, Link, ValidateLinks};

/// An authored sequence of material operations, interpreted into rendering passes.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialProgram<S> {
    /// Material operations in authored order.
    pub steps: Vec<S>,
}
impl<S> Default for MaterialProgram<S> {
    fn default() -> Self {
        Self { steps: Vec::new() }
    }
}
/// One typed operation in the universal profile's material program.
///
/// Each dialect retains its shader semantics. Composition and external shader
/// steps may expand into multiple rendering passes when interpreted.
#[derive(Clone, Debug)]
pub enum MaterialStep<'a> {
    /// Warcraft III shader inputs and rendering settings.
    Mdlx(Box<MdlxMaterialStep<'a>>),
    /// World of Warcraft model shader inputs and rendering settings.
    M2(Box<M2MaterialStep<'a>>),
    /// World of Warcraft world shader inputs and rendering settings.
    Wmo(Box<WmoMaterialStep<'a>>),
    /// M3 shader or material-composition operation.
    M3(m3::MaterialStep<'a>),
    /// Diablo III shader inputs and ordered phase declarations.
    Diablo3(Box<diablo3::MaterialStep<'a>>),
    /// Diablo IV external shader inputs and parameters.
    Diablo4(Box<diablo4::MaterialStep>),
}
/// The universal profile's complete material program.
pub type MaterialSurface<'a> = MaterialProgram<MaterialStep<'a>>;
impl<S: ValidateLinks> ValidateLinks for MaterialProgram<S> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.steps.visit_links(visitor);
    }
}
impl ValidateLinks for MaterialStep<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Mdlx(v) => v.visit_links(visitor),
            Self::M2(v) => v.visit_links(visitor),
            Self::Wmo(v) => v.visit_links(visitor),
            Self::M3(v) => v.visit_links(visitor),
            Self::Diablo3(v) => v.visit_links(visitor),
            Self::Diablo4(v) => v.visit_links(visitor),
        }
    }
}
impl MaterialStep<'_> {
    /// Checks the dialect's local invariants independently of graph relationships.
    pub fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Mdlx(_) => Ok(()),
            Self::M2(v) => v.validate(),
            Self::Wmo(v) => v.validate(),
            Self::M3(v) => v.validate(),
            Self::Diablo3(v) => v.validate(),
            Self::Diablo4(v) => v.validate(),
        }
    }
}
impl SurfacePayload for MaterialSurface<'_> {
    fn validate(&self) -> Result<(), Error> {
        self.steps.iter().try_for_each(MaterialStep::validate)
    }
}

fn incompatible(message: &str) -> Error {
    Error::new("surface", message)
}
impl<'a> TryFrom<LayeredSurface<'a>> for MaterialSurface<'a> {
    type Error = Error;
    fn try_from(value: LayeredSurface<'a>) -> Result<Self, Error> {
        value.validate()?;
        let LayeredSurface {
            priority_plane,
            shader,
            constant_color,
            two_sided,
            sort_near_z,
            sort_far_z,
            full_resolution,
            unclassified,
            layers,
        } = value;
        Ok(Self {
            steps: layers
                .into_iter()
                .map(|layer| {
                    MaterialStep::Mdlx(Box::new(MdlxMaterialStep::from_layer(
                        layer,
                        MaterialDeclaration {
                            priority_plane,
                            constant_color,
                            two_sided,
                            sort_near_z,
                            sort_far_z,
                            full_resolution,
                            unclassified,
                        },
                        shader.clone(),
                    )))
                })
                .collect(),
        })
    }
}
impl<'a> From<M2MaterialStep<'a>> for MaterialSurface<'a> {
    fn from(value: M2MaterialStep<'a>) -> Self {
        Self {
            steps: vec![MaterialStep::M2(Box::new(value))],
        }
    }
}
impl<'a> From<WmoMaterialStep<'a>> for MaterialSurface<'a> {
    fn from(value: WmoMaterialStep<'a>) -> Self {
        Self {
            steps: vec![MaterialStep::Wmo(Box::new(value))],
        }
    }
}
impl<'a> TryFrom<MaterialSurface<'a>> for LayeredSurface<'a> {
    type Error = Error;
    fn try_from(value: MaterialSurface<'a>) -> Result<Self, Error> {
        let mut steps = value.steps.into_iter();
        let Some(first) = steps.next() else {
            return Ok(Self::default());
        };
        let MaterialStep::Mdlx(first) = first else {
            return Err(incompatible("surface contains a non-MDLX step"));
        };
        let mut layers = Vec::new();
        let mut shader = match first.as_ref() {
            MdlxMaterialStep::Unresolved(step) => Some(step.shader_name.clone()),
            _ => None,
        };
        for pass in steps {
            let MaterialStep::Mdlx(pass) = pass else {
                return Err(incompatible("surface contains a non-MDLX step"));
            };
            if !first.same_declaration(&pass) {
                return Err(incompatible(
                    "MDLX steps have different material-wide draw declarations",
                ));
            }
            if let MdlxMaterialStep::Unresolved(step) = pass.as_ref() {
                if shader
                    .as_ref()
                    .is_some_and(|name| name != &step.shader_name)
                {
                    return Err(incompatible(
                        "MDLX unresolved steps require different material shader names",
                    ));
                }
                shader = Some(step.shader_name.clone());
            }
            layers.push((
                matches!(pass.as_ref(), MdlxMaterialStep::Classic(_)),
                pass.to_layer(),
            ));
        }
        let MaterialDeclaration {
            priority_plane,
            constant_color,
            two_sided,
            sort_near_z,
            sort_far_z,
            full_resolution,
            unclassified,
        } = first.declaration().clone();
        layers.insert(
            0,
            (
                matches!(first.as_ref(), MdlxMaterialStep::Classic(_)),
                first.to_layer(),
            ),
        );
        let shader = shader.unwrap_or_default();
        let layers = layers
            .into_iter()
            .map(|(classic, mut layer)| {
                // A shared unresolved name must not override a known classic step.
                if classic && !shader.is_empty() {
                    layer.shader = Some(0);
                }
                layer
            })
            .collect();
        Ok(Self {
            layers,
            priority_plane,
            shader,
            constant_color,
            two_sided,
            sort_near_z,
            sort_far_z,
            full_resolution,
            unclassified,
        })
    }
}
impl<'a> TryFrom<MaterialSurface<'a>> for M2MaterialStep<'a> {
    type Error = Error;
    fn try_from(value: MaterialSurface<'a>) -> Result<Self, Error> {
        let MaterialStep::M2(value) = single_step(value)? else {
            return Err(incompatible("surface is not an M2 shader step"));
        };
        Ok(*value)
    }
}
impl<'a> TryFrom<MaterialSurface<'a>> for WmoMaterialStep<'a> {
    type Error = Error;
    fn try_from(value: MaterialSurface<'a>) -> Result<Self, Error> {
        let MaterialStep::Wmo(value) = single_step(value)? else {
            return Err(incompatible("surface is not a WMO shader step"));
        };
        Ok(*value)
    }
}
fn single_step(value: MaterialSurface<'_>) -> Result<MaterialStep<'_>, Error> {
    let mut stack = value;
    if stack.steps.len() != 1 {
        return Err(incompatible("surface requires exactly one step"));
    }
    Ok(stack.steps.pop().expect("one step"))
}

impl<'a> From<m3::MaterialStep<'a>> for MaterialStep<'a> {
    fn from(value: m3::MaterialStep<'a>) -> Self {
        Self::M3(value)
    }
}
impl<'a> TryFrom<MaterialStep<'a>> for m3::MaterialStep<'a> {
    type Error = Error;
    fn try_from(value: MaterialStep<'a>) -> Result<Self, Error> {
        let MaterialStep::M3(step) = value else {
            return Err(incompatible("step is not in the M3 shader dialect"));
        };
        Ok(step)
    }
}
impl<'a> From<m3::MaterialStep<'a>> for MaterialSurface<'a> {
    fn from(value: m3::MaterialStep<'a>) -> Self {
        Self {
            steps: vec![value.into()],
        }
    }
}
impl<'a> TryFrom<MaterialSurface<'a>> for m3::MaterialStep<'a> {
    type Error = Error;
    fn try_from(value: MaterialSurface<'a>) -> Result<Self, Error> {
        single_step(value)?.try_into()
    }
}
impl<'a> From<diablo3::MaterialStep<'a>> for MaterialSurface<'a> {
    fn from(value: diablo3::MaterialStep<'a>) -> Self {
        Self {
            steps: vec![MaterialStep::Diablo3(Box::new(value))],
        }
    }
}
impl<'a> TryFrom<MaterialSurface<'a>> for diablo3::MaterialStep<'a> {
    type Error = Error;
    fn try_from(value: MaterialSurface<'a>) -> Result<Self, Error> {
        let MaterialStep::Diablo3(shader) = single_step(value)? else {
            return Err(incompatible("not a Diablo III shader programme"));
        };
        Ok(*shader)
    }
}
impl<'a> From<MaterialProgram<diablo4::MaterialStep>> for MaterialSurface<'a> {
    fn from(value: MaterialProgram<diablo4::MaterialStep>) -> Self {
        Self {
            steps: value
                .steps
                .into_iter()
                .map(|shader| MaterialStep::Diablo4(Box::new(shader)))
                .collect(),
        }
    }
}
impl TryFrom<MaterialSurface<'_>> for MaterialProgram<diablo4::MaterialStep> {
    type Error = Error;
    fn try_from(value: MaterialSurface<'_>) -> Result<Self, Error> {
        if value.steps.len() > 1 {
            return Err(incompatible(
                "Diablo IV surface admits at most one shader programme",
            ));
        }
        let steps = value
            .steps
            .into_iter()
            .map(|pass| match pass {
                MaterialStep::Diablo4(shader) => Ok(*shader),
                _ => Err(incompatible("not a Diablo IV shader programme")),
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { steps })
    }
}

impl SurfacePayload for MaterialProgram<M2MaterialStep<'_>> {
    fn validate(&self) -> Result<(), Error> {
        self.steps.iter().try_for_each(M2MaterialStep::validate)
    }
}
impl<'a> From<MaterialProgram<M2MaterialStep<'a>>> for MaterialSurface<'a> {
    fn from(value: MaterialProgram<M2MaterialStep<'a>>) -> Self {
        Self {
            steps: value
                .steps
                .into_iter()
                .map(|s| MaterialStep::M2(Box::new(s)))
                .collect(),
        }
    }
}
impl<'a> TryFrom<MaterialSurface<'a>> for MaterialProgram<M2MaterialStep<'a>> {
    type Error = Error;
    fn try_from(value: MaterialSurface<'a>) -> Result<Self, Error> {
        Ok(Self {
            steps: value
                .steps
                .into_iter()
                .map(|s| match s {
                    MaterialStep::M2(v) => Ok(*v),
                    _ => Err(incompatible("surface contains a non-M2 material step")),
                })
                .collect::<Result<_, _>>()?,
        })
    }
}
impl<S> From<S> for MaterialProgram<S> {
    fn from(step: S) -> Self {
        Self { steps: vec![step] }
    }
}
