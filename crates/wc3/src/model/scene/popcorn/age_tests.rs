use super::*;
use crate::model::formats::pkb::{Document, EffectCompileOptions, EffectEvolution};
use crate::model::scene::popcorn::graph::{BoundProgram, EffectField, EffectProgram};
use crate::model::scene::popcorn::packet::tests::fixture;
use crate::model::scene::popcorn::{Host, Operation, Program, State};
use std::fs::read;
use std::path::Path;
use std::sync::Arc;

struct Fields<'a> {
    bound: &'a BoundProgram,
    samplers: &'a [EffectSampler],
    values: Vec<Value>,
    time: f32,
}
impl Host for Fields<'_> {
    type Error = &'static str;
    fn load(&mut self, external: u16) -> Result<Value, Self::Error> {
        match self.bound.externals[usize::from(external)] {
            External::Field(field) => Ok(self.values[field]),
            External::Time => Ok(Value::from(self.time)),
            _ => Err("unexpected external in original scalar test"),
        }
    }
    fn store(&mut self, external: u16, value: Value) -> Result<(), Self::Error> {
        let External::Field(field) = self.bound.externals[usize::from(external)] else {
            return Err("not a field");
        };
        self.values[field] = value;
        Ok(())
    }
    fn call(
        &mut self,
        function: u16,
        receiver: Option<u16>,
        arguments: &[Value],
        _: &[u8],
        _: u8,
    ) -> Result<Option<Value>, Self::Error> {
        assert_eq!(
            self.bound.callbacks[usize::from(function)],
            Callback::Sample
        );
        let External::Sampler(sampler) = self.bound.externals[usize::from(receiver.unwrap())]
        else {
            return Err("not a sampler");
        };
        sample(
            &self.samplers[sampler],
            f32::from_bits(arguments[0].words()[0]),
        )
        .map(Some)
    }
}
fn zero() -> Value {
    Value::from(0f32)
}
fn initial(layer: &EffectLayer) -> Vec<Value> {
    layer
        .fields
        .iter()
        .map(|field| Value::from_words(field.value_type.unwrap(), field.initial))
        .collect()
}
fn appearance() -> EffectProgram {
    let mut program = fixture(1);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[0];
    let bound = layer.evolve.as_mut().unwrap();
    let stop = bound
        .program
        .operations
        .iter()
        .position(|operation| {
            matches!(operation.instruction, Instruction::Load { external: 5, .. })
        })
        .unwrap();
    bound.program.operations = bound.program.operations[..stop].into();
    bound.externals[0] = External::Field(layer.life);
    program
}
fn run(
    plan: &AgePlan,
    captures: &[Value],
    samplers: &[EffectSampler],
    input: AgeSample,
) -> Vec<Value> {
    // Poison all reusable words. Complete successful evaluation must not depend
    // on a previous particle or on any prior query's private scratch contents.
    let mut scratch =
        vec![Value::from_words(ValueType::Float4, [f32::NAN.to_bits(); 4]); plan.nodes.len()];
    let mut outputs = vec![zero(); plan.outputs.len()];
    plan.evaluate(captures, samplers, input, &mut scratch, &mut outputs)
        .unwrap();
    for (node, value) in plan.nodes.iter().zip(&scratch) {
        assert_eq!(
            node.kind,
            value.kind(),
            "every evaluated DAG node retains its checked type"
        );
    }
    outputs
}
#[test]
fn ordered_own_field_stores_curve_and_float3_match_original_scalar() {
    let program = appearance();
    let layer = &program.inner.layers[0];
    let bound = layer.evolve.as_ref().unwrap();
    let plan = AgePlan::prepare(layer, &program.inner.samplers).unwrap();
    assert_eq!(plan.phases, 0);
    let birth = initial(layer);
    let mut captures = vec![zero(); plan.captures.len()];
    plan.capture(&mut captures, |field| Some(birth[field]))
        .unwrap();
    for life in [-0.25, 0., 0.125, 0.25, 0.5, 1., 2.] {
        let mut host = Fields {
            bound,
            samplers: &program.inner.samplers,
            values: birth.clone(),
            time: 7.,
        };
        host.values[layer.life] = Value::from(life);
        let mut state = State::for_programs([&bound.program]);
        bound.program.execute(&mut state, &mut host).unwrap();
        let outputs = run(
            &plan,
            &captures,
            &program.inner.samplers,
            AgeSample {
                effect_time: 7.,
                life,
            },
        );
        for (binding, value) in plan.outputs.iter().zip(outputs) {
            assert_eq!(
                value, host.values[binding.field],
                "field{} life{life}",
                binding.field
            );
        }
    }
}
fn phase_program() -> EffectProgram {
    let mut program = appearance();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[0];
    let mut fields = layer.fields.to_vec();
    fields[2].initial = [2f32.to_bits(), 0, 0, 0];
    fields[3].initial = [3f32.to_bits(), 0, 0, 0];
    fields.push(EffectField {
        name: b"nonzero initialized clock".to_vec().into_boxed_slice(),
        native_type: 0,
        value_type: Some(ValueType::Float),
        initial: [5f32.to_bits(), 0, 0, 0],
    });
    layer.fields = fields.into_boxed_slice();
    let dest = |slot| Destination {
        scope: 1,
        slot,
        kind: ValueType::Float,
    };
    let src = |slot| Source {
        address: Address::Slot { scope: 1, slot },
        kind: ValueType::Float,
    };
    let ops = vec![
        Instruction::Load {
            destination: dest(0),
            external: 0,
        },
        Instruction::Load {
            destination: dest(1),
            external: 1,
        },
        Instruction::Load {
            destination: dest(2),
            external: 2,
        },
        Instruction::Load {
            destination: dest(3),
            external: 3,
        },
        Instruction::Binary {
            destination: dest(2),
            left: src(2),
            right: src(3),
            operation: BinaryOperation::Subtract,
        },
        Instruction::Binary {
            destination: dest(1),
            left: src(1),
            right: src(2),
            operation: BinaryOperation::Multiply,
        },
        Instruction::Binary {
            destination: dest(0),
            left: src(0),
            right: src(1),
            operation: BinaryOperation::Add,
        },
        Instruction::Store {
            source: src(0),
            external: 0,
        },
        Instruction::Load {
            destination: dest(0),
            external: 2,
        },
        Instruction::Store {
            source: src(0),
            external: 3,
        },
    ];
    layer.evolve = Some(BoundProgram {
        program: Program {
            counts: [0, 4, 0, 0],
            max_arguments: 0,
            operations: ops
                .into_iter()
                .enumerate()
                .map(|(i, instruction)| Operation {
                    offset: i * 7,
                    instruction,
                })
                .collect(),
        },
        externals: vec![
            External::Field(3),
            External::Field(2),
            External::Time,
            External::Field(6),
        ]
        .into_boxed_slice(),
        callbacks: Box::new([]),
    });
    program
}
#[test]
fn linear_phase_uses_initialized_phase_clock_and_rate_not_birth_timestamp() {
    let program = phase_program();
    let layer = &program.inner.layers[0];
    let bound = layer.evolve.as_ref().unwrap();
    let plan = AgePlan::prepare(layer, &[]).unwrap();
    assert_eq!(plan.phases, 1);
    let birth = initial(layer);
    let mut captures = vec![zero(); plan.captures.len()];
    plan.capture(&mut captures, |field| Some(birth[field]))
        .unwrap();
    let mut host = Fields {
        bound,
        samplers: &[],
        values: birth,
        time: 5.,
    };
    let mut state = State::for_programs([&bound.program]);
    for time in [5., 5.125, 5.75, 8., 12.] {
        host.time = time;
        bound.program.execute(&mut state, &mut host).unwrap();
        let values = run(
            &plan,
            &captures,
            &[],
            AgeSample {
                effect_time: f64::from(time),
                life: 0.25,
            },
        );
        for (binding, value) in plan.outputs.iter().zip(values) {
            assert_eq!(value, host.values[binding.field]);
        }
        assert_eq!(host.values[3], Value::from(3. + 2. * (time - 5.)));
    }
}
#[test]
fn nonlinear_history_unpaired_clock_and_early_old_phase_observer_reject() {
    let mut program = phase_program();
    let layer = &mut Arc::get_mut(&mut program.inner).unwrap().layers[0];
    layer.evolve.as_mut().unwrap().program.operations[6].instruction = Instruction::Binary {
        destination: Destination {
            scope: 1,
            slot: 0,
            kind: ValueType::Float,
        },
        left: Source {
            address: Address::Slot { scope: 1, slot: 0 },
            kind: ValueType::Float,
        },
        right: Source {
            address: Address::Slot { scope: 1, slot: 1 },
            kind: ValueType::Float,
        },
        operation: BinaryOperation::Multiply,
    };
    assert!(AgePlan::prepare(layer, &[]).is_none());
    let mut program = phase_program();
    let layer = &mut Arc::get_mut(&mut program.inner).unwrap().layers[0];
    let operations = &mut layer.evolve.as_mut().unwrap().program.operations;
    *operations = operations[..operations.len() - 1].into();
    assert!(AgePlan::prepare(layer, &[]).is_none());
    let mut program = phase_program();
    let layer = &mut Arc::get_mut(&mut program.inner).unwrap().layers[0];
    // The old phase is visible in another canonical output before advancement.
    layer.evolve.as_mut().unwrap().externals = vec![
        External::Field(3),
        External::Field(2),
        External::Time,
        External::Field(6),
        External::Field(7),
    ]
    .into_boxed_slice();
    let mut fields = layer.fields.to_vec();
    fields.push(fields[3].clone());
    layer.fields = fields.into_boxed_slice();
    let bound = layer.evolve.as_mut().unwrap();
    let mut operations = bound.program.operations.to_vec();
    operations.insert(
        1,
        Operation {
            offset: 3,
            instruction: Instruction::Store {
                source: Source {
                    address: Address::Slot { scope: 1, slot: 0 },
                    kind: ValueType::Float,
                },
                external: 4,
            },
        },
    );
    bound.program.operations = operations;
    assert!(AgePlan::prepare(layer, &[]).is_none());
}
#[test]
fn live_attribute_lifecycle_write_and_side_effect_callbacks_are_not_frozen() {
    for external in [
        External::Time,
        External::Delta,
        External::Attribute(0),
        External::SceneContext,
    ] {
        let mut program = appearance();
        let graph = Arc::get_mut(&mut program.inner).unwrap();
        let layer = &mut graph.layers[0];
        layer.evolve.as_mut().unwrap().externals[0] = external;
        assert!(AgePlan::prepare(layer, &graph.samplers).is_none());
    }
    let mut program = appearance();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[0];
    layer.evolve.as_mut().unwrap().externals[1] = External::Field(layer.life);
    assert!(AgePlan::prepare(layer, &graph.samplers).is_none());
    let mut program = appearance();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[0];
    layer.evolve.as_mut().unwrap().callbacks[0] = Callback::Rand;
    assert!(AgePlan::prepare(layer, &graph.samplers).is_none());
}
#[test]
fn curve_error_preserves_native_offset_and_leaves_all_caller_outputs_untouched() {
    let program = appearance();
    let layer = &program.inner.layers[0];
    let plan = AgePlan::prepare(layer, &program.inner.samplers).unwrap();
    let birth = initial(layer);
    let mut captures = vec![zero(); plan.captures.len()];
    plan.capture(&mut captures, |field| Some(birth[field]))
        .unwrap();
    let sentinel = Value::from(-71f32);
    let mut outputs = vec![sentinel; plan.outputs.len()];
    let mut scratch = vec![zero(); plan.nodes.len()];
    let error = plan
        .evaluate(
            &captures,
            &program.inner.samplers,
            AgeSample {
                effect_time: 0.,
                life: f32::NAN,
            },
            &mut scratch,
            &mut outputs,
        )
        .unwrap_err();
    let curve_offset = layer
        .evolve
        .as_ref()
        .unwrap()
        .program
        .operations
        .iter()
        .find(|operation| matches!(operation.instruction, Instruction::Call { .. }))
        .unwrap()
        .offset;
    assert_eq!(error.offset, Some(curve_offset));
    assert!(outputs.iter().all(|value| *value == sentinel));
    let expected = run(
        &plan,
        &captures,
        &program.inner.samplers,
        AgeSample {
            effect_time: 0.,
            life: 0.3,
        },
    );
    plan.evaluate(
        &captures,
        &program.inner.samplers,
        AgeSample {
            effect_time: 0.,
            life: 0.3,
        },
        &mut scratch,
        &mut outputs,
    )
    .unwrap();
    assert_eq!(outputs, expected);
}
#[test]
fn persistent_incoming_registers_and_scalar_word_padding_have_distinct_rules() {
    let mut program = appearance();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[0];
    let bound = layer.evolve.as_mut().unwrap();
    bound.program.counts[2] = 1;
    let mut operations = bound.program.operations.to_vec();
    operations.insert(
        0,
        Operation {
            offset: 2,
            instruction: Instruction::Store {
                source: Source {
                    address: Address::Slot { scope: 2, slot: 0 },
                    kind: ValueType::Float,
                },
                external: 1,
            },
        },
    );
    bound.program.operations = operations;
    assert!(
        AgePlan::prepare(layer, &graph.samplers).is_none(),
        "init/private persistent history cannot become an implicit birth capture"
    );

    let program = appearance();
    let layer = &program.inner.layers[0];
    let plan = AgePlan::prepare(layer, &program.inner.samplers).unwrap();
    let birth = initial(layer);
    let mut captures = vec![zero(); plan.captures.len()];
    plan.capture(&mut captures, |field| Some(birth[field]))
        .unwrap();
    let mut scratch = vec![Value::from_words(ValueType::Float4, [0xdeadbeef; 4]); plan.nodes.len()];
    let mut outputs = vec![zero(); plan.outputs.len()];
    for life in [0.125, 0.25, 0.125] {
        plan.evaluate(
            &captures,
            &program.inner.samplers,
            AgeSample {
                effect_time: 0.,
                life,
            },
            &mut scratch,
            &mut outputs,
        )
        .unwrap();
        assert_eq!(
            outputs,
            run(
                &plan,
                &captures,
                &program.inner.samplers,
                AgeSample {
                    effect_time: 0.,
                    life
                }
            )
        );
    }
}

