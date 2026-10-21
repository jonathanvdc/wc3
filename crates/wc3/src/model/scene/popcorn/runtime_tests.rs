use super::*;
use crate::model::formats::pkb::{Document, EffectCompileOptions};
use crate::model::scene::popcorn::{Address, Destination, Instruction, Operation, Program, Source};

fn sprite() -> EffectProgram {
    Document::read(include_bytes!("../../../../tests/fixtures/pkb/sprite.pkb"))
        .unwrap()
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap()
}
fn inputs() -> SceneInputs {
    SceneInputs::default()
}
#[test]
fn initialization_is_once_and_pool_reuse_changes_identity() {
    let mut program = sprite();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    // Initialization increments a canonical counter, making accidental reinitialization observable.
    let init = &mut graph.layers[0].init;
    init.externals = vec![External::Field(3)].into_boxed_slice();
    init.program.counts = [0, 1, 0, 0];
    let target = Destination {
        scope: 1,
        slot: 0,
        kind: ValueType::Float,
    };
    let field = Source {
        address: Address::Slot { scope: 1, slot: 0 },
        kind: ValueType::Float,
    };
    init.program.operations = vec![
        Operation {
            offset: 0,
            instruction: Instruction::Load {
                destination: target,
                external: 0,
            },
        },
        Operation {
            offset: 1,
            instruction: Instruction::Binary {
                destination: target,
                left: field,
                right: Source {
                    address: Address::Constant([1f32.to_bits(), 0, 0, 0]),
                    kind: ValueType::Float,
                },
                operation: 0,
            },
        },
        Operation {
            offset: 2,
            instruction: Instruction::Store {
                source: field,
                external: 0,
            },
        },
    ];
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 1,
                ..Default::default()
            },
            7,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    let first = instance.particles(EffectLayerId(0)).next().unwrap().id;
    for _ in 0..30 {
        instance.advance(&program, 1. / 60., inputs()).unwrap();
    }
    let particle = instance.particles(EffectLayerId(0)).next().unwrap();
    assert_eq!(
        f32::from_bits(particle.field(EffectFieldId(3)).unwrap().words()[0]),
        3.
    );
    instance.stop();
    instance.advance(&program, 2., inputs()).unwrap();
    assert!(instance.is_finished());
    instance.start(&program, 0, inputs()).unwrap();
    let next = instance.particles(EffectLayerId(0)).next().unwrap();
    assert_eq!(next.id.slot, first.slot);
    assert_ne!(next.id.generation, first.generation);
    assert_eq!(
        f32::from_bits(next.field(EffectFieldId(3)).unwrap().words()[0]),
        3.
    );
}
#[test]
fn work_and_step_omissions_advance_lifetime_without_backlog() {
    let program = sprite();
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 1,
                steps_per_update: 1,
                work_per_update: 1,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    instance.advance(&program, 4., inputs()).unwrap();
    assert_eq!(instance.active_count(), 0);
    assert_eq!(instance.stats().retired, 1);
    assert!(instance.stats().skipped_seconds > 3.9);
    assert!(instance.stats().omitted_executions > 0);
    assert_eq!(instance.time(), 4.);
    instance.advance(&program, 0.01, inputs()).unwrap();
    assert_eq!(instance.stats().spawned, 1);
}
#[test]
fn root_fanout_obeys_pool_and_pending_bounds() {
    let mut program = sprite();
    Arc::get_mut(&mut program.inner).unwrap().events[0].targets =
        vec![EffectLayerId(0); 8].into_boxed_slice();
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 2,
                pending_events: 3,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    assert_eq!(instance.active_count(), 2);
    assert_eq!(instance.stats().spawned, 2);
    assert_eq!(instance.stats().omitted_births, 6);
}
#[test]
fn instance_rejects_wrong_program_and_bad_inputs_without_resetting_live_state() {
    let program = sprite();
    let other = sprite();
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 1,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    assert!(instance.advance(&other, 0.1, inputs()).is_err());
    assert!(instance.advance(&program, f64::NAN, inputs()).is_err());
    assert!(instance.start(&program, 42, inputs()).is_err());
    assert_eq!(instance.active_count(), 1);
    assert_eq!(instance.time(), 0.);
}
#[test]
fn shared_scratch_allows_large_local_programmes_with_small_particle_budget() {
    let mut program = sprite();
    Arc::get_mut(&mut program.inner).unwrap().layers[0]
        .init
        .program
        .counts = [4096, 4096, 0, 0];
    // 8192 local registers consume 128 KiB once, instead of once per pool slot.
    let limits = EffectLimits {
        particles: 64,
        pending_events: 8,
        occurrences: 8,
        handles: 8,
        storage_bytes: 256 * 1024,
        ..Default::default()
    };
    let mut instance = program.instantiate(limits, 0).unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    assert_eq!(instance.active_count(), 1);
}

