use std::convert::Infallible;
use std::slice::from_ref;
use wc3::model::formats::pkb::{
    Argument, CompileLimits, Instruction, LocatedInstruction, Register, Script,
};
use wc3::model::scene::popcorn::{Failure, Function, Host, State, Value, ValueType};

fn register(bank: u8, scope: u8, slot: u16) -> Register {
    Register((u32::from(bank) << 24) | (u32::from(scope) << 16) | u32::from(slot))
}
fn script<'a>(constants: &'a [u8], instructions: Vec<Instruction>) -> Script<'a> {
    Script {
        reserved: [0; 2],
        register_counts: [0, 8, 8, 8, 16],
        constants,
        instructions: instructions
            .into_iter()
            .enumerate()
            .map(|(index, instruction)| LocatedInstruction {
                offset: index * 17,
                instruction,
            })
            .collect(),
        padding: &[],
    }
}
fn constants(values: &[[u32; 4]]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.iter().copied().chain([0; 4]))
        .flat_map(u32::to_le_bytes)
        .collect()
}
fn scalar(value: f32) -> Value {
    Value::from_words(ValueType::Float, [value.to_bits(), 0, 0, 0])
}
fn store(source: Register, external: u16) -> Instruction {
    Instruction::Store { source, external }
}
type RecordedCall = (u16, Option<u16>, Vec<Value>, Vec<u8>, u8);

#[derive(Default)]
struct Fields {
    values: Vec<Value>,
    calls: Vec<RecordedCall>,
    killed: bool,
}
impl Host for Fields {
    type Error = Infallible;
    fn load(&mut self, external: u16) -> Result<Value, Self::Error> {
        Ok(self.values[usize::from(external)])
    }
    fn store(&mut self, external: u16, value: Value) -> Result<(), Self::Error> {
        self.values[usize::from(external)] = value;
        Ok(())
    }
    fn call(
        &mut self,
        function: u16,
        receiver: Option<u16>,
        arguments: &[Value],
        controls: &[u8],
        flags: u8,
    ) -> Result<Option<Value>, Self::Error> {
        self.calls.push((
            function,
            receiver,
            arguments.to_vec(),
            controls.to_vec(),
            flags,
        ));
        if function == 0 {
            self.killed = true;
            Ok(None)
        } else {
            Ok(Some(scalar(9.0)))
        }
    }
}

#[test]
fn initialization_stores_zero_float_and_integer_pair_from_one_native_constant() {
    let bytes = constants(&[[0; 4]]);
    let native = script(
        &bytes,
        vec![
            store(register(0x20, 0, 0), 0),
            store(register(0x20, 0x10, 0), 1),
            store(register(0x1b, 0, 0), 2),
        ],
    );
    let types = [ValueType::Float, ValueType::Float, ValueType::Int2];
    let program = native.compile(&types, &[]).unwrap();
    let mut host = Fields {
        values: vec![
            scalar(1.0),
            scalar(2.0),
            Value::from_words(ValueType::Int2, [3, 4, 0, 0]),
        ],
        ..Default::default()
    };
    program
        .execute(&mut program.new_state(), &mut host)
        .unwrap();
    assert_eq!(
        host.values,
        types.map(|kind| Value::from_words(kind, [0; 4]))
    );
}

