//! Explicit conversion policies, reports, planning, and native adapters.
mod context;
mod plan;
mod policy;
mod report;
pub use context::*;
pub use plan::*;
pub use policy::*;
pub use report::*;

/// World of Warcraft animated-model imports and typed exports.
pub mod m2;
/// StarCraft II indexed-model imports and typed exports.
pub mod m3;
/// Warcraft III model imports and typed exports.
pub mod mdlx;

/// World model geometry imports and typed root/group exports.
pub mod wmo;

/// Exact curve restrictions for bounded native animation clocks.
pub mod animation;
/// Logical primitive and attribute normalization at native output boundaries.
pub mod geometry;

/// Diablo native model imports and typed exports.
pub mod diablo;

use zerovec::{ule::AsULE, ZeroVec};

/// Materializes a projection once at the native conversion boundary.
fn materialize<T: AsULE>(len: usize, get: impl Fn(usize) -> T) -> ZeroVec<'static, T> {
    (0..len).map(get).collect()
}

fn owned<T: AsULE>(values: impl AsRef<[T]>) -> ZeroVec<'static, T> {
    ZeroVec::alloc_from_slice(values.as_ref())
}
