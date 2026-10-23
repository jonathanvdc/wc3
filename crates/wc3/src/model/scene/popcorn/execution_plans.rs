//! Weak execution-plan reuse belonging to one immutable graph, never a registry.
use super::EffectGraph;
use crate::model::scene::popcorn::execution_plan::ExecutionPlan;
use crate::model::scene::popcorn::ExecutionOptions;
use std::sync::{Arc, Mutex, Weak};

#[derive(Debug)]
pub(super) struct ExecutionPlans {
    entries: Mutex<Vec<(ExecutionOptions, Weak<ExecutionPlan>)>>,
}
impl ExecutionPlans {
    pub(super) fn new(initial: &Arc<ExecutionPlan>) -> Self {
        Self {
            entries: Mutex::new(vec![(initial.options(), Arc::downgrade(initial))]),
        }
    }
    pub(super) fn prepare(
        &self,
        graph: &EffectGraph,
        options: ExecutionOptions,
    ) -> Arc<ExecutionPlan> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        entries.retain(|(_, plan)| plan.strong_count() != 0);
        if let Some(plan) = entries
            .iter()
            .find_map(|(key, plan)| (*key == options).then(|| plan.upgrade()).flatten())
        {
            return plan;
        }
        // Preparation has no application callbacks. Serialize this cold work to
        // avoid duplicate executable modules for concurrent identical requests.
        // A panic before insertion leaves entries valid; a later caller can retry.
        let plan = Arc::new(ExecutionPlan::prepare(graph, options));
        entries.push((options, Arc::downgrade(&plan)));
        plan
    }
}

#[cfg(test)]
#[path = "execution_plans_tests.rs"]
mod tests;