#[test]
fn floating_comparisons_use_source_type_and_boolean_not_selects_false_first() {
    let bytes = constants(&[
        [(-2.0f32).to_bits(), 0, 0, 0],
        [(-1.0f32).to_bits(), 0, 0, 0],
        [f32::NAN.to_bits(), 0, 0, 0],
    ]);
    let a = register(0x20, 0, 0);
    let b = register(0x20, 0, 1);
    let nan = register(0x20, 0, 2);
    let condition = register(0x02, 1, 0);
    let result = register(0x20, 1, 1);
    let program = script(
        &bytes,
        vec![
            Instruction::Binary {
                operation: 12,
                destination: condition,
                left: a,
                right: b,
            },
            store(condition, 0),
            Instruction::Binary {
                operation: 11,
                destination: condition,
                left: condition,
                right: Register::VOID,
            },
            store(condition, 1),
            Instruction::Select {
                destination: result,
                if_false: a,
                if_true: b,
                condition,
            },
            store(result, 2),
            Instruction::Binary {
                operation: 16,
                destination: condition,
                left: nan,
                right: nan,
            },
            store(condition, 3),
        ],
    )
    .compile(
        &[
            ValueType::Bool,
            ValueType::Bool,
            ValueType::Float,
            ValueType::Bool,
        ],
        &[],
    )
    .unwrap();
    let mut host = Fields {
        values: vec![Value::from_words(ValueType::Bool, [0; 4]); 4],
        ..Default::default()
    };
    program
        .execute(&mut program.new_state(), &mut host)
        .unwrap();
    assert_eq!(host.values[0].words()[0], u32::MAX);
    assert_eq!(host.values[1].words()[0], 0);
    assert_eq!(host.values[2], scalar(-2.0));
    assert_eq!(host.values[3].words()[0], 0);
}

#[test]
fn integer_bitwise_not_is_distinct_from_boolean_not_and_overflow_wraps() {
    let bytes = constants(&[[1, 0, 0, 0], [i32::MAX as u32, 0, 0, 0]]);
    let one = register(0x1a, 0, 0);
    let max = register(0x1a, 0, 1);
    let result = register(0x1a, 1, 0);
    let program = script(
        &bytes,
        vec![
            Instruction::Binary {
                operation: 11,
                destination: result,
                left: one,
                right: Register::VOID,
            },
            store(result, 0),
            Instruction::Binary {
                operation: 0,
                destination: result,
                left: max,
                right: one,
            },
            store(result, 1),
        ],
    )
    .compile(&[ValueType::Int; 2], &[])
    .unwrap();
    let mut host = Fields {
        values: vec![Value::from_words(ValueType::Int, [0; 4]); 2],
        ..Default::default()
    };
    program
        .execute(&mut program.new_state(), &mut host)
        .unwrap();
    assert_eq!(host.values[0].words()[0], !1u32);
    assert_eq!(host.values[1].words()[0], i32::MIN as u32);
}

#[test]
fn vector_construction_swizzle_literals_and_numeric_conversion_keep_distinct_meanings() {
    let bytes = constants(&[[2.0f32.to_bits(), 0, 0, 0]]);
    let source = register(0x20, 0, 0);
    let vector = register(0x22, 1, 0);
    let integer = register(0x1a, 1, 1);
    let program = script(
        &bytes,
        vec![
            Instruction::Construct {
                destination: vector,
                sources: vec![source],
            },
            store(vector, 0),
            Instruction::Swizzle {
                selectors: [0, 4 | (5 << 3), 0],
                destination: vector,
                source: vector,
            },
            store(vector, 1),
            Instruction::Convert {
                destination: integer,
                source,
            },
            store(integer, 2),
            Instruction::Reinterpret {
                destination: integer,
                source,
            },
            store(integer, 3),
        ],
    )
    .compile(
        &[
            ValueType::Float3,
            ValueType::Float3,
            ValueType::Int,
            ValueType::Int,
        ],
        &[],
    )
    .unwrap();
    let mut host = Fields {
        values: vec![scalar(0.0); 4],
        ..Default::default()
    };
    program
        .execute(&mut program.new_state(), &mut host)
        .unwrap();
    assert_eq!(
        host.values[0].words(),
        [2.0f32.to_bits(), 2.0f32.to_bits(), 2.0f32.to_bits(), 0]
    );
    assert_eq!(
        host.values[1].words(),
        [0, 1.0f32.to_bits(), 2.0f32.to_bits(), 0]
    );
    assert_eq!(host.values[2].words()[0], 2);
    assert_eq!(host.values[3].words()[0], 2.0f32.to_bits());
}

