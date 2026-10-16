//! Bounded numerical integration on an absolute emitter clock.
//!
//! Rates and activation are sampled at the midpoint of each 1/60-second quantum.
//! Partial updates share the same sample point, so splitting an update does not
//! alter integrated rates. This approximates animated curves; it does not claim
//! analytic integration at key or visibility discontinuities. A long update
//! visits at most 4096 quanta and explicitly reports its omitted tail.
use super::super::events::EventTraversal;
use wc3::model::ClipId;

const QUANTUM: f64 = 1. / 60.;
const MAX_STEPS: usize = 4096;

pub(super) struct EmissionStep {
    pub start: f64,
    pub end: f64,
    pub tick: i64,
    pub sample_time: f64,
}
pub(super) struct EmissionClock {
    next: f64,
    end: f64,
    steps: usize,
}
impl EmissionClock {
    pub fn new(start: f64, end: f64) -> Self {
        Self {
            next: start,
            end,
            steps: 0,
        }
    }
    pub fn omitted_seconds(&self) -> f64 {
        (self.end - self.next).max(0.)
    }
}
impl Iterator for EmissionClock {
    type Item = EmissionStep;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.next.is_finite()
            || !self.end.is_finite()
            || self.next >= self.end
            || self.steps >= MAX_STEPS
        {
            return None;
        }
        let tick = (self.next / QUANTUM + 1e-9).floor();
        let end = self.end.min((tick + 1.) * QUANTUM);
        if end <= self.next {
            return None;
        }
        let step = EmissionStep {
            start: self.next,
            end,
            tick: tick as i64,
            sample_time: (tick + 0.5) * QUANTUM,
        };
        self.next = end;
        self.steps += 1;
        Some(step)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::simulation::{EmissionPhase, EmissionRounding};

    fn births(parts: &[f64], rounding: EmissionRounding) -> Vec<f64> {
        let mut phase = EmissionPhase::default();
        let mut result = Vec::new();
        let mut start = 0.;
        for &end in parts {
            for step in EmissionClock::new(start, end) {
                let rate = 90. + 40. * (step.sample_time * 3.).sin();
                let schedule = phase.continuous(rate, step.end - step.start, 8192, rounding);
                for index in 0..schedule.count as usize {
                    result.push(step.start + schedule.offset(index));
                }
            }
            start = end;
        }
        result
    }
    #[test]
    fn updates_share_rate_samples_and_birth_times() {
        for rounding in [
            EmissionRounding::Model,
            EmissionRounding::Quad,
            EmissionRounding::Ribbon,
        ] {
            let whole = births(&[2.], rounding);
            let split = births(&[0.003, 0.017, 0.101, 0.7, 1.001, 2.], rounding);
            assert_eq!(whole.len(), split.len());
            for (a, b) in whole.iter().zip(split) {
                assert!((a - b).abs() < 1e-10, "{a} != {b}");
            }
        }
    }
    #[test]
    fn long_updates_have_a_finite_explicit_budget() {
        let mut clock = EmissionClock::new(0., 3600.);
        assert_eq!(clock.by_ref().count(), MAX_STEPS);
        assert!(clock.omitted_seconds() > 3500.);
        assert!(clock.next().is_none());
    }
}

/// Explicit separation between authored clock jumps and visual simulation time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum EffectChange {
    #[default]
    Continuous,
    Seek,
    Reset,
}
#[derive(Default)]
pub(crate) struct EffectClock {
    initialized: bool,
    raw: f64,
    time: f64,
    revision: u64,
    clip: Option<ClipId>,
}
impl EffectClock {
    pub fn project(
        &self,
        raw: f64,
        clip: Option<ClipId>,
        traversal: EventTraversal,
    ) -> (f64, EffectChange) {
        if !self.initialized {
            return (0., EffectChange::Continuous);
        }
        let revised = self.revision != traversal.revision;
        let reset = self.clip != clip || raw < self.raw || (revised && traversal.include_origin);
        if reset {
            return (0., EffectChange::Reset);
        }
        if revised {
            (
                self.time + (raw - traversal.origin.shared).max(0.),
                EffectChange::Seek,
            )
        } else {
            (
                self.time + (raw - self.raw).max(0.),
                EffectChange::Continuous,
            )
        }
    }
    pub fn observe(
        &mut self,
        raw: f64,
        clip: Option<ClipId>,
        traversal: EventTraversal,
    ) -> (f64, EffectChange) {
        let result = self.project(raw, clip, traversal);
        self.initialized = true;
        self.raw = raw;
        self.time = result.0;
        self.revision = traversal.revision;
        self.clip = clip;
        result
    }
}

#[cfg(test)]
mod discontinuity_tests {
    use super::*;
    use crate::universal::events::EventClocks;
    #[test]
    fn explicit_forward_seek_skips_time_but_subsequent_forward_motion_advances() {
        let mut clock = EffectClock::default();
        let initial = EventTraversal::default();
        assert_eq!(
            clock.observe(0., None, initial),
            (0., EffectChange::Continuous)
        );
        assert_eq!(
            clock.observe(1., None, initial),
            (1., EffectChange::Continuous)
        );
        let seek = EventTraversal {
            revision: 1,
            origin: EventClocks {
                clip: 10.,
                timeline: 10.,
                shared: 10.,
            },
            include_origin: false,
        };
        assert_eq!(clock.project(10.25, None, seek), (1.25, EffectChange::Seek));
        assert_eq!(clock.observe(10.25, None, seek), (1.25, EffectChange::Seek));
        assert_eq!(
            clock.observe(10.5, None, seek),
            (1.5, EffectChange::Continuous)
        );
    }
    #[test]
    fn raw_forward_edits_remain_traversal_while_reverse_and_restart_reset() {
        let mut clock = EffectClock::default();
        let initial = EventTraversal::default();
        clock.observe(0., None, initial);
        assert_eq!(
            clock.observe(30., None, initial),
            (30., EffectChange::Continuous)
        );
        assert_eq!(clock.observe(5., None, initial), (0., EffectChange::Reset));
        clock.observe(6., None, initial);
        let restart = EventTraversal {
            revision: 1,
            ..initial
        };
        assert_eq!(clock.observe(0., None, restart), (0., EffectChange::Reset));
    }
}
