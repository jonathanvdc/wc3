//! Direct native timelines and checked logical animation programs.
use crate::model::PropertyValues;
use crate::model::{profiles::m3 as surface, ValueSlot};
use std::collections::HashMap;

/// Native identity assigned to a logical animated destination.
#[derive(Clone, Copy)]
pub(super) struct AnimationId {
    pub id: u32,
    pub interpolation: surface::values::Interpolation,
    pub keyed: bool,
}
pub(super) struct AnimationIds {
    pub values: PropertyValues,
    entries: HashMap<ValueSlot, AnimationId>,
}
impl AnimationIds {
    fn new(values: &PropertyValues) -> Self {
        Self {
            values: values.clone(),
            entries: HashMap::new(),
        }
    }
}
impl std::ops::Deref for AnimationIds {
    type Target = HashMap<ValueSlot, AnimationId>;
    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}
impl std::ops::DerefMut for AnimationIds {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

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
