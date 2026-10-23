use super::super::{codegen, Entry, ModuleOwner};
use super::*;
use crate::model::formats::pkb::{Document, EffectCompileOptions};
use crate::model::scene::popcorn::curve::sample;
use crate::model::scene::popcorn::graph::{
    BoundProgram, Callback, EffectGraph, EffectSampler, External,
};
use crate::model::scene::popcorn::{
    Address, Destination, Instruction, Operation, Source, Value, ValueType,
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module};
use std::mem::transmute;

struct CurveHost<'a> {
    sampler: &'a EffectSampler,
    stores: Vec<u32>,
    samples: usize,
}
impl Host for CurveHost<'_> {
    type Error = &'static str;
    fn load(&mut self, _: u16) -> Result<Value, Self::Error> {
        unreachable!()
    }
    fn store(&mut self, _: u16, value: Value) -> Result<(), Self::Error> {
        self.stores.push(value.words()[0]);
        Ok(())
    }
    fn call(
        &mut self,
        _: u16,
        _: Option<u16>,
        values: &[Value],
        _: &[u8],
        _: u8,
    ) -> Result<Option<Value>, Self::Error> {
        self.samples += 1;
        sample(self.sampler, f32::from_bits(values[0].words()[0])).map(Some)
    }
}
fn compile(
    bound: &BoundProgram,
    graph: &EffectGraph,
    samplers: &[EffectSampler],
    budget: usize,
) -> (ModuleOwner, Entry, FieldLayout) {
    let layout = FieldLayout::for_graph(graph).unwrap();
    let builder =
        JITBuilder::with_flags(&[("opt_level", "speed")], default_libcall_names()).unwrap();
    let mut owner = ModuleOwner(Some(JITModule::new(builder)));
    let module = owner.0.as_mut().unwrap();
    let mut context = module.make_context();
    let mut remaining = budget;
    codegen::lower_bound(
        module,
        bound,
        &graph.layers[1].fields,
        &layout,
        samplers,
        &mut remaining,
        &mut context,
    );
    let id = module
        .declare_function("bound_curve", Linkage::Local, &context.func.signature)
        .unwrap();
    module.define_function(id, &mut context).unwrap();
    module.finalize_definitions().unwrap();
    // SAFETY: exact private ABI; returned executable owner outlives each run.
    let entry = unsafe { transmute::<*const u8, Entry>(module.get_finalized_function(id)) };
    (owner, entry, layout)
}
fn bound(kind: ValueType) -> BoundProgram {
    BoundProgram {
        program: Program {
            counts: [0, 0, 2, 0],
            max_arguments: 1,
            operations: vec![
                Operation {
                    offset: 5,
                    instruction: Instruction::Store {
                        external: 0,
                        source: Source {
                            kind: ValueType::Float,
                            address: Address::Constant([7, 0, 0, 0]),
                        },
                    },
                },
                Operation {
                    offset: 29,
                    instruction: Instruction::Call {
                        destination: Some(Destination {
                            kind,
                            scope: 2,
                            slot: 0,
                        }),
                        result_kind: Some(kind),
                        receiver: Some(1),
                        function: 0,
                        arguments: vec![Source {
                            kind: ValueType::Float,
                            address: Address::Slot { scope: 2, slot: 1 },
                        }],
                        controls: vec![0],
                        flags: 0,
                    },
                },
            ],
        },
        externals: vec![External::Field(0), External::Sampler(0)].into_boxed_slice(),
        callbacks: vec![Callback::Sample].into_boxed_slice(),
    }
}
#[test]
fn generated_curve_matches_shared_order_knots_and_checked_nonfinite_prefix() {
    let document = Document::read(include_bytes!(
        "../../../../../benches/fixtures/particle_runtime/burst-1.pkb"
    ))
    .unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    let effect = document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap();
    let coordinates = [
        -2.,
        -0.,
        0.,
        f32::from_bits(1),
        0.125,
        f32::from_bits(0.5f32.to_bits() - 1),
        0.5,
        f32::from_bits(0.5f32.to_bits() + 1),
        0.875,
        1.,
        3.,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NAN,
    ];
    for kind in [
        ValueType::Float,
        ValueType::Float2,
        ValueType::Float3,
        ValueType::Float4,
    ] {
        for hermite in [false, true] {
            let sampler = EffectSampler::Curve {
                kind,
                times: vec![0., 0.5, 1.].into_boxed_slice(),
                values: vec![
                    [0.125, -0.25, 0.5, 1.],
                    [0.75, 0.25, -0.5, 0.5],
                    [0.5, 1., 0.25, -0.125],
                ]
                .into_boxed_slice(),
                tangents: if hermite {
                    (0..6)
                        .map(|row| {
                            let row = row as f32;
                            [
                                0.125 + row * 0.25,
                                -0.5 + row * 0.125,
                                0.75 - row * 0.0625,
                                -0.125 - row * 0.25,
                            ]
                        })
                        .collect()
                } else {
                    Box::new([])
                },
            };
            let samplers = [sampler];
            let bound = bound(kind);
            let (_owner, entry, layout) = compile(
                &bound,
                &effect.inner,
                &samplers,
                codegen::CURVE_INLINE_BUDGET,
            );
            let planes = layout
                .component_ordinals()
                .iter()
                .flatten()
                .copied()
                .filter(|o| *o != FieldLayout::ABSENT)
                .max()
                .unwrap()
                + 1;
            for coordinate in coordinates {
                let mut data = vec![99; planes];
                let mut state = bound.program.new_state();
                state.scopes[2][0] = [777; 4];
                state.scopes[2][1] = [coordinate.to_bits(), 0, 0, 0];
                let mut host = CurveHost {
                    sampler: &samplers[0],
                    stores: Vec::new(),
                    samples: 0,
                };
                let result = run_with_inputs(
                    entry,
                    &bound.program,
                    &mut state,
                    &mut host,
                    DirectInputs {
                        fields: data.as_mut_ptr(),
                        stride: 1,
                        slot: 0,
                        time: 0,
                        delta: 0,
                    },
                );
                assert_eq!(
                    data[layout.component_ordinals()[0][0]],
                    7,
                    "preceding canonical write retained"
                );
                assert!(host.stores.is_empty(), "direct store must not replay");
                match sample(&samplers[0], coordinate) {
                    Ok(expected) => {
                        assert_eq!(result, Ok(()));
                        assert_eq!(host.samples, 0, "finite bound Sample must be inlined");
                        assert_eq!(
                            state.scopes[2][0],
                            expected.words(),
                            "coordinate={coordinate} kind={kind:?} hermite={hermite}"
                        );
                    }
                    Err(error) => {
                        assert_eq!(
                            result,
                            Err(ExecutionError {
                                offset: 29,
                                kind: Failure::Host(error)
                            })
                        );
                        assert_eq!(host.samples, 1);
                        assert_eq!(state.scopes[2][0], [777; 4]);
                    }
                }
            }
            // Null direct capability always preserves custom Host dispatch, even
            // though this entry was compiled with canonical sampler bindings.
            let mut state = bound.program.new_state();
            state.scopes[2][1] = [0.125f32.to_bits(), 0, 0, 0];
            let mut host = CurveHost {
                sampler: &samplers[0],
                stores: Vec::new(),
                samples: 0,
            };
            assert_eq!(run(entry, &bound.program, &mut state, &mut host), Ok(()));
            assert_eq!(host.stores, [7]);
            assert_eq!(host.samples, 1);
            assert_eq!(
                state.scopes[2][0],
                sample(&samplers[0], 0.125).unwrap().words()
            );
            // A spent graph-wide expansion budget retains the checked Call.
            let (_owner, entry, _) = compile(&bound, &effect.inner, &samplers, 0);
            let mut state = bound.program.new_state();
            state.scopes[2][1] = [0.125f32.to_bits(), 0, 0, 0];
            let mut host = CurveHost {
                sampler: &samplers[0],
                stores: Vec::new(),
                samples: 0,
            };
            let mut data = vec![99; planes];
            assert_eq!(
                run_with_inputs(
                    entry,
                    &bound.program,
                    &mut state,
                    &mut host,
                    DirectInputs {
                        fields: data.as_mut_ptr(),
                        stride: 1,
                        slot: 0,
                        time: 0,
                        delta: 0
                    }
                ),
                Ok(())
            );
            assert_eq!(host.samples, 1);
            assert!(host.stores.is_empty());
        }
    }
}

