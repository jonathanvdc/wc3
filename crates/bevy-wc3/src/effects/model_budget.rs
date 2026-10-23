//! Shared limits for ordinary actor trees created by model particles.
use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Optional limits on the outer application model root. Pending and ready
/// model-particle actors and their rigid parts and automatic attachments share
/// these limits. Preexisting application-owned assembly parts are not charged.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelParticleLimits {
    /// Pending plus ready rigs across this tree, including static descendants.
    pub actors_per_tree: u32,
    /// New occurrence inspection, descendant admission, and later ready attempts.
    /// Existing ready actor poses do not spend this allowance.
    pub birth_work_per_frame: u32,
    /// Examined fixed intervals, programme steps, and keyed-burst keys or
    /// occurrences shared by this tree's effect sources.
    pub emission_steps_per_frame: u32,
}
impl Default for ModelParticleLimits {
    fn default() -> Self {
        Self {
            actors_per_tree: 4096,
            birth_work_per_frame: 512,
            emission_steps_per_frame: 16384,
        }
    }
}
/// Observable tree accounting. Work uses the actual app frame; omission and
/// skipped-source-time totals are cumulative for this tree generation.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct ModelParticleTreeStats {
    /// Pending plus ready ordinary rigs, including static descendants.
    pub actors: usize,
    /// Accepted actors still waiting for population.
    pub pending: usize,
    /// Admission work used in the current app frame.
    pub birth_work: u32,
    /// Source intervals examined in the current app frame.
    pub emission_steps: u32,
    /// Requests omitted by the tree's resident ceiling.
    pub omitted_actor_limit: u64,
    /// Known requests omitted before further admission work.
    pub omitted_birth_work: u64,
    /// Sum across emitter sources, rather than elapsed scene wall time.
    pub skipped_emission_seconds: f64,
}
impl ModelParticleTreeStats {
    /// Total omitted actor requests, counting each exclusive reason once.
    pub fn omitted_actor_requests(&self) -> u64 {
        self.omitted_actor_limit
            .saturating_add(self.omitted_birth_work)
    }
    /// Populated accepted actors.
    pub fn ready(&self) -> usize {
        self.actors.saturating_sub(self.pending)
    }
}
#[derive(Default, Debug)]
struct FrameQuota(AtomicU64);
impl FrameQuota {
    fn used(&self, frame: FrameCount) -> u32 {
        let state = self.0.load(Ordering::Relaxed);
        if (state >> 32) as u32 == frame.0 {
            state as u32
        } else {
            0
        }
    }
    fn charge(&self, frame: FrameCount, limit: u32) -> bool {
        let mut state = self.0.load(Ordering::Relaxed);
        loop {
            let used = if (state >> 32) as u32 == frame.0 {
                state as u32
            } else {
                0
            };
            if used >= limit {
                return false;
            }
            let next = (u64::from(frame.0) << 32) | u64::from(used + 1);
            match self
                .0
                .compare_exchange_weak(state, next, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => return true,
                Err(current) => state = current,
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActorBudgetError {
    Actors,
    BirthWork,
}
#[derive(Debug)]
pub(crate) struct ActorBudget {
    root: Entity,
    actor_limit: AtomicU32,
    birth_limit: AtomicU32,
    step_limit: AtomicU32,
    actors: AtomicUsize,
    pending: AtomicUsize,
    births: FrameQuota,
    steps: FrameQuota,
    omitted_actors: AtomicU64,
    omitted_work: AtomicU64,
    skipped_seconds: AtomicU64,
}
fn add(counter: &AtomicU64, amount: u64) {
    #[allow(
        deprecated,
        reason = "Atomic::try_update is newer than the Rust 1.97 minimum"
    )]
    let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
        Some(old.saturating_add(amount))
    });
}
impl ActorBudget {
    pub fn new(root: Entity, limits: ModelParticleLimits) -> Arc<Self> {
        Arc::new(Self {
            root,
            actor_limit: AtomicU32::new(limits.actors_per_tree),
            birth_limit: AtomicU32::new(limits.birth_work_per_frame),
            step_limit: AtomicU32::new(limits.emission_steps_per_frame),
            actors: AtomicUsize::new(0),
            pending: AtomicUsize::new(0),
            births: Default::default(),
            steps: Default::default(),
            omitted_actors: AtomicU64::new(0),
            omitted_work: AtomicU64::new(0),
            skipped_seconds: AtomicU64::new(0),
        })
    }
    /// Configuration changes happen once on the main thread before the tree step.
    pub fn set_limits(&self, limits: ModelParticleLimits) {
        self.actor_limit
            .store(limits.actors_per_tree, Ordering::Relaxed);
        self.birth_limit
            .store(limits.birth_work_per_frame, Ordering::Relaxed);
        self.step_limit
            .store(limits.emission_steps_per_frame, Ordering::Relaxed);
    }
    pub fn root(&self) -> Entity {
        self.root
    }
    pub fn remaining_births(&self, frame: FrameCount) -> usize {
        self.birth_limit
            .load(Ordering::Relaxed)
            .saturating_sub(self.births.used(frame)) as usize
    }
    pub fn step(&self, frame: FrameCount) -> bool {
        self.steps
            .charge(frame, self.step_limit.load(Ordering::Relaxed))
    }
    pub fn skip_seconds(&self, seconds: f64) {
        if !seconds.is_finite() || seconds <= 0. {
            return;
        }
        #[allow(
            deprecated,
            reason = "Atomic::try_update is newer than the Rust 1.97 minimum"
        )]
        let _ = self
            .skipped_seconds
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
                Some((f64::from_bits(old) + seconds).min(f64::MAX).to_bits())
            });
    }
    pub fn omit(&self, reason: ActorBudgetError, count: u64) {
        match reason {
            ActorBudgetError::Actors => add(&self.omitted_actors, count),
            ActorBudgetError::BirthWork => self.omit_work(count),
        }
    }
    pub fn omit_work(&self, count: u64) {
        add(&self.omitted_work, count);
    }
    pub fn reserve(
        self: &Arc<Self>,
        frame: FrameCount,
    ) -> Result<ModelActorLease, ActorBudgetError> {
        if !self
            .births
            .charge(frame, self.birth_limit.load(Ordering::Relaxed))
        {
            self.omit_work(1);
            return Err(ActorBudgetError::BirthWork);
        }
        let limit = self.actor_limit.load(Ordering::Relaxed) as usize;
        #[allow(
            deprecated,
            reason = "Atomic::try_update is newer than the Rust 1.97 minimum"
        )]
        let result = self
            .actors
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                if value < limit {
                    Some(value + 1)
                } else {
                    None
                }
            });
        if result.is_err() {
            add(&self.omitted_actors, 1);
            return Err(ActorBudgetError::Actors);
        }
        self.pending.fetch_add(1, Ordering::Relaxed);
        Ok(ModelActorLease {
            budget: self.clone(),
            frame: frame.0,
            pending: true,
        })
    }
    pub fn stats(&self, frame: FrameCount) -> ModelParticleTreeStats {
        ModelParticleTreeStats {
            actors: self.actors.load(Ordering::Relaxed),
            pending: self.pending.load(Ordering::Relaxed),
            birth_work: self.births.used(frame),
            emission_steps: self.steps.used(frame),
            omitted_actor_limit: self.omitted_actors.load(Ordering::Relaxed),
            omitted_birth_work: self.omitted_work.load(Ordering::Relaxed),
            skipped_emission_seconds: f64::from_bits(self.skipped_seconds.load(Ordering::Relaxed)),
        }
    }
}
#[derive(Component, Debug)]
pub(crate) struct ModelActorLease {
    budget: Arc<ActorBudget>,
    frame: u32,
    pending: bool,
}
impl ModelActorLease {
    pub fn is_current(&self, tree: Option<&ModelActorTree>) -> bool {
        tree.is_some_and(|tree| Arc::ptr_eq(&tree.budget, &self.budget))
    }
    pub fn budget(&self) -> &Arc<ActorBudget> {
        &self.budget
    }
    /// Retried readiness spends work, but deferred accepted actors are not omissions.
    pub fn admit_ready(&mut self, frame: FrameCount) -> bool {
        if self.frame == frame.0 {
            return true;
        }
        if !self
            .budget
            .births
            .charge(frame, self.budget.birth_limit.load(Ordering::Relaxed))
        {
            return false;
        }
        self.frame = frame.0;
        true
    }
    pub fn ready(&mut self) {
        if self.pending {
            self.pending = false;
            self.budget.pending.fetch_sub(1, Ordering::Relaxed);
        }
    }
    pub fn is_pending(&self) -> bool {
        self.pending
    }
}
impl Drop for ModelActorLease {
    fn drop(&mut self) {
        self.budget.actors.fetch_sub(1, Ordering::Relaxed);
        if self.pending {
            self.budget.pending.fetch_sub(1, Ordering::Relaxed);
        }
    }
}
#[derive(Component)]
pub(crate) struct ModelActorTree {
    pub budget: Arc<ActorBudget>,
    pub pending: VecDeque<Entity>,
}
#[derive(Clone)]
pub(crate) struct ActorFrame {
    pub budget: Arc<ActorBudget>,
    pub frame: FrameCount,
    pub owned: bool,
}
impl ActorFrame {
    pub fn reserve(&self) -> Result<ModelActorLease, ActorBudgetError> {
        self.budget.reserve(self.frame)
    }
    pub fn remaining(&self) -> usize {
        self.budget.remaining_births(self.frame)
    }
    pub fn step(&self) -> bool {
        self.budget.step(self.frame)
    }
}

#[cfg(test)]
#[path = "model_budget_tests.rs"]
mod tests;
