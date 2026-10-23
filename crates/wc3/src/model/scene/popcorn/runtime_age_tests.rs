use super::*;
use crate::model::scene::popcorn::packet::tests::fixture;
use crate::model::scene::popcorn::{
    Address, BinaryOperation, Destination, ExecutionBackend, ExecutionFallback, ExecutionOptions,
    ExecutionPreference, Instruction, NumericPolicy, Operation, Program, Source,
};

fn options(preference: ExecutionPreference) -> ExecutionOptions {
    ExecutionOptions {
        preference,
        numeric_policy: NumericPolicy::Graphics,
    }
}
fn closed(count: usize, phase: bool) -> EffectProgram {
    let mut program = fixture(count);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[0];
    let reg = |slot, kind| Destination {
        scope: 1,
        slot,
        kind,
    };
    let src = |slot, kind| Source {
        address: Address::Slot { scope: 1, slot },
        kind,
    };
    let mut instructions = vec![
        Instruction::Load {
            destination: reg(0, ValueType::Float),
            external: 0,
        },
        Instruction::Call {
            destination: Some(reg(2, ValueType::Float4)),
            result_kind: Some(ValueType::Float4),
            receiver: Some(4),
            function: 0,
            arguments: vec![src(0, ValueType::Float)],
            controls: vec![0],
            flags: 0,
        },
        Instruction::Store {
            external: 3,
            source: src(2, ValueType::Float4),
        },
        Instruction::Binary {
            destination: reg(1, ValueType::Float),
            left: src(0, ValueType::Float),
            right: Source {
                address: Address::Constant([1f32.to_bits(), 0, 0, 0]),
                kind: ValueType::Float,
            },
            operation: BinaryOperation::Add,
        },
        Instruction::Store {
            external: 1,
            source: src(1, ValueType::Float),
        },
    ];
    let mut externals = vec![
        External::Field(0),
        External::Field(3),
        External::Field(4),
        External::Field(5),
        External::Sampler(0),
    ];
    if phase {
        let mut fields = layer.fields.to_vec();
        for (name, initial) in [("phase", 0.25f32), ("clock", 0.), ("rate", 1.)] {
            fields.push(EffectField {
                name: name.as_bytes().into(),
                native_type: 0,
                value_type: Some(ValueType::Float),
                initial: [initial.to_bits(), 0, 0, 0],
            });
        }
        layer.fields = fields.into_boxed_slice();
        layer.fields[1].initial = [0; 4];
        externals.extend([
            External::Time,
            External::Field(6),
            External::Field(7),
            External::Field(8),
        ]);
        instructions.extend([
            Instruction::Load {
                destination: reg(3, ValueType::Float),
                external: 5,
            },
            Instruction::Load {
                destination: reg(4, ValueType::Float),
                external: 7,
            },
            Instruction::Binary {
                destination: reg(3, ValueType::Float),
                left: src(3, ValueType::Float),
                right: src(4, ValueType::Float),
                operation: BinaryOperation::Subtract,
            },
            Instruction::Load {
                destination: reg(4, ValueType::Float),
                external: 8,
            },
            Instruction::Binary {
                destination: reg(3, ValueType::Float),
                left: src(4, ValueType::Float),
                right: src(3, ValueType::Float),
                operation: BinaryOperation::Multiply,
            },
            Instruction::Load {
                destination: reg(4, ValueType::Float),
                external: 6,
            },
            Instruction::Binary {
                destination: reg(3, ValueType::Float),
                left: src(4, ValueType::Float),
                right: src(3, ValueType::Float),
                operation: BinaryOperation::Add,
            },
            Instruction::Store {
                external: 6,
                source: src(3, ValueType::Float),
            },
            Instruction::Load {
                destination: reg(3, ValueType::Float),
                external: 5,
            },
            Instruction::Store {
                external: 7,
                source: src(3, ValueType::Float),
            },
        ]);
    }
    layer.evolve = Some(BoundProgram {
        program: Program {
            counts: [0, 6, 0, 0],
            max_arguments: 1,
            operations: instructions
                .into_iter()
                .enumerate()
                .map(|(i, instruction)| Operation {
                    offset: i * 7,
                    instruction,
                })
                .collect(),
        },
        externals: externals.into_boxed_slice(),
        callbacks: vec![Callback::Sample].into_boxed_slice(),
    });
    super::super::packet::prepare(&mut graph.layers, &graph.samplers);
    program
}
fn instance(program: &EffectProgram) -> EffectInstance {
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 16,
                step_seconds: 1. / 64.,
                ..Default::default()
            },
            42,
        )
        .unwrap();
    instance
        .start(program, program.default_entry(), SceneInputs::default())
        .unwrap();
    instance
}
fn equal_public(a: &EffectInstance, b: &EffectInstance) {
    assert_eq!(a.time(), b.time());
    assert_eq!(a.active_count(), b.active_count());
    assert_eq!(format!("{:?}", a.stats()), format!("{:?}", b.stats()));
    for (left, right) in a
        .particles(EffectLayerId(0))
        .zip(b.particles(EffectLayerId(0)))
    {
        assert_eq!(left.id, right.id);
        assert_eq!(left.parent, right.parent);
        assert_eq!(left.birth_time, right.birth_time);
        assert_eq!(left.birth_serial, right.birth_serial);
        for field in 0..a.layout[0].len() {
            assert_eq!(
                left.field(EffectFieldId(field as u32)),
                right.field(EffectFieldId(field as u32)),
                "field {field}"
            );
        }
    }
}
#[test]
fn raw_birth_planes_overlay_queries_and_fixed_plan_survive_wrapper_drop() {
    let base = closed(3, true);
    let cpu = base
        .clone()
        .with_execution(options(ExecutionPreference::Interpreter));
    let age = base.with_execution(options(ExecutionPreference::ImmutableAge));
    assert!(age.immutable_recipe(EffectLayerId(0)).is_some());
    let mut a = instance(&age);
    let mut b = instance(&cpu);
    assert!(a.immutable_active());
    assert_eq!(a.immutable_time(), None);
    assert!(a
        .particles(EffectLayerId(0))
        .all(|p| p.immutable_sample().is_none()));
    equal_public(&a, &b);
    let size_initial = a.fields.read(3, 0);
    drop(age);
    for _ in 0..32 {
        // A same-graph ordinary wrapper cannot replace the instance's fixed plan.
        a.advance(&cpu, 1. / 64., SceneInputs::default()).unwrap();
        b.advance(&cpu, 1. / 64., SceneInputs::default()).unwrap();
        equal_public(&a, &b);
        assert_eq!(a.immutable_time(), Some(a.time()));
        assert_eq!(
            a.fields.read(3, 0),
            size_initial,
            "virtual writes must retain initialized capture planes"
        );
        let p = a.particles(EffectLayerId(0)).next().unwrap();
        let view = p.immutable_sample().unwrap();
        assert!(view
            .recipe()
            .same_recipe(&a.immutable_recipe(EffectLayerId(0)).unwrap()));
        assert_eq!(
            p.field(EffectFieldId(3)).unwrap().words()[0],
            (1. + view.sample().life).to_bits()
        );
        for index in 0..view.recipe().node_count() {
            assert!(view.recipe().node(index).is_some());
        }
    }
    assert_eq!(a.execution_stats().immutable_age_invocations, 3 * 32);
    a.advance(&cpu, 0., SceneInputs::default()).unwrap();
    equal_public(&a, &b);
}
#[test]
fn quota_prefix_restores_all_captures_before_cpu_latch_and_restart_reuses_slot() {
    let base = closed(3, false);
    let cpu = base
        .clone()
        .with_execution(options(ExecutionPreference::Interpreter));
    let age = base.with_execution(options(ExecutionPreference::ImmutableAge));
    let mut a = instance(&age);
    let mut b = instance(&cpu);
    a.advance(&cpu, 1. / 64., SceneInputs::default()).unwrap();
    b.advance(&cpu, 1. / 64., SceneInputs::default()).unwrap();
    a.limits.work_per_update = 1;
    b.limits.work_per_update = 1;
    a.advance(&cpu, 1. / 64., SceneInputs::default()).unwrap();
    b.advance(&cpu, 1. / 64., SceneInputs::default()).unwrap();
    assert!(!a.immutable_active());
    assert_eq!(a.execution_stats().immutable_age_fallbacks, 1);
    equal_public(&a, &b);
    a.limits.work_per_update = EffectLimits::default().work_per_update;
    b.limits.work_per_update = a.limits.work_per_update;
    a.advance(&cpu, 1. / 64., SceneInputs::default()).unwrap();
    b.advance(&cpu, 1. / 64., SceneInputs::default()).unwrap();
    equal_public(&a, &b);
    let old = a.particles(EffectLayerId(0)).next().unwrap().id;
    a.start(&age, age.default_entry(), SceneInputs::default())
        .unwrap();
    assert!(a.immutable_active());
    assert!(a
        .particles(EffectLayerId(0))
        .all(|p| p.immutable_sample().is_none()));
    assert_ne!(a.particles(EffectLayerId(0)).next().unwrap().id, old);
    assert_eq!(
        a.particles(EffectLayerId(0))
            .next()
            .unwrap()
            .field(EffectFieldId(3))
            .unwrap()
            .words(),
        [0; 4]
    );
}
#[test]
fn skipped_step_tail_freezes_old_outputs_while_lifetime_advances() {
    let base = closed(1, false);
    let cpu = base
        .clone()
        .with_execution(options(ExecutionPreference::Interpreter));
    let age = base.with_execution(options(ExecutionPreference::ImmutableAge));
    let mut a = instance(&age);
    let mut b = instance(&cpu);
    a.limits.steps_per_update = 1;
    b.limits.steps_per_update = 1;
    a.advance(&cpu, 3. / 64., SceneInputs::default()).unwrap();
    b.advance(&cpu, 3. / 64., SceneInputs::default()).unwrap();
    assert!(!a.immutable_active());
    equal_public(&a, &b);
    let p = a.particles(EffectLayerId(0)).next().unwrap();
    assert_eq!(
        f32::from_bits(p.field(EffectFieldId(0)).unwrap().words()[0]),
        3. / 256.
    );
    assert_eq!(
        f32::from_bits(p.field(EffectFieldId(3)).unwrap().words()[0]),
        1. + 1. / 256.
    );
}
#[test]
fn strict_policy_uses_ordinary_execution_with_typed_reason() {
    let program = closed(1, false).with_execution(ExecutionOptions {
        preference: ExecutionPreference::ImmutableAge,
        numeric_policy: NumericPolicy::Strict,
    });
    assert!(program.immutable_recipe(EffectLayerId(0)).is_none());
    let report = program
        .execution_reports()
        .iter()
        .find(|r| r.layer == EffectLayerId(0) && r.stage == ExecutionStage::Evolution)
        .unwrap();
    assert_ne!(report.backend, ExecutionBackend::ImmutableAge);
    assert_eq!(report.fallback, Some(ExecutionFallback::NumericPolicy));
    let mut a = instance(&program);
    assert!(!a.immutable_active());
    a.advance(&program, 1. / 64., SceneInputs::default())
        .unwrap();
    assert_eq!(a.execution_stats().immutable_age_invocations, 0);
}
#[test]
fn failed_time_guard_restores_old_phase_then_executes_original_error_prefix_once() {
    let base = closed(1, true);
    let cpu = base
        .clone()
        .with_execution(options(ExecutionPreference::Interpreter));
    let age = base.with_execution(options(ExecutionPreference::ImmutableAge));
    let mut a = instance(&age);
    let mut b = instance(&cpu);
    for instance in [&mut a, &mut b] {
        instance
            .advance(&cpu, 0.25, SceneInputs::default())
            .unwrap();
    }
    equal_public(&a, &b);
    for instance in [&mut a, &mut b] {
        // A bounded fixture representing a very long finite clock reached with
        // individually finite steps; do not loop astronomical history in tests.
        instance.time = 1e40;
        instance.particles[0].cursor = 1e40;
        instance.limits.step_seconds = 1e30;
        instance.fields.write(0, 0, [0.25f32.to_bits(), 0, 0, 0]);
    }
    let error = a.advance(&cpu, 1e30, SceneInputs::default()).unwrap_err();
    let ordinary = b.advance(&cpu, 1e30, SceneInputs::default()).unwrap_err();
    assert_eq!(error, ordinary);
    assert_eq!(error.offset, Some(35));
    assert_eq!(error.message, "effect callback produced a nonfinite scalar");
    assert!(!a.immutable_active());
    assert_eq!(a.execution_stats().immutable_age_fallbacks, 1);
    equal_public(&a, &b);
    let p = a.particles(EffectLayerId(0)).next().unwrap();
    assert_eq!(
        p.field(EffectFieldId(3)).unwrap().words()[0],
        1.25f32.to_bits(),
        "stores preceding failing Time load remain committed"
    );
}