#[test]
fn single_key_inlines_and_over_limit_curve_uses_checked_service() {
    let document = Document::read(include_bytes!(
        "../../../../../benches/fixtures/particle_runtime/burst-1.pkb"
    ))
    .unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    let effect = document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap();
    for keys in [1, 9] {
        let sampler = EffectSampler::Curve {
            kind: ValueType::Float4,
            times: (0..keys).map(|i| i as f32 * 0.25).collect(),
            values: (0..keys).map(|i| [i as f32 * 0.125; 4]).collect(),
            tangents: Box::new([]),
        };
        let samplers = [sampler];
        let bound = bound(ValueType::Float4);
        let (_owner, entry, layout) = compile(
            &bound,
            &effect.inner,
            &samplers,
            codegen::CURVE_INLINE_BUDGET,
        );
        let planes = layout
            .component_ordinals()
            .iter()
            .flatten()
            .copied()
            .filter(|o| *o != FieldLayout::ABSENT)
            .max()
            .unwrap()
            + 1;
        for coordinate in [-1f32, 0., 0.125, 2.] {
            let mut state = bound.program.new_state();
            state.scopes[2][1] = [coordinate.to_bits(), 0, 0, 0];
            let mut data = vec![99; planes];
            let mut host = CurveHost {
                sampler: &samplers[0],
                stores: Vec::new(),
                samples: 0,
            };
            assert_eq!(
                run_with_inputs(
                    entry,
                    &bound.program,
                    &mut state,
                    &mut host,
                    DirectInputs {
                        fields: data.as_mut_ptr(),
                        stride: 1,
                        slot: 0,
                        time: 0,
                        delta: 0
                    }
                ),
                Ok(())
            );
            assert_eq!(
                state.scopes[2][0],
                sample(&samplers[0], coordinate).unwrap().words()
            );
            assert_eq!(host.samples, usize::from(keys > 8));
        }
    }
}

