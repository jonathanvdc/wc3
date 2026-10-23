//! Include as a private `native.rs` test module. Actual generated code, not an ABI stand-in.
use super::*;
use crate::model::scene::popcorn::{
    Address, Destination, Failure, Instruction, Operation, Source, Value, ValueType,
};

#[derive(Debug, PartialEq, Eq)]
struct HostFault(u32);
#[derive(Default)]
struct GenericHost {
    stores: Vec<u32>,
    loads: usize,
}
impl Host for GenericHost {
    type Error = HostFault;
    fn load(&mut self, _: u16) -> Result<Value, Self::Error> {
        self.loads += 1;
        Err(HostFault(37))
    }
    fn store(&mut self, _: u16, value: Value) -> Result<(), Self::Error> {
        self.stores.push(value.words()[0]);
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
        unreachable!()
    }
}
#[test]
fn generated_generic_host_error_retains_prefix_without_replay() {
    let source = Source {
        kind: ValueType::Int,
        address: Address::Slot { scope: 2, slot: 0 },
    };
    let destination = Destination {
        kind: ValueType::Int,
        scope: 2,
        slot: 0,
    };
    let program = Program {
        counts: [0, 0, 1, 0],
        max_arguments: 0,
        operations: vec![
            Operation {
                offset: 5,
                instruction: Instruction::Reinterpret {
                    destination,
                    source: Source {
                        kind: ValueType::Int,
                        address: Address::Constant([123, 0, 0, 0]),
                    },
                },
            },
            Operation {
                offset: 11,
                instruction: Instruction::Store {
                    external: 0,
                    source,
                },
            },
            Operation {
                offset: 29,
                instruction: Instruction::Load {
                    external: 1,
                    destination,
                },
            },
            Operation {
                offset: 41,
                instruction: Instruction::Store {
                    external: 0,
                    source,
                },
            },
        ],
    };
    let builder =
        JITBuilder::with_flags(&[("opt_level", "speed")], default_libcall_names()).unwrap();
    let mut owner = ModuleOwner(Some(JITModule::new(builder)));
    let module = owner.0.as_mut().unwrap();
    let mut context = module.make_context();
    codegen::lower(module, &program, &mut context);
    let id = module
        .declare_function(
            "generic_failure_prefix",
            Linkage::Local,
            &context.func.signature,
        )
        .unwrap();
    module.define_function(id, &mut context).unwrap();
    module.finalize_definitions().unwrap();
    // SAFETY: codegen emitted exactly this private ABI; owner lives beyond run.
    let entry = unsafe { transmute::<*const u8, Entry>(module.get_finalized_function(id)) };
    let mut native_state = program.new_state();
    let mut native_host = GenericHost::default();
    let result = bridge::run(entry, &program, &mut native_state, &mut native_host);
    assert_eq!(
        result,
        Err(ExecutionError {
            offset: 29,
            kind: Failure::Host(HostFault(37))
        })
    );
    assert_eq!(
        native_host.stores,
        [123],
        "earlier host store must never be replayed"
    );
    assert_eq!(native_host.loads, 1);
    assert_eq!(native_state.scopes[2][0], [123, 0, 0, 0]);
    let mut scalar_state = program.new_state();
    let mut scalar_host = GenericHost::default();
    assert_eq!(program.execute(&mut scalar_state, &mut scalar_host), result);
    assert_eq!(scalar_state.scopes, native_state.scopes);
    assert_eq!(scalar_host.stores, native_host.stores);
    assert_eq!(scalar_host.loads, native_host.loads);
}
