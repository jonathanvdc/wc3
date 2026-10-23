use super::*;
use crate::model::scene::popcorn::{
    Address, Destination, Instruction, Operation, Source, Value, ValueType,
};

#[derive(Default)]
struct Recorder {
    writes: Vec<u32>,
    fail: bool,
    panic: bool,
}
impl Host for Recorder {
    type Error = &'static str;
    fn load(&mut self, _: u16) -> Result<Value, Self::Error> {
        if self.panic {
            panic!("bridge host panic");
        }
        if self.fail {
            return Err("bridge host failure");
        }
        Ok(Value::from_words(ValueType::Float, [7; 4]))
    }
    fn store(&mut self, _: u16, value: Value) -> Result<(), Self::Error> {
        self.writes.push(value.words()[0]);
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
fn program() -> Program {
    Program {
        counts: [1, 1, 1, 1],
        max_arguments: 3,
        operations: vec![
            Operation {
                offset: 11,
                instruction: Instruction::Store {
                    external: 0,
                    source: Source {
                        kind: ValueType::Float,
                        address: Address::Slot { scope: 2, slot: 0 },
                    },
                },
            },
            Operation {
                offset: 29,
                instruction: Instruction::Load {
                    external: 0,
                    destination: Destination {
                        kind: ValueType::Float,
                        scope: 3,
                        slot: 0,
                    },
                },
            },
            Operation {
                offset: 41,
                instruction: Instruction::Store {
                    external: 0,
                    source: Source {
                        kind: ValueType::Float,
                        address: Address::Slot { scope: 3, slot: 0 },
                    },
                },
            },
        ],
    }
}
// Original ABI stand-in: checks initialized locals, writes one persistent native
// word, then dispatches checked services. This is bridge proof, not JIT coverage.
unsafe extern "C" fn entry(raw: *mut Invocation) -> u32 {
    let invocation = unsafe { &mut *raw };
    if unsafe { *invocation.scopes[0] != 0 || *invocation.scopes[1] != 0 } {
        return 1;
    }
    unsafe {
        *invocation.scopes[2] = 123;
    }
    for index in 0..3 {
        let status = unsafe { (invocation.service)(invocation.context, index) };
        if status != 0 {
            return status;
        }
    }
    0
}
#[test]
fn native_buffer_writes_and_services_share_storage_and_exact_failure_prefix() {
    let program = program();
    for fail in [false, true] {
        let mut state = program.new_state();
        state.scopes[0][0] = [99; 4];
        state.scopes[1][0] = [98; 4];
        state.scopes[3][0] = [77; 4];
        state
            .arguments
            .push(Value::from_words(ValueType::Int, [9; 4]));
        let args = state.arguments.clone();
        let pointers = state.scopes.each_ref().map(|s| s.as_ptr());
        let mut host = Recorder {
            fail,
            ..Default::default()
        };
        let result = run(entry, &program, &mut state, &mut host);
        assert_eq!(state.scopes.each_ref().map(|s| s.as_ptr()), pointers);
        assert_eq!(state.arguments, args);
        if fail {
            assert_eq!(
                result,
                Err(ExecutionError {
                    offset: 29,
                    kind: Failure::Host("bridge host failure")
                })
            );
            assert_eq!(host.writes, [123]);
            assert_eq!(state.scopes[3][0], [77; 4]);
        } else {
            assert_eq!(result, Ok(()));
            assert_eq!(host.writes, [123, 7]);
            assert_eq!(state.scopes[3][0], [7, 0, 0, 0]);
        }
    }
}
#[test]
fn incompatible_storage_does_not_reset_or_enter_generated_code() {
    let program = program();
    let mut state = program.new_state();
    state.scopes[0][0] = [91; 4];
    state.scopes[3].clear();
    let mut host = Recorder::default();
    assert_eq!(
        run(entry, &program, &mut state, &mut host),
        Err(ExecutionError {
            offset: 0,
            kind: Failure::IncompatibleState
        })
    );
    assert_eq!(state.scopes[0][0], [91; 4]);
    assert!(host.writes.is_empty());
    let mut state = program.new_state();
    state.arguments = Vec::new();
    assert!(matches!(
        run(entry, &program, &mut state, &mut host),
        Err(ExecutionError {
            kind: Failure::IncompatibleState,
            ..
        })
    ));
    assert!(host.writes.is_empty());
}
#[test]
fn host_panic_is_captured_by_service_and_resumed_after_native_returns() {
    let program = program();
    let mut state = program.new_state();
    state.scopes[3][0] = [71; 4];
    let mut host = Recorder {
        panic: true,
        ..Default::default()
    };
    let result = catch_unwind(AssertUnwindSafe(|| {
        run(entry, &program, &mut state, &mut host)
    }));
    let payload = result.unwrap_err();
    assert_eq!(payload.downcast_ref::<&str>(), Some(&"bridge host panic"));
    assert_eq!(host.writes, [123]);
    assert_eq!(state.scopes[3][0], [71; 4]);
}
