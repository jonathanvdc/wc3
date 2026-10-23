//! Immutable executable selection, shared by instances but independent of schema identity.
use super::callbacks::ExecutionHost;
use super::execution::with_scratch;
use super::graph::{EffectGraph, EffectLayerId};
use super::native::NativeCode;
use super::packet::{PureEvolution, Scratch as PacketScratch};
use super::ExecutionFallback;
use super::{
    ExecutionBackend, ExecutionError, ExecutionOptions, ExecutionPreference, ExecutionReport,
    ExecutionStage, ExecutionStats, Program, State,
};
use std::panic::{catch_unwind, AssertUnwindSafe};

#[derive(Debug)]
pub(super) struct ExecutionPlan {
    options: ExecutionOptions,
    reports: Box<[ExecutionReport]>,
    native: Option<NativeCode>,
}
impl ExecutionPlan {
    pub(super) fn prepare(graph: &EffectGraph, options: ExecutionOptions) -> Self {
        let mut native = None;
        let fallback = if options.preference == ExecutionPreference::Interpreter {
            None
        } else {
            match catch_unwind(AssertUnwindSafe(|| {
                NativeCode::prepare(graph, options.preference == ExecutionPreference::Automatic)
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
                let evolution_backend = if native.as_ref().is_some_and(|code| code.has_simd(index))
                {
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
                        fallback: fallback.clone(),
                    }
                })
            })
            .collect();
        Self {
            options,
            reports,
            native,
        }
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
