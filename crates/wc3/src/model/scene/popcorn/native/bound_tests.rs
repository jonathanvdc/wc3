use super::super::{codegen, Entry, ModuleOwner};
use super::*;
use crate::model::formats::pkb::{Document, EffectCompileOptions};
use crate::model::scene::popcorn::graph::{BoundProgram, EffectField, External};
use crate::model::scene::popcorn::{
    Address, Destination, Instruction, Operation, Source, Value, ValueType,
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module};
use std::mem::transmute;
use std::sync::Arc;

#[derive(Default)]
struct CheckedHost {
    calls: Vec<u16>,
    nonfinite: bool,
}
impl Host for CheckedHost {
    type Error = &'static str;
    fn load(&mut self, slot: u16) -> Result<Value, Self::Error> {
        self.calls.push(slot);
        if slot == 2 && self.nonfinite {
            return Err("effect callback produced a nonfinite scalar");
        }
        Ok(if slot == 0 {
            Value::from_words(ValueType::Float3, [31, 32, 33, 0])
        } else {
            Value::from_words(ValueType::Float, [17, 0, 0, 0])
        })
    }
    fn store(&mut self, slot: u16, _: Value) -> Result<(), Self::Error> {
        self.calls.push(slot);
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
fn compile(
    bound: &BoundProgram,
    fields: &[EffectField],
    layout: &FieldLayout,
) -> (ModuleOwner, Entry) {
    let builder =
        JITBuilder::with_flags(&[("opt_level", "speed")], default_libcall_names()).unwrap();
    let mut owner = ModuleOwner(Some(JITModule::new(builder)));
    let module = owner.0.as_mut().unwrap();
    let mut context = module.make_context();
    let mut budget = codegen::CURVE_INLINE_BUDGET;
    codegen::lower_bound(
        module,
        bound,
        fields,
        layout,
        &[],
        &mut budget,
        &mut context,
    );
    let id = module
        .declare_function("bound_fields", Linkage::Local, &context.func.signature)
        .unwrap();
    module.define_function(id, &mut context).unwrap();
    module.finalize_definitions().unwrap();
    // SAFETY: same private ABI; returned owner must remain alive through run.
    let entry = unsafe { transmute::<*const u8, Entry>(module.get_finalized_function(id)) };
    (owner, entry)
}
#[test]
fn bound_fields_zero_union_padding_and_nonfinite_time_retains_direct_prefix() {
    let document = Document::read(include_bytes!(
        "../../../../../benches/fixtures/particle_runtime/burst-1.pkb"
    ))
    .unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    let mut effect = document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap();
    let graph = Arc::get_mut(&mut effect.inner).unwrap();
    // Deliberately wider same-index field in another layer exercises physical
    // union planes; layer1's scalar store must clear these higher components.
    let wider = graph.layers[1].fields[3].clone();
    let mut fields = graph.layers[0].fields.to_vec();
    fields.resize(4, wider);
    graph.layers[0].fields = fields.into_boxed_slice();
    graph.layers[0].fields[3].value_type = Some(ValueType::Float4);
    let layout = FieldLayout::for_graph(graph).unwrap();
    let fields = &graph.layers[1].fields;
    assert_eq!(fields[2].value_type, Some(ValueType::Float3));
    assert_eq!(fields[3].value_type, Some(ValueType::Float));
    let bound = BoundProgram {
        program: Program {
            counts: [0, 0, 3, 0],
            max_arguments: 0,
            operations: vec![
                Operation {
                    offset: 5,
                    instruction: Instruction::Store {
                        external: 1,
                        source: Source {
                            kind: ValueType::Float,
                            address: Address::Constant([7, 0, 0, 0]),
                        },
                    },
                },
                Operation {
                    offset: 11,
                    instruction: Instruction::Load {
                        external: 0,
                        destination: Destination {
                            kind: ValueType::Float3,
                            scope: 2,
                            slot: 0,
                        },
                    },
                },
                Operation {
                    offset: 29,
                    instruction: Instruction::Load {
                        external: 2,
                        destination: Destination {
                            kind: ValueType::Float,
                            scope: 2,
                            slot: 1,
                        },
                    },
                },
                Operation {
                    offset: 41,
                    instruction: Instruction::Load {
                        external: 3,
                        destination: Destination {
                            kind: ValueType::Float,
                            scope: 2,
                            slot: 2,
                        },
                    },
                },
            ],
        },
        externals: vec![
            External::Field(2),
            External::Field(3),
            External::Time,
            External::Delta,
        ]
        .into_boxed_slice(),
        callbacks: Box::new([]),
    };
    let (_owner, entry) = compile(&bound, fields, &layout);
    let capacity = 3;
    let planes = layout
        .component_ordinals()
        .iter()
        .flatten()
        .copied()
        .filter(|o| *o != FieldLayout::ABSENT)
        .max()
        .unwrap()
        + 1;
    for nonfinite in [false, true] {
        let mut data = vec![99; planes * capacity];
        let slot = 1;
        for (ordinal, word) in layout.component_ordinals()[2].iter().zip([11, 12, 13]) {
            data[*ordinal * capacity + slot] = word;
        }
        let mut state = bound.program.new_state();
        state.scopes[2][1] = [77, 0, 0, 0];
        let mut host = CheckedHost {
            nonfinite,
            ..Default::default()
        };
        let inputs = DirectInputs {
            fields: data.as_mut_ptr(),
            stride: capacity,
            slot,
            time: if nonfinite {
                f32::INFINITY.to_bits()
            } else {
                0.25f32.to_bits()
            },
            delta: 0.125f32.to_bits(),
        };
        let result = run_with_inputs(entry, &bound.program, &mut state, &mut host, inputs);
        assert_eq!(state.scopes[2][0], [11, 12, 13, 0]);
        for (component, ordinal) in layout.component_ordinals()[3].iter().enumerate() {
            assert_eq!(
                data[*ordinal * capacity + slot],
                if component == 0 { 7 } else { 0 }
            );
            assert_eq!(data[*ordinal * capacity], 99, "neighbor slot untouched");
        }
        if nonfinite {
            assert_eq!(
                result,
                Err(ExecutionError {
                    offset: 29,
                    kind: Failure::Host("effect callback produced a nonfinite scalar")
                })
            );
            assert_eq!(host.calls, [2]);
            assert_eq!(state.scopes[2][1], [77, 0, 0, 0]);
        } else {
            assert_eq!(result, Ok(()));
            assert!(host.calls.is_empty());
            assert_eq!(state.scopes[2][1], [0.25f32.to_bits(), 0, 0, 0]);
            assert_eq!(state.scopes[2][2], [0.125f32.to_bits(), 0, 0, 0]);
        }
    }
    let mut state = bound.program.new_state();
    let mut host = CheckedHost::default();
    assert_eq!(run(entry, &bound.program, &mut state, &mut host), Ok(()));
    assert_eq!(
        host.calls,
        [1, 0, 2, 3],
        "generic host must not use canonical binding shortcuts"
    );
    assert_eq!(state.scopes[2][0], [31, 32, 33, 0]);
}
