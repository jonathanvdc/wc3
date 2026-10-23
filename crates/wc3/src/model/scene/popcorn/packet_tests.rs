use super::*;
use crate::model::formats::pkb::{Document, EffectCompileOptions};
use crate::model::scene::popcorn::graph::{EffectField, EffectLayerId, EffectProgram};
use crate::model::scene::popcorn::{Operation as VmOperation, Program};
use std::sync::Arc;

pub(crate) fn fixture(count: usize) -> EffectProgram {
    let document =
        Document::read(include_bytes!("../../../../tests/fixtures/pkb/sprite.pkb")).unwrap();
    let mut program = document
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[0];
    layer.life = 0;
    layer.inverse_life = 1;
    layer.standalone = false;
    layer.spawner = false;
    layer.rendered = true;
    layer.fields = [
        (ValueType::Float, [0.; 4]),
        (ValueType::Float, [0.25, 0., 0., 0.]),
        (ValueType::Float, [0.05, 0., 0., 0.]),
        (ValueType::Float, [0.; 4]),
        (ValueType::Float3, [-0., 0.2, 0.3, 0.]),
        (ValueType::Float4, [0.; 4]),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (kind, initial))| EffectField {
        name: format!("original{index}").into_bytes().into_boxed_slice(),
        native_type: 0,
        value_type: Some(kind),
        initial: initial.map(f32::to_bits),
    })
    .collect();
    layer.init = BoundProgram {
        program: Program {
            counts: [0, 6, 0, 0],
            max_arguments: 1,
            operations: Vec::new(),
        },
        externals: Box::new([]),
        callbacks: Box::new([]),
    };
    let reg = |slot, kind| Destination {
        scope: 1,
        slot,
        kind,
    };
    let src = |slot, kind| Source {
        address: Address::Slot { scope: 1, slot },
        kind,
    };
    let operations = vec![
        Instruction::Load {
            destination: reg(0, ValueType::Float),
            external: 0,
        },
        Instruction::Store {
            source: src(0, ValueType::Float),
            external: 1,
        },
        Instruction::Binary {
            destination: reg(0, ValueType::Float),
            left: src(0, ValueType::Float),
            right: Source {
                address: Address::Constant([0.125f32.to_bits(), 0, 0, 0]),
                kind: ValueType::Float,
            },
            operation: BinaryOperation::Add,
        },
        Instruction::Store {
            source: src(0, ValueType::Float),
            external: 1,
        },
        Instruction::Load {
            destination: reg(1, ValueType::Float),
            external: 1,
        },
        Instruction::Call {
            destination: Some(reg(2, ValueType::Float4)),
            result_kind: Some(ValueType::Float4),
            receiver: Some(4),
            function: 0,
            arguments: vec![src(1, ValueType::Float)],
            controls: vec![26],
            flags: 2,
        },
        Instruction::Binary {
            destination: reg(2, ValueType::Float4),
            left: src(2, ValueType::Float4),
            right: Source {
                address: Address::Constant([1.5f32.to_bits(), 0, 0, 0]),
                kind: ValueType::Float,
            },
            operation: BinaryOperation::Multiply,
        },
        Instruction::Store {
            source: src(2, ValueType::Float4),
            external: 3,
        },
        Instruction::Load {
            destination: reg(3, ValueType::Float3),
            external: 2,
        },
        Instruction::Store {
            source: src(3, ValueType::Float3),
            external: 2,
        },
        Instruction::Load {
            destination: reg(4, ValueType::Float),
            external: 5,
        },
        Instruction::Load {
            destination: reg(5, ValueType::Float),
            external: 6,
        },
        Instruction::Binary {
            destination: reg(4, ValueType::Float),
            left: src(4, ValueType::Float),
            right: src(5, ValueType::Float),
            operation: BinaryOperation::Subtract,
        },
        Instruction::Store {
            source: src(4, ValueType::Float),
            external: 1,
        },
    ];
    layer.evolve = Some(BoundProgram {
        program: Program {
            counts: [0, 6, 0, 0],
            max_arguments: 1,
            operations: operations
                .into_iter()
                .enumerate()
                .map(|(offset, instruction)| VmOperation {
                    offset: offset * 7,
                    instruction,
                })
                .collect(),
        },
        externals: vec![
            External::Field(2),
            External::Field(3),
            External::Field(4),
            External::Field(5),
            External::Sampler(0),
            External::Time,
            External::Delta,
        ]
        .into_boxed_slice(),
        callbacks: vec![Callback::Sample].into_boxed_slice(),
    });
    graph.samplers = vec![EffectSampler::Curve {
        kind: ValueType::Float4,
        times: vec![0., 0.25, 1.].into_boxed_slice(),
        values: vec![
            [0.1, 0.2, 0.3, 0.4],
            [0.2, 0.4, 0.6, 0.8],
            [1., 0.5, 0., 0.25],
        ]
        .into_boxed_slice(),
        tangents: vec![[0.2, 0.1, -0., 0.7]; 6].into_boxed_slice(),
    }]
    .into_boxed_slice();
    graph.events[graph.entries[0].event].targets = vec![EffectLayerId(0); count].into_boxed_slice();
    prepare(&mut graph.layers, &graph.samplers);
    assert!(graph.layers[0].pure_evolution.is_some());
    program
}
#[test]
fn plan_checks_canonical_lifecycle_aliases_callbacks_resources_and_persistent_operands() {
    let mut program = fixture(PARTICLE_BATCH_SIZE);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let original = graph.layers[0].evolve.as_ref().unwrap().clone();
    let mut controller = graph.layers[0].clone();
    controller.spawner = true;
    assert!(PureEvolution::prepare(&controller, &original, &graph.samplers).is_none());
    controller.spawner = false;
    controller.rendered = false;
    assert!(PureEvolution::prepare(&controller, &original, &graph.samplers).is_none());
    for field_index in [graph.layers[0].life, graph.layers[0].inverse_life] {
        let mut bound = original.clone();
        bound.externals[1] = External::Field(field_index);
        assert!(PureEvolution::prepare(&graph.layers[0], &bound, &graph.samplers).is_none());
    }
    let mut bound = original.clone();
    bound.callbacks[0] = Callback::Rand;
    assert!(PureEvolution::prepare(&graph.layers[0], &bound, &graph.samplers).is_none());
    let mut bound = original.clone();
    bound.program.counts[2] = 1;
    assert!(PureEvolution::prepare(&graph.layers[0], &bound, &graph.samplers).is_some());
    let mut bound = original.clone();
    if let Instruction::Store { source, .. } = &mut bound.program.operations[1].instruction {
        source.address = Address::Slot { scope: 3, slot: 0 };
    }
    assert!(PureEvolution::prepare(&graph.layers[0], &bound, &graph.samplers).is_none());
    let mut bound = original.clone();
    if let Instruction::Binary { operation, .. } = &mut bound.program.operations[2].instruction {
        *operation = BinaryOperation::Divide;
    }
    assert!(PureEvolution::prepare(&graph.layers[0], &bound, &graph.samplers).is_none());
    graph.samplers[0] = EffectSampler::Events(vec![0.].into_boxed_slice());
    assert!(PureEvolution::prepare(&graph.layers[0], &original, &graph.samplers).is_none());
}
#[test]
fn packet_scratch_accounts_exact_owned_allocations_and_ineligible_graph_allocates_none() {
    let program = fixture(PARTICLE_BATCH_SIZE);
    let shape = Scratch::shape(&program.inner.layers).unwrap();
    let scratch = Scratch::new(shape);
    assert_eq!(
        Scratch::bytes(shape),
        Some(
            size_of::<Scratch>()
                + (scratch.registers.capacity() + scratch.fields.capacity()) * size_of::<Planes>()
        )
    );
    let mut layers = program.inner.layers.to_vec();
    layers[0].pure_evolution = None;
    assert_eq!(Scratch::shape(&layers), None);
    assert!(Scratch::bytes((usize::MAX, 1)).is_none());
}