#[test]
fn plan_limits_are_optimization_fallback_not_partial_output_plans() {
    let mut program = appearance();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[0];
    let bound = layer.evolve.as_mut().unwrap();
    let op = bound.program.operations[2].clone();
    bound
        .program
        .operations
        .splice(0..0, (0..MAX_NODES).map(|_| op.clone()));
    assert!(AgePlan::prepare(layer, &graph.samplers).is_none());
}
#[test]
#[ignore = "requires retained original native Lich asset; compile all selected evolution alternatives"]
fn native_lich_all_selected_modes_close_complete_outputs_and_match_one_scalar_evaluation() {
    let path=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/dependencies/files/war3.w3mod/_hd.w3mod/sharedfx/lich/lichattack--f9ecf5d6868f.pkb");
    let bytes = read(path).unwrap();
    let document = Document::read(&bytes).unwrap();
    for evolution in [
        EffectEvolution::Physics,
        EffectEvolution::Fixed,
        EffectEvolution::Varying,
    ] {
        let program = document
            .compile_effect(
                1,
                &EffectCompileOptions {
                    evolution,
                    ..Default::default()
                },
            )
            .unwrap();
        let layer = &program.inner.layers[3];
        let bound = layer.evolve.as_ref().unwrap();
        let plan = AgePlan::prepare(layer, &program.inner.samplers)
            .expect("actual renderer layer must be a positive generic extraction");
        assert_eq!(plan.phases, 1);
        let mut birth = initial(layer);
        for value in &mut birth {
            let words = match value.kind() {
                ValueType::Float => [0.125f32.to_bits(), 0, 0, 0],
                ValueType::Float3 => [0.2f32.to_bits(), 0.3f32.to_bits(), 0.4f32.to_bits(), 0],
                ValueType::Float4 => [0.5f32.to_bits(); 4],
                _ => value.words(),
            };
            *value = Value::from_words(value.kind(), words);
        }
        let mut captures = vec![zero(); plan.captures.len()];
        plan.capture(&mut captures, |field| Some(birth[field]))
            .unwrap();
        for life in [0., 0.22495063, 0.5, 0.999] {
            let mut host = Fields {
                bound,
                samplers: &program.inner.samplers,
                values: birth.clone(),
                time: 0.75,
            };
            host.values[layer.life] = Value::from(life);
            let mut state = State::for_programs([&bound.program]);
            bound.program.execute(&mut state, &mut host).unwrap();
            let outputs = run(
                &plan,
                &captures,
                &program.inner.samplers,
                AgeSample {
                    effect_time: 0.75,
                    life,
                },
            );
            for (binding, value) in plan.outputs.iter().zip(outputs) {
                assert_eq!(
                    value, host.values[binding.field],
                    "{evolution:?} field{}",
                    binding.field
                );
            }
        }
    }
}

