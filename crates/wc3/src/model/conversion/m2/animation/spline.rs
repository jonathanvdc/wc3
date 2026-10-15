//! Dialect-selected camera spline value storage.
use super::*;
use zerovec::ZeroVec;
/// Builds typed native spline records from logical values and control tangents.
pub trait SplineFactory<T: Value>: Sized
where
    [T; 3]: Value,
{
    /// Constructs complete per-sequence tangent-triplet leaves.
    fn build(leaves: Vec<Vec<[T; 3]>>) -> Self;
}

impl<T: Value> SplineFactory<T> for native::records::LegacySplineValues<'static, T>
where
    [T; 3]: Value,
{
    fn build(leaves: Vec<Vec<[T; 3]>>) -> Self {
        Self {
            values: ZeroVec::alloc_from_slice(&leaves.into_iter().flatten().collect::<Vec<_>>()),
        }
    }
}

impl<T: Value> SplineFactory<T> for native::records::ModernSplineValues<'static, T>
where
    [T; 3]: Value,
{
    fn build(leaves: Vec<Vec<[T; 3]>>) -> Self {
        Self {
            values: leaves
                .into_iter()
                .map(|v| Array::Inline(ZeroVec::alloc_from_slice(&v)))
                .collect(),
        }
    }
}
