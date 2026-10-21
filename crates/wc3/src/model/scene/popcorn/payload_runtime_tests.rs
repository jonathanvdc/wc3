use super::*;
use crate::model::scene::popcorn::graph::{Callback, EventRoute};
use crate::model::scene::popcorn::payload::{Field as PayloadField, Layout as PayloadLayout};
use glam::{Mat4, Quat, Vec3, Vec4};
use std::mem::take;
fn field(name: &[u8], kind: ValueType, native_type: u32, role: u32) -> PayloadField {
    PayloadField {
        name: name.into(),
        kind: Some(kind),
        native_type,
        footprint: kind.components() as u32 * 4,
        role,
    }
}
fn constant(value: Value) -> Source {
    Source {
        address: Address::Constant(value.words()),
        kind: value.kind(),
    }
}
fn register(slot: usize, kind: ValueType) -> Source {
    Source {
        address: Address::Slot { scope: 1, slot },
        kind,
    }
}
fn call(
    slot: usize,
    kind: ValueType,
    receiver: Option<u16>,
    function: u16,
    arguments: Vec<Source>,
) -> Operation {
    Operation {
        offset: 0,
        instruction: Instruction::Call {
            destination: Some(Destination {
                scope: 1,
                slot,
                kind,
            }),
            result_kind: Some(kind),
            receiver,
            function,
            arguments,
            controls: vec![],
            flags: 0,
        },
    }
}
fn routed() -> EffectProgram {
    let mut program = triggered(true, 0.5);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let outgoing = PayloadLayout::new(
        vec![
            field(b"Position", ValueType::Float3, 33, 1),
            field(b"Color", ValueType::Float4, 34, 0),
        ]
        .into(),
    )
    .unwrap();
    let root = PayloadLayout::new(
        vec![
            field(b"UnusedSize", ValueType::Float, 31, 0),
            field(b"Position", ValueType::Float3, 33, 1),
            field(b"Color", ValueType::Float4, 34, 0),
        ]
        .into(),
    )
    .unwrap();
    graph.events[1].maps = vec![outgoing.map_to(&root).unwrap()].into();
    graph.events[1].payload = outgoing;
    graph.layers[1].payload = root;
    let evolve = graph.layers[0].evolve.as_mut().unwrap();
    evolve.program.counts[1] = 6;
    evolve.callbacks = vec![
        Callback::Trigger,
        Callback::InitPayload,
        Callback::Kick,
        Callback::BuildElement,
        Callback::Append,
    ]
    .into();
    let position = call(
        4,
        ValueType::Int,
        None,
        3,
        vec![
            register(0, ValueType::Int3),
            constant(Value::from(Vec3::X * 2.)),
            constant(Value::from(Vec3::X * 6.)),
            constant(Value::from(258i32)),
            register(2, ValueType::Context),
        ],
    );
    let color = call(
        5,
        ValueType::Int,
        None,
        3,
        vec![
            register(0, ValueType::Int3),
            constant(Value::from(Vec4::ZERO)),
            constant(Value::from(Vec4::new(4., 8., 2., 3.))),
            constant(Value::from(1i32)),
            register(2, ValueType::Context),
        ],
    );
    let append = |slot: i32, element| {
        call(
            1,
            ValueType::Int,
            Some(2),
            4,
            vec![
                register(1, ValueType::Int),
                constant(Value::from(slot)),
                register(element, ValueType::Int),
                register(2, ValueType::Context),
            ],
        )
    };
    let kick = evolve.program.operations.pop().unwrap();
    evolve
        .program
        .operations
        .extend([position, color, append(0, 4), append(1, 5), kick]);
    let init = &mut graph.layers[1].init;
    init.externals = vec![
        External::ParticleContext,
        External::Field(2),
        External::Field(5),
    ]
    .into();
    init.callbacks = vec![Callback::Transform, Callback::Extract(ValueType::Float4)].into();
    init.program.counts = [0, 2, 0, 0];
    init.program.max_arguments = 3;
    init.program.operations = vec![
        Operation {
            offset: 0,
            instruction: Instruction::Load {
                destination: Destination {
                    scope: 1,
                    slot: 0,
                    kind: ValueType::Context,
                },
                external: 0,
            },
        },
        call(
            1,
            ValueType::Float3,
            None,
            0,
            vec![
                constant(Value::from(Vec3::X)),
                constant(Value::from(19i32)),
                register(0, ValueType::Context),
            ],
        ),
        Operation {
            offset: 1,
            instruction: Instruction::Store {
                source: register(1, ValueType::Float3),
                external: 1,
            },
        },
        call(
            1,
            ValueType::Float4,
            None,
            1,
            vec![constant(Value::from(2i32)), register(0, ValueType::Context)],
        ),
        Operation {
            offset: 2,
            instruction: Instruction::Store {
                source: register(1, ValueType::Float4),
                external: 2,
            },
        },
    ];
    program
}
#[test]
fn queued_payloads_interpolate_and_map_into_sparse_root_layout_before_init() {
    let program = routed();
    let mut instance = program
        .instantiate(
            EffectLimits {
                step_seconds: 1.,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    let inputs = SceneInputs {
        world_from_effect: Mat4::from_scale_rotation_translation(
            Vec3::splat(3.),
            Quat::from_rotation_z(1.),
            Vec3::splat(100.),
        ),
        ..Default::default()
    };
    instance.start(&program, 0, inputs).unwrap();
    instance.advance(&program, 0.25, inputs).unwrap();
    let particle = instance.particles(EffectLayerId(1)).next().unwrap();
    assert_eq!(particle.birth_time, 0.125);
    assert_eq!(
        particle.field(EffectFieldId(2)).unwrap(),
        Value::from(Vec3::X * 5.)
    );
    assert_eq!(
        particle.field(EffectFieldId(5)).unwrap(),
        Value::from(Vec4::new(2., 4., 1., 1.5))
    );
}
#[test]
fn shared_fanout_payloads_release_on_omission_and_hidden_births_without_backlog() {
    let mut program = routed();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    graph.events[1].targets = vec![EffectLayerId(1); 2].into();
    graph.events[1].maps = vec![vec![1, 2].into_boxed_slice(); 2].into();
    let mut instance = program
        .instantiate(
            EffectLimits {
                particles: 8,
                pending_events: 1,
                step_seconds: 1.,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    for _ in 0..4 {
        instance.advance(&program, 0.25, inputs()).unwrap();
    }
    assert_eq!(instance.particles(EffectLayerId(1)).count(), 4);
    assert_eq!(instance.stats().omitted_births, 4);
    instance.set_emitting(false);
    for _ in 0..20 {
        instance.advance(&program, 0.25, inputs()).unwrap();
    }
    assert_eq!(instance.stats().suppressed_births, 40);
    instance.set_emitting(true);
    instance.advance(&program, 0.25, inputs()).unwrap();
    assert_eq!(instance.particles(EffectLayerId(1)).count(), 1);
    instance.start(&program, 0, inputs()).unwrap();
    instance.advance(&program, 0.25, inputs()).unwrap();
    assert_eq!(instance.particles(EffectLayerId(1)).count(), 1);
}
#[test]
fn payload_word_budget_omits_whole_births_and_resets_each_update() {
    let program = routed();
    for (budget, births) in [(1, 0), (50, 0), (100, 2)] {
        let mut instance = program
            .instantiate(
                EffectLimits {
                    payload_words_per_update: budget,
                    step_seconds: 1.,
                    ..Default::default()
                },
                0,
            )
            .unwrap();
        instance.start(&program, 0, inputs()).unwrap();
        instance.advance(&program, 0.25, inputs()).unwrap();
        instance.advance(&program, 0.25, inputs()).unwrap();
        assert_eq!(instance.particles(EffectLayerId(1)).count(), births);
        assert_eq!(
            instance.stats().omitted_births,
            if births == 0 { 2 } else { 0 }
        );
    }
}
#[test]
fn element_capabilities_cannot_be_appended_to_an_unrelated_batch() {
    let mut program = routed();
    let evolve = Arc::get_mut(&mut program.inner).unwrap().layers[0]
        .evolve
        .as_mut()
        .unwrap();
    // Build the endpoints for the original batch, then replace the receiving draft
    // with one created by a second trigger invocation in this same script.
    let second_trigger = evolve.program.operations[4].clone();
    let second_draft = evolve.program.operations[6].clone();
    evolve.program.operations.insert(9, second_draft);
    evolve.program.operations.insert(9, second_trigger);
    let mut instance = program.instantiate(Default::default(), 0).unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    let failure = instance.advance(&program, 0.25, inputs()).unwrap_err();
    assert!(failure.message.contains("another batch"));
    assert_eq!(instance.stats().spawned, 1);
}
#[test]
fn nonzero_native_control_has_independent_draft_ownership() {
    let mut program = routed();
    let evolve = Arc::get_mut(&mut program.inner).unwrap().layers[0]
        .evolve
        .as_mut()
        .unwrap();
    let Instruction::Call { arguments, .. } = &mut evolve.program.operations[6].instruction else {
        panic!()
    };
    arguments[0] = constant(Value::from(2i32));
    let mut instance = program.instantiate(Default::default(), 0).unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    instance.advance(&program, 1. / 60., inputs()).unwrap();
    assert_eq!(instance.particles(EffectLayerId(1)).count(), 1);
}
#[test]
fn queued_ownership_is_released_before_a_child_initializer_routes_its_own_payload() {
    let mut program = routed();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let visible = graph.layers[1].clone();
    let mut forwarding = graph.layers[0].clone();
    forwarding.payload = visible.payload.clone();
    forwarding.init = forwarding.evolve.take().unwrap();
    forwarding.init.externals[2] = External::Event(2);
    graph.layers = vec![graph.layers[0].clone(), forwarding, visible].into();
    let route = EventRoute {
        targets: vec![EffectLayerId(2)].into(),
        payload: graph.events[1].payload.clone(),
        maps: graph.events[1].maps.clone(),
    };
    let mut events = take(&mut graph.events).into_vec();
    events.push(route);
    graph.events = events.into();
    let mut instance = program
        .instantiate(
            EffectLimits {
                pending_events: 1,
                step_seconds: 1.,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    instance.advance(&program, 0.25, inputs()).unwrap();
    assert_eq!(instance.particles(EffectLayerId(2)).count(), 1);
    assert_eq!(instance.stats().omitted_births, 0);
}
