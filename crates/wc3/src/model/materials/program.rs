//! Ordered material programs with explicit shader dialects.
use super::m2::MaterialStep as M2MaterialStep;
use super::mdlx::{MaterialDeclaration, MaterialStep as MdlxMaterialStep};
use super::wmo::MaterialStep as WmoMaterialStep;
use crate::model::common::{Detached, IntoOwned};
use crate::model::error::{Location, ProfileError, ProfileKind, ProfileViolation};
use crate::model::profiles::{NoProperties, SurfacePayload};
use crate::model::{Error, Link, ValidateLinks};

/// An authored sequence of material operations, interpreted into rendering passes.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialProgram<S, D = NoProperties> {
    /// Declarations owned by this material, independent of the number of steps.
    pub declaration: D,
    /// Material operations in authored order.
    pub steps: Vec<S>,
}
impl<'owned, S: Detached<'owned>, D: Detached<'owned>> Detached<'owned> for MaterialProgram<S, D> {
    type Detached = MaterialProgram<S::Detached, D::Detached>;
    fn into_detached(self) -> Self::Detached {
        MaterialProgram {
            declaration: self.declaration.into_detached(),
            steps: self.steps.into_detached(),
        }
    }
}
impl<S, D: Default> Default for MaterialProgram<S, D> {
    fn default() -> Self {
        Self {
            declaration: D::default(),
            steps: Vec::new(),
        }
    }
}
/// One typed operation in the universal profile's material program.
///
/// Each dialect retains its shader semantics. Composition and external shader
/// steps may expand into multiple rendering passes when interpreted.
#[derive(Clone, Debug, IntoOwned)]
pub enum MaterialStep<'a> {
    /// Warcraft III shader inputs and rendering settings.
    Mdlx(Box<MdlxMaterialStep>),
    /// World of Warcraft model shader inputs and rendering settings.
    M2(Box<M2MaterialStep<'a>>),
    /// World of Warcraft world shader inputs and rendering settings.
    Wmo(Box<WmoMaterialStep<'a>>),
}
/// The universal profile's complete material program.
pub type MaterialSurface<'a> = MaterialProgram<MaterialStep<'a>, Option<MaterialDeclaration<'a>>>;
impl<S: ValidateLinks, D: ValidateLinks> ValidateLinks for MaterialProgram<S, D> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.declaration.visit_links(visitor);
        self.steps.visit_links(visitor);
    }
}
impl ValidateLinks for MaterialStep<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Mdlx(v) => v.visit_links(visitor),
            Self::M2(v) => v.visit_links(visitor),
            Self::Wmo(v) => v.visit_links(visitor),
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
        }
    }
}
impl SurfacePayload for MaterialSurface<'_> {
    fn validate(&self) -> Result<(), Error> {
        self.steps.iter().enumerate().try_for_each(|(index, step)| {
            step.validate()
                .map_err(|error| error.with_context(Location::new().field("steps").index(index)))
        })
    }
}