fn generator() -> EffectProgram {
    let mut program = sprite();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let visible = graph.layers[0].clone();
    let mut controller = visible.clone();
    controller.rendered = false;
    controller.spawner = true;
    controller.fields[1].initial = [0; 4];
    let mut fields = controller.fields.to_vec();
    fields.push(super::super::EffectField {
        name: b"accumulator".as_slice().into(),
        native_type: 26,
        value_type: Some(ValueType::Int2),
        initial: [0; 4],
    });
    controller.fields = fields.into_boxed_slice();
    let source = |slot, kind| Source {
        address: Address::Slot { scope: 1, slot },
        kind,
    };
    let constant = |word, kind| Source {
        address: Address::Constant([word, 0, 0, 0]),
        kind,
    };
    let float = |value: f32| constant(value.to_bits(), ValueType::Float);
    let call = |slot: Option<usize>, kind, receiver, function, args| Instruction::Call {
        destination: slot.map(|slot| Destination {
            scope: 1,
            slot,
            kind,
        }),
        result_kind: slot.map(|_| kind),
        receiver,
        function,
        arguments: args,
        controls: vec![],
        flags: 0,
    };
    let mut instructions = Vec::new();
    for (slot, external, kind) in [
        (0, 3, ValueType::Int2),
        (1, 4, ValueType::Float),
        (2, 0, ValueType::Context),
        (3, 1, ValueType::Context),
    ] {
        instructions.push(Instruction::Load {
            destination: Destination {
                scope: 1,
                slot,
                kind,
            },
            external,
        });
    }
    instructions.push(call(
        Some(0),
        ValueType::Int3,
        None,
        0,
        vec![
            source(0, ValueType::Int2),
            source(1, ValueType::Float),
            float(0.1),
            float(0.),
            float(1.),
            source(2, ValueType::Context),
        ],
    ));
    instructions.push(Instruction::Store {
        source: source(0, ValueType::Int2),
        external: 3,
    });
    instructions.push(call(
        Some(1),
        ValueType::Int,
        Some(2),
        1,
        vec![
            constant(0, ValueType::Int),
            source(0, ValueType::Int3),
            source(2, ValueType::Context),
        ],
    ));
    instructions.push(call(
        None,
        ValueType::Int,
        Some(2),
        2,
        vec![
            source(1, ValueType::Int),
            source(3, ValueType::Context),
            source(2, ValueType::Context),
        ],
    ));
    controller.evolve = Some(BoundProgram {
        program: Program {
            counts: [0, 4, 0, 0],
            max_arguments: 6,
            operations: instructions
                .into_iter()
                .enumerate()
                .map(|(offset, instruction)| Operation {
                    offset,
                    instruction,
                })
                .collect(),
        },
        externals: vec![
            External::ParticleContext,
            External::RandomContext,
            External::Event(1),
            External::Field(6),
            External::Delta,
        ]
        .into_boxed_slice(),
        callbacks: vec![
            super::super::graph::Callback::Generate,
            super::super::graph::Callback::InitPayload,
            super::super::graph::Callback::Kick,
        ]
        .into_boxed_slice(),
    });
    graph.layers = vec![controller, visible].into_boxed_slice();
    graph.events = vec![
        super::super::graph::EventRoute {
            payload: Default::default(),
            maps: Default::default(),
            targets: vec![EffectLayerId(0)].into_boxed_slice(),
        },
        super::super::graph::EventRoute {
            payload: Default::default(),
            maps: Default::default(),
            targets: vec![EffectLayerId(1)].into_boxed_slice(),
        },
    ]
    .into_boxed_slice();
    graph.layers[1].standalone = false;
    program
}
#[test]
fn subframe_births_partition_consistently_and_visibility_gate_has_no_backlog() {
    let program = generator();
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 64,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    instance.advance(&program, 0.35, inputs()).unwrap();
    let births: Vec<_> = instance
        .particles(EffectLayerId(1))
        .map(|particle| {
            assert!(particle.parent.is_some());
            particle.birth_time
        })
        .collect();
    assert_eq!(births.len(), 3);
    for (actual, expected) in births.iter().zip([0.1, 0.2, 0.3]) {
        assert!((actual - expected).abs() < 1e-6);
    }
    instance.set_emitting(false);
    instance.advance(&program, 0.4, inputs()).unwrap();
    assert_eq!(instance.stats().suppressed_births, 4);
    assert_eq!(instance.particles(EffectLayerId(1)).count(), 3);
    instance.set_emitting(true);
    instance.advance(&program, 0.2, inputs()).unwrap();
    assert_eq!(instance.particles(EffectLayerId(1)).count(), 5);
    assert_eq!(instance.stats().spawned, 6);
    instance.stop();
    instance.advance(&program, 3., inputs()).unwrap();
    assert!(instance.is_finished());
}
#[test]
fn fanout_spawns_distinct_particles_and_capacity_drops_do_not_defer() {
    let mut program = generator();
    Arc::get_mut(&mut program.inner).unwrap().events[1].targets =
        vec![EffectLayerId(1), EffectLayerId(1)].into_boxed_slice();
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 3,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    instance.advance(&program, 0.35, inputs()).unwrap();
    let particles: Vec<_> = instance.particles(EffectLayerId(1)).collect();
    assert_eq!(particles.len(), 2);
    assert_ne!(particles[0].id, particles[1].id);
    assert_eq!(particles[0].parent, particles[1].parent);
    assert_eq!(instance.stats().omitted_births, 4);
}

