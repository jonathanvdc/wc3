//! Channel destinations that can actually change material values.
use std::collections::HashSet;
use wc3::model::{Model, Property, PropertyTarget};

pub(super) struct AnimationTargets(HashSet<PropertyTarget>);
impl AnimationTargets {
    pub(super) fn new(model: &Model<'_>) -> Self {
        Self(
            model
                .timeline_channels()
                .iter()
                .chain(model.shared_channels())
                .chain(model.clips().flat_map(|(_, clip)| clip.channels()))
                .chain(
                    model
                        .controllers()
                        .flat_map(|(_, controller)| &controller.channels),
                )
                .map(|channel| channel.target().clone())
                .collect(),
        )
    }

    pub(super) fn contains<T>(&self, binding: &Option<Property<T>>) -> bool {
        binding
            .as_ref()
            .is_some_and(|property| self.0.contains(property.target()))
    }

    /// Prune a cloned runtime binding while preserving its authored rest value.
    pub(super) fn retain<T>(&self, binding: &mut Option<Property<T>>) {
        if !self.contains(binding) {
            *binding = None;
        }
    }
}