fn incompatible(target: ProfileKind, step: &MaterialStep<'_>, index: usize) -> Error {
    let actual = match step {
        MaterialStep::Mdlx(_) => ProfileKind::Mdlx,
        MaterialStep::M2(_) => ProfileKind::M2,
        MaterialStep::Wmo(_) => ProfileKind::Wmo,
    };
    Error::at(
        Location::new().field("steps").index(index),
        ProfileError::new(target, ProfileViolation::MaterialDialect { actual }),
    )
}
impl<'a> From<MdlxMaterialStep> for MaterialSurface<'a> {
    fn from(value: MdlxMaterialStep) -> Self {
        Self {
            declaration: None,
            steps: vec![MaterialStep::Mdlx(Box::new(value))],
        }
    }
}
impl<'a> From<M2MaterialStep<'a>> for MaterialSurface<'a> {
    fn from(value: M2MaterialStep<'a>) -> Self {
        Self {
            declaration: None,
            steps: vec![MaterialStep::M2(Box::new(value))],
        }
    }
}
impl<'a> From<WmoMaterialStep<'a>> for MaterialSurface<'a> {
    fn from(value: WmoMaterialStep<'a>) -> Self {
        Self {
            declaration: None,
            steps: vec![MaterialStep::Wmo(Box::new(value))],
        }
    }
}
impl<'a> TryFrom<MaterialSurface<'a>> for M2MaterialStep<'a> {
    type Error = Error;
    fn try_from(value: MaterialSurface<'a>) -> Result<Self, Error> {
        let value = single_step(value, ProfileKind::M2)?;
        let MaterialStep::M2(value) = value else {
            return Err(incompatible(ProfileKind::M2, &value, 0));
        };
        Ok(*value)
    }
}
impl<'a> TryFrom<MaterialSurface<'a>> for WmoMaterialStep<'a> {
    type Error = Error;
    fn try_from(value: MaterialSurface<'a>) -> Result<Self, Error> {
        let value = single_step(value, ProfileKind::Wmo)?;
        let MaterialStep::Wmo(value) = value else {
            return Err(incompatible(ProfileKind::Wmo, &value, 0));
        };
        Ok(*value)
    }
}
fn single_step(value: MaterialSurface<'_>, target: ProfileKind) -> Result<MaterialStep<'_>, Error> {
    require_no_declaration(&value, target)?;
    let mut stack = value;
    if stack.steps.len() != 1 {
        return Err(Error::at(
            Location::new().field("steps"),
            ProfileError::new(
                target,
                ProfileViolation::MaterialStepCount {
                    expected: 1,
                    actual: stack.steps.len(),
                },
            ),
        ));
    }
    Ok(stack.steps.pop().expect("one step"))
}

impl SurfacePayload for MaterialProgram<M2MaterialStep<'_>> {
    fn validate(&self) -> Result<(), Error> {
        self.steps.iter().enumerate().try_for_each(|(index, step)| {
            step.validate()
                .map_err(|error| error.with_context(Location::new().field("steps").index(index)))
        })
    }
}
impl<'a> From<MaterialProgram<M2MaterialStep<'a>>> for MaterialSurface<'a> {
    fn from(value: MaterialProgram<M2MaterialStep<'a>>) -> Self {
        Self {
            declaration: None,
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
        require_no_declaration(&value, ProfileKind::M2)?;
        Ok(Self {
            declaration: NoProperties,
            steps: value
                .steps
                .into_iter()
                .enumerate()
                .map(|(index, step)| match step {
                    MaterialStep::M2(v) => Ok(*v),
                    step => Err(incompatible(ProfileKind::M2, &step, index)),
                })
                .collect::<Result<_, _>>()?,
        })
    }
}
impl<S, D: Default> From<S> for MaterialProgram<S, D> {
    fn from(step: S) -> Self {
        Self {
            declaration: D::default(),
            steps: vec![step],
        }
    }
}

impl SurfacePayload for MaterialProgram<MdlxMaterialStep, MaterialDeclaration<'_>> {
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}
impl<'a> From<MaterialProgram<MdlxMaterialStep, MaterialDeclaration<'a>>> for MaterialSurface<'a> {
    fn from(value: MaterialProgram<MdlxMaterialStep, MaterialDeclaration<'a>>) -> Self {
        Self {
            declaration: Some(value.declaration),
            steps: value
                .steps
                .into_iter()
                .map(|step| MaterialStep::Mdlx(Box::new(step)))
                .collect(),
        }
    }
}
impl<'a> TryFrom<MaterialSurface<'a>>
    for MaterialProgram<MdlxMaterialStep, MaterialDeclaration<'a>>
{
    type Error = Error;
    fn try_from(value: MaterialSurface<'a>) -> Result<Self, Error> {
        Ok(Self {
            declaration: value.declaration.unwrap_or_default(),
            steps: value
                .steps
                .into_iter()
                .enumerate()
                .map(|(index, step)| match step {
                    MaterialStep::Mdlx(step) => Ok(*step),
                    step => Err(incompatible(ProfileKind::Mdlx, &step, index)),
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

fn require_no_declaration(value: &MaterialSurface<'_>, target: ProfileKind) -> Result<(), Error> {
    if value
        .declaration
        .as_ref()
        .is_some_and(|declaration| declaration != &MaterialDeclaration::default())
    {
        return Err(Error::at(
            Location::new().field("declaration"),
            ProfileError::new(target, ProfileViolation::MaterialDeclaration),
        ));
    }
    Ok(())
}