#[test]
fn invalid_initialized_lifetime_faults_before_any_step_or_budget_omission() {
    for invalid in [f32::NAN, f32::NEG_INFINITY, -1.] {
        let mut program = sprite();
        Arc::get_mut(&mut program.inner).unwrap().layers[0].fields[1].initial[0] =
            invalid.to_bits();
        let mut instance = program
            .instantiate(
                EffectLimits {
                    particles: 1,
                    ..Default::default()
                },
                0,
            )
            .unwrap();
        assert!(instance.start(&program, 0, inputs()).is_err());
        assert!(!instance.is_running());
        assert!(instance.advance(&program, 10., inputs()).is_err());
        assert_eq!(instance.time(), 0.);
    }
}

#[test]
fn authored_instant_controller_runs_terminal_evolution_once_after_positive_time() {
    for evolve_on_death in [false, true] {
        let mut program = generator();
        let graph = Arc::get_mut(&mut program.inner).unwrap();
        graph.layers[0].fields[1].initial = [f32::INFINITY.to_bits(), 0, 0, 0];
        graph.layers[0].evolve_on_death = evolve_on_death;
        let mut instance = program
            .instantiate(
                EffectLimits {
                    particles: 8,
                    step_seconds: 0.1,
                    ..Default::default()
                },
                0,
            )
            .unwrap();
        instance.start(&program, 0, inputs()).unwrap();
        assert_eq!(instance.active_count(), 1);
        instance.advance(&program, 0., inputs()).unwrap();
        assert_eq!(instance.active_count(), 1);
        instance.advance(&program, 0.1, inputs()).unwrap();
        assert_eq!(instance.particles(EffectLayerId(0)).count(), 0);
        assert_eq!(
            instance.particles(EffectLayerId(1)).count(),
            usize::from(evolve_on_death)
        );
        let spawned = instance.stats().spawned;
        instance.advance(&program, 0.1, inputs()).unwrap();
        assert_eq!(instance.stats().spawned, spawned);
    }
}