#[test]
fn calls_resolve_receiver_separately_from_context_and_continue_after_retirement_request() {
    let context = register(0, 3, 0);
    let result = register(0x20, 3, 1);
    let program = script(
        &[],
        vec![
            Instruction::Load {
                destination: context,
                external: 2,
            },
            Instruction::Call {
                flags: 0,
                receiver: -1,
                function: 0,
                result: Register::VOID,
                arguments: vec![Argument {
                    flags: 8,
                    register: context,
                }],
            },
            Instruction::Call {
                flags: 0,
                receiver: 1,
                function: 1,
                result,
                arguments: vec![Argument {
                    flags: 24,
                    register: context,
                }],
            },
            store(result, 3),
            Instruction::Call {
                flags: 2,
                receiver: -1,
                function: 1,
                result: Register::VOID,
                arguments: vec![Argument {
                    flags: 8,
                    register: context,
                }],
            },
        ],
    )
    .compile(
        &[
            ValueType::Context,
            ValueType::Context,
            ValueType::Context,
            ValueType::Float,
        ],
        &[
            Function {
                receiver: None,
                arguments: vec![ValueType::Context],
                result: None,
            },
            Function {
                receiver: Some(0),
                arguments: vec![ValueType::Context],
                result: Some(ValueType::Float),
            },
        ],
    )
    .unwrap();
    let mut host = Fields {
        values: vec![Value::from_words(ValueType::Context, [17, 0, 0, 0]); 4],
        ..Default::default()
    };
    program
        .execute(&mut program.new_state(), &mut host)
        .unwrap();
    assert!(host.killed);
    assert_eq!(host.values[3], scalar(9.0));
    assert_eq!(host.calls[0].1, None);
    assert_eq!(host.calls[1].1, Some(0));
    assert_eq!(host.calls[1].2[0].words()[0], 17);
    assert_eq!(host.calls[1].3, [24]);
    assert_eq!(host.calls[1].4, 0);
    assert_eq!(host.calls.len(), 3);
}

#[test]
fn persistent_stream_storage_survives_invocation_and_clear_reuses_particle() {
    let bytes = constants(&[[1.0f32.to_bits(), 0, 0, 0]]);
    let stream = register(0x20, 3, 0);
    let local = register(0x20, 1, 0);
    let one = register(0x20, 0, 0);
    let program = script(
        &bytes,
        vec![
            Instruction::Binary {
                operation: 0,
                destination: stream,
                left: stream,
                right: one,
            },
            Instruction::Binary {
                operation: 0,
                destination: local,
                left: local,
                right: one,
            },
            store(stream, 0),
            store(local, 1),
        ],
    )
    .compile(&[ValueType::Float; 2], &[])
    .unwrap();
    let mut state = program.new_state();
    let mut host = Fields {
        values: vec![scalar(0.0); 2],
        ..Default::default()
    };
    for _ in 0..3 {
        program.execute(&mut state, &mut host).unwrap();
    }
    assert_eq!(host.values, [scalar(3.0), scalar(1.0)]);
    state.clear();
    program.execute(&mut state, &mut host).unwrap();
    assert_eq!(host.values, [scalar(1.0), scalar(1.0)]);
}

