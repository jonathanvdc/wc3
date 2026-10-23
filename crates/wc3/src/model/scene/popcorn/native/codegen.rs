//! Lower checked scalar instructions to machine operations and ordered host barriers.
use super::bridge::Invocation;
use crate::model::scene::popcorn::graph::{BoundProgram, EffectField, External};
use crate::model::scene::popcorn::numeric::{BinaryOperation, UnaryOperation};
use crate::model::scene::popcorn::runtime::FieldLayout;
use crate::model::scene::popcorn::{
    Address, Destination, Family, Instruction, Program, Source, ValueType,
};
use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::{types, AbiParam, InstBuilder, MemFlagsData, Signature, Value};
use cranelift_codegen::ir::{Block, SigRef, Type};
use cranelift_codegen::Context;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::Module;
use std::array::from_fn;
use std::mem::{offset_of, size_of};

#[cfg(test)]
pub(super) fn lower(module: &mut impl Module, program: &Program, context: &mut Context) {
    lower_impl(module, program, context, None);
}

pub(super) fn lower_bound(
    module: &mut impl Module,
    bound: &BoundProgram,
    fields: &[EffectField],
    layout: &FieldLayout,
    context: &mut Context,
) {
    let bindings: Vec<_> = bound
        .program
        .operations
        .iter()
        .map(|op| {
            let (external, kind, store) = match &op.instruction {
                Instruction::Load {
                    external,
                    destination,
                } => (*external, destination.kind, false),
                Instruction::Store { external, source } => (*external, source.kind, true),
                _ => return None,
            };
            match *bound.externals.get(usize::from(external))? {
                External::Field(field) if fields.get(field)?.value_type == Some(kind) => {
                    Some(Direct::Field {
                        ordinals: *layout.component_ordinals().get(field)?,
                        store,
                    })
                }
                External::Time if !store && kind == ValueType::Float => Some(Direct::Time),
                External::Delta if !store && kind == ValueType::Float => Some(Direct::Delta),
                _ => None,
            }
        })
        .collect();
    lower_impl(module, &bound.program, context, Some(&bindings));
}
#[derive(Clone, Copy)]
enum Direct {
    Field { ordinals: [usize; 4], store: bool },
    Time,
    Delta,
}
fn lower_impl(
    module: &mut impl Module,
    program: &Program,
    context: &mut Context,
    bindings: Option<&[Option<Direct>]>,
) {
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
    let scopes = from_fn(|i| {
        builder.ins().load(
            pointer,
            MemFlagsData::new(),
            frame,
            (offset_of!(Invocation, scopes) + i * size_of::<usize>()) as i32,
        )
    });
    let service = builder.ins().load(
        pointer,
        MemFlagsData::new(),
        frame,
        offset_of!(Invocation, service) as i32,
    );
    let opaque = builder.ins().load(
        pointer,
        MemFlagsData::new(),
        frame,
        offset_of!(Invocation, context) as i32,
    );
    let mut signature = Signature::new(module.target_config().default_call_conv);
    signature
        .params
        .extend([AbiParam::new(pointer), AbiParam::new(types::I32)]);
    signature.returns.push(AbiParam::new(types::I32));
    let signature = builder.import_signature(signature);
    for (index, operation) in program.operations.iter().enumerate() {
        let mut lower = Lower {
            builder: &mut builder,
            scopes,
            frame,
            service,
            opaque,
            service_signature: signature,
            failed,
        };
        let direct = bindings.and_then(|b| b[index]);
        if direct.is_some_and(|binding| lower.direct(&operation.instruction, binding, index)) {
            continue;
        }
        if !lower.instruction(&operation.instruction) {
            let index = lower.builder.ins().iconst(types::I32, index as i64);
            let call = lower
                .builder
                .ins()
                .call_indirect(signature, service, &[opaque, index]);
            let status = lower.builder.inst_results(call)[0];
            let next = lower.builder.create_block();
            lower.builder.ins().brif(status, failed, &[], next, &[]);
            lower.builder.switch_to_block(next);
        }
    }
    let zero = builder.ins().iconst(types::I32, 0);
    builder.ins().return_(&[zero]);
    builder.switch_to_block(failed);
    let one = builder.ins().iconst(types::I32, 1);
    builder.ins().return_(&[one]);
    builder.seal_all_blocks();
    builder.finalize(module.target_config());
}

