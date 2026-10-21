//! M2 sampled-rate squirts, independent of keyed MDLX emission.
//! The host observes once per advancing actor frame and places requested births
//! at that frame's endpoint. Paused/render-only frames never arm or consume work.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EmissionMode {
    Continuous,
    Squirt,
}

use super::birth::BirthPlan;

const THRESHOLD: f32 = 0.02;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SquirtUpdate {
    Advance,
    Hold,
    /// Rebase the destination sample without replaying the skipped cursor jump.
    Seek,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SquirtError {
    NonFiniteInput,
    InvalidJitter,
    CountOverflow,
}

/// One checked actor-frame sample, including the ordinary Bevy birth gate.
#[derive(Clone, Copy, Debug)]
pub(super) struct SquirtSample {
    rate: f32,
    allowed: bool,
}
impl SquirtSample {
    pub(super) fn inactive() -> Self {
        Self {
            rate: 0.,
            allowed: false,
        }
    }

    pub(super) fn new(rate: f32, allowed: bool) -> Result<Self, SquirtError> {
        if !rate.is_finite() {
            return Err(SquirtError::NonFiniteInput);
        }
        Ok(Self {
            rate: if allowed { rate.max(0.) } else { 0. },
            allowed,
        })
    }
}

pub(super) struct SquirtState {
    previous_rate: Option<f32>,
    startup_pending: bool,
    burst_ordinal: u64,
}
impl Default for SquirtState {
    fn default() -> Self {
        Self {
            previous_rate: None,
            startup_pending: true,
            burst_ordinal: 0,
        }
    }
}
impl SquirtState {
    #[cfg(test)]
    pub(super) fn packets(&self) -> u64 {
        self.burst_ordinal
    }

    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    /// Observe exactly once per actor update, never once per integration quantum.
    /// A returned request is consumed this update even if admission is denied.
    pub(super) fn observe(
        &mut self,
        sample: SquirtSample,
        update: SquirtUpdate,
    ) -> Option<SquirtBurst> {
        match update {
            SquirtUpdate::Hold => return None,
            SquirtUpdate::Seek => {
                self.previous_rate = Some(sample.rate);
                self.startup_pending = false;
                return None;
            }
            SquirtUpdate::Advance => {}
        }
        let armed = self.startup_pending
            || self
                .previous_rate
                .is_some_and(|previous| previous <= THRESHOLD && sample.rate > THRESHOLD);
        self.previous_rate = Some(sample.rate);
        self.startup_pending = false;
        // Disabled startup cannot jitter into births. Unlike a disabled sample,
        // an enabled authored zero rate may still have positive additive jitter.
        if !armed || !sample.allowed {
            return None;
        }
        let ordinal = self.burst_ordinal;
        self.burst_ordinal = self.burst_ordinal.wrapping_add(1);
        Some(SquirtBurst {
            rate: sample.rate,
            ordinal,
        })
    }
}

/// An update-local request. Planning consumes it; no residual count is retained.
#[derive(Debug)]
pub(super) struct SquirtBurst {
    rate: f32,
    pub ordinal: u64,
}
impl SquirtBurst {
    /// Native scalar count convention: additive signed rate variation,
    /// truncate toward zero. The caller owns RNG and advances it exactly once.
    /// Distance/global scaling is outside this interpreter.
    pub(super) fn plan(
        self,
        variation: f32,
        jitter: f32,
        limit: usize,
    ) -> Result<BirthPlan, SquirtError> {
        if !variation.is_finite() || !jitter.is_finite() {
            return Err(SquirtError::NonFiniteInput);
        }
        if !(-1. ..1.).contains(&jitter) {
            return Err(SquirtError::InvalidJitter);
        }
        let amount = self.rate + jitter * variation;
        if !amount.is_finite() {
            return Err(SquirtError::NonFiniteInput);
        }
        let integral = f64::from(amount.max(0.).trunc());
        if integral >= u64::MAX as f64 {
            return Err(SquirtError::CountOverflow);
        }
        let requested = integral as u64;
        Ok(BirthPlan::new(requested, limit))
    }
}

#[cfg(test)]
#[path = "squirt_tests.rs"]
mod tests;
