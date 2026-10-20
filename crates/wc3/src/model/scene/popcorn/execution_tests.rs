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