struct Lower<'a, 'b> {
    builder: &'a mut FunctionBuilder<'b>,
    scopes: [Value; 4],
    frame: Value,
    service: Value,
    opaque: Value,
    service_signature: SigRef,
    failed: Block,
}
impl Lower<'_, '_> {
    fn invocation_word(&mut self, offset: usize) -> Value {
        self.builder
            .ins()
            .load(types::I32, MemFlagsData::new(), self.frame, offset as i32)
    }
    fn direct(&mut self, instruction: &Instruction, binding: Direct, index: usize) -> bool {
        let pointer = self.scopes[0];
        let pointer_type = self.builder.func.dfg.value_type(pointer);
        let fields = self.builder.ins().load(
            pointer_type,
            MemFlagsData::new(),
            self.frame,
            offset_of!(Invocation, fields) as i32,
        );
        // Null means this is the generic Host ABI: never interpret its externals
        // as canonical fields or effect time, even with a bound compiled entry.
        let present = self.builder.ins().icmp_imm_s(IntCC::NotEqual, fields, 0);
        let time_word = match binding {
            Direct::Time => Some(self.invocation_word(offset_of!(Invocation, time))),
            Direct::Delta => Some(self.invocation_word(offset_of!(Invocation, delta))),
            _ => None,
        };
        let admitted = if let Some(word) = time_word {
            let exponent = self.builder.ins().band_imm_s(word, 0x7f800000);
            let finite = self
                .builder
                .ins()
                .icmp_imm_s(IntCC::NotEqual, exponent, 0x7f800000);
            self.builder.ins().band(present, finite)
        } else {
            present
        };
        let fast = self.builder.create_block();
        let slow = self.builder.create_block();
        let next = self.builder.create_block();
        self.builder.ins().brif(admitted, fast, &[], slow, &[]);
        self.builder.switch_to_block(fast);
        match (binding, instruction) {
            (
                Direct::Field {
                    ordinals,
                    store: false,
                },
                Instruction::Load { destination, .. },
            ) => {
                let words: Vec<_> = ordinals
                    .iter()
                    .take(destination.kind.components())
                    .map(|ordinal| {
                        let address = self.field_address(fields, *ordinal, pointer_type);
                        self.builder
                            .ins()
                            .load(types::I32, MemFlagsData::new(), address, 0)
                    })
                    .collect();
                self.write(*destination, &words);
            }
            (
                Direct::Field {
                    ordinals,
                    store: true,
                },
                Instruction::Store { source, .. },
            ) => {
                // FieldLayout is a union across layers. Present higher planes
                // must be zeroed even when this layer's typed field is narrower.
                for (component, ordinal) in ordinals.into_iter().enumerate() {
                    if ordinal != FieldLayout::ABSENT {
                        let value = self.word(*source, component);
                        let address = self.field_address(fields, ordinal, pointer_type);
                        self.builder
                            .ins()
                            .store(MemFlagsData::new(), value, address, 0);
                    }
                }
            }
            (Direct::Time | Direct::Delta, Instruction::Load { destination, .. }) => {
                self.write(*destination, &[time_word.expect("time binding")]);
            }
            _ => unreachable!("validated direct binding"),
        }
        self.builder.ins().jump(next, &[]);
        self.builder.switch_to_block(slow);
        // Checked service owns nonfinite Time/Delta errors and exact offsets.
        // Only this instruction runs; preceding native writes are never replayed.
        let index = self.builder.ins().iconst(types::I32, index as i64);
        let call = self.builder.ins().call_indirect(
            self.service_signature,
            self.service,
            &[self.opaque, index],
        );
        let status = self.builder.inst_results(call)[0];
        self.builder.ins().brif(status, self.failed, &[], next, &[]);
        self.builder.switch_to_block(next);
        true
    }
    fn field_address(&mut self, fields: Value, ordinal: usize, pointer_type: Type) -> Value {
        let stride = self.builder.ins().load(
            pointer_type,
            MemFlagsData::new(),
            self.frame,
            offset_of!(Invocation, stride) as i32,
        );
        let slot = self.builder.ins().load(
            pointer_type,
            MemFlagsData::new(),
            self.frame,
            offset_of!(Invocation, slot) as i32,
        );
        let plane = self.builder.ins().imul_imm_s(stride, ordinal as i64);
        let word = self.builder.ins().iadd(plane, slot);
        let bytes = self.builder.ins().imul_imm_s(word, 4);
        self.builder.ins().iadd(fields, bytes)
    }

