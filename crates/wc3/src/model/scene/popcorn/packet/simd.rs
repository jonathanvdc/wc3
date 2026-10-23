//! Const-specialized SIMD lowering of the existing checked pure tile plan.
//! Native module ownership and runtime packet admission remain with the caller.
use super::{
    BatchScratch, Input, InstructionPlan, Location, Operand, PureEvolution, PARTICLE_BATCH_SIZE,
    VALUE_COMPONENTS,
};
use crate::model::scene::popcorn::graph::EffectSampler;
use crate::model::scene::popcorn::native::interpolation::{
    constant, coordinate, hermite, hermite_weights, linear,
};
use crate::model::scene::popcorn::{BinaryOperation, ValueType};
use cranelift_codegen::ir::condcodes::FloatCC;
use cranelift_codegen::ir::{types, AbiParam, Block, InstBuilder, MemFlagsData, Type, Value};
use cranelift_codegen::Context;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::Module;
use std::array::from_fn;
use std::mem::{offset_of, size_of};

const SIMD_LANES: usize = 4;
const MAX_CURVE_KEYS: usize = 16;
type Components = [Value; VALUE_COMPONENTS];

/// One synchronous invocation over one already staged complete particle batch.
/// No canonical storage or host capability can be reached through this ABI.
pub(crate) type Frame = BatchFrame<PARTICLE_BATCH_SIZE>;

#[repr(C)]
pub(crate) struct BatchFrame<const BATCH: usize> {
    fields: *mut u32,
    deltas: [f32; BATCH],
    time: f32,
}
impl<const BATCH: usize> BatchFrame<BATCH> {
    /// The generated entry and this plan must share the owning immutable graph.
    /// `call` is synchronous; no reference or entry pointer is stored in Scratch.
    pub(in crate::model::scene::popcorn) fn with(
        scratch: &mut BatchScratch<BATCH>,
        plan: &PureEvolution,
        time: f64,
        call: impl FnOnce(&mut Self) -> u32,
    ) -> Result<(), ()> {
        if scratch.count != BATCH || scratch.fields.len() < plan.fields.len() {
            return Err(());
        }
        #[cfg(test)]
        {
            scratch.attempts += 1;
        }
        let mut frame = Self {
            fields: scratch.fields.as_mut_ptr().cast(),
            deltas: from_fn(|lane| scratch.lanes[lane].delta as f32),
            time: time as f32,
        };
        if call(&mut frame) == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}
/// Concrete defaults keep runtime/native ownership independent of const generics.
type DefaultCompiler = SimdCompiler<PARTICLE_BATCH_SIZE, SIMD_LANES>;
#[cfg(test)]
pub(crate) fn eligible(plan: &PureEvolution, samplers: &[EffectSampler]) -> bool {
    DefaultCompiler::eligible(plan, samplers)
}
pub(crate) fn cost(plan: &PureEvolution, samplers: &[EffectSampler]) -> Option<usize> {
    DefaultCompiler::cost(plan, samplers)
}
pub(crate) fn lower(
    module: &mut impl Module,
    plan: &PureEvolution,
    samplers: &[EffectSampler],
    context: &mut Context,
) -> Result<(), &'static str> {
    DefaultCompiler::lower(module, plan, samplers, context)
}

