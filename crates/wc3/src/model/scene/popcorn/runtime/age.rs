//! Fixed-instance immutable sample ownership and exact restoration boundaries.
use super::{EffectInstance, EffectLayerId};
use crate::model::scene::popcorn::age::MAX_OUTPUTS;
use crate::model::scene::popcorn::{
    ImmutableBirth, ImmutableParticleView, ImmutableRecipe, ImmutableSample, Value, ValueType,
};
use std::mem::size_of;

#[derive(Clone, Copy, Default)]
struct AuthorizedClock {
    effect_time: f64,
    integrated_delta: f64,
}

#[derive(Clone, Copy, Default)]
pub(super) struct AgeState {
    birth: ImmutableBirth,
    clock: AuthorizedClock,
    life: f32,
    generation: u32,
    valid: bool,
    evaluated: bool,
}
impl AgeState {
    fn sample(self) -> ImmutableSample {
        ImmutableSample {
            effect_time: self.clock.effect_time,
            integrated_delta: self.clock.integrated_delta,
            life: self.life,
        }
    }
    fn authorize(&mut self, sample: ImmutableSample) {
        self.clock = AuthorizedClock {
            effect_time: sample.effect_time,
            integrated_delta: sample.integrated_delta,
        };
        self.life = sample.life;
    }
}
pub(super) struct AgePool {
    slots: Vec<AgeState>,
    disabled: bool,
}
impl AgePool {
    pub(super) fn bytes(capacity: usize) -> Option<usize> {
        capacity
            .checked_mul(size_of::<AgeState>())
            .filter(|bytes| *bytes <= isize::MAX as usize)?
            .checked_add(size_of::<Self>())
    }
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            slots: vec![AgeState::default(); capacity],
            disabled: false,
        }
    }
    pub(super) fn reset(&mut self) {
        self.disabled = false;
        self.slots.fill(AgeState::default());
    }
    pub(super) fn clear_slot(&mut self, slot: usize) {
        self.slots[slot] = AgeState::default();
    }
}
impl EffectInstance {
    /// Whether this fixed instance still admits immutable evolution. Resource,
    /// quota or checked-guard fallback remains latched until the next start.
    pub fn immutable_active(&self) -> bool {
        self.age.as_ref().is_some_and(|pool| !pool.disabled)
    }
    /// Shared authorized clock only when every living eligible lane has evolved
    /// at this observation. A newborn or frozen sample returns None.
    pub fn immutable_time(&self) -> Option<f64> {
        let pool = self.age.as_ref().filter(|pool| !pool.disabled)?;
        for &slot in &self.active {
            if self.execution.age(self.particles[slot].layer).is_none() {
                continue;
            }
            let state = pool.slots[slot];
            if !state.valid || !state.evaluated || state.clock.effect_time != self.time {
                return None;
            }
        }
        Some(self.time)
    }
    /// Borrow this instance's fixed prepared recipe, unless runtime admission
    /// has fallen back. A same-graph wrapper cannot switch this choice.
    pub fn immutable_recipe(&self, layer: EffectLayerId) -> Option<ImmutableRecipe<'_>> {
        self.immutable_active().then_some(())?;
        Some(ImmutableRecipe {
            plan: self.execution.age_shared(layer)?,
            samplers: &self.definition.samplers,
        })
    }
    pub(super) fn immutable_view(&self, slot: usize) -> Option<ImmutableParticleView<'_>> {
        let pool = self.age.as_ref().filter(|pool| !pool.disabled)?;
        let state = pool.slots[slot];
        let particle = &self.particles[slot];
        if !state.valid || !state.evaluated || state.generation != particle.generation {
            return None;
        }
        Some(ImmutableParticleView {
            recipe: self.immutable_recipe(particle.layer)?,
            fields: &self.fields,
            slot,
            birth: state.birth,
            sample: state.sample(),
        })
    }
    pub(super) fn capture_age(&mut self, slot: usize) {
        if !self.immutable_active() {
            return;
        }
        let particle = &self.particles[slot];
        let Some(plan) = self.execution.age(particle.layer) else {
            return;
        };
        let layer = &self.definition.layers[particle.layer.0 as usize];
        let birth = ImmutableBirth {
            time: particle.birth as f32,
            life: f32::from_bits(self.fields.read(layer.life, slot)[0]),
            inverse_life: f32::from_bits(self.fields.read(layer.inverse_life, slot)[0]),
        };
        let captures_finite = plan.captures.iter().all(|capture| {
            let value = Value::from_words(capture.kind, self.fields.read(capture.field, slot));
            !matches!(
                capture.kind,
                ValueType::Float
                    | ValueType::Float2
                    | ValueType::Float3
                    | ValueType::Float4
                    | ValueType::Quaternion
            ) || value.words()[..capture.kind.components()]
                .iter()
                .all(|word| f32::from_bits(*word).is_finite())
        });
        let invariant_permitted = plan.captures_permitted(|field| {
            Value::from_words(
                layer.fields[field]
                    .value_type
                    .expect("prepared immutable capture type"),
                self.fields.read(field, slot),
            )
        });
        if !birth.time.is_finite()
            || birth.time < 0.
            || !birth.life.is_finite()
            || !(0. ..1.).contains(&birth.life)
            || !birth.inverse_life.is_finite()
            || birth.inverse_life < 0.
            || !captures_finite
            || !invariant_permitted
        {
            self.disable_age();
            return;
        }
        let sample = birth.sample(particle.birth);
        self.age.as_mut().expect("admitted age pool").slots[slot] = AgeState {
            birth,
            clock: AuthorizedClock {
                effect_time: sample.effect_time,
                integrated_delta: 0.,
            },
            life: sample.life,
            generation: particle.generation,
            valid: true,
            evaluated: false,
        };
    }
    /// Caller has already charged work and updated authoritative lifetime.
    pub(super) fn authorize_age(&mut self, slot: usize, time: f64, delta: f64) -> bool {
        if !self.immutable_active() {
            return false;
        }
        let particle = &self.particles[slot];
        let Some(plan) = self.execution.age(particle.layer) else {
            return false;
        };
        let state = self.age.as_ref().expect("age pool").slots[slot];
        let mut sample = state.birth.sample(time);
        if plan.integrated_delta {
            let step = delta as f32;
            if !step.is_finite() || step < 0. {
                self.disable_age();
                return false;
            }
            sample.integrated_delta = state.clock.integrated_delta + f64::from(step);
        }
        if !state.valid || state.generation != particle.generation || !plan.permitted(sample.into())
        {
            self.disable_age();
            return false;
        }
        let state = &mut self.age.as_mut().expect("age pool").slots[slot];
        state.authorize(sample);
        state.evaluated = true;
        self.scratch.clear();
        self.execution_stats.immutable_age_invocations = self
            .execution_stats
            .immutable_age_invocations
            .saturating_add(1);
        true
    }
    /// Restore complete old virtual outputs before any capture plane is changed.
    /// Earlier invocations are not replayed. Current source execution then owns
    /// its exact prefix/error, and all later work uses ordinary CPU until reset.
    pub(super) fn disable_age(&mut self) {
        if !self.immutable_active() {
            return;
        }
        let execution = self.execution.clone();
        for &slot in &self.active {
            let Some(view) = self.immutable_view(slot) else {
                continue;
            };
            let count = view.recipe().output_count();
            let mut values = [Value::from(0f32); MAX_OUTPUTS];
            view.evaluate(&mut values[..count]);
            let plan = execution
                .age(self.particles[slot].layer)
                .expect("admitted age recipe");
            for (output, value) in plan.outputs.iter().zip(values) {
                self.fields.write(output.field, slot, value.words());
            }
        }
        self.age.as_mut().expect("age pool").disabled = true;
        self.execution_stats.immutable_age_fallbacks = self
            .execution_stats
            .immutable_age_fallbacks
            .saturating_add(1);
    }
}

#[cfg(all(test, target_pointer_width = "64"))]
mod layout_tests {
    use super::*;

    #[test]
    fn moment_clock_reuses_previous_authorization_padding() {
        // Per-slot storage is part of admission's memory budget. Compare against
        // the previous semantic payload without depending on Rust field offsets.
        #[allow(dead_code)]
        struct PreviousSample {
            effect_time: f64,
            life: f32,
        }
        #[allow(dead_code)]
        struct PreviousState {
            birth: ImmutableBirth,
            sample: PreviousSample,
            generation: u32,
            valid: bool,
            evaluated: bool,
        }
        assert!(size_of::<AgeState>() <= size_of::<PreviousState>());
    }
}
