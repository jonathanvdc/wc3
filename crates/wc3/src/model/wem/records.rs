//! Native WEM section records and their independent revision dialects.
mod dialect;
pub use dialect::*;
mod core;
pub mod targets;
pub use core::*;
mod animation;
pub use animation::*;
mod material;
pub use material::*;
mod scene;
pub use scene::*;
mod native;
pub use native::*;
mod geometry;
pub use geometry::*;
mod physics;
pub use physics::*;
mod primitive;
pub use primitive::*;
mod variants;
pub use variants::*;
mod registry;
pub use registry::*;

mod enums;
pub use enums::*;

mod native_values;
pub use native_values::*;

mod storage_values;
pub use storage_values::*;
