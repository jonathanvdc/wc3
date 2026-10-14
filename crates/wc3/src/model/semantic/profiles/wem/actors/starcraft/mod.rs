//! Independently authored StarCraft behavior snapshots and their actual links.
mod values;
pub use values::*;
mod particle;
pub use particle::Particle;
mod ribbon;
pub use ribbon::Ribbon;
mod spline;
pub use spline::RibbonPoint;
mod links;
use crate::model::semantic::{CurveValue, FeatureId, Link, Property, ValidateLinks};
/// Baseline and reset values independent of animation-channel initialization.
#[derive(Clone, Debug, Default)]
pub struct Control<T> {
    /// Value before a channel supplies an override.
    pub initial: T,
    /// Value selected when the effect property resets.
    pub reset: T,
    /// Actual independently authored animation destination when declared.
    pub binding: Option<Property<T>>,
}
impl<T: CurveValue + 'static> ValidateLinks for Control<T> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.binding.visit_links(visitor);
        if !self.initial.is_valid() || !self.reset.is_valid() {
            visitor(Link::InvalidValue);
        }
    }
}
/// Variation function applied to an emitter parameter.
#[derive(Clone, Debug, Default)]
pub struct Variation {
    /// Variation behavior.
    pub kind: VariationKind,
    /// Variation magnitude.
    pub amplitude: Control<f32>,
    /// Variation frequency.
    pub frequency: Control<f32>,
}
/// Another emission point whose inherited controls belong only to its source.
#[derive(Clone, Debug)]
pub struct ParticleCopy {
    /// Actual primary particle behavior supplying shared controls.
    pub source: FeatureId,
    /// Emission rate at this emission point.
    pub emission_rate: Control<f32>,
    /// Burst population at this emission point.
    pub squirt_amount: Control<u32>,
}
/// Complete StarCraft attached authored behavior.
#[derive(Clone, Debug)]
pub enum Feature<'a> {
    /// Primary particle system.
    Particle(Box<Particle<'a>>),
    /// Additional emission point of one particle system.
    ParticleCopy(Box<ParticleCopy>),
    /// Ribbon geometry and motion controls.
    Ribbon(Box<Ribbon<'a>>),
}
