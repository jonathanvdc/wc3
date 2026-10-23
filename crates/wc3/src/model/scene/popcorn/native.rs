//! Optional native invocation code; the Rust runtime retains lifecycle authority.
use super::callbacks::ExecutionHost;
use super::graph::EffectGraph;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use super::packet::simd::{self, Frame};
use super::packet::{PureEvolution, Scratch as PacketScratch};
use super::runtime::FieldLayout;
#[cfg(test)]
use super::Host;
use super::{ExecutionError, ExecutionFallback, ExecutionStats, Program, State};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::mem::transmute;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

mod bridge;
mod codegen;
pub(super) mod interpolation;

pub(super) const OPERATION_LIMIT: usize = 32_768;
type Entry = unsafe extern "C" fn(*mut bridge::Invocation) -> u32;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
type TileEntry = unsafe extern "C" fn(*mut Frame) -> u32;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
const SIMD_CODEGEN_BUDGET: usize = 65_536;

/// No entry point escapes this owner. The mutex only makes the non-Sync module
/// safely shareable; execution never locks or accesses the mutable compiler.
pub(super) struct NativeCode {
    _module: Mutex<ModuleOwner>,
    entries: Box<[Option<Entry>]>,
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    tiles: Box<[Option<TileEntry>]>,
    field_layout: FieldLayout,
}
impl Debug for NativeCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("NativeCode")
            .field("stages", &self.entries.len())
            .finish_non_exhaustive()
    }
}
struct ModuleOwner(Option<JITModule>);
impl Drop for ModuleOwner {
    fn drop(&mut self) {
        if let Some(module) = self.0.take() {
            // The last owner can drop only after every borrowed invocation has
            // returned. Function pointers are private to the same owner.
            unsafe { module.free_memory() };
        }
    }
}
impl NativeCode {
    pub(super) fn prepare(
        graph: &EffectGraph,
        prepare_simd: bool,
    ) -> Result<Self, ExecutionFallback> {
        let operations = graph
            .layers
            .iter()
            .map(|layer| {
                layer.init.program.instruction_count()
                    + layer
                        .evolve
                        .as_ref()
                        .map_or(0, |bound| bound.program.instruction_count())
            })
            .sum();
        if operations > OPERATION_LIMIT {
            return Err(ExecutionFallback::BudgetExceeded {
                operations,
                limit: OPERATION_LIMIT,
            });
        }
        Self::compile(graph, prepare_simd).map_err(|message| ExecutionFallback::CompilationFailed {
            message: message.into_boxed_str(),
        })
    }
    fn compile(graph: &EffectGraph, prepare_simd: bool) -> Result<Self, String> {
        let _ = prepare_simd; // Scalar native targets without SIMD keep the same API.
        let field_layout = FieldLayout::for_graph(graph)
            .ok_or_else(|| "native field layout exceeds its checked address range".to_owned())?;
        let builder = JITBuilder::with_flags(&[("opt_level", "speed")], default_libcall_names())
            .map_err(|e| e.to_string())?;
        let mut owner = ModuleOwner(Some(JITModule::new(builder)));
        let module = owner.0.as_mut().expect("new native module");
        let mut context = module.make_context();
        let mut functions = Vec::with_capacity(graph.layers.len() * 2);
        let mut curve_budget = codegen::CURVE_INLINE_BUDGET;
        for (layer, data) in graph.layers.iter().enumerate() {
            for (stage, bound) in [Some(&data.init), data.evolve.as_ref()]
                .into_iter()
                .enumerate()
            {
                let Some(bound) = bound else {
                    functions.push(None);
                    continue;
                };
                let program = &bound.program;
                if program.counts.iter().any(|count| {
                    count
                        .checked_mul(16)
                        .is_none_or(|bytes| bytes > i32::MAX as usize)
                }) {
                    return Err("native register offset exceeds its checked address range".into());
                }
                codegen::lower_bound(
                    module,
                    bound,
                    &data.fields,
                    &field_layout,
                    &graph.samplers,
                    &mut curve_budget,
                    &mut context,
                );
                let id = module
                    .declare_function(
                        &format!("particle_{layer}_{stage}"),
                        Linkage::Local,
                        &context.func.signature,
                    )
                    .map_err(|e| e.to_string())?;
                module
                    .define_function(id, &mut context)
                    .map_err(|e| e.to_string())?;
                module.clear_context(&mut context);
                functions.push(Some(id));
            }
        }
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        let tile_functions = {
            let mut remaining = SIMD_CODEGEN_BUDGET;
            graph
                .layers
                .iter()
                .enumerate()
                .map(|(layer, data)| {
                    if !prepare_simd {
                        return None;
                    }
                    let plan = data.pure_evolution.as_deref()?;
                    let cost = simd::cost(plan, &graph.samplers)?;
                    if cost > remaining {
                        return None;
                    }
                    // Charge before the attempt. Rejected kernels do not turn into
                    // an unbounded preparation retry or a whole-graph failure.
                    remaining -= cost;
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        simd::lower(module, plan, &graph.samplers, &mut context).ok()?;
                        let id = module
                            .declare_function(
                                &format!("particle_tile_{layer}"),
                                Linkage::Local,
                                &context.func.signature,
                            )
                            .ok()?;
                        module.define_function(id, &mut context).ok()?;
                        Some(id)
                    }))
                    .ok()
                    .flatten();
                    // Failed declarations have no callers. Cranelift finalizes only
                    // successfully defined functions; scalar entries remain valid.
                    module.clear_context(&mut context);
                    result
                })
                .collect::<Vec<_>>()
        };
        module.finalize_definitions().map_err(|e| e.to_string())?;
        let entries = functions
            .into_iter()
            .map(|id| {
                id.map(|id| {
                    let address = module.get_finalized_function(id);
                    // Every definition above has the native C ABI (Invocation*) -> u32.
                    // The code owner keeps its executable allocation alive until drop.
                    unsafe { transmute::<*const u8, Entry>(address) }
                })
            })
            .collect();
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        let tiles = tile_functions
            .into_iter()
            .map(|id| {
                id.map(|id| {
                    let address = module.get_finalized_function(id);
                    // The same owner retains this synchronous (Frame*) -> u32 entry.
                    unsafe { transmute::<*const u8, TileEntry>(address) }
                })
            })
            .collect();
        Ok(Self {
            _module: Mutex::new(owner),
            entries,
            #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
            tiles,
            field_layout,
        })
    }
    pub(super) fn has_simd(&self, layer: usize) -> bool {
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            self.tiles.get(layer).is_some_and(Option::is_some)
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            let _ = layer;
            false
        }
    }
    pub(super) fn execute_simd(
        &self,
        layer: usize,
        plan: &PureEvolution,
        scratch: &mut PacketScratch,
        time: f64,
        stats: &mut ExecutionStats,
    ) -> Option<Result<(), ()>> {
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            let entry = (*self.tiles.get(layer)?)?;
            Some(Frame::with(scratch, plan, time, |frame| {
                stats.simd_attempted_batches = stats.simd_attempted_batches.saturating_add(1);
                // Entry/plan/field layout belong to this immutable executable
                // owner; frame's mutable scratch borrow lasts through return.
                unsafe { entry(frame) }
            }))
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            let _ = (layer, plan, scratch, time, stats);
            None
        }
    }
    pub(super) fn execute_bound(
        &self,
        index: usize,
        program: &Program,
        state: &mut State,
        host: &mut ExecutionHost<'_>,
    ) -> Result<(), ExecutionError<&'static str>> {
        let entry = self.entries[index].expect("prepared native stage");
        bridge::run_bound(entry, program, state, host, &self.field_layout)
    }
}

#[cfg(test)]
#[path = "native/failure_tests.rs"]
mod failure_tests;
