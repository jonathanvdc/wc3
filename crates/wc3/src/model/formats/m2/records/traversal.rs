//! Field validation and context-preserving animation traversal.
use crate::model::formats::m2::codec::{Array, Value};
use crate::model::formats::m2::ReadError;
use zerovec::ZeroVec;

/// Applies an operation to animation leaves with their native sequence context.
pub trait ArrayVisitor<'data> {
    /// Operation failure.
    type Error;
    /// Visits one encoded or externally referenced leaf.
    fn visit<T: Value>(
        &mut self,
        sequence: usize,
        global_sequence: u16,
        values: &mut Array<'data, T>,
    ) -> Result<(), Self::Error>;
}
/// Traverses encoded animation leaves without erasing their value type.
pub trait VisitArrays<'data> {
    /// Visits leaves in native order.
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, visitor: &mut V) -> Result<(), V::Error>;
}
/// Validates locally available native record relationships.
pub trait Validate {
    /// Checks field relationships without loading other documents.
    fn validate(&self) -> Result<(), ReadError>;
}
macro_rules! scalars {
    ($($ty:ty),* $(,)?)=>{$(
        impl Validate for $ty {fn validate(&self)->Result<(),ReadError>{Ok(())}}
        impl<'data> VisitArrays<'data> for $ty {fn visit_arrays<V:ArrayVisitor<'data>>(&mut self,_:&mut V)->Result<(),V::Error>{Ok(())}}
    )*};
}
scalars!((), u8, i8, u16, i16, u32, i32, f32);
impl<T: Validate, const N: usize> Validate for [T; N] {
    fn validate(&self) -> Result<(), ReadError> {
        for v in self {
            v.validate()?;
        }
        Ok(())
    }
}
impl<'data, T: VisitArrays<'data>, const N: usize> VisitArrays<'data> for [T; N] {
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, v: &mut V) -> Result<(), V::Error> {
        for f in self {
            f.visit_arrays(v)?;
        }
        Ok(())
    }
}
impl<T: Validate> Validate for Vec<T> {
    fn validate(&self) -> Result<(), ReadError> {
        for v in self {
            v.validate()?;
        }
        Ok(())
    }
}
impl<'data, T: VisitArrays<'data>> VisitArrays<'data> for Vec<T> {
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, v: &mut V) -> Result<(), V::Error> {
        for f in self {
            f.visit_arrays(v)?;
        }
        Ok(())
    }
}
impl<T: Validate> Validate for Option<T> {
    fn validate(&self) -> Result<(), ReadError> {
        if let Some(value) = self {
            value.validate()?;
        }
        Ok(())
    }
}
impl<'data, T: VisitArrays<'data>> VisitArrays<'data> for Option<T> {
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, visitor: &mut V) -> Result<(), V::Error> {
        if let Some(value) = self {
            value.visit_arrays(visitor)?;
        }
        Ok(())
    }
}
impl<T: Validate + ?Sized> Validate for Box<T> {
    fn validate(&self) -> Result<(), ReadError> {
        (**self).validate()
    }
}
impl<'data, T: VisitArrays<'data> + ?Sized> VisitArrays<'data> for Box<T> {
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, visitor: &mut V) -> Result<(), V::Error> {
        (**self).visit_arrays(visitor)
    }
}
impl<T: Value> Validate for ZeroVec<'_, T> {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}
impl<'data, T: Value> VisitArrays<'data> for ZeroVec<'data, T> {
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, _: &mut V) -> Result<(), V::Error> {
        Ok(())
    }
}
