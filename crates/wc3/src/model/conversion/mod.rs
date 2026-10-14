//! Explicit conversion policies, reports, planning, and native adapters.
mod asset;
mod context;
mod plan;
mod policy;
mod report;
pub use asset::*;
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

/// Editable-model authoring import and export support.
pub mod wem;
