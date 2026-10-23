//! Optional native invocation code; the Rust runtime retains lifecycle authority.
use super::callbacks::ExecutionHost;
use super::graph::EffectGraph;
use super::runtime::FieldLayout;
#[cfg(test)]
use super::Host;
use super::{ExecutionError, ExecutionFallback, Program, State};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::mem::transmute;
use std::sync::Mutex;

mod bridge;
mod codegen;
pub(super) mod interpolation;

pub(super) const OPERATION_LIMIT: usize = 32_768;
type Entry = unsafe extern "C" fn(*mut bridge::Invocation) -> u32;

/// No entry point escapes this owner. The mutex only makes the non-Sync module
/// safely shareable; execution never locks or accesses the mutable compiler.
pub(super) struct NativeCode {
    _module: Mutex<ModuleOwner>,
    entries: Box<[Option<Entry>]>,
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
    pub(super) fn prepare(graph: &EffectGraph) -> Result<Self, ExecutionFallback> {
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
        Self::compile(graph).map_err(|message| ExecutionFallback::CompilationFailed {
            message: message.into_boxed_str(),
        })
    }
    fn compile(graph: &EffectGraph) -> Result<Self, String> {
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
        Ok(Self {
            _module: Mutex::new(owner),
            entries,
            field_layout,
        })
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
