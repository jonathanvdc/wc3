use super::*;
use crate::model::scene::popcorn::graph::BoundProgram;
use crate::model::scene::popcorn::packet::{prepare, tests::fixture};
use crate::model::scene::popcorn::{
    Address, BinaryOperation, Destination, ExecutionOptions, ExecutionPreference,
    ImmutableExpression, Instruction, NumericPolicy, Operation, Program, Source,
};

fn delta_program(count: usize, preference: ExecutionPreference) -> EffectProgram {
    let mut program = fixture(count);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[0];
    layer.fields[2].initial = Value::from(2f32).words();
    layer.fields[3].initial = Value::from(3f32).words();
    let destination = |slot| Destination {
        scope: 1,
        slot,
        kind: ValueType::Float,
    };
    let source = |slot| Source {
        address: Address::Slot { scope: 1, slot },
        kind: ValueType::Float,
    };
    let operations = [
        Instruction::Load {
            destination: destination(0),
            external: 0,
        },
        Instruction::Load {
            destination: destination(1),
            external: 1,
        },
        Instruction::Load {
            destination: destination(2),
            external: 2,
        },
        Instruction::Binary {
            destination: destination(1),
            left: source(1),
            right: source(2),
            operation: BinaryOperation::Multiply,
        },
        Instruction::Binary {
            destination: destination(0),
            left: source(0),
            right: source(1),
            operation: BinaryOperation::Add,
        },
        Instruction::Store {
            source: source(0),
            external: 0,
        },
    ];
    layer.evolve = Some(BoundProgram {
        program: Program {
            counts: [0, 3, 0, 0],
            max_arguments: 0,
            operations: operations
                .into_iter()
                .enumerate()
                .map(|(index, instruction)| Operation {
                    offset: index * 7,
                    instruction,
                })
                .collect(),
        },
        externals: vec![External::Field(3), External::Field(2), External::Delta].into_boxed_slice(),
        callbacks: Box::new([]),
    });
    prepare(&mut graph.layers, &graph.samplers);
    program.with_execution(ExecutionOptions {
        preference,
        numeric_policy: NumericPolicy::Graphics,
    })
}
fn limits() -> EffectLimits {
    EffectLimits {
        particles: 8,
        step_seconds: 1.,
        ..EffectLimits::default()
    }
}
fn position(instance: &EffectInstance) -> f32 {
    f32::from_bits(
        instance
            .particles(EffectLayerId(0))
            .next()
            .unwrap()
            .field(EffectFieldId(3))
            .unwrap()
            .words()[0],
    )
}

#[test]
fn authorized_delta_sum_matches_variable_step_source_and_reset() {
    let ordinary = delta_program(1, ExecutionPreference::Interpreter);
    let optimized = delta_program(1, ExecutionPreference::ImmutableAge);
    let recipe = optimized.immutable_recipe(EffectLayerId(0)).unwrap();
    assert!((0..recipe.node_count()).any(|index| matches!(
        recipe.node(index).unwrap().expression,
        ImmutableExpression::IntegratedDelta
    )));
    let mut source = ordinary.instantiate(limits(), 0).unwrap();
    let mut closed = optimized.instantiate(limits(), 0).unwrap();
    source
        .start(&ordinary, ordinary.default_entry(), SceneInputs::default())
        .unwrap();
    closed
        .start(
            &optimized,
            optimized.default_entry(),
            SceneInputs::default(),
        )
        .unwrap();
    assert_eq!(position(&closed), 3.);
    closed
        .advance(&optimized, 0., SceneInputs::default())
        .unwrap();
    assert_eq!(closed.execution_stats().immutable_age_invocations, 0);
    let mut elapsed = 0.;
    for step in [0.125, 0.375, 0.25] {
        source
            .advance(&ordinary, step, SceneInputs::default())
            .unwrap();
        closed
            .advance(&optimized, step, SceneInputs::default())
            .unwrap();
        elapsed += f64::from(step as f32);
        assert_eq!(position(&closed), position(&source));
        assert_eq!(position(&closed), 3. + 2. * elapsed as f32);
        let sample = closed
            .particles(EffectLayerId(0))
            .next()
            .unwrap()
            .immutable_sample()
            .unwrap();
        assert_eq!(sample.sample().integrated_delta, elapsed);
    }
    assert_eq!(closed.execution_stats().immutable_age_fallbacks, 0);
    closed
        .start(
            &optimized,
            optimized.default_entry(),
            SceneInputs::default(),
        )
        .unwrap();
    assert_eq!(position(&closed), 3.);
    closed
        .advance(&optimized, 0.25, SceneInputs::default())
        .unwrap();
    assert_eq!(position(&closed), 3.5);
    assert_eq!(
        closed
            .particles(EffectLayerId(0))
            .next()
            .unwrap()
            .immutable_sample()
            .unwrap()
            .sample()
            .integrated_delta,
        0.25
    );
}

