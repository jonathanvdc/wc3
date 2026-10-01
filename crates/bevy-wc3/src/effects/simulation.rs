use bevy::math::Affine3A;
use bevy::prelude::*;
use bytemuck::Pod;
use std::collections::VecDeque;
use std::sync::Arc;

use crate::animation::Wc3Animation;
use crate::effect_ring::RecordRing;

pub(crate) fn simulation_delta(time: &Time, animation: &Wc3Animation) -> f64 {
    let dt = time.delta_secs_f64() * animation.speed;
    if animation.playing && dt.is_finite() {
        dt.max(0.0)
    } else {
        0.0
    }
}

#[derive(Debug)]
pub(crate) struct SimulationClock {
    pub(crate) time: f64,
    last_sequence: usize,
    last_elapsed_ms: f64,
}

impl Default for SimulationClock {
    fn default() -> Self {
        Self {
            time: 0.0,
            last_sequence: usize::MAX,
            last_elapsed_ms: 0.0,
        }
    }
}

impl SimulationClock {
    /// Report discontinuities without deciding what happens to existing objects.
    pub(crate) fn observe(&mut self, animation: &Wc3Animation) -> bool {
        let reset =
            self.last_sequence != animation.sequence || animation.elapsed_ms < self.last_elapsed_ms;
        self.last_sequence = animation.sequence;
        self.last_elapsed_ms = animation.elapsed_ms;
        reset
    }

    pub(crate) fn advance(&mut self, dt: f64) {
        if dt.is_finite() && dt > 0.0 {
            self.time += dt;
        }
    }
}

pub(crate) fn sequence_ended(animation: &Wc3Animation) -> bool {
    animation
        .sequences
        .get(animation.sequence)
        .is_some_and(|sequence| {
            sequence.flags.non_looping()
                && animation.elapsed_ms
                    >= f64::from(sequence.interval[1].saturating_sub(sequence.interval[0]))
        })
}

pub(crate) fn birth_animation(animation: &Wc3Animation, age: f64) -> Wc3Animation {
    let mut birth = animation.clone();
    birth.elapsed_ms -= age * 1000.0;
    birth
}

/// Integer-millisecond tracks need stable sampling at exact key boundaries.
pub(crate) fn sample_clock(mut animation: Wc3Animation) -> Wc3Animation {
    let rounded = animation.elapsed_ms.round();
    if (animation.elapsed_ms - rounded).abs() < 1e-6 {
        animation.elapsed_ms = rounded;
    }
    animation
}

pub(crate) fn split_time(time: f64) -> [f32; 2] {
    let high = time as f32;
    [high, (time - f64::from(high)) as f32]
}

pub(crate) struct EmitterRng(u64);
impl EmitterRng {
    pub(crate) fn new(object_id: u32) -> Self {
        Self(u64::from(object_id) + 1)
    }
    pub(crate) fn random(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u32 << 24) as f32
    }
    pub(crate) fn signed(&mut self) -> f32 {
        self.random() * 2.0 - 1.0
    }
}

/// Preserve each format's existing boundary and overflow behavior explicitly.
#[derive(Clone, Copy)]
pub(crate) enum EmissionRounding {
    /// PREM discards births beyond capacity, retaining only the fractional phase.
    Model,
    /// PRE2 subtracts the bounded count and retains up to one deferred birth.
    Quad,
    /// Ribbons keep an unbounded schedule and select the most recent live births.
    Ribbon,
}

#[derive(Default)]
pub(crate) struct EmissionPhase {
    remainder: f64,
}

pub(crate) struct BirthSchedule {
    pub(crate) count: f64,
    rate: f64,
    remainder: f64,
    dt: f64,
}
impl BirthSchedule {
    pub(crate) fn offset(&self, index: usize) -> f64 {
        ((index as f64 + 1.0 - self.remainder) / self.rate).clamp(0.0, self.dt)
    }
    pub(crate) fn age(&self, index: usize) -> f64 {
        self.dt - self.offset(index)
    }