#[test]
fn instant_initialization_routes_children_before_positive_interval_retirement() {
    let mut program = generator();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let controller = &mut graph.layers[0];
    controller.fields[1].initial = [f32::INFINITY.to_bits(), 0, 0, 0];
    let mut init = controller.evolve.take().unwrap();
    init.program.operations[1].instruction = Instruction::Reinterpret {
        destination: Destination {
            scope: 1,
            slot: 1,
            kind: ValueType::Float,
        },
        source: Source {
            address: Address::Constant([0.1f32.to_bits(), 0, 0, 0]),
            kind: ValueType::Float,
        },
    };
    controller.init = init;
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 8,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    assert_eq!(instance.active_count(), 2);
    assert_eq!(
        instance
            .particles(EffectLayerId(1))
            .next()
            .unwrap()
            .birth_time,
        0.
    );
    instance.advance(&program, 0., inputs()).unwrap();
    assert_eq!(instance.active_count(), 2);
    instance.advance(&program, 0.01, inputs()).unwrap();
    assert_eq!(instance.particles(EffectLayerId(0)).count(), 0);
    assert_eq!(instance.particles(EffectLayerId(1)).count(), 1);
    assert_eq!(instance.stats().spawned, 2);
}
#[test]
fn renderer_bearing_instant_lifetime_is_observable_at_zero_then_retires() {
    let mut program = sprite();
    Arc::get_mut(&mut program.inner).unwrap().layers[0].fields[1].initial =
        [f32::INFINITY.to_bits(), 0, 0, 0];
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 1,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    assert_eq!(instance.particles(EffectLayerId(0)).count(), 1);
    instance.advance(&program, 0., inputs()).unwrap();
    assert_eq!(instance.particles(EffectLayerId(0)).count(), 1);
    instance.advance(&program, 0.01, inputs()).unwrap();
    assert_eq!(instance.particles(EffectLayerId(0)).count(), 0);
}

fn triggered(condition: bool, fraction: f32) -> EffectProgram {
    use super::super::graph::Callback;
    let mut program = generator();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let evolve = graph.layers[0].evolve.as_mut().unwrap();
    evolve.callbacks[0] = Callback::Trigger;
    let Instruction::Call { arguments, .. } = &mut evolve.program.operations[4].instruction else {
        panic!()
    };
    *arguments = vec![
        Source {
            address: Address::Constant([if condition { u32::MAX } else { 0 }, 0, 0, 0]),
            kind: ValueType::Bool,
        },
        Source {
            address: Address::Constant([fraction.to_bits(), 0, 0, 0]),
            kind: ValueType::Float,
        },
        Source {
            address: Address::Slot { scope: 1, slot: 2 },
            kind: ValueType::Context,
        },
    ];
    program
}
#[test]
fn trigger_has_no_edge_latch_and_false_retains_a_valid_empty_batch() {
    for condition in [false, true] {
        let program = triggered(condition, 1.);
        let mut instance = program
            .instantiate(
                EffectLimits {
                    step_seconds: 1.,
                    ..Default::default()
                },
                0,
            )
            .unwrap();
        instance.start(&program, 0, inputs()).unwrap();
        instance.advance(&program, 0., inputs()).unwrap();
        assert_eq!(instance.active_count(), 1);
        instance.advance(&program, 0.25, inputs()).unwrap();
        instance.advance(&program, 0.25, inputs()).unwrap();
        let births: Vec<_> = instance
            .particles(EffectLayerId(1))
            .map(|particle| particle.birth_time)
            .collect();
        assert_eq!(births, if condition { vec![0.25, 0.5] } else { vec![] });
    }
}
#[test]
fn trigger_clamps_interval_fraction_and_rejects_nonfinite_values() {
    for (fraction, expected) in [(-4., 0.), (0., 0.), (0.5, 0.125), (1., 0.25), (4., 0.25)] {
        let program = triggered(true, fraction);
        let mut instance = program
            .instantiate(
                EffectLimits {
                    step_seconds: 1.,
                    ..Default::default()
                },
                0,
            )
            .unwrap();
        instance.start(&program, 0, inputs()).unwrap();
        instance.advance(&program, 0.25, inputs()).unwrap();
        assert_eq!(
            instance
                .particles(EffectLayerId(1))
                .next()
                .unwrap()
                .birth_time,
            expected
        );
    }
    for fraction in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let program = triggered(true, fraction);
        let mut instance = program.instantiate(Default::default(), 0).unwrap();
        instance.start(&program, 0, inputs()).unwrap();
        assert!(instance.advance(&program, 0.25, inputs()).is_err());
    }
}

