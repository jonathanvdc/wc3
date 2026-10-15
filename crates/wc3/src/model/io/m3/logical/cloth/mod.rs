//! Cloth solvers bind scene geometry and nodes without native table indices.
mod read;
mod write;
pub(super) use read::read;
pub(super) use write::write;
