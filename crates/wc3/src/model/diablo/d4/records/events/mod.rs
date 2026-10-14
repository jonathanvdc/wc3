//! Native trigger event records, grouped by affected system.
use super::*;
/// Native common event records.
pub mod common;
pub use common::*;
/// Native interface event records.
pub mod interface;
pub use interface::*;
/// Native scene event records.
pub mod scene;
pub use scene::*;
/// Native effects event records.
pub mod effects;
pub use effects::*;
/// Native audio event records.
pub mod audio;
pub use audio::*;