#[test]
fn lifecycle_advanced_rate_cannot_be_captured_as_an_immutable_phase_coefficient() {
    let mut program = phase_program();
    let layer = &mut Arc::get_mut(&mut program.inner).unwrap().layers[0];
    layer.evolve.as_mut().unwrap().externals[1] = External::Field(layer.life);
    assert!(AgePlan::prepare(layer, &[]).is_none());
}

#[test]
fn immutable_boolean_capture_constant_and_reinterpreted_words_preserve_exact_bits() {
    use crate::model::scene::popcorn::packet::tests::boolean_copy_fixture;
    for word in [0, u32::MAX] {
        let mut program = boolean_copy_fixture(word);
        let graph = Arc::get_mut(&mut program.inner).unwrap();
        let layer = &mut graph.layers[0];
        let bound = layer.evolve.as_mut().unwrap();
        let mut externals = bound.externals.to_vec();
        externals.push(External::Field(3));
        bound.externals = externals.into_boxed_slice();
        // Reuse the raw Boolean register as Float exactly as the checked VM
        // allows; all-ones is a NaN bit-pattern, not numeric Boolean 1.0.
        let mut operations = bound.program.operations.to_vec();
        operations.push(Operation {
            offset: 101,
            instruction: Instruction::Store {
                source: Source {
                    address: Address::Slot { scope: 1, slot: 0 },
                    kind: ValueType::Float,
                },
                external: 5,
            },
        });
        bound.program.operations = operations;
        let layer = &program.inner.layers[0];
        let bound = layer.evolve.as_ref().unwrap();
        let plan = AgePlan::prepare(layer, &program.inner.samplers).unwrap();
        assert!(plan.total_life_curves(&program.inner.samplers));
        assert!(plan.nodes.iter().any(|node| matches!(
            node.expression,
            Expression::Words {
                kind: ValueType::Float,
                ..
            }
        )));
        let birth = initial(layer);
        let mut captures = vec![zero(); plan.captures.len()];
        plan.capture(&mut captures, |field| Some(birth[field]))
            .unwrap();
        let mut host = Fields {
            bound,
            samplers: &program.inner.samplers,
            values: birth,
            time: 1.,
        };
        let mut state = State::for_programs([&bound.program]);
        bound.program.execute(&mut state, &mut host).unwrap();
        let outputs = run(
            &plan,
            &captures,
            &program.inner.samplers,
            AgeSample {
                effect_time: 1.,
                life: 0.25,
            },
        );
        for (binding, value) in plan.outputs.iter().zip(outputs) {
            assert_eq!(value.words(), host.values[binding.field].words());
            assert_eq!(value.kind(), host.values[binding.field].kind());
        }
        assert_eq!(host.values[7].words(), [word, 0, 0, 0]);
        assert_eq!(host.values[9].words(), [word, 0, u32::MAX, 0]);
        assert_eq!(host.values[10].words(), [word, 0, 0, 0]);
        assert_eq!(host.values[3].words(), [word, 0, 0, 0]);
        let mut invalid = layer.clone();
        let invalid_program = &mut invalid.evolve.as_mut().unwrap().program;
        let mut operations = invalid_program.operations.to_vec();
        operations.push(Operation {
            offset: 102,
            instruction: Instruction::Binary {
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
        });
        invalid_program.operations = operations;
        assert!(AgePlan::prepare(&invalid, &program.inner.samplers).is_none());
    }
}

#[test]
fn typed_self_copies_keep_output_normalization_but_arithmetic_feedback_is_not_frozen() {
    let program = appearance();
    let layer = &program.inner.layers[0];
    let plan = AgePlan::prepare(layer, &program.inner.samplers).unwrap();
    let field = 4;
    assert!(plan.outputs.iter().any(|output| output.field == field));
    assert!(plan
        .captures
        .iter()
        .any(|capture| capture.field == field && capture.kind == ValueType::Float3));
    let birth = initial(layer);
    let mut captures = vec![zero(); plan.captures.len()];
    plan.capture(&mut captures, |field| Some(birth[field]))
        .unwrap();
    let values = run(
        &plan,
        &captures,
        &program.inner.samplers,
        AgeSample {
            effect_time: 0.,
            life: 0.25,
        },
    );
    let position = plan
        .outputs
        .iter()
        .position(|output| output.field == field)
        .unwrap();
    assert_eq!(values[position].kind(), ValueType::Float3);
    assert_eq!(
        values[position].words(),
        [(-0f32).to_bits(), 0.2f32.to_bits(), 0.3f32.to_bits(), 0]
    );
    // The scalar source now reads its own incoming field3 and writes field3
    // plus0.125. Unlike the exact Float3 identity copy, this is recurrence.
    let mut changed = layer.clone();
    changed.evolve.as_mut().unwrap().externals[0] = External::Field(3);
    assert!(AgePlan::prepare(&changed, &program.inner.samplers).is_none());
}

#[test]
fn authorization_checks_finite_life_and_f32_time_even_without_an_explicit_time_node() {
    for program in [appearance(), phase_program()] {
        let plan = AgePlan::prepare(&program.inner.layers[0], &program.inner.samplers).unwrap();
        for sample in [
            AgeSample {
                effect_time: -0.,
                life: -0.,
            },
            AgeSample {
                effect_time: f64::from(f32::MAX),
                life: 1.,
            },
            AgeSample {
                effect_time: f64::from(f32::MIN_POSITIVE),
                life: -1.,
            },
        ] {
            assert!(
                plan.permitted(sample),
                "finite shader inputs do not imply a clamped lifecycle domain"
            );
        }
        for life in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(!plan.permitted(AgeSample {
                effect_time: 0.,
                life
            }));
        }
        for effect_time in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MAX,
            -f64::MAX,
            f64::from(f32::MAX) * 2.,
        ] {
            assert!(!plan.permitted(AgeSample {
                effect_time,
                life: 0.25
            }));
        }
    }
}
