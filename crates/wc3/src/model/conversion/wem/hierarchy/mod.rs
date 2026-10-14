//! Hierarchy placement, attached payloads, and saved authoring poses.
mod base;

pub(super) mod poses;

mod setup;

pub(super) mod payloads;

mod read;
pub(super) use read::read;

mod write;
pub(super) use write::{supports, write, Context as WriteContext};
