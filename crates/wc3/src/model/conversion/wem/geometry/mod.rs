//! Native polygon topology is preserved without constructing draw instances.
mod read;
mod write;
pub(super) use read::read;
pub(super) use write::write;

#[cfg(test)]
mod tests;