#[test]
fn denied_step_restores_old_moment_and_never_catches_up() {
    let program = delta_program(1, ExecutionPreference::ImmutableAge);
    let mut instance = program.instantiate(limits(), 0).unwrap();
    instance
        .start(&program, program.default_entry(), SceneInputs::default())
        .unwrap();
    instance
        .advance(&program, 0.25, SceneInputs::default())
        .unwrap();
    assert_eq!(position(&instance), 3.5);
    instance
        .advance_with_step_budget(&program, 0.5, SceneInputs::default(), || false)
        .unwrap();
    assert!(!instance.immutable_active());
    assert_eq!(position(&instance), 3.5);
    instance
        .advance(&program, 0.25, SceneInputs::default())
        .unwrap();
    assert_eq!(position(&instance), 4.);
}

#[test]
fn actual_cursor_delta_is_committed_once_and_finite_guard_restores_before_replay() {
    let program = delta_program(1, ExecutionPreference::ImmutableAge);
    let mut instance = program.instantiate(limits(), 0).unwrap();
    instance
        .start(&program, program.default_entry(), SceneInputs::default())
        .unwrap();
    // Direct private lifecycle entry isolates a partial first interval without
    // authoring an unrelated event generator. It uses the actual runtime cursor.
    let slot = instance.active[0];
    instance.particles[slot].cursor = 0.1;
    let mut work = 0;
    assert!(!instance
        .evolve(&program, slot, 0.25, SceneInputs::default(), &mut work)
        .unwrap());
    let expected = f64::from(0.15f32);
    assert_eq!(
        instance
            .immutable_view(slot)
            .unwrap()
            .sample()
            .integrated_delta,
        expected
    );
    assert_eq!(position(&instance), 3. + 2. * 0.15f32);
    assert!(!instance
        .evolve(&program, slot, 0.25, SceneInputs::default(), &mut work)
        .unwrap());
    assert_eq!(
        instance
            .immutable_view(slot)
            .unwrap()
            .sample()
            .integrated_delta,
        expected,
        "same-cursor invocation adds zero exactly once"
    );
    // Failed prospective moment validation must not publish or destroy the old
    // authorized sample. Existing fallback materializes precisely that state.
    assert!(!instance.authorize_age(slot, 0.5, f64::MAX));
    assert!(!instance.immutable_active());
    assert_eq!(position(&instance), 3. + 2. * 0.15f32);
}

fn routed_program(preference: ExecutionPreference) -> EffectProgram {
    use crate::model::formats::pkb::{Document, EffectCompileOptions};
    let native = Document::read(include_bytes!(
        "../../../../tests/fixtures/pkb/delta-affine.pkb"
    ))
    .unwrap();
    native
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap()
        .with_execution(ExecutionOptions {
            preference,
            numeric_policy: NumericPolicy::Graphics,
        })
}
fn child_rotation(instance: &EffectInstance) -> f32 {
    f32::from_bits(
        instance
            .particles(EffectLayerId(1))
            .next()
            .unwrap()
            .field(EffectFieldId(10))
            .unwrap()
            .words()[0],
    )
}

#[test]
fn native_routed_partial_birth_and_recycled_slot_receive_only_their_own_moments() {
    let source_program = routed_program(ExecutionPreference::Interpreter);
    let program = routed_program(ExecutionPreference::ImmutableAge);
    let limits = EffectLimits {
        particles: 3,
        step_seconds: 1.,
        ..EffectLimits::default()
    };
    let mut source = source_program.instantiate(limits, 0).unwrap();
    let mut instance = program.instantiate(limits, 0).unwrap();
    source
        .start(
            &source_program,
            source_program.default_entry(),
            SceneInputs::default(),
        )
        .unwrap();
    instance
        .start(&program, program.default_entry(), SceneInputs::default())
        .unwrap();
    let mut previous: Option<EffectParticleId> = None;
    let mut observed_reuse = false;
    for _ in 0..12 {
        source
            .advance(&source_program, 0.25, SceneInputs::default())
            .unwrap();
        instance
            .advance(&program, 0.25, SceneInputs::default())
            .unwrap();
        let particle = instance.particles(EffectLayerId(1)).next().unwrap();
        let expected_birth = instance.time() - 0.25 + 0.25 * f64::from(0.4f32);
        assert_eq!(particle.birth_time, expected_birth);
        let expected_delta = f64::from((instance.time() - particle.birth_time) as f32);
        assert_eq!(
            particle
                .immutable_sample()
                .unwrap()
                .sample()
                .integrated_delta,
            expected_delta
        );
        assert_eq!(child_rotation(&instance), 3. + 2. * expected_delta as f32);
        assert_eq!(child_rotation(&instance), child_rotation(&source));
        assert_eq!(instance.stats().omitted_births, 0);
        assert_eq!(instance.stats().omitted_executions, 0);
        if let Some(old) = previous {
            if particle.id.slot == old.slot {
                assert_ne!(particle.id.generation, old.generation);
                observed_reuse = true;
            }
        }
        previous = Some(particle.id);
    }
    assert!(
        observed_reuse,
        "real routed deaths must recycle a bounded slot"
    );
}

