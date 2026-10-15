//! Storage components for native animation conversion.
use super::*;
use zerovec::ZeroVec;

/// Construction of a native timestamp family from complete sequence leaves.
pub trait TimelineFactory: Sized {
    /// Builds per-sequence leaves or one legacy timeline with explicit windows.
    fn build(leaves: Vec<Vec<u32>>, global: bool) -> Self;
}

impl TimelineFactory for native::records::ModernTimeline<'static> {
    fn build(leaves: Vec<Vec<u32>>, _: bool) -> Self {
        Self {
            timestamps: leaves
                .into_iter()
                .map(|times| Array::Inline(ZeroVec::alloc_from_slice(&times)))
                .collect(),
        }
    }
}

impl TimelineFactory for native::records::LegacyTimeline<'static> {
    fn build(leaves: Vec<Vec<u32>>, global: bool) -> Self {
        let mut ranges = Vec::new();
        let mut times = Vec::new();
        for leaf in leaves {
            let start = times.len() as u32;
            let count = leaf.len() as u32;
            times.extend(leaf);
            ranges.push(if count == 0 {
                [0, 0]
            } else {
                [start, start + count - 1]
            });
        }
        Self {
            ranges: if global {
                ZeroVec::new()
            } else {
                ZeroVec::alloc_from_slice(&ranges)
            },
            timestamps: ZeroVec::alloc_from_slice(&times),
        }
    }
}

/// Construction of ordinary or tangent-triplet native key leaves.
pub trait ValuesFactory<T: Value>: Sized
where
    [T; 3]: Value,
{
    /// Creates keys with the selected runtime interpolation shape.
    fn build(values: Vec<Vec<T>>, splines: Vec<Vec<[T; 3]>>, is_spline: bool) -> Self;
}

impl<T: Value> ValuesFactory<T> for native::records::LegacyValues<'static, T>
where
    [T; 3]: Value,
{
    fn build(values: Vec<Vec<T>>, splines: Vec<Vec<[T; 3]>>, is_spline: bool) -> Self {
        if is_spline {
            Self::Splines(ZeroVec::alloc_from_slice(
                &splines.into_iter().flatten().collect::<Vec<_>>(),
            ))
        } else {
            Self::Values(ZeroVec::alloc_from_slice(
                &values.into_iter().flatten().collect::<Vec<_>>(),
            ))
        }
    }
}

impl<T: Value> ValuesFactory<T> for native::records::TrackValues<'static, T>
where
    [T; 3]: Value,
{
    fn build(values: Vec<Vec<T>>, splines: Vec<Vec<[T; 3]>>, is_spline: bool) -> Self {
        if is_spline {
            Self::Splines(
                splines
                    .into_iter()
                    .map(|v| Array::Inline(ZeroVec::alloc_from_slice(&v)))
                    .collect(),
            )
        } else {
            Self::Values(
                values
                    .into_iter()
                    .map(|v| Array::Inline(ZeroVec::alloc_from_slice(&v)))
                    .collect(),
            )
        }
    }
}
