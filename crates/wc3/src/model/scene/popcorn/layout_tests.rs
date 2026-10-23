use super::*;
use crate::model::scene::popcorn::{
    BinaryOperation, Host, Operation, State, UnaryOperation, Value, ValueType,
};

type Call = (u16, Option<u16>, Vec<Value>, Vec<u8>, u8);
#[derive(Default, Debug, PartialEq)]
struct Trace {
    writes: Vec<(u16, Value)>,
    calls: Vec<Call>,
}
impl Host for Trace {
    type Error = &'static str;
    fn load(&mut self, external: u16) -> Result<Value, Self::Error> {
        match external {
            0 => Ok(1.5f32.into()),
            1 => Ok(Value::from_words(
                ValueType::Float4,
                [1., 2., 3., 4.].map(f32::to_bits),
            )),
            3 => Ok(true.into()),
            _ => Err("missing input"),
        }
    }
    fn store(&mut self, external: u16, value: Value) -> Result<(), Self::Error> {
        self.writes.push((external, value));
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
        Ok(arguments.first().copied())
    }
}
fn slot(scope: usize, slot: usize, kind: ValueType) -> Source {
    Source {
        address: Address::Slot { scope, slot },
        kind,
    }
}
fn output(scope: usize, slot: usize, kind: ValueType) -> Destination {
    Destination { scope, slot, kind }
}
fn constant(value: Value) -> Source {
    Source {
        address: Address::Constant(value.words()),
        kind: value.kind(),
    }
}
fn programme(instructions: Vec<Instruction>) -> Program {
    Program {
        counts: [256, 128, 2048, 4096],
        max_arguments: 2,
        operations: instructions
            .into_iter()
            .enumerate()
            .map(|(index, instruction)| Operation {
                offset: index * 17,
                instruction,
            })
            .collect(),
    }
}
fn pair() -> (Program, Program) {
    use ValueType::{Bool, Float, Float3, Float4, Int};
    let init = programme(vec![
        Instruction::Load {
            destination: output(2, 900, Float),
            external: 0,
        },
        Instruction::Load {
            destination: output(3, 1000, Float4),
            external: 1,
        },
        Instruction::Store {
            source: slot(2, 999, Float),
            external: 2,
        },
    ]);
    let evolve = programme(vec![
        Instruction::Store {
            source: slot(3, 1000, Float4),
            external: 4,
        },
        Instruction::Load {
            destination: output(0, 80, Bool),
            external: 3,
        },
        Instruction::Binary {
            destination: output(2, 900, Float),
            left: slot(2, 900, Float),
            right: constant(0.25f32.into()),
            operation: BinaryOperation::Add,
        },
        Instruction::Convert {
            destination: output(1, 30, Int),
            source: slot(2, 900, Float),
        },
        Instruction::Reinterpret {
            destination: output(1, 31, Float),
            source: slot(1, 30, Int),
        },
        Instruction::Construct {
            destination: output(3, 1010, Float4),
            sources: vec![
                slot(2, 900, Float),
                constant(2.0f32.into()),
                constant(3.0f32.into()),
                constant(4.0f32.into()),
            ],
        },
        Instruction::Swizzle {
            destination: output(1, 40, Float3),
            source: slot(3, 1010, Float4),
            lanes: [2, 1, 0, 0],
        },
        Instruction::Unary {
            destination: output(1, 41, Float3),
            source: slot(1, 40, Float3),
            operation: UnaryOperation::Saturate,
        },
        Instruction::Binary {
            destination: output(1, 42, Float3),
            left: slot(1, 41, Float3),
            right: constant(Value::from_words(Float3, [0.5f32.to_bits(); 4])),
            operation: BinaryOperation::Minimum,
        },
        Instruction::Select {
            destination: output(3, 1000, Float4),
            if_false: constant(Value::from_words(Float4, [0; 4])),
            if_true: slot(3, 1010, Float4),
            condition: slot(0, 80, Bool),
        },
        Instruction::Call {
            destination: Some(output(1, 90, Float4)),
            result_kind: Some(Float4),
            receiver: Some(5),
            function: 3,
            arguments: vec![slot(3, 1000, Float4), slot(1, 42, Float3)],
            controls: vec![4, 7],
            flags: 6,
        },
        Instruction::Store {
            source: slot(1, 90, Float4),
            external: 6,
        },
        Instruction::Store {
            source: slot(2, 900, Float),
            external: 7,
        },
        Instruction::Store {
            source: slot(1, 31, Float),
            external: 8,
        },
    ]);
    (init, evolve)
}
fn execute(init: &Program, evolve: &Program) -> Trace {
    let mut state = State::for_programs([init, evolve]);
    let mut trace = Trace::default();
    for rounds in [4, 2] {
        state.clear();
        init.execute(&mut state, &mut trace).unwrap();
        for _ in 0..rounds {
            evolve.execute(&mut state, &mut trace).unwrap();
        }
    }
    trace
}
#[test]
fn dense_layout_preserves_aliases_cross_stage_state_types_and_host_effects() {
    let (mut init, mut evolve) = pair();
    let expected = execute(&init, &evolve);
    let original_init = init.clone();
    let original_evolve = evolve.clone();
    compact_registers([&mut init, &mut evolve]);
    assert_eq!(init.counts, [1, 6, 2, 2]);
    assert_eq!(evolve.counts, init.counts);
    assert_eq!(execute(&init, &evolve), expected);
    assert_eq!(original_init.counts, [256, 128, 2048, 4096]);
    assert_eq!(
        original_evolve.instruction_count(),
        evolve.instruction_count()
    );
    assert_eq!(
        original_evolve
            .operations
            .iter()
            .map(|op| op.offset)
            .collect::<Vec<_>>(),
        evolve
            .operations
            .iter()
            .map(|op| op.offset)
            .collect::<Vec<_>>()
    );
    assert_eq!(expected.writes[0], (2, 0.0f32.into()));
    assert_eq!(expected.calls.len(), 6);
}
#[test]
fn compaction_keeps_checked_failure_locations_and_preceding_writes() {
    let (mut init, _) = pair();
    init.operations.push(Operation {
        offset: 0x221,
        instruction: Instruction::Load {
            destination: output(1, 100, ValueType::Float),
            external: 99,
        },
    });
    let original = init.clone();
    compact_registers([&mut init]);
    let mut before = Trace::default();
    let mut after = Trace::default();
    let expected = original
        .execute(&mut original.new_state(), &mut before)
        .unwrap_err();
    let actual = init.execute(&mut init.new_state(), &mut after).unwrap_err();
    assert_eq!(actual, expected);
    assert_eq!(actual.offset, 0x221);
    assert_eq!(after, before);
}
#[test]
fn layers_compact_independently_and_empty_programmes_need_no_registers() {
    let (mut first, _) = pair();
    let mut second = programme(vec![Instruction::Store {
        source: slot(2, 900, ValueType::Float),
        external: 0,
    }]);
    compact_registers([&mut first]);
    compact_registers([&mut second]);
    assert_eq!(second.counts, [0, 0, 1, 0]);
    assert_ne!(first.counts, second.counts);
    let mut empty = programme(vec![]);
    compact_registers([&mut empty]);
    assert_eq!(empty.counts, [0; 4]);
    empty
        .execute(&mut empty.new_state(), &mut Trace::default())
        .unwrap();
}