#[test]
fn work_quota_after_an_authorized_prefix_restores_all_delta_outputs() {
    let program = delta_program(3, ExecutionPreference::ImmutableAge);
    let mut instance = program.instantiate(limits(), 0).unwrap();
    instance
        .start(&program, program.default_entry(), SceneInputs::default())
        .unwrap();
    instance
        .advance(&program, 0.25, SceneInputs::default())
        .unwrap();
    instance.limits.work_per_update = 1;
    instance
        .advance(&program, 0.25, SceneInputs::default())
        .unwrap();
    assert!(!instance.immutable_active());
    let values: Vec<_> = instance
        .particles(EffectLayerId(0))
        .map(|p| f32::from_bits(p.field(EffectFieldId(3)).unwrap().words()[0]))
        .collect();
    assert_eq!(values, [4., 3.5, 3.5]);
    assert_eq!(instance.stats().omitted_executions, 2);
    instance.limits.work_per_update = EffectLimits::default().work_per_update;
    instance
        .advance(&program, 0.25, SceneInputs::default())
        .unwrap();
    let values: Vec<_> = instance
        .particles(EffectLayerId(0))
        .map(|p| f32::from_bits(p.field(EffectFieldId(3)).unwrap().words()[0]))
        .collect();
    assert_eq!(values, [4.5, 4., 4.]);
}

#[test]
fn natural_clock_fault_keeps_current_delta_store_without_replaying_old_steps() {
    let mut program = delta_program(1, ExecutionPreference::Interpreter);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[0];
    layer.fields[1].initial = Value::from(0f32).words();
    layer.fields[2].initial = Value::from(1e-38f32).words();
    let mut fields = layer.fields.to_vec();
    for name in ["clock", "paired phase"] {
        fields.push(EffectField {
            name: name.as_bytes().into(),
            native_type: 31,
            value_type: Some(ValueType::Float),
            initial: [0; 4],
        });
    }
    layer.fields = fields.into_boxed_slice();
    let bound = layer.evolve.as_mut().unwrap();
    bound.externals = vec![
        External::Field(3),
        External::Field(2),
        External::Delta,
        External::Field(6),
        External::Field(7),
        External::Time,
    ]
    .into_boxed_slice();
    let d = |slot| Destination {
        scope: 1,
        slot,
        kind: ValueType::Float,
    };
    let s = |slot| Source {
        address: Address::Slot { scope: 1, slot },
        kind: ValueType::Float,
    };
    let suffix = [
        Instruction::Load {
            destination: d(3),
            external: 5,
        },
        Instruction::Load {
            destination: d(4),
            external: 3,
        },
        Instruction::Binary {
            destination: d(3),
            left: s(3),
            right: s(4),
            operation: BinaryOperation::Subtract,
        },
        Instruction::Load {
            destination: d(4),
            external: 1,
        },
        Instruction::Binary {
            destination: d(3),
            left: s(3),
            right: s(4),
            operation: BinaryOperation::Multiply,
        },
        Instruction::Load {
            destination: d(5),
            external: 4,
        },
        Instruction::Binary {
            destination: d(5),
            left: s(5),
            right: s(3),
            operation: BinaryOperation::Add,
        },
        Instruction::Store {
            source: s(5),
            external: 4,
        },
        Instruction::Load {
            destination: d(3),
            external: 5,
        },
        Instruction::Store {
            source: s(3),
            external: 3,
        },
    ];
    bound.program.counts[1] = 6;
    bound
        .program
        .operations
        .extend(
            suffix
                .into_iter()
                .enumerate()
                .map(|(index, instruction)| Operation {
                    offset: 42 + 7 * index,
                    instruction,
                }),
        );
    prepare(&mut graph.layers, &graph.samplers);
    let source_program = program.clone().with_execution(ExecutionOptions {
        preference: ExecutionPreference::Interpreter,
        numeric_policy: NumericPolicy::Graphics,
    });
    let program = program.with_execution(ExecutionOptions {
        preference: ExecutionPreference::ImmutableAge,
        numeric_policy: NumericPolicy::Graphics,
    });
    assert!(program.immutable_recipe(EffectLayerId(0)).is_some());
    let limits = EffectLimits {
        particles: 8,
        step_seconds: 2e38,
        ..EffectLimits::default()
    };
    let mut source = source_program.instantiate(limits, 0).unwrap();
    let mut instance = program.instantiate(limits, 0).unwrap();
    source
        .start(
            &source_program,
            source_program.default_entry(),
            SceneInputs::default(),
        )
        .unwrap();
    instance
        .start(&program, program.default_entry(), SceneInputs::default())
        .unwrap();
    source
        .advance(&source_program, 2e38, SceneInputs::default())
        .unwrap();
    instance
        .advance(&program, 2e38, SceneInputs::default())
        .unwrap();
    let previous = position(&instance);
    let expected = source
        .advance(&source_program, 2e38, SceneInputs::default())
        .unwrap_err();
    let actual = instance
        .advance(&program, 2e38, SceneInputs::default())
        .unwrap_err();
    assert_eq!(actual.offset, Some(42));
    assert_eq!(actual.offset, expected.offset);
    assert_eq!(actual.message, expected.message);
    assert!(
        position(&instance) > previous,
        "current Delta store precedes failing Time load"
    );
    assert_eq!(position(&instance), position(&source));
    assert!(!instance.immutable_active());
}

