//! Native M2 records whose field representations are selected by layout families.

pub mod layout;
pub use layout::*;

mod animation;
pub use animation::*;
mod effects;
pub use effects::*;
mod geometry;
pub use geometry::*;
mod materials;
pub use materials::*;
mod scene;
pub use scene::*;
mod storage;
pub use storage::*;
mod traversal;
pub use traversal::*;

#[cfg(test)]
#[path = "records/tests.rs"]
mod tests;

mod types;
pub use types::*;