    pub(crate) fn first_birth(&self, start: f64) -> f64 {
        start + (1.0 - self.remainder) / self.rate
    }
    pub(crate) fn birth(&self, first: f64, index: f64) -> f64 {
        first + index / self.rate
    }
    /// Skip dead births and keep the most recent capacity births for trails.
    pub(crate) fn skip(&self, first: f64, end: f64, lifetime: f64, capacity: usize) -> f64 {
        let dead = ((end - lifetime - first) * self.rate).floor() + 1.0;
        dead.max(0.0)
            .max(self.count - capacity as f64)
            .min(self.count)
    }
}
impl EmissionPhase {
    pub(crate) fn reset(&mut self) {
        self.remainder = 0.0;
    }
    pub(crate) fn continuous(
        &mut self,
        rate: f64,
        dt: f64,
        limit: usize,
        rounding: EmissionRounding,
    ) -> BirthSchedule {
        let remainder = self.remainder;
        let count = self.accumulate(rate * dt, limit, rounding);
        BirthSchedule {
            count,
            rate,
            remainder,
            dt,
        }
    }
    pub(crate) fn burst(&mut self, amount: f64, limit: usize) -> usize {
        self.accumulate(amount, limit, EmissionRounding::Quad) as usize
    }
    fn accumulate(&mut self, amount: f64, limit: usize, rounding: EmissionRounding) -> f64 {
        let total = self.remainder + amount;
        if !total.is_finite() {
            self.reset();
            return 0.0;
        }
        let tolerance = match rounding {
            EmissionRounding::Ribbon => 1e-9,
            _ => 16.0 * f64::EPSILON * total.max(1.0),
        };
        let births = (total + tolerance).floor();
        let count = match rounding {
            EmissionRounding::Ribbon => births,
            _ => (births as usize).min(limit) as f64,
        };
        self.remainder = match rounding {
            EmissionRounding::Model => (total - births).clamp(0.0, 1.0),
            EmissionRounding::Quad => {
                let remainder = (total - count).clamp(0.0, 1.0);
                if remainder < tolerance {
                    0.0
                } else {
                    remainder
                }
            }
            EmissionRounding::Ribbon => (total - births).max(0.0),
        };
        count
    }
}

/// Keep chronological CPU metadata and immutable GPU records in lockstep.
/// Expiry predicates and capacity policy belong to the caller.
pub(crate) struct LiveRecords<R, M> {
    pub(crate) records: Arc<RecordRing<R>>,
    metadata: VecDeque<M>,
}
impl<R, M> Default for LiveRecords<R, M> {
    fn default() -> Self {
        Self {
            records: Arc::default(),
            metadata: VecDeque::new(),
        }
    }
}
impl<R: Pod, M> LiveRecords<R, M> {
    pub(crate) fn metadata(&self) -> &VecDeque<M> {
        &self.metadata
    }
    pub(crate) fn len(&self) -> usize {
        self.metadata.len()
    }
    pub(crate) fn pop_front(&mut self) {
        self.metadata.pop_front().expect("live record metadata");
        Arc::make_mut(&mut self.records).pop_front();
    }
    pub(crate) fn retire(&mut self, mut expired: impl FnMut(&M) -> bool) {
        while self.metadata.front().is_some_and(&mut expired) {
            self.pop_front();
        }
    }
    pub(crate) fn push(&mut self, record: R, metadata: M, limit: usize) {
        Arc::make_mut(&mut self.records).push(record, limit);
        self.metadata.push_back(metadata);
    }
}

/// A sampled local pose continues up the hierarchy; a world pose ends it.
pub(crate) enum SampledPose {
    Local(GlobalTransform, Option<Entity>),
    World(GlobalTransform),
}

pub(crate) fn compose_emitter_transform(
    mut entity: Entity,
    mut lookup: impl FnMut(Entity) -> Option<SampledPose>,
) -> Option<GlobalTransform> {
    let mut local = Affine3A::IDENTITY;
    loop {
        match lookup(entity)? {
            SampledPose::World(world) => {
                return Some(GlobalTransform::from(Mat4::from(world.affine() * local)))
            }
            SampledPose::Local(pose, parent) => {
                local = pose.affine() * local;
                let Some(parent) = parent else {
                    return Some(GlobalTransform::from(Mat4::from(local)));
                };
                entity = parent;
            }
        }
    }
}

#[cfg(test)]
#[path = "simulation_tests.rs"]
mod tests;