    fn word(&mut self, source: Source, component: usize) -> Value {
        if component >= source.kind.components() {
            return self.builder.ins().iconst(types::I32, 0);
        }
        match source.address {
            Address::Constant(words) => self
                .builder
                .ins()
                .iconst(types::I32, i64::from(words[component])),
            Address::Slot { scope, slot } => self.builder.ins().load(
                types::I32,
                MemFlagsData::new(),
                self.scopes[scope],
                (slot * 16 + component * 4) as i32,
            ),
        }
    }
    fn component(&mut self, source: Source, component: usize) -> Value {
        self.word(source, component.min(source.kind.components() - 1))
    }
    fn float(&mut self, value: Value) -> Value {
        self.builder
            .ins()
            .bitcast(types::F32, MemFlagsData::new(), value)
    }
    fn bits(&mut self, value: Value) -> Value {
        self.builder
            .ins()
            .bitcast(types::I32, MemFlagsData::new(), value)
    }
    fn truth(&mut self, value: Value) -> Value {
        let yes = self.builder.ins().iconst(types::I32, -1);
        let no = self.builder.ins().iconst(types::I32, 0);
        self.builder.ins().select(value, yes, no)
    }
    fn write(&mut self, destination: Destination, values: &[Value]) {
        let zero = self.builder.ins().iconst(types::I32, 0);
        for component in 0..4 {
            let value = values.get(component).copied().unwrap_or(zero);
            self.builder.ins().store(
                MemFlagsData::new(),
                value,
                self.scopes[destination.scope],
                (destination.slot * 16 + component * 4) as i32,
            );
        }
    }
    fn instruction(&mut self, instruction: &Instruction) -> bool {
        match instruction {
            Instruction::Reinterpret {
                destination,
                source,
            } => {
                let words: Vec<_> = (0..destination.kind.components())
                    .map(|i| self.word(*source, i))
                    .collect();
                self.write(*destination, &words);
            }
            Instruction::Construct {
                destination,
                sources,
            } => {
                let mut words = Vec::new();
                for source in sources {
                    for i in 0..source.kind.components() {
                        words.push(self.word(*source, i));
                    }
                }
                if words.len() == 1 {
                    words.resize(destination.kind.components(), words[0]);
                }
                self.write(*destination, &words);
            }
            Instruction::Swizzle {
                destination,
                source,
                lanes,
            } => {
                let words: Vec<_> = (0..destination.kind.components())
                    .map(|i| match lanes[i] {
                        0..=3 => self.word(*source, usize::from(lanes[i])),
                        4 => self.builder.ins().iconst(types::I32, 0),
                        5 => self.builder.ins().iconst(
                            types::I32,
                            match destination.kind.family() {
                                Family::Float => i64::from(1f32.to_bits()),
                                Family::Bool => -1,
                                _ => 1,
                            },
                        ),
                        _ => unreachable!("checked swizzle"),
                    })
                    .collect();
                self.write(*destination, &words);
            }
            Instruction::Select {
                destination,
                if_false,
                if_true,
                condition,
            } => {
                let words: Vec<_> = (0..destination.kind.components())
                    .map(|i| {
                        let c = self.component(*condition, i);
                        let a = self.component(*if_false, i);
                        let b = self.component(*if_true, i);
                        self.builder.ins().select(c, b, a)
                    })
                    .collect();
                self.write(*destination, &words);
            }
            Instruction::Binary {
                destination,
                left,
                right,
                operation,
            } => {
                if matches!(
                    operation,
                    BinaryOperation::Minimum
                        | BinaryOperation::Maximum
                        | BinaryOperation::Dot
                        | BinaryOperation::Remainder
                ) || (*operation == BinaryOperation::Divide
                    && left.kind.family() != Family::Float)
                {
                    return false;
                }
                let mut words = Vec::new();
                for i in 0..destination.kind.components() {
                    let a = self.component(*left, i);
                    let b = self.component(*right, i);
                    let value = self.binary(*operation, left.kind.family(), a, b);
                    words.push(value);
                }
                self.write(*destination, &words);
            }
            Instruction::Unary {
                destination,
                source,
                operation,
            } => {
                if !matches!(
                    operation,
                    UnaryOperation::Negate
                        | UnaryOperation::LogicalNot
                        | UnaryOperation::BitNot
                        | UnaryOperation::IsFinite
                ) {
                    return false;
                }
                let mut words = Vec::new();
                for i in 0..destination.kind.components() {
                    let a = self.component(*source, i);
                    let result = match operation {
                        UnaryOperation::Negate if source.kind.family() == Family::Float => {
                            let a = self.float(a);
                            let v = self.builder.ins().fneg(a);
                            self.bits(v)
                        }
                        UnaryOperation::Negate => self.builder.ins().ineg(a),
                        UnaryOperation::LogicalNot => {
                            let v = self.builder.ins().icmp_imm_s(IntCC::Equal, a, 0);
                            self.truth(v)
                        }
                        UnaryOperation::BitNot => self.builder.ins().bnot(a),
                        UnaryOperation::IsFinite => {
                            let mask = self.builder.ins().band_imm_s(a, 0x7f800000);
                            let v =
                                self.builder
                                    .ins()
                                    .icmp_imm_s(IntCC::NotEqual, mask, 0x7f800000);
                            self.truth(v)
                        }
                        _ => unreachable!(),
                    };
                    words.push(result);
                }
                self.write(*destination, &words);
            }
            _ => return false,
        }
        true
    }
    fn binary(&mut self, operation: BinaryOperation, family: Family, a: Value, b: Value) -> Value {
        use BinaryOperation::*;
        if matches!(operation, LogicalAnd | LogicalOr) {
            let a = self.builder.ins().icmp_imm_s(IntCC::NotEqual, a, 0);
            let b = self.builder.ins().icmp_imm_s(IntCC::NotEqual, b, 0);
            let v = if operation == LogicalAnd {
                self.builder.ins().band(a, b)
            } else {
                self.builder.ins().bor(a, b)
            };
            return self.truth(v);
        }
        if family == Family::Float {
            let a = self.float(a);
            let b = self.float(b);
            let arithmetic = match operation {
                Add => Some(self.builder.ins().fadd(a, b)),
                Subtract => Some(self.builder.ins().fsub(a, b)),
                Multiply => Some(self.builder.ins().fmul(a, b)),
                Divide => Some(self.builder.ins().fdiv(a, b)),
                _ => None,
            };
            if let Some(value) = arithmetic {
                return self.bits(value);
            }
            let condition = match operation {
                Less => FloatCC::LessThan,
                LessEqual => FloatCC::LessThanOrEqual,
                Greater => FloatCC::GreaterThan,
                GreaterEqual => FloatCC::GreaterThanOrEqual,
                Equal => FloatCC::Equal,
                _ => unreachable!("checked floating binary"),
            };
            let v = self.builder.ins().fcmp(condition, a, b);
            return self.truth(v);
        }
        match operation {
            Add => self.builder.ins().iadd(a, b),
            Subtract => self.builder.ins().isub(a, b),
            Multiply => self.builder.ins().imul(a, b),
            BitAnd => self.builder.ins().band(a, b),
            BitOr => self.builder.ins().bor(a, b),
            Equal if family == Family::Bool => {
                let a = self.builder.ins().icmp_imm_s(IntCC::NotEqual, a, 0);
                let b = self.builder.ins().icmp_imm_s(IntCC::NotEqual, b, 0);
                let v = self.builder.ins().icmp(IntCC::Equal, a, b);
                self.truth(v)
            }
            Less | LessEqual | Greater | GreaterEqual | Equal => {
                let cc = match operation {
                    Less => IntCC::SignedLessThan,
                    LessEqual => IntCC::SignedLessThanOrEqual,
                    Greater => IntCC::SignedGreaterThan,
                    GreaterEqual => IntCC::SignedGreaterThanOrEqual,
                    _ => IntCC::Equal,
                };
                let v = self.builder.ins().icmp(cc, a, b);
                self.truth(v)
            }
            _ => unreachable!("checked integer binary"),
        }
    }
}
