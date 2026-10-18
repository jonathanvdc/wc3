//! Import native documents into checked models and plan explicit native exports.
use super::conversion::*;
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

use zerovec::{ule::AsULE, ZeroVec};

/// Materializes a projection once at the native conversion boundary.
fn materialize<T: AsULE>(len: usize, get: impl Fn(usize) -> T) -> ZeroVec<'static, T> {
    (0..len).map(get).collect()
}

fn owned<T: AsULE>(values: impl AsRef<[T]>) -> ZeroVec<'static, T> {
    ZeroVec::alloc_from_slice(values.as_ref())
}