/// Cold specialization owns the frame stride, vector type and emitted group count.
struct SimdCompiler<const BATCH: usize, const LANES: usize>;
impl<const BATCH: usize, const LANES: usize> SimdCompiler<BATCH, LANES> {
    fn vector_type() -> Option<Type> {
        if BATCH == 0 || LANES < 2 || !BATCH.is_multiple_of(LANES) {
            return None;
        }
        types::F32.by(u32::try_from(LANES).ok()?)
    }
    /// Return no tile entry for oversized curves; the existing Rust pure tile still
    /// evaluates them with its original per-lane checked sampler. This is an
    /// optimization admission bound, not a programme support limitation.
    fn eligible(plan: &PureEvolution, samplers: &[EffectSampler]) -> bool {
        if Self::vector_type().is_none() {
            return false;
        }
        if plan
            .fields
            .len()
            .checked_mul(VALUE_COMPONENTS)
            .and_then(|words| words.checked_mul(BATCH))
            .and_then(|words| words.checked_mul(size_of::<u32>()))
            .is_none_or(|bytes| bytes > i32::MAX as usize)
        {
            return false;
        }
        plan.operations.iter().all(|operation| {
            let InstructionPlan::Curve { sampler, .. } = operation.instruction else {
                return true;
            };
            let Some(EffectSampler::Curve {
                kind: ValueType::Float4,
                times,
                values,
                tangents,
            }) = samplers.get(sampler)
            else {
                return false;
            };
            !times.is_empty()
                && times.len() <= MAX_CURVE_KEYS
                && times.len() == values.len()
                && (tangents.is_empty() || tangents.len() == times.len() * 2)
        })
    }
    /// Conservative cold estimate of emitted vector IR work, not a timing claim.
    /// Charges every configured vector group and the static Float4 curve expansion.
    fn cost(plan: &PureEvolution, samplers: &[EffectSampler]) -> Option<usize> {
        if !Self::eligible(plan, samplers) {
            return None;
        }
        let mut per_group = plan
            .fields
            .iter()
            .filter(|field| field.written)
            .count()
            .checked_mul(16)?;
        for operation in &plan.operations {
            let operation_cost = match operation.instruction {
                InstructionPlan::Curve { sampler, .. } => {
                    let EffectSampler::Curve {
                        times, tangents, ..
                    } = &samplers[sampler]
                    else {
                        return None;
                    };
                    let per_component = if tangents.is_empty() {
                        16usize
                    } else {
                        56usize
                    };
                    times
                        .len()
                        .saturating_sub(1)
                        .checked_mul(VALUE_COMPONENTS)?
                        .checked_mul(per_component)?
                        .checked_add(64)?
                }
                InstructionPlan::Binary { .. } => 24,
                _ => 16,
            };
            per_group = per_group.checked_add(operation_cost)?;
        }
        per_group.checked_mul(BATCH / LANES)
    }
    /// Emit the derived number of SIMD groups. Register values use compiler SSA,
    /// not the retained Rust register planes. Eligibility already proves local zero
    /// initialization and persistent first-write-before-read in the selected entry.
    fn lower(
        module: &mut impl Module,
        plan: &PureEvolution,
        samplers: &[EffectSampler],
        context: &mut Context,
    ) -> Result<(), &'static str> {
        if !Self::eligible(plan, samplers) {
            return Err("pure tile exceeds SIMD curve/storage admission");
        }
        let vector_type = Self::vector_type().ok_or("unsupported SIMD batch/lane configuration")?;
        let pointer = module.target_config().pointer_type();
        context.func.signature.params.push(AbiParam::new(pointer));
        context
            .func
            .signature
            .returns
            .push(AbiParam::new(types::I32));
        let mut functions = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut context.func, &mut functions);
        let entry = builder.create_block();
        let failed = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let frame = builder.block_params(entry)[0];
        let fields = builder.ins().load(
            pointer,
            MemFlagsData::new(),
            frame,
            offset_of!(BatchFrame<BATCH>, fields) as i32,
        );
        let mut lower = Lower::<BATCH, LANES> {
            builder: &mut builder,
            failed,
            fields,
            vector_type,
        };
        // Time is immutable across the complete speculative tile. A failed guard
        // discards this tile and replays the original scalar source, so checking it
        // before the first group cannot change canonical stores or fault prefixes.
        // Kernels without a Time load must not inspect this otherwise unused input.
        let time = if plan.operations.iter().any(|operation| {
            matches!(
                operation.instruction,
                InstructionPlan::Load {
                    input: Input::Time,
                    ..
                }
            )
        }) {
            let value = lower.builder.ins().load(
                types::F32,
                MemFlagsData::new(),
                frame,
                offset_of!(BatchFrame<BATCH>, time) as i32,
            );
            let value = lower.builder.ins().splat(lower.vector_type, value);
            lower.finite(value);
            Some(value)
        } else {
            None
        };
        for group in 0..BATCH / LANES {
            let zero = lower.constant(0.);
            let mut registers = vec![None; plan.registers];
            registers[..plan.locals].fill(Some([zero; VALUE_COMPONENTS]));
            let mut values = vec![None; plan.fields.len()];
            for operation in &plan.operations {
                match operation.instruction {
                    InstructionPlan::Load { output, input } => {
                        let mut result = [zero; VALUE_COMPONENTS];
                        match input {
                            Input::Field(field) => {
                                let input = if let Some(input) = values[field] {
                                    input
                                } else {
                                    let input =
                                        lower.fields(field, group, plan.fields[field].components);
                                    values[field] = Some(input);
                                    input
                                };
                                result[..output.width].copy_from_slice(&input[..output.width]);
                            }
                            Input::Time => {
                                result[0] =
                                    time.expect("Time loads were checked before group lowering");
                            }
                            Input::Delta => {
                                let offset = offset_of!(BatchFrame<BATCH>, deltas)
                                    + group * LANES * size_of::<f32>();
                                let value = lower.builder.ins().load(
                                    lower.vector_type,
                                    MemFlagsData::new(),
                                    frame,
                                    offset as i32,
                                );
                                lower.finite(value);
                                result[0] = value;
                            }
                        }
                        registers[output.slot] = Some(result);
                    }
                    InstructionPlan::Store { field, input } => {
                        let mut result = [zero; VALUE_COMPONENTS];
                        for (component, value) in result.iter_mut().enumerate().take(input.width) {
                            *value = lower.operand(input, component, &registers)?;
                        }
                        values[field] = Some(result);
                    }
                    InstructionPlan::Binary {
                        output,
                        left,
                        right,
                        operation,
                    } => {
                        let mut result = [zero; VALUE_COMPONENTS];
                        for (component, value) in result.iter_mut().enumerate().take(output.width) {
                            let left = lower.operand(left, component, &registers)?;
                            let right = lower.operand(right, component, &registers)?;
                            *value = match operation {
                                BinaryOperation::Add => lower.builder.ins().fadd(left, right),
                                BinaryOperation::Subtract => lower.builder.ins().fsub(left, right),
                                BinaryOperation::Multiply => lower.builder.ins().fmul(left, right),
                                _ => return Err("non-pure arithmetic in checked tile"),
                            };
                        }
                        registers[output.slot] = Some(result);
                    }
                    InstructionPlan::Curve {
                        output,
                        input,
                        sampler,
                    } => {
                        let coordinate = lower.operand(input, 0, &registers)?;
                        lower.finite(coordinate);
                        registers[output.slot] = Some(lower.curve(&samplers[sampler], coordinate)?);
                    }
                }
            }
            // Only speculative field overlays are modified. A later group failure
            // discards all groups and the existing runtime replays scalar exactly
            // once, before canonical fields, life, cursor or work are committed.
            for (field, _binding) in plan
                .fields
                .iter()
                .enumerate()
                .filter(|(_, binding)| binding.written)
            {
                let result = values[field].ok_or("written tile field has no SSA value")?;
                for (component, value) in result.into_iter().enumerate() {
                    let offset = Self::plane_offset(field, component, group)?;
                    lower
                        .builder
                        .ins()
                        .store(MemFlagsData::new(), value, fields, offset);
                }
            }
        }
        let zero = lower.builder.ins().iconst(types::I32, 0);
        lower.builder.ins().return_(&[zero]);
        lower.builder.switch_to_block(failed);
        let one = lower.builder.ins().iconst(types::I32, 1);
        lower.builder.ins().return_(&[one]);
        builder.seal_all_blocks();
        builder.finalize(module.target_config());
        Ok(())
    }
    fn plane_offset(field: usize, component: usize, group: usize) -> Result<i32, &'static str> {
        let words = field
            .checked_mul(VALUE_COMPONENTS)
            .and_then(|v| v.checked_add(component))
            .and_then(|v| v.checked_mul(BATCH))
            .and_then(|v| v.checked_add(group * LANES))
            .ok_or("SIMD field offset overflow")?;
        let bytes = words
            .checked_mul(size_of::<u32>())
            .ok_or("SIMD field offset overflow")?;
        i32::try_from(bytes).map_err(|_| "SIMD field offset exceeds native addressing")
    }
}
struct Lower<'a, 'b, const BATCH: usize, const LANES: usize> {
    builder: &'a mut FunctionBuilder<'b>,
    failed: Block,
    fields: Value,
    vector_type: Type,
}
impl<const BATCH: usize, const LANES: usize> Lower<'_, '_, BATCH, LANES> {
    fn constant(&mut self, value: f32) -> Value {
        constant(self.builder, self.vector_type, value)
    }
    fn operand(
        &mut self,
        operand: Operand,
        component: usize,
        registers: &[Option<Components>],
    ) -> Result<Value, &'static str> {
        let component = component.min(operand.width - 1);
        match operand.location {
            Location::Constant(words) => Ok(self.constant(f32::from_bits(words[component]))),
            Location::Register(slot) => Ok(registers
                .get(slot)
                .copied()
                .flatten()
                .ok_or("SIMD persistent register read precedes its write")?[component]),
        }
    }
    fn fields(&mut self, field: usize, group: usize, components: usize) -> Components {
        from_fn(|component| {
            if component >= components {
                return self.constant(0.);
            }
            self.builder.ins().load(
                self.vector_type,
                MemFlagsData::new(),
                self.fields,
                SimdCompiler::<BATCH, LANES>::plane_offset(field, component, group)
                    .expect("eligible checked plane offset"),
            )
        })
    }
    fn finite(&mut self, value: Value) {
        let abs = self.builder.ins().fabs(value);
        let max = self.constant(f32::MAX);
        let finite = self.builder.ins().fcmp(FloatCC::LessThanOrEqual, abs, max);
        let all = self.builder.ins().vall_true(finite);
        let next = self.builder.create_block();
        self.builder.ins().brif(all, next, &[], self.failed, &[]);
        self.builder.switch_to_block(next);
    }
    fn select(&mut self, mask: Value, yes: Value, no: Value) -> Value {
        let bits = self
            .builder
            .ins()
            .bitcast(self.vector_type, MemFlagsData::new(), mask);
        self.builder.ins().bitselect(bits, yes, no)
    }
    fn curve(&mut self, sampler: &EffectSampler, x: Value) -> Result<Components, &'static str> {
        let EffectSampler::Curve {
            kind: ValueType::Float4,
            times,
            values,
            tangents,
        } = sampler
        else {
            return Err("SIMD curve is not immutable Float4");
        };
        let mut result = from_fn(|component| self.constant(values[values.len() - 1][component]));
        for interval in 0..times.len() - 1 {
            let start = self.constant(times[interval]);
            let end = self.constant(times[interval + 1]);
            let above = self.builder.ins().fcmp(
                if interval == 0 {
                    FloatCC::GreaterThan
                } else {
                    FloatCC::GreaterThanOrEqual
                },
                x,
                start,
            );
            let below = self.builder.ins().fcmp(FloatCC::LessThan, x, end);
            let mask = self.builder.ins().band(above, below);
            let t = coordinate(self.builder, x, start, end);
            let weights = if tangents.is_empty() {
                None
            } else {
                Some(hermite_weights(self.builder, t))
            };
            for component in 0..VALUE_COMPONENTS {
                let first = self.constant(values[interval][component]);
                let last = self.constant(values[interval + 1][component]);
                let value = if let Some(weights) = weights {
                    let out = self.constant(tangents[interval * 2 + 1][component]);
                    let input = self.constant(tangents[(interval + 1) * 2][component]);
                    hermite(self.builder, [first, out, last, input], weights)
                } else {
                    linear(self.builder, first, last, t)
                };
                result[component] = self.select(mask, value, result[component]);
            }
        }
        // The scalar helper returns the first key directly at/before its first
        // timestamp. Do not evaluate that endpoint through a polynomial: signed
        // zero and cancellation behavior can differ from a direct key load.
        let first_time = self.constant(times[0]);
        let before = self
            .builder
            .ins()
            .fcmp(FloatCC::LessThanOrEqual, x, first_time);
        for component in 0..VALUE_COMPONENTS {
            let first = self.constant(values[0][component]);
            result[component] = self.select(before, first, result[component]);
        }
        Ok(result)
    }
}

#[cfg(test)]
#[path = "simd_tests.rs"]
mod tests;