#[test]
fn subdivided_external_advance_and_repeated_delta_load_integrate_once_per_invocation() {
    let mut program = delta_program(1, ExecutionPreference::Interpreter);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let bound = graph.layers[0].evolve.as_mut().unwrap();
    bound.program.operations.push(Operation {
        offset: 77,
        instruction: Instruction::Load {
            destination: Destination {
                scope: 1,
                slot: 2,
                kind: ValueType::Float,
            },
            external: 2,
        },
    });
    prepare(&mut graph.layers, &graph.samplers);
    let program = program.with_execution(ExecutionOptions {
        preference: ExecutionPreference::ImmutableAge,
        numeric_policy: NumericPolicy::Graphics,
    });
    let mut instance = program
        .instantiate(
            EffectLimits {
                step_seconds: 0.125,
                ..limits()
            },
            0,
        )
        .unwrap();
    instance
        .start(&program, program.default_entry(), SceneInputs::default())
        .unwrap();
    instance
        .advance(&program, 0.3, SceneInputs::default())
        .unwrap();
    let sample = instance
        .particles(EffectLayerId(0))
        .next()
        .unwrap()
        .immutable_sample()
        .unwrap();
    let expected = 0.25 + f64::from(0.05f32);
    assert_eq!(sample.sample().integrated_delta, expected);
    assert_eq!(position(&instance), 3. + 2. * expected as f32);
    assert_eq!(instance.execution_stats().immutable_age_invocations, 3);
}

#[test]
fn positive_elapsed_underflowing_vm_delta_does_not_move_or_advance_the_moment() {
    let source_program = delta_program(1, ExecutionPreference::Interpreter);
    let program = delta_program(1, ExecutionPreference::ImmutableAge);
    let mut source = source_program.instantiate(limits(), 0).unwrap();
    let mut instance = program.instantiate(limits(), 0).unwrap();
    source
        .start(
            &source_program,
            source_program.default_entry(),
            SceneInputs::default(),
        )
        .unwrap();
    instance
        .start(&program, program.default_entry(), SceneInputs::default())
        .unwrap();
    let tiny = f64::MIN_POSITIVE;
    assert!(tiny > 0.);
    assert_eq!(tiny as f32, 0.);
    source
        .advance(&source_program, tiny, SceneInputs::default())
        .unwrap();
    instance
        .advance(&program, tiny, SceneInputs::default())
        .unwrap();
    assert_eq!(instance.time(), tiny);
    assert_eq!(instance.execution_stats().immutable_age_invocations, 1);
    assert_eq!(position(&instance), 3.);
    assert_eq!(position(&instance), position(&source));
    assert_eq!(
        instance
            .particles(EffectLayerId(0))
            .next()
            .unwrap()
            .immutable_sample()
            .unwrap()
            .sample()
            .integrated_delta,
        0.
    );
    source
        .advance(&source_program, 0.25, SceneInputs::default())
        .unwrap();
    instance
        .advance(&program, 0.25, SceneInputs::default())
        .unwrap();
    assert_eq!(position(&instance), 3.5);
    assert_eq!(position(&instance), position(&source));
    assert_eq!(
        instance
            .particles(EffectLayerId(0))
            .next()
            .unwrap()
            .immutable_sample()
            .unwrap()
            .sample()
            .integrated_delta,
        0.25
    );
}
