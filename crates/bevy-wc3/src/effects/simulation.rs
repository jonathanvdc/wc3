use bytemuck::Pod;
use std::collections::VecDeque;
use std::sync::Arc;

use crate::effects::records::RecordRing;

pub(crate) fn split_time(time: f64) -> [f32; 2] {
    let high = time as f32;
    [high, (time - f64::from(high)) as f32]
}

pub(crate) struct EmitterRng(u64);
impl EmitterRng {
    pub(crate) fn new(object_id: u32) -> Self {
        Self(u64::from(object_id) + 1)
    }
    /// Independent packet for one scheduled occurrence. Packet zero uses the
    /// emitter seed; omitted births cannot shift later packets.
    pub(crate) fn at_occurrence(object_id: u32, ordinal: u64) -> Self {
        let base = u64::from(object_id) + 1;
        let mut mixed = ordinal.wrapping_mul(0x9e3779b97f4a7c15);
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d049bb133111eb);
        mixed ^= mixed >> 31;
        let seed = base ^ mixed;
        Self(if seed == 0 { base } else { seed })
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
    /// All integral authored occurrences, including those discarded by a cap.
    pub(crate) authored_count: f64,
    rate: f64,
    remainder: f64,
    dt: f64,
}
impl BirthSchedule {
    pub(crate) fn offset(&self, index: usize) -> f64 {
        ((index as f64 + 1.0 - self.remainder) / self.rate).clamp(0.0, self.dt)
    }
    #[cfg(test)]
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
        let (count, authored_count) = self.accumulate(rate * dt, limit, rounding);
        BirthSchedule {
            count,
            authored_count,
            rate,
            remainder,
            dt,
        }
    }
    pub(crate) fn burst(&mut self, amount: f64, limit: usize) -> usize {
        self.accumulate(amount, limit, EmissionRounding::Quad).0 as usize
    }
    fn accumulate(&mut self, amount: f64, limit: usize, rounding: EmissionRounding) -> (f64, f64) {
        let total = self.remainder + amount;
        if !total.is_finite() {
            self.reset();
            return (0.0, 0.0);
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
        (count, births)
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

#[cfg(test)]
#[path = "simulation_tests.rs"]
mod tests;
