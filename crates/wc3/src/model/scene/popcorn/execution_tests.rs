use super::*;
use crate::model::scene::popcorn::{Operation, ValueType};
struct Reject;
impl Host for Reject {
    type Error = &'static str;
    fn load(&mut self, _: u16) -> Result<Value, Self::Error> {
        Err("rejected")
    }
    fn store(&mut self, _: u16, _: Value) -> Result<(), Self::Error> {
        Ok(())
    }
    fn call(
        &mut self,
        _: u16,
        _: Option<u16>,
        _: &[Value],
        _: &[u8],
        _: u8,
    ) -> Result<Option<Value>, Self::Error> {
        Ok(None)
    }
}
#[test]
fn split_storage_preserves_individual_persistent_registers_and_restores_on_error() {
    let mut program = Program {
        counts: [4, 3, 2, 1],
        max_arguments: 7,
        operations: vec![],
    };
    let mut first = State::persistent_for_programs([&program]);
    let mut second = State::persistent_for_programs([&program]);
    let mut scratch = State::scratch_for_programs([&program]);
    first.scopes[2][0] = [123; 4];
    second.scopes[2][0] = [456; 4];
    let scratch_pointer = scratch.scopes[0].as_ptr();
    program.operations.push(Operation {
        offset: 19,
        instruction: Instruction::Load {
            destination: Destination {
                scope: 0,
                slot: 0,
                kind: ValueType::Float,
            },
            external: 0,
        },
    });
    let error = program
        .execute_with_scratch(&mut first, &mut scratch, &mut Reject)
        .unwrap_err();
    assert_eq!(error.offset, 19);
    assert_eq!(error.kind, Failure::Host("rejected"));
    assert_eq!(scratch.scopes[0].as_ptr(), scratch_pointer);
    assert!(first.scopes[0].is_empty());
    assert_eq!(first.arguments.capacity(), 0);
    assert_eq!(first.scopes[2][0], [123; 4]);
    program.operations.clear();
    program
        .execute_with_scratch(&mut second, &mut scratch, &mut Reject)
        .unwrap();
    assert_eq!(second.scopes[2][0], [456; 4]);
    assert_eq!(scratch.scopes[0].as_ptr(), scratch_pointer);
}

#[test]
fn typed_logical_and_bitwise_operations_preserve_distinct_word_contracts() {
    assert_eq!(
        unary(UnaryOperation::LogicalNot, 0x8000_0000, Family::Bool),
        0
    );
    assert_eq!(unary(UnaryOperation::LogicalNot, 0, Family::Bool), u32::MAX);
    assert_eq!(
        unary(UnaryOperation::BitNot, 0x8000_0000, Family::Int),
        0x7fff_ffff
    );
    assert_eq!(
        binary(BinaryOperation::LogicalAnd, 1, 2, Family::Bool),
        Some(u32::MAX)
    );
    assert_eq!(binary(BinaryOperation::BitAnd, 1, 2, Family::Int), Some(0));
    assert_eq!(
        binary(BinaryOperation::LogicalOr, 1, 2, Family::Bool),
        Some(u32::MAX)
    );
    assert_eq!(binary(BinaryOperation::BitOr, 1, 2, Family::Int), Some(3));
    assert_eq!(
        unary(UnaryOperation::Negate, i32::MIN as u32, Family::Int),
        i32::MIN as u32
    );
    assert_eq!(
        unary(
            UnaryOperation::SignedFraction,
            (-1.25f32).to_bits(),
            Family::Float
        ),
        (-0.25f32).to_bits()
    );
}

#[test]
fn typed_integer_failure_keeps_persistent_write_prefix_and_original_offset() {
    let source = |word| Source {
        kind: ValueType::Int,
        address: Address::Constant([word, 0, 0, 0]),
    };
    let first = Destination {
        scope: 2,
        slot: 0,
        kind: ValueType::Int,
    };
    let second = Destination {
        scope: 3,
        slot: 0,
        kind: ValueType::Int,
    };
    let program = Program {
        counts: [0, 0, 1, 1],
        max_arguments: 0,
        operations: vec![
            Operation {
                offset: 17,
                instruction: Instruction::Unary {
                    destination: first,
                    source: source(7),
                    operation: UnaryOperation::Negate,
                },
            },
            Operation {
                offset: 91,
                instruction: Instruction::Binary {
                    destination: second,
                    left: source(1),
                    right: source(0),
                    operation: BinaryOperation::Divide,
                },
            },
            Operation {
                offset: 101,
                instruction: Instruction::Unary {
                    destination: first,
                    source: source(9),
                    operation: UnaryOperation::Negate,
                },
            },
        ],
    };
    let mut state = program.new_state();
    let error = program.execute(&mut state, &mut Reject).unwrap_err();
    assert_eq!(error.offset, 91);
    assert_eq!(error.kind, Failure::IntegerDivisionByZero);
    assert_eq!(state.scopes[2][0], [(-7i32) as u32, 0, 0, 0]);
    assert_eq!(state.scopes[3][0], [0; 4]);
}

#[test]
fn typed_reduction_and_pair_operations_preserve_result_shapes_and_padding() {
    let constant = |kind, words| Source {
        kind,
        address: Address::Constant(words),
    };
    let program = Program {
        counts: [0, 0, 0, 2],
        max_arguments: 0,
        operations: vec![
            Operation {
                offset: 7,
                instruction: Instruction::Binary {
                    destination: Destination {
                        scope: 3,
                        slot: 0,
                        kind: ValueType::Float,
                    },
                    left: constant(ValueType::Float3, [1., 2., 3., 99.].map(f32::to_bits)),
                    right: constant(ValueType::Float3, [4., 5., 6., 99.].map(f32::to_bits)),
                    operation: BinaryOperation::Dot,
                },
            },
            Operation {
                offset: 19,
                instruction: Instruction::Unary {
                    destination: Destination {
                        scope: 3,
                        slot: 1,
                        kind: ValueType::Float2,
                    },
                    source: constant(ValueType::Float, [0, 123, 456, 789]),
                    operation: UnaryOperation::SineCosine,
                },
            },
        ],
    };
    let mut state = program.new_state();
    program.execute(&mut state, &mut Reject).unwrap();
    assert_eq!(state.scopes[3][0], [32f32.to_bits(), 0, 0, 0]);
    assert_eq!(state.scopes[3][1], [0, 1f32.to_bits(), 0, 0]);
    assert_eq!(
        binary(BinaryOperation::Add, i32::MAX as u32, 1, Family::Int),
        Some(i32::MIN as u32)
    );
    assert_eq!(
        binary(
            BinaryOperation::Divide,
            i32::MIN as u32,
            (-1i32) as u32,
            Family::Int
        ),
        Some(i32::MIN as u32)
    );
    assert_eq!(
        binary(
            BinaryOperation::Remainder,
            i32::MIN as u32,
            (-1i32) as u32,
            Family::Int
        ),
        Some(0)
    );
}
