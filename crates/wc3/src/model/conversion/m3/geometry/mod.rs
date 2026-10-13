//! Native geometry conversion by vertex layout and draw division.
mod packed;
use packed::packed_vertices;
mod import;
pub(super) use import::semantic;
mod export;
pub(super) use export::encode;

mod controls;
pub(super) use controls::attach;