#[test]
fn trigger_batches_are_bounded_even_when_empty_and_do_not_reuse_keys() {
    for condition in [false, true] {
        let mut program = triggered(condition, 0.);
        let graph = Arc::get_mut(&mut program.inner).unwrap();
        let evolve = graph.layers[0].evolve.as_mut().unwrap();
        let repeated = evolve.program.operations[4].clone();
        evolve.program.operations.insert(5, repeated);
        let mut instance = program
            .instantiate(
                EffectLimits {
                    handles: 1,
                    ..Default::default()
                },
                0,
            )
            .unwrap();
        instance.start(&program, 0, inputs()).unwrap();
        let failure = instance.advance(&program, 0.25, inputs()).unwrap_err();
        assert!(failure.to_string().contains("batch capacity"));
    }
}
#[test]
fn triggers_route_in_subframe_time_order_and_keep_distinct_invocation_batches() {
    let mut program = triggered(true, 1.);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let evolve = graph.layers[0].evolve.as_mut().unwrap();
    // Commit the later event first, then a second event at interval start.
    let mut repeated = evolve.program.operations[4..].to_vec();
    let Instruction::Call { arguments, .. } = &mut repeated[0].instruction else {
        panic!()
    };
    arguments[1].address = Address::Constant([0., 0., 0., 0.].map(f32::to_bits));
    evolve.program.operations.extend(repeated);
    let mut instance = program
        .instantiate(
            EffectLimits {
                step_seconds: 1.,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    instance.advance(&program, 0.25, inputs()).unwrap();
    let births: Vec<_> = instance
        .particles(EffectLayerId(1))
        .map(|particle| particle.birth_time)
        .collect();
    assert_eq!(births, [0., 0.25]);
}
#[test]
fn payload_drafts_created_by_trigger_expire_between_script_invocations() {
    let mut program = triggered(true, 1.);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let controller = &mut graph.layers[0];
    let mut fields = controller.fields.to_vec();
    fields.push(super::super::EffectField {
        name: b"savedDraft".as_slice().into(),
        native_type: 24,
        value_type: Some(ValueType::Int),
        initial: [0; 4],
    });
    controller.fields = fields.into_boxed_slice();
    let mut init = controller.evolve.as_ref().unwrap().clone();
    let mut externals = init.externals.to_vec();
    externals.push(External::Field(7));
    init.externals = externals.into_boxed_slice();
    init.program.operations.pop(); // Retain trigger/initPayload, postpone kick.
    init.program.operations.push(Operation {
        offset: 8,
        instruction: Instruction::Store {
            source: Source {
                address: Address::Slot { scope: 1, slot: 1 },
                kind: ValueType::Int,
            },
            external: 5,
        },
    });
    let mut evolve = controller.evolve.take().unwrap();
    evolve.externals = init.externals.clone();
    let kick = evolve.program.operations.pop().unwrap();
    evolve.program.operations = vec![
        Operation {
            offset: 0,
            instruction: Instruction::Load {
                destination: Destination {
                    scope: 1,
                    slot: 1,
                    kind: ValueType::Int,
                },
                external: 5,
            },
        },
        init.program.operations[2].clone(),
        init.program.operations[3].clone(),
        kick,
    ];
    controller.init = init;
    controller.evolve = Some(evolve);
    let mut instance = program.instantiate(Default::default(), 0).unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    let failure = instance.advance(&program, 0.25, inputs()).unwrap_err();
    assert!(failure.to_string().contains("draft capability is stale"));
}
#[test]
fn trigger_in_initialization_can_route_at_zero_without_warmup_replay() {
    let mut program = triggered(true, 0.5);
    let controller = &mut Arc::get_mut(&mut program.inner).unwrap().layers[0];
    controller.init = controller.evolve.as_ref().unwrap().clone();
    let mut instance = program.instantiate(Default::default(), 0).unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    assert_eq!(
        instance
            .particles(EffectLayerId(1))
            .next()
            .unwrap()
            .birth_time,
        0.
    );
    let spawned = instance.stats().spawned;
    instance.advance(&program, 0., inputs()).unwrap();
    assert_eq!(instance.stats().spawned, spawned);
}

#[path = "payload_runtime_tests.rs"]
mod payload;
#[path = "vrand_runtime_tests.rs"]
mod vrand;

#[test]
fn shared_step_permission_omits_tail_without_replay_and_keeps_local_ceiling() {
    let program = sprite();
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 1,
                steps_per_update: 1,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    let mut calls = 0;
    instance
        .advance_with_step_budget(&program, 0., inputs(), || {
            calls += 1;
            true
        })
        .unwrap();
    assert_eq!(calls, 0, "a zero-delta readiness update spends no steps");
    instance
        .advance_with_step_budget(&program, 0.1, inputs(), || {
            calls += 1;
            true
        })
        .unwrap();
    assert_eq!(
        calls, 1,
        "the existing per-instance step ceiling still applies"
    );
    assert!(instance.stats().skipped_seconds > 0.08);
    let skipped = instance.stats().skipped_seconds;
    instance
        .advance_with_step_budget(&program, 4., inputs(), || false)
        .unwrap();
    assert_eq!(
        instance.active_count(),
        0,
        "denied time still retires expired particles"
    );
    assert_eq!(instance.time(), 4.1);
    assert!((instance.stats().skipped_seconds - skipped - 4.).abs() < 1e-10);
    instance
        .advance_with_step_budget(&program, 0.01, inputs(), || true)
        .unwrap();
    assert_eq!(
        instance.stats().spawned,
        1,
        "the omitted interval has no deferred births"
    );
}

#[test]
fn birth_serial_orders_same_time_births_after_slot_reuse_and_resets_cleanly() {
    let program = sprite();
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 3,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    let original = instance.particles(EffectLayerId(0)).next().unwrap().id;
    let birth = Spawn {
        time: 0.,
        layer: EffectLayerId(0),
        parent: None,
        seed: 0,
        order: 0,
        payload: None,
        payload_event: 0,
        payload_target: 0,
        payload_fraction: 0.,
        payload_ordinal: 0,
    };
    instance
        .birth(&program, birth, 0., inputs(), &mut 0)
        .unwrap();
    instance.retire_at(0);
    instance
        .birth(&program, birth, 0., inputs(), &mut 0)
        .unwrap();
    let mut living: Vec<_> = instance
        .particles(EffectLayerId(0))
        .map(|particle| (particle.birth_serial, particle.id, particle.birth_time))
        .collect();
    living.sort_by_key(|particle| particle.0);
    assert_eq!(
        living.iter().map(|particle| particle.0).collect::<Vec<_>>(),
        [1, 2]
    );
    assert!(
        living[0].1.slot > living[1].1.slot,
        "slot order does not encode chronology"
    );
    assert_eq!(living[0].2, living[1].2);
    instance.start(&program, 0, inputs()).unwrap();
    let restarted = instance.particles(EffectLayerId(0)).next().unwrap();
    assert_eq!(restarted.birth_serial, 0);
    assert_ne!(restarted.id, original);
    instance.next_serial = u64::MAX;
    let free = instance.free.len();
    assert!(instance
        .birth(&program, birth, 0., inputs(), &mut 0)
        .unwrap_err()
        .message
        .contains("serial exhausted"));
    assert_eq!(
        instance.free.len(),
        free,
        "identity exhaustion must not consume a pool slot"
    );
}

#[path = "scene_runtime_tests.rs"]
mod scene_runtime_tests;

#[path = "spatial_runtime_tests.rs"]
mod spatial_runtime_tests;
