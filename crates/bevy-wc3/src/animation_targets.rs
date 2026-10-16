//! Shared node-channel presence and conservative material constant analysis.
use std::collections::{HashMap, HashSet};
use wc3::model::profiles::Universal;
use wc3::model::{Animated, BindingValue, Channel, Interpolation, Model, PropertyTarget};

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

    pub(crate) fn contains<B: BindingValue<Profile = Universal>>(
        &self,
        binding: &Option<B>,
    ) -> bool {
        binding
            .as_ref()
            .is_some_and(|property| self.targets.contains(property.target()))
    }

    /// Prune a cloned runtime binding while preserving its authored rest value.
    pub(crate) fn retain<B: BindingValue<Profile = Universal>>(&self, binding: &mut Option<B>) {
        if !self.contains(binding) {
            *binding = None;
        }
    }
    /// A scalar binding is redundant only when every possible programme writes
    /// exactly its fallback value. Different clip/controller constants remain live.
    pub(crate) fn scalar_changes<B: BindingValue<Value = f32, Profile = Universal>>(
        &self,
        default: f32,
        binding: &Option<B>,
    ) -> bool {
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
        self.retain_scalar_binding(value.default, &mut value.binding);
    }

    pub(crate) fn retain_scalar_binding<B: BindingValue<Value = f32, Profile = Universal>>(
        &self,
        default: f32,
        binding: &mut Option<B>,
    ) {
        if !self.scalar_changes(default, binding) {
            *binding = None;
        }
    }
}