pub(crate) fn persistent_fixture(count: usize) -> EffectProgram {
    let mut program = fixture(count);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[0];
    let bound = layer.evolve.as_mut().unwrap();
    // Keep two locals; split four other original temporary identities across
    // both persistent scopes. Scalar allocations are deliberately retained.
    for operation in &mut bound.program.operations {
        let rewrite = |scope: &mut usize, slot: &mut usize| {
            if *scope == 1 && *slot < 4 {
                let old = *slot;
                *scope = 2 + old % 2;
                *slot = old / 2;
            } else if *scope == 1 {
                *slot -= 4;
            }
        };
        match &mut operation.instruction {
            Instruction::Load { destination, .. } => {
                rewrite(&mut destination.scope, &mut destination.slot)
            }
            Instruction::Store { source, .. } => {
                if let Address::Slot { scope, slot } = &mut source.address {
                    rewrite(scope, slot)
                }
            }
            Instruction::Binary {
                destination,
                left,
                right,
                ..
            } => {
                rewrite(&mut destination.scope, &mut destination.slot);
                for source in [left, right] {
                    if let Address::Slot { scope, slot } = &mut source.address {
                        rewrite(scope, slot);
                    }
                }
            }
            Instruction::Call {
                destination,
                arguments,
                ..
            } => {
                if let Some(destination) = destination {
                    rewrite(&mut destination.scope, &mut destination.slot);
                }
                for source in arguments {
                    if let Address::Slot { scope, slot } = &mut source.address {
                        rewrite(scope, slot);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
    bound.program.counts = [0, 2, 2, 2];
    layer.init.program.counts = [0, 2, 2, 2];
    // Incoming persistent state is legitimate during initialization. It is
    // always zero at birth and must not prevent an independent evolve proof.
    layer.init.externals = vec![External::Field(3)].into_boxed_slice();
    layer.init.program.operations = vec![VmOperation {
        offset: 0,
        instruction: Instruction::Store {
            source: Source {
                address: Address::Slot { scope: 2, slot: 0 },
                kind: ValueType::Float,
            },
            external: 0,
        },
    }];
    prepare(&mut graph.layers, &graph.samplers);
    assert!(layer_plan(&graph.layers[0]).is_some());
    program
}
fn layer_plan(layer: &EffectLayer) -> Option<&PureEvolution> {
    layer.evolve.as_ref().and(layer.pure_evolution.as_deref())
}
#[test]
fn persistent_eligibility_requires_sources_before_destination_within_evolution_only() {
    let mut program = persistent_fixture(PARTICLE_BATCH_SIZE);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let original = graph.layers[0].evolve.as_ref().unwrap().clone();
    assert_eq!(original.program.counts, [0, 2, 2, 2]);
    assert_eq!(layer_plan(&graph.layers[0]).unwrap().registers, 6);
    for instruction in [
        Instruction::Store {
            source: Source {
                address: Address::Slot { scope: 2, slot: 0 },
                kind: ValueType::Float,
            },
            external: 1,
        },
        Instruction::Binary {
            destination: Destination {
                scope: 2,
                slot: 0,
                kind: ValueType::Float,
            },
            left: Source {
                address: Address::Slot { scope: 2, slot: 0 },
                kind: ValueType::Float,
            },
            right: Source {
                address: Address::Constant([0; 4]),
                kind: ValueType::Float,
            },
            operation: BinaryOperation::Add,
        },
        Instruction::Call {
            destination: Some(Destination {
                scope: 2,
                slot: 0,
                kind: ValueType::Float4,
            }),
            result_kind: Some(ValueType::Float4),
            receiver: Some(4),
            function: 0,
            arguments: vec![Source {
                address: Address::Slot { scope: 2, slot: 0 },
                kind: ValueType::Float,
            }],
            controls: vec![26],
            flags: 2,
        },
    ] {
        let mut bound = original.clone();
        bound.program.operations.insert(
            0,
            VmOperation {
                offset: 999,
                instruction,
            },
        );
        assert!(PureEvolution::prepare(&graph.layers[0], &bound, &graph.samplers).is_none());
    }
}

#[test]
fn preparation_clears_the_selected_plan_when_evolution_is_removed() {
    let mut program = fixture(PARTICLE_BATCH_SIZE);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    assert!(graph.layers[0].pure_evolution.is_some());
    graph.layers[0].evolve = None;
    prepare(&mut graph.layers, &graph.samplers);
    assert!(graph.layers[0].pure_evolution.is_none());
    assert_eq!(Scratch::shape(&graph.layers), None);
}

// Boolean values are copied as opaque words, never converted through float ALU.
pub(crate) fn boolean_copy_fixture(word: u32) -> EffectProgram {
    let mut program = fixture(PARTICLE_BATCH_SIZE);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[0];
    let mut fields = layer.fields.to_vec();
    for (kind, initial) in [
        (ValueType::Bool, [word, 17, 18, 19]),
        (ValueType::Bool, [42; 4]),
        (ValueType::Bool3, [word, 0, u32::MAX, 23]),
        (ValueType::Bool3, [42; 4]),
        (ValueType::Bool, [42; 4]),
    ] {
        fields.push(EffectField {
            name: format!("bool{}", fields.len())
                .into_bytes()
                .into_boxed_slice(),
            native_type: 0,
            value_type: Some(kind),
            initial,
        });
    }
    layer.fields = fields.into_boxed_slice();
    let destination = |slot, kind| Destination {
        scope: 1,
        slot,
        kind,
    };
    let source = |slot, kind| Source {
        address: Address::Slot { scope: 1, slot },
        kind,
    };
    let operations = vec![
        Instruction::Load {
            destination: destination(0, ValueType::Bool),
            external: 0,
        },
        Instruction::Store {
            source: source(0, ValueType::Bool),
            external: 1,
        },
        Instruction::Load {
            destination: destination(1, ValueType::Bool3),
            external: 2,
        },
        Instruction::Store {
            source: source(1, ValueType::Bool3),
            external: 3,
        },
        Instruction::Store {
            source: Source {
                address: Address::Constant([word, 31, 32, 33]),
                kind: ValueType::Bool,
            },
            external: 4,
        },
    ];
    layer.evolve = Some(BoundProgram {
        program: Program {
            counts: [0, 2, 0, 0],
            max_arguments: 0,
            operations: operations
                .into_iter()
                .enumerate()
                .map(|(offset, instruction)| VmOperation {
                    offset,
                    instruction,
                })
                .collect(),
        },
        externals: (6..11).map(External::Field).collect(),
        callbacks: Box::new([]),
    });
    prepare(&mut graph.layers, &graph.samplers);
    assert!(graph.layers[0].pure_evolution.is_some());
    program
}

#[test]
fn boolean_copy_packets_preserve_zero_and_all_ones_and_normalize_inactive_components() {
    for word in [0, u32::MAX, 1, 0x7f80_0001, 0x7fc1_2345] {
        let program = boolean_copy_fixture(word);
        let plan = program.inner.layers[0].pure_evolution.as_ref().unwrap();
        let mut scratch = Scratch::new((plan.registers, plan.fields.len()));
        scratch.poison();
        scratch.count = PARTICLE_BATCH_SIZE;
        for (index, binding) in plan.fields.iter().enumerate() {
            let words = program.inner.layers[0].fields[binding.canonical].initial;
            for (component, value) in words.into_iter().enumerate() {
                scratch.fields[index][component].fill(value);
            }
        }
        scratch.execute(plan, &program.inner.samplers, 0.).unwrap();
        for (canonical, expected) in [
            (7, [word, 0, 0, 0]),
            (9, [word, 0, u32::MAX, 0]),
            (10, [word, 0, 0, 0]),
        ] {
            let field = plan
                .fields
                .iter()
                .position(|field| field.canonical == canonical)
                .unwrap();
            for (component, value) in expected.into_iter().enumerate() {
                assert_eq!(
                    scratch.fields[field][component],
                    [value; PARTICLE_BATCH_SIZE]
                );
            }
        }
        let layer = &program.inner.layers[0];
        let original = layer.evolve.as_ref().unwrap();
        for instruction in [
            Instruction::Binary {
                destination: Destination {
                    scope: 1,
                    slot: 0,
                    kind: ValueType::Bool,
                },
                left: Source {
                    address: Address::Slot { scope: 1, slot: 0 },
                    kind: ValueType::Bool,
                },
                right: Source {
                    address: Address::Constant([word, 0, 0, 0]),
                    kind: ValueType::Bool,
                },
                operation: BinaryOperation::Add,
            },
            Instruction::Swizzle {
                destination: Destination {
                    scope: 1,
                    slot: 1,
                    kind: ValueType::Bool3,
                },
                source: Source {
                    address: Address::Slot { scope: 1, slot: 1 },
                    kind: ValueType::Bool3,
                },
                lanes: [2, 1, 0, 0],
            },
        ] {
            let mut bound = original.clone();
            bound.program.operations.push(VmOperation {
                offset: 99,
                instruction,
            });
            assert!(PureEvolution::prepare(layer, &bound, &program.inner.samplers).is_none());
        }
    }
}

#[test]
fn actual_native_benchmark_child_has_a_shared_pure_plan_including_enabled_copy() {
    let document = Document::read(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/benches/fixtures/particle_runtime/burst-1.pkb"
    )))
    .unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    let program = document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap();
    let layer = &program.inner.layers[1];
    let plan = layer
        .pure_evolution
        .as_ref()
        .expect("actual benchmark, not a similar hand-built graph");
    assert!(plan
        .fields
        .iter()
        .any(|field| field.written
            && layer.fields[field.canonical].value_type == Some(ValueType::Bool)));
}

#[test]
fn ordered_field_dependencies_exclude_write_only_and_post_store_loads() {
    let program = fixture(PARTICLE_BATCH_SIZE);
    let plan = program.inner.layers[0].pure_evolution.as_ref().unwrap();
    let binding = |canonical| {
        plan.fields
            .iter()
            .find(|field| field.canonical == canonical)
            .unwrap()
    };
    // original2 is read before any store; original3 is written before its
    // curve-coordinate read; original5 is only the curve's final output.
    assert!(binding(2).read_before_write);
    assert_eq!(binding(2).components, 1);
    assert!(!binding(3).read_before_write);
    assert!(binding(3).written);
    assert!(!binding(5).read_before_write);
    assert!(binding(5).written);
    assert!(binding(4).read_before_write);
    assert_eq!(binding(4).components, 3);
    assert!(binding(4).written);
}