#[test]
fn unary_functions_keep_negative_fraction_and_finite_boolean() {
    let bytes = constants(&[
        [(-1.25f32).to_bits(), 0, 0, 0],
        [f32::INFINITY.to_bits(), 0, 0, 0],
        [0; 4],
    ]);
    let input = register(0x20, 0, 0);
    let result = register(0x20, 1, 0);
    let boolean = register(0x02, 1, 1);
    let pair = register(0x21, 1, 2);
    let program = script(
        &bytes,
        vec![
            Instruction::UnaryFunction {
                function: 23,
                destination: result,
                source: input,
            },
            store(result, 0),
            Instruction::UnaryFunction {
                function: 51,
                destination: boolean,
                source: register(0x20, 0, 1),
            },
            store(boolean, 1),
            Instruction::UnaryFunction {
                function: 7,
                destination: pair,
                source: register(0x20, 0, 2),
            },
            store(pair, 2),
        ],
    )
    .compile(&[ValueType::Float, ValueType::Bool, ValueType::Float2], &[])
    .unwrap();
    let mut host = Fields {
        values: vec![scalar(0.0); 3],
        ..Default::default()
    };
    program
        .execute(&mut program.new_state(), &mut host)
        .unwrap();
    assert_eq!(host.values[0], scalar(-0.25));
    assert_eq!(host.values[1].words()[0], 0);
    assert_eq!(host.values[2].words(), [0, 1.0f32.to_bits(), 0, 0]);
}

#[test]
fn malformed_addresses_layouts_and_controls_are_rejected_before_execution() {
    let bytes = constants(&[[0; 4]]);
    let local = register(0x20, 1, 0);
    for destination in [
        register(0x20, 0, 0),
        register(0x20, 1, 100),
        register(0x20, 5, 0),
        register(0xff, 1, 0),
    ] {
        assert!(script(
            &bytes,
            vec![Instruction::Load {
                destination,
                external: 0
            }]
        )
        .compile(&[ValueType::Float], &[])
        .is_err());
    }
    assert!(script(
        &[],
        vec![Instruction::Load {
            destination: local,
            external: 0
        }]
    )
    .compile(&[ValueType::Int], &[])
    .is_err());
    let function = Function {
        receiver: None,
        arguments: vec![],
        result: None,
    };
    for receiver in [-2, 9] {
        assert!(script(
            &[],
            vec![Instruction::Call {
                flags: 0,
                receiver,
                function: 0,
                result: Register::VOID,
                arguments: vec![]
            }]
        )
        .compile(&[], from_ref(&function))
        .is_err());
    }
    let mut enormous = script(&[], vec![]);
    enormous.register_counts = [0, u32::MAX, 0, 0, 0];
    assert!(enormous.compile(&[], &[]).is_err());
    assert!(script(&[], vec![store(local, 0)])
        .compile_with_limits(
            &[ValueType::Float],
            &[],
            CompileLimits {
                storage_slots: 0,
                instructions: 100
            }
        )
        .is_err());
}

#[test]
fn runtime_errors_identify_native_offset_without_panics_or_silent_host_coercion() {
    let source = register(0x1a, 1, 0);
    let program = script(
        &[],
        vec![Instruction::Binary {
            operation: 3,
            destination: source,
            left: source,
            right: source,
        }],
    )
    .compile(&[], &[])
    .unwrap();
    let mut host = Fields::default();
    assert_eq!(
        program
            .execute(&mut program.new_state(), &mut host)
            .unwrap_err()
            .kind,
        Failure::IntegerDivisionByZero
    );
    let empty = script(&[], vec![]).compile(&[], &[]).unwrap();
    let mut empty_native = script(&[], vec![]);
    empty_native.register_counts = [0; 5];
    let tiny = empty_native.compile(&[], &[]).unwrap();
    assert_eq!(
        program
            .execute(&mut tiny.new_state(), &mut host)
            .unwrap_err()
            .kind,
        Failure::IncompatibleState
    );
    let program = script(
        &[],
        vec![Instruction::Load {
            destination: register(0x20, 1, 0),
            external: 0,
        }],
    )
    .compile(&[ValueType::Float], &[])
    .unwrap();
    let mut host = Fields {
        values: vec![Value::from_words(ValueType::Int, [0; 4])],
        ..Default::default()
    };
    assert_eq!(
        program
            .execute(&mut State::for_programs([&program, &empty]), &mut host)
            .unwrap_err()
            .kind,
        Failure::HostType
    );
}