#[test]
fn readonly_values_keep_graph_resources_after_all_programme_wrappers_drop() {
    let retained = {
        let program = closed(1, false).with_execution(options(ExecutionPreference::ImmutableAge));
        let mut instance = instance(&program);
        instance
            .advance(&program, 1. / 64., SceneInputs::default())
            .unwrap();
        instance
    };
    let particle = retained.particles(EffectLayerId(0)).next().unwrap();
    assert!(particle.immutable_sample().is_some());
    assert_eq!(
        particle.field(EffectFieldId(3)).unwrap().words()[0],
        (1f32 + 1. / 256.).to_bits()
    );
    assert!(retained
        .immutable_recipe(EffectLayerId(0))
        .unwrap()
        .curve(0)
        .is_some());
}

#[test]
#[ignore = "requires acquired HD LichAttack; validates all selected native evolution modes"]
fn acquired_lich_initialization_and_complete_field_views_preserve_structure_in_all_modes() {
    use crate::model::formats::pkb::{Document, EffectCompileOptions, EffectEvolution};
    use std::fs::read;
    use std::path::Path;
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/dependencies/files/war3.w3mod/_hd.w3mod/sharedfx/lich/lichattack--f9ecf5d6868f.pkb");
    let bytes = read(path).unwrap();
    let document = Document::read(&bytes).unwrap();
    for evolution in [
        EffectEvolution::Physics,
        EffectEvolution::Fixed,
        EffectEvolution::Varying,
    ] {
        let base = document
            .compile_effect(
                1,
                &EffectCompileOptions {
                    evolution,
                    ..Default::default()
                },
            )
            .unwrap();
        let cpu = base
            .clone()
            .with_execution(options(ExecutionPreference::Interpreter));
        let age = base.with_execution(options(ExecutionPreference::ImmutableAge));
        assert!(age.immutable_recipe(EffectLayerId(3)).is_some());
        let limits = EffectLimits {
            particles: 128,
            step_seconds: 1. / 64.,
            ..Default::default()
        };
        let mut a = age.instantiate(limits, 42).unwrap();
        let mut b = cpu.instantiate(limits, 42).unwrap();
        a.start(&age, age.default_entry(), SceneInputs::default())
            .unwrap();
        b.start(&cpu, cpu.default_entry(), SceneInputs::default())
            .unwrap();
        for _ in 0..120 {
            a.advance(&cpu, 1. / 64., SceneInputs::default()).unwrap();
            b.advance(&cpu, 1. / 64., SceneInputs::default()).unwrap();
            assert_eq!(a.time(), b.time());
            assert_eq!(a.active_count(), b.active_count());
            assert_eq!(format!("{:?}", a.stats()), format!("{:?}", b.stats()));
            for (index, layer) in cpu.inner.layers.iter().enumerate() {
                for (left, right) in a
                    .particles(EffectLayerId(index as u32))
                    .zip(b.particles(EffectLayerId(index as u32)))
                {
                    assert_eq!(left.id, right.id);
                    assert_eq!(left.parent, right.parent);
                    assert_eq!(left.birth_time, right.birth_time);
                    assert_eq!(left.birth_serial, right.birth_serial);
                    for field in 0..layer.fields.len() {
                        let x = left.field(EffectFieldId(field as u32));
                        let y = right.field(EffectFieldId(field as u32));
                        let (Some(x), Some(y)) = (x, y) else {
                            assert_eq!(x, y);
                            continue;
                        };
                        assert_eq!(x.kind(), y.kind());
                        if index != 3
                            || field == layer.life
                            || field == layer.inverse_life
                            || !matches!(
                                x.kind(),
                                ValueType::Float
                                    | ValueType::Float2
                                    | ValueType::Float3
                                    | ValueType::Float4
                            )
                        {
                            assert_eq!(x, y);
                        } else {
                            for (x, y) in x.words().into_iter().zip(y.words()) {
                                let x = f32::from_bits(x);
                                let y = f32::from_bits(y);
                                assert!(x.is_finite() && y.is_finite() && (x-y).abs() <= 1e-4 * y.abs().max(1.), "declared analytic appearance difference exceeded bound: {x} vs {y}");
                            }
                        }
                    }
                }
            }
        }
        assert!(a.execution_stats().immutable_age_invocations > 0);
    }
}
