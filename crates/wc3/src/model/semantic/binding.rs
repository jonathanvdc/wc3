//! Profile-specific joint binding with borrowed matrix inspection.
use crate::model::common::IntoZeroVec;
use crate::model::semantic::profiles::{Either, Unavailable};
use crate::model::semantic::{Error, Link, ValidateLinks, ZeroVec};
use std::fmt::Debug;

/// A transform mapping an authored vertex into a joint's bind space.
pub type BindMatrix = [[f32; 4]; 4];
const IDENTITY: BindMatrix = [
    [1., 0., 0., 0.],
    [0., 1., 0., 0.],
    [0., 0., 1., 0.],
    [0., 0., 0., 1.],
];

/// Binding interpretation admitted by a semantic profile.
pub trait BindingSchema<'data>: Clone + Debug + Send + Sync + ValidateLinks {
    /// Inspects inverse bind transforms in joint order.
    fn matrices(&self, joints: usize) -> BindMatrices<'_, 'data>;
    /// Checks that the binding describes the specified joint palette.
    fn validate(&self, joints: usize) -> Result<(), Error>;
}

/// Borrowed matrix access; identity bindings do not allocate a matrix array.
#[derive(Clone, Copy, Debug)]
pub enum BindMatrices<'view, 'data> {
    /// Every joint uses the authored model coordinates directly.
    Identity(usize),
    /// Explicit inverse transforms in palette order.
    Explicit(&'view ZeroVec<'data, BindMatrix>),
}
impl BindMatrices<'_, '_> {
    /// Number of transforms in the joint palette.
    pub fn len(&self) -> usize {
        match self {
            Self::Identity(count) => *count,
            Self::Explicit(values) => values.len(),
        }
    }
    /// Whether the palette is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Reads one inverse bind transform.
    pub fn get(&self, index: usize) -> Option<BindMatrix> {
        match self {
            Self::Identity(count) => (index < *count).then_some(IDENTITY),
            Self::Explicit(values) => values.get(index),
        }
    }
    /// Iterates transforms without creating a second array.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = BindMatrix> + '_ {
        (0..self.len()).map(|index| self.get(index).expect("checked bind matrix index"))
    }
}

/// Model-space joint binding used by Warcraft III and M2 geometry.
#[derive(Clone, Copy, Debug, Default)]
pub struct IdentityBinding;
impl ValidateLinks for IdentityBinding {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl<'data> BindingSchema<'data> for IdentityBinding {
    fn matrices(&self, joints: usize) -> BindMatrices<'_, 'data> {
        BindMatrices::Identity(joints)
    }
    fn validate(&self, _: usize) -> Result<(), Error> {
        Ok(())
    }
}

/// An explicitly authored inverse bind transform per joint.
#[derive(Clone, Debug)]
pub struct ExplicitBinding<'data> {
    matrices: ZeroVec<'data, BindMatrix>,
}
impl<'data> ExplicitBinding<'data> {
    /// Constructs a finite inverse bind stream; the owning skin checks its cardinality.
    pub fn new(matrices: impl IntoZeroVec<'data, BindMatrix>) -> Result<Self, Error> {
        let value = Self {
            matrices: matrices.into_zero_vec(),
        };
        value.validate(value.matrices.len())?;
        Ok(value)
    }
    /// Authored transforms, retaining their borrowed storage.
    pub fn values(&self) -> &ZeroVec<'data, BindMatrix> {
        &self.matrices
    }
}
impl ValidateLinks for ExplicitBinding<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl<'data> BindingSchema<'data> for ExplicitBinding<'data> {
    fn matrices(&self, _: usize) -> BindMatrices<'_, 'data> {
        BindMatrices::Explicit(&self.matrices)
    }
    fn validate(&self, joints: usize) -> Result<(), Error> {
        if joints != self.matrices.len()
            || self
                .matrices
                .iter()
                .flatten()
                .flatten()
                .any(|value| !value.is_finite())
        {
            Err(Error::new(
                "skin.binding",
                "expected one finite inverse bind transform per joint",
            ))
        } else {
            Ok(())
        }
    }
}

/// All built-in bind interpretations, preserving their authored distinction.
#[derive(Clone, Debug)]
pub enum UniversalBinding<'data> {
    /// Implicit model-space binding.
    Identity(IdentityBinding),
    /// Explicit joint-space transforms.
    Explicit(ExplicitBinding<'data>),
}
impl ValidateLinks for UniversalBinding<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl<'data> BindingSchema<'data> for UniversalBinding<'data> {
    fn matrices(&self, joints: usize) -> BindMatrices<'_, 'data> {
        match self {
            Self::Identity(value) => value.matrices(joints),
            Self::Explicit(value) => value.matrices(joints),
        }
    }
    fn validate(&self, joints: usize) -> Result<(), Error> {
        match self {
            Self::Identity(value) => value.validate(joints),
            Self::Explicit(value) => value.validate(joints),
        }
    }
}
impl<'data, A: BindingSchema<'data>, B: BindingSchema<'data>> BindingSchema<'data>
    for Either<A, B>
{
    fn matrices(&self, joints: usize) -> BindMatrices<'_, 'data> {
        match self {
            Self::Left(value) => value.matrices(joints),
            Self::Right(value) => value.matrices(joints),
        }
    }
    fn validate(&self, joints: usize) -> Result<(), Error> {
        match self {
            Self::Left(value) => value.validate(joints),
            Self::Right(value) => value.validate(joints),
        }
    }
}
impl<'data> BindingSchema<'data> for Unavailable {
    fn matrices(&self, _: usize) -> BindMatrices<'_, 'data> {
        match *self {}
    }
    fn validate(&self, _: usize) -> Result<(), Error> {
        match *self {}
    }
}
