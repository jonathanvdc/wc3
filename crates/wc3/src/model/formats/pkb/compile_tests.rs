use super::*;
use crate::model::formats::pkb::LocatedInstruction;

fn register(bank: u8, slot: u16) -> Register {
    Register((u32::from(bank) << 24) | (1 << 16) | u32::from(slot))
}
fn script(instruction: NativeInstruction) -> Script<'static> {
    Script {
        reserved: [0; 2],
        register_counts: [0, 4, 4, 4, 4],
        constants: &[],
        instructions: vec![LocatedInstruction {
            offset: 77,
            instruction,
        }],
        padding: &[],
    }
}
#[test]
fn native_basic_selectors_lower_to_typed_operations_at_original_offsets() {
    for (selector, operation) in [
        (0, BinaryOperation::Add),
        (1, BinaryOperation::Subtract),
        (2, BinaryOperation::Multiply),
        (3, BinaryOperation::Divide),
        (4, BinaryOperation::Remainder),
        (8, BinaryOperation::BitAnd),
        (9, BinaryOperation::BitOr),
        (12, BinaryOperation::Less),
        (13, BinaryOperation::LessEqual),
        (14, BinaryOperation::Greater),
        (15, BinaryOperation::GreaterEqual),
        (16, BinaryOperation::Equal),
    ] {
        let output = if selector >= 12 { 0x02 } else { 0x1a };
        let program = script(NativeInstruction::Binary {
            destination: register(output, 0),
            left: register(0x1a, 1),
            right: register(0x1a, 2),
            operation: selector,
        })
        .compile(&[], &[])
        .unwrap();
        assert_eq!(program.operations[0].offset, 77);
        let Instruction::Binary {
            operation: actual, ..
        } = program.operations[0].instruction
        else {
            panic!("typed binary");
        };
        assert_eq!(actual, operation);
    }
    for (selector, operation) in [
        (8, BinaryOperation::LogicalAnd),
        (9, BinaryOperation::LogicalOr),
    ] {
        let program = script(NativeInstruction::Binary {
            destination: register(0x02, 0),
            left: register(0x02, 1),
            right: register(0x02, 2),
            operation: selector,
        })
        .compile(&[], &[])
        .unwrap();
        let Instruction::Binary {
            operation: actual, ..
        } = program.operations[0].instruction
        else {
            panic!("typed logical");
        };
        assert_eq!(actual, operation);
    }
}
#[test]
fn native_unary_forms_normalize_without_reading_the_unused_right_operand() {
    for (selector, bank, operation) in [
        (5, 0x20, UnaryOperation::Negate),
        (5, 0x1a, UnaryOperation::Negate),
        (11, 0x02, UnaryOperation::LogicalNot),
        (11, 0x1a, UnaryOperation::BitNot),
    ] {
        let program = script(NativeInstruction::Binary {
            destination: register(bank, 0),
            left: register(bank, 1),
            right: Register(u32::MAX),
            operation: selector,
        })
        .compile(&[], &[])
        .unwrap();
        let Instruction::Unary {
            operation: actual,
            source,
            ..
        } = program.operations[0].instruction
        else {
            panic!("normalized unary");
        };
        assert_eq!(actual, operation);
        assert!(matches!(
            source.address,
            Address::Slot { scope: 1, slot: 1 }
        ));
        assert_eq!(program.operations[0].offset, 77);
    }
}
#[test]
fn mathematical_function_selectors_share_the_same_typed_alu_forms() {
    for (selector, output, operation) in [
        (0, 0x20, UnaryOperation::SquareRoot),
        (1, 0x20, UnaryOperation::ReciprocalSquareRoot),
        (7, 0x21, UnaryOperation::SineCosine),
        (13, 0x20, UnaryOperation::Exponential),
        (17, 0x20, UnaryOperation::Reciprocal),
        (23, 0x20, UnaryOperation::SignedFraction),
        (24, 0x20, UnaryOperation::Saturate),
        (51, 0x02, UnaryOperation::IsFinite),
    ] {
        let program = script(NativeInstruction::UnaryFunction {
            destination: register(output, 0),
            source: register(0x20, 1),
            function: selector,
        })
        .compile(&[], &[])
        .unwrap();
        let Instruction::Unary {
            operation: actual, ..
        } = program.operations[0].instruction
        else {
            panic!("typed unary function");
        };
        assert_eq!(actual, operation);
    }
    for (selector, operation) in [
        (27, BinaryOperation::Minimum),
        (28, BinaryOperation::Maximum),
        (29, BinaryOperation::Dot),
    ] {
        let program = script(NativeInstruction::BinaryFunction {
            destination: register(0x20, 0),
            left: register(0x20, 1),
            right: register(0x20, 2),
            function: selector,
        })
        .compile(&[], &[])
        .unwrap();
        let Instruction::Binary {
            operation: actual, ..
        } = program.operations[0].instruction
        else {
            panic!("typed binary function");
        };
        assert_eq!(actual, operation);
    }
    let error = script(NativeInstruction::Binary {
        destination: register(0x20, 0),
        left: register(0x20, 1),
        right: register(0x20, 2),
        operation: 99,
    })
    .compile(&[], &[])
    .unwrap_err();
    assert_eq!(error.offset, 77);
    assert_eq!(error.message, "unsupported basic-operation selector");
}
