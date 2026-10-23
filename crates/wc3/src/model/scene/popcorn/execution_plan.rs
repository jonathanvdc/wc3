//! Immutable executable selection, shared by instances but independent of schema identity.
use super::age::AgePlan;
use super::callbacks::ExecutionHost;
use super::execution::with_scratch;
use super::graph::{EffectGraph, EffectLayerId};
use super::native::NativeCode;
use super::packet::{PureEvolution, Scratch as PacketScratch};
use super::ExecutionFallback;
use super::NumericPolicy;
use super::{
    ExecutionBackend, ExecutionError, ExecutionOptions, ExecutionPreference, ExecutionReport,
    ExecutionStage, ExecutionStats, Program, State,
};
use super::{SpecializationReason, SpecializationRejection, SpecializationStatus};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct ExecutionPlan {
    options: ExecutionOptions,
    reports: Box<[ExecutionReport]>,
    native: Option<NativeCode>,
    ages: Box<[Option<Arc<AgePlan>>]>,
    immutable_statuses: Box<[SpecializationStatus]>,
}
impl ExecutionPlan {
    pub(super) fn prepare(graph: &EffectGraph, options: ExecutionOptions) -> Self {
        let mut ages = Vec::new();
        let mut immutable_statuses = Vec::new();
        if options.preference == ExecutionPreference::ImmutableAge
            && options.numeric_policy == NumericPolicy::Graphics
        {
            ages.reserve_exact(graph.layers.len());
            immutable_statuses.reserve_exact(graph.layers.len());
            for layer in &graph.layers {
                if layer.evolve.is_none() {
                    ages.push(None);
                    immutable_statuses.push(SpecializationStatus::NotApplicable);
                    continue;
                }
                match AgePlan::analyze(layer, &graph.samplers) {
                    Ok(plan) => {
                        ages.push(Some(Arc::new(plan)));
                        immutable_statuses.push(SpecializationStatus::Accepted);
                    }
                    Err(reason) => {
                        ages.push(None);
                        immutable_statuses.push(SpecializationStatus::Rejected(reason));
                    }
                }
            }
        }
        let mut native = None;
        let fallback = if options.preference == ExecutionPreference::Interpreter {
            None
        } else {
            match catch_unwind(AssertUnwindSafe(|| {
                NativeCode::prepare(
                    graph,
                    matches!(
                        options.preference,
                        ExecutionPreference::Automatic | ExecutionPreference::ImmutableAge
                    ),
                )
            })) {
                Ok(Ok(code)) => {
                    native = Some(code);
                    None
                }
                Ok(Err(reason)) => Some(reason),
                Err(_) => Some(ExecutionFallback::CompilationFailed {
                    message: "native preparation panicked before execution".into(),
                }),
            }
        };
        let backend = if native.is_some() {
            ExecutionBackend::ScalarJit
        } else {
            ExecutionBackend::Interpreter
        };
        let reports = graph
            .layers
            .iter()
            .enumerate()
            .flat_map(|(index, layer)| {
                let evolution_backend = if ages.get(index).is_some_and(Option::is_some) {
                    ExecutionBackend::ImmutableAge
                } else if native.as_ref().is_some_and(|code| code.has_simd(index)) {
                    ExecutionBackend::SimdJit
                } else {
                    backend
                };
                [
                    Some(ExecutionStage::Initialization),
                    layer.evolve.as_ref().map(|_| ExecutionStage::Evolution),
                ]
                .into_iter()
                .flatten()
                .map({
                    let fallback = fallback.clone();
                    move |stage| ExecutionReport {
                        layer: EffectLayerId(index as u32),
                        stage,
                        backend: if stage == ExecutionStage::Evolution {
                            evolution_backend
                        } else {
                            backend
                        },
                        fallback: if stage == ExecutionStage::Evolution
                            && evolution_backend == ExecutionBackend::ImmutableAge
                        {
                            None
                        } else if stage == ExecutionStage::Evolution
                            && options.preference == ExecutionPreference::ImmutableAge
                            && options.numeric_policy == NumericPolicy::Strict
                        {
                            Some(ExecutionFallback::NumericPolicy)
                        } else {
                            fallback.clone()
                        },
                    }
                })
            })
            .collect();
        Self {
            options,
            reports,
            native,
            ages: ages.into_boxed_slice(),
            immutable_statuses: immutable_statuses.into_boxed_slice(),
        }
    }
    pub(super) fn immutable_status(&self, layer: EffectLayerId) -> SpecializationStatus {
        if self.options.preference != ExecutionPreference::ImmutableAge {
            return SpecializationStatus::NotExamined;
        }
        if self.options.numeric_policy == NumericPolicy::Strict {
            return SpecializationStatus::Rejected(SpecializationRejection::new(
                SpecializationReason::NumericPolicy,
            ));
        }
        self.immutable_statuses
            .get(layer.0 as usize)
            .copied()
            .unwrap_or(SpecializationStatus::NotExamined)
    }
    pub(super) fn age(&self, layer: EffectLayerId) -> Option<&AgePlan> {
        self.age_shared(layer).map(Arc::as_ref)
    }
    pub(super) fn age_shared(&self, layer: EffectLayerId) -> Option<&Arc<AgePlan>> {
        self.ages.get(layer.0 as usize)?.as_ref()
    }
    pub(super) fn has_age(&self) -> bool {
        self.ages.iter().any(Option::is_some)
    }
    pub(super) fn options(&self) -> ExecutionOptions {
        self.options
    }
    pub(super) fn reports(&self) -> &[ExecutionReport] {
        &self.reports
    }
    pub(super) fn is_native(&self) -> bool {
        self.native.is_some()
    }
    /// Scalar-only native preparation is the explicit comparison route. Native
    /// fallback still keeps the established Rust packet implementation.
    pub(super) fn allows_packets(&self) -> bool {
        self.options.preference != ExecutionPreference::ScalarJit || self.native.is_none()
    }
    pub(super) fn execute_packet(
        &self,
        layer: EffectLayerId,
        plan: &PureEvolution,
        scratch: &mut PacketScratch,
        time: f64,
        stats: &mut ExecutionStats,
    ) -> Option<Result<(), ()>> {
        self.native
            .as_ref()?
            .execute_simd(layer.0 as usize, plan, scratch, time, stats)
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn execute(
        &self,
        layer: EffectLayerId,
        stage: ExecutionStage,
        program: &Program,
        state: &mut State,
        scratch: &mut State,
        host: &mut ExecutionHost<'_>,
    ) -> Result<(), ExecutionError<&'static str>> {
        if let Some(code) = &self.native {
            let index = layer.0 as usize * 2 + usize::from(stage == ExecutionStage::Evolution);
            return with_scratch(state, scratch, |state| {
                code.execute_bound(index, program, state, host)
            });
        }
        program.execute_with_scratch(state, scratch, host)
    }
}
