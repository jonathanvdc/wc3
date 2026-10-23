//! Import native documents into checked models and plan explicit native exports.
use super::conversion::*;
/// Incremental native model loading with caller-owned resource access.
pub mod asset;
/// ANIMATION import and export support.
pub mod animation;

/// GEOMETRY import and export support.
pub mod geometry;
/// M2 import and export support.
pub mod m2;

/// MDLX import and export support.
pub mod mdlx;
/// WMO import and export support.
pub mod wmo;

use crate::model::common::{PackedArray, PackedValue};

/// Materializes a projection once at the native conversion boundary.
fn materialize<T: PackedValue>(len: usize, get: impl Fn(usize) -> T) -> PackedArray<'static, T> {
    (0..len).map(get).collect()
}

fn owned<T: PackedValue>(values: impl AsRef<[T]>) -> PackedArray<'static, T> {
    PackedArray::alloc_from_slice(values.as_ref())
}
