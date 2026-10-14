//! Direct native timelines and checked logical animation programs.
use crate::model::semantic::{profiles::m3 as surface, PropertyTarget};
use std::collections::HashMap;

/// Native identity assigned to a logical animated destination.
#[derive(Clone, Copy)]
pub(super) struct AnimationId {
    pub id: u32,
    pub interpolation: surface::values::Interpolation,
    pub keyed: bool,
}
pub(super) type AnimationIds = HashMap<PropertyTarget, AnimationId>;

mod blocks;
mod events;
mod export;
mod extents;
mod import;
mod keys;
mod raw_export;
mod raw_keys;
mod structured_export;
pub(super) use export::{prepare, write};
pub(super) use import::read;
#[cfg(test)]
mod tests;
