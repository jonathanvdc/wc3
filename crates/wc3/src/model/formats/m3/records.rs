//! Native M3 record families and independent section-revision dispatch.
mod common;
pub use common::*;
pub mod animation;
pub mod targets;
pub use animation::*;
pub mod mesh;
pub use mesh::*;
pub mod material;
pub use material::*;
pub mod effect;
pub use effect::*;
pub mod physics;
pub use physics::*;
pub mod scene;
pub use scene::*;
pub mod model;
pub use model::*;
pub mod primitive;
pub use primitive::*;
mod registry;
pub use registry::*;

mod dialect;
pub use dialect::*;

pub mod values;
pub use values::*;
