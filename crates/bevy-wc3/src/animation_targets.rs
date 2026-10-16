//! Shared node-channel presence and conservative material constant analysis.
use crate::compiled::NodeChannels;
use std::collections::{HashMap, HashSet};
use wc3::model::{Animated, Channel, Interpolation, Model, NodeId, Property, PropertyTarget};

pub(crate) struct AnimationTargets {
    targets: HashSet<PropertyTarget>,
    scalar_constants: HashMap<PropertyTarget, Option<f32>>,
}
impl AnimationTargets {
    pub(crate) fn new(model: &Model<'_>) -> Self {
        let mut targets = HashSet::new();
        let mut scalar_constants = HashMap::<PropertyTarget, Option<f32>>::new();
        for channel in model
            .timeline_channels()
            .iter()
            .chain(model.shared_channels())
            .chain(model.clips().flat_map(|(_, clip)| clip.channels()))
            .chain(
                model
                    .controllers()
                    .flat_map(|(_, controller)| &controller.channels),
            )
        {
            targets.insert(channel.target().clone());
            if let Channel::Scalar(track) = channel {
                let values = track.curve.values();
                let constant = values.get(0).filter(|first| {
                    first.is_finite()
                        && (values.len() == 1
                            || matches!(
                                track.curve.interpolation(),
                                Interpolation::Step | Interpolation::Linear
                            ))
                        && values
                            .iter()
                            .all(|value| value.to_bits() == first.to_bits())
                });
                scalar_constants
                    .entry(channel.target().clone())
                    .and_modify(|previous| {
                        if previous.map(f32::to_bits) != constant.map(f32::to_bits) {
                            *previous = None;
                        }
                    })
                    .or_insert(constant);
            }
        }
        Self {
            targets,
            scalar_constants,
        }
    }

    pub(crate) fn node_channels(&self, node: NodeId) -> NodeChannels {
        NodeChannels {
            translation: self.targets.contains(&PropertyTarget::Translation(node)),
            rotation: self.targets.contains(&PropertyTarget::Rotation(node)),
            scale: self.targets.contains(&PropertyTarget::Scale(node)),
            visibility: self.targets.contains(&PropertyTarget::Visibility(node)),
        }
    }

    pub(crate) fn contains<T>(&self, binding: &Option<Property<T>>) -> bool {
        binding
            .as_ref()
            .is_some_and(|property| self.targets.contains(property.target()))
    }

    /// Prune a cloned runtime binding while preserving its authored rest value.
    pub(crate) fn retain<T>(&self, binding: &mut Option<Property<T>>) {
        if !self.contains(binding) {
            *binding = None;
        }
    }
    /// A scalar binding is redundant only when every possible programme writes
    /// exactly its fallback value. Different clip/controller constants remain live.
    pub(crate) fn scalar_changes(&self, default: f32, binding: &Option<Property<f32>>) -> bool {
        self.contains(binding)
            && !binding.as_ref().is_some_and(|property| {
                self.scalar_constants
                    .get(property.target())
                    .copied()
                    .flatten()
                    .is_some_and(|value| value.to_bits() == default.to_bits())
            })
    }

    pub(crate) fn retain_scalar(&self, value: &mut Animated<f32>) {
        self.retain_scalar_binding(value.default, &mut value.property);
    }

    pub(crate) fn retain_scalar_binding(&self, default: f32, binding: &mut Option<Property<f32>>) {
        if !self.scalar_changes(default, binding) {
            *binding = None;
        }
    }
}
