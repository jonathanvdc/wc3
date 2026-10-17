//! Shared node-channel presence and conservative material constant analysis.
use std::collections::{HashMap, HashSet};
use wc3::model::{Animatable, Animated, Channel, Interpolation, Model, ValueSlot, Values};

pub(crate) struct AnimationTargets<'a> {
    pub(crate) values: &'a Values,
    targets: HashSet<ValueSlot>,
    scalar_constants: HashMap<ValueSlot, Option<f32>>,
}
impl<'a> AnimationTargets<'a> {
    pub(crate) fn new(model: &'a Model<'_>) -> Self {
        let mut targets = HashSet::new();
        let mut scalar_constants = HashMap::<ValueSlot, Option<f32>>::new();
        for channel in model
            .timeline_channels()
            .iter()
            .chain(model.shared_channels())
            .chain(model.clips().iter().flat_map(|(_, clip)| clip.channels()))
            .chain(
                model
                    .controllers()
                    .iter()
                    .flat_map(|(_, controller)| &controller.channels),
            )
        {
            targets.insert(channel.target());
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
                    .entry(channel.target())
                    .and_modify(|previous| {
                        if previous.map(f32::to_bits) != constant.map(f32::to_bits) {
                            *previous = None;
                        }
                    })
                    .or_insert(constant);
            }
        }
        Self {
            values: model.values(),
            targets,
            scalar_constants,
        }
    }

    pub(crate) fn contains<T: Animatable>(&self, slot: &Animated<T>) -> bool {
        self.targets.contains(&slot.slot())
            || match slot.slot() {
                ValueSlot::Scalar(id) => self
                    .values
                    .boolean_source(id)
                    .is_some_and(|source| self.contains(&source)),
                _ => false,
            }
    }
    pub(crate) fn scalar_changes(&self, default: f32, slot: &Animated<f32>) -> bool {
        self.contains(slot)
            && self
                .scalar_constants
                .get(&slot.slot())
                .copied()
                .flatten()
                .is_none_or(|value| value.to_bits() != default.to_bits())
    }
    pub(crate) fn scalar(&self, slot: &Animated<f32>) -> bool {
        self.scalar_changes(*self.values.get(*slot).expect("validated slot"), slot)
    }
}