#[test]
fn eight_nonuniform_keys_choose_exact_intervals_and_distinct_tangent_sides() {
    let document = Document::read(include_bytes!(
        "../../../../../benches/fixtures/particle_runtime/burst-1.pkb"
    ))
    .unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    let effect = document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap();
    let times = [-2., -0.5, 0., 0.125, 0.75, 1.5, 3., 8.];
    // Include the ULP on each side of every knot, plus one point inside each
    // unequal interval. This exercises both branches at every tree depth.
    let mut coordinates = vec![-3., 9.];
    for &key in &times {
        let key: f32 = key;
        coordinates.extend([key.next_down(), key, key.next_up()]);
    }
    coordinates.extend(times.windows(2).map(|pair| (pair[0] + pair[1]) * 0.5));
    for hermite in [false, true] {
        let sampler = EffectSampler::Curve {
            kind: ValueType::Float4,
            times: times.into(),
            values: (0..8)
                .map(|key| {
                    let key = key as f32;
                    [
                        0.125 + key * 0.25,
                        -key * 0.125,
                        0.5 - key * 0.0625,
                        1. - key * 0.125,
                    ]
                })
                .collect(),
            tangents: if hermite {
                // Every incoming/outgoing row differs; selecting the adjacent
                // knot's wrong side must change the interpolated result.
                (0..16)
                    .map(|row| {
                        let row = row as f32;
                        [
                            0.125 + row * 0.0625,
                            -0.5 - row * 0.125,
                            0.75 - row * 0.03125,
                            -0.125 + row * 0.25,
                        ]
                    })
                    .collect()
            } else {
                Box::new([])
            },
        };
        let samplers = [sampler];
        let bound = bound(ValueType::Float4);
        let (_owner, entry, layout) = compile(
            &bound,
            &effect.inner,
            &samplers,
            codegen::CURVE_INLINE_BUDGET,
        );
        let planes = layout
            .component_ordinals()
            .iter()
            .flatten()
            .copied()
            .filter(|ordinal| *ordinal != FieldLayout::ABSENT)
            .max()
            .unwrap()
            + 1;
        for &coordinate in &coordinates {
            let mut data = vec![99; planes];
            let mut state = bound.program.new_state();
            state.scopes[2][1] = [coordinate.to_bits(), 0, 0, 0];
            let mut host = CurveHost {
                sampler: &samplers[0],
                stores: Vec::new(),
                samples: 0,
            };
            let result = run_with_inputs(
                entry,
                &bound.program,
                &mut state,
                &mut host,
                DirectInputs {
                    fields: data.as_mut_ptr(),
                    stride: 1,
                    slot: 0,
                    time: 0,
                    delta: 0,
                },
            );
            assert_eq!(result, Ok(()));
            assert_eq!(
                host.samples, 0,
                "eight keys must exercise generated sampling"
            );
            assert!(host.stores.is_empty());
            assert_eq!(data[layout.component_ordinals()[0][0]], 7);
            assert_eq!(
                state.scopes[2][0],
                sample(&samplers[0], coordinate).unwrap().words(),
                "coordinate={coordinate} hermite={hermite}"
            );
        }
    }
}
