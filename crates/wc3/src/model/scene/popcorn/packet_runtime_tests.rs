use super::*;
use crate::model::formats::pkb::{Document, EffectCompileOptions, EffectEvolution};
use crate::model::scene::popcorn::curve::sample;
use crate::model::scene::popcorn::packet::{prepare, tests::fixture};
use crate::model::scene::popcorn::{
    Address, BinaryOperation, Destination, Instruction, Operation, Source,
};
use std::fs::read;
use std::path::Path;

fn equivalent(a: &EffectInstance, b: &EffectInstance, program: &EffectProgram) {
    assert_eq!(a.attributes, b.attributes);
    assert_eq!(a.seed, b.seed);
    assert_eq!(a.fields, b.fields);
    assert_eq!(a.active, b.active);
    assert_eq!(a.free, b.free);
    assert_eq!(format!("{:?}", a.pending), format!("{:?}", b.pending));
    assert_eq!(format!("{:?}", a.stats), format!("{:?}", b.stats));
    assert_eq!(
        (
            a.time.to_bits(),
            a.running,
            a.emitting,
            a.faulted,
            a.order,
            a.next_serial
        ),
        (
            b.time.to_bits(),
            b.running,
            b.emitting,
            b.faulted,
            b.order,
            b.next_serial
        )
    );
    for (slot, (left, right)) in a.particles.iter().zip(&b.particles).enumerate() {
        for field in 0..program.inner.layers[left.layer.0 as usize]
            .payload
            .fields
            .len()
        {
            assert_eq!(a.payload.read(slot, field), b.payload.read(slot, field));
        }
        let (a, b) = (left, right);
        assert_eq!(
            (
                a.layer,
                a.generation,
                a.birth.to_bits(),
                a.serial,
                a.cursor.to_bits(),
                a.random.seed,
                a.random.counter,
                a.parent,
                a.payload_frame
            ),
            (
                b.layer,
                b.generation,
                b.birth.to_bits(),
                b.serial,
                b.cursor.to_bits(),
                b.random.seed,
                b.random.counter,
                b.parent,
                b.payload_frame
            )
        );
        let layer = &program.inner.layers[a.layer.0 as usize];
        let plan = layer.evolve.as_ref().and(layer.pure_evolution.as_deref());
        assert!(a.state.same_observable(&b.state, |scope, slot| plan
            .is_some_and(|plan| plan.persistent_writes.contains(&(scope, slot)))));
    }
}
fn pair(program: &EffectProgram, count: usize) -> (EffectInstance, EffectInstance) {
    let limits = EffectLimits {
        particles: count.max(1) + 4,
        step_seconds: 0.125,
        ..Default::default()
    };
    let mut a = program.instantiate(limits, 42).unwrap();
    let mut b = program.instantiate(limits, 42).unwrap();
    assert!(b.packet.is_some());
    a.packet = None;
    a.start(program, 0, SceneInputs::default()).unwrap();
    b.start(program, 0, SceneInputs::default()).unwrap();
    equivalent(&a, &b, program);
    (a, b)
}
fn advance(program: &EffectProgram, a: &mut EffectInstance, b: &mut EffectInstance, delta: f64) {
    if let Some(packet) = &mut b.packet {
        packet.poison();
    }
    assert_eq!(
        a.advance(program, delta, SceneInputs::default()),
        b.advance(program, delta, SceneInputs::default())
    );
    equivalent(a, b, program);
}
#[test]
fn packets_match_scalar_at_width_boundaries_across_poison_reset_stop_and_pool_reuse() {
    for count in [0, WIDTH - 1, WIDTH, WIDTH + 1, WIDTH * 2 + 1] {
        let program = fixture(count);
        let (mut a, mut b) = pair(&program, count);
        for restart in 0..2 {
            if restart > 0 {
                a.start(&program, 0, SceneInputs::default()).unwrap();
                b.start(&program, 0, SceneInputs::default()).unwrap();
            }
            for &slot in &a.active {
                let coordinate = [-0.125f32, 0.125, 0.875, -1.125, 1.875, -0.0][slot % 6];
                let words = [coordinate.to_bits(), 0, 0, 0];
                a.fields.write(2, slot, words);
                b.fields.write(2, slot, words);
            }
            for dt in [0., 0.01, 0.125, 0.25, 0.] {
                advance(&program, &mut a, &mut b, dt);
            }
            a.stop();
            b.stop();
            advance(&program, &mut a, &mut b, 5.);
        }
        let packet = b.packet.as_ref().unwrap();
        if count >= WIDTH {
            assert!(packet.committed > 0, "eligible widths must actually commit");
        } else {
            assert_eq!(packet.attempts, 0);
            assert_eq!(packet.committed, 0);
        }
        assert_eq!(packet.fallbacks, 0);
    }
}
#[test]
fn terminal_and_invalid_lifetimes_preserve_swap_order_and_exact_failure_cursor() {
    for index in [0, WIDTH / 2, WIDTH - 1, WIDTH + 3] {
        for invalid in [false, true] {
            let program = fixture(WIDTH * 2 + 1);
            let (mut a, mut b) = pair(&program, WIDTH * 2 + 1);
            let slot = a.active[index];
            let life = if invalid { f32::NAN } else { 0.99f32 };
            a.fields.write(0, slot, [life.to_bits(), 0, 0, 0]);
            b.fields.write(0, slot, [life.to_bits(), 0, 0, 0]);
            advance(&program, &mut a, &mut b, 0.125);
            if invalid {
                assert!(a.faulted);
                assert_eq!(a.particles[slot].cursor, 0.125);
            } else {
                assert!(!a.active.contains(&slot));
                advance(&program, &mut a, &mut b, 0.125);
            }
        }
    }
}
#[test]
fn failed_speculation_replays_once_with_scalar_store_prefix_and_original_offset() {
    for index in [0, WIDTH / 2, WIDTH - 1, WIDTH + 3] {
        let program = fixture(WIDTH * 2 + 1);
        let (mut a, mut b) = pair(&program, WIDTH * 2 + 1);
        let slot = a.active[index];
        a.fields.write(2, slot, [f32::NAN.to_bits(), 0, 0, 0]);
        b.fields.write(2, slot, [f32::NAN.to_bits(), 0, 0, 0]);
        let left = a
            .advance(&program, 0.125, SceneInputs::default())
            .unwrap_err();
        let right = b
            .advance(&program, 0.125, SceneInputs::default())
            .unwrap_err();
        assert_eq!(left, right);
        assert_eq!(left.offset, Some(35));
        equivalent(&a, &b, &program);
        assert_eq!(b.packet.as_ref().unwrap().attempts, index / WIDTH + 1);
        let packet = b.packet.as_ref().unwrap();
        assert_eq!(packet.fallbacks, 1);
        // Only earlier complete tiles can commit; the failing tile and every
        // later lane remain wholly under scalar replay/error-prefix semantics.
        assert_eq!(packet.committed, (index / WIDTH) * WIDTH);
        assert_eq!(
            a.advance(&program, 0.125, SceneInputs::default()),
            b.advance(&program, 0.125, SceneInputs::default())
        );
        a.start(&program, 0, SceneInputs::default()).unwrap();
        b.start(&program, 0, SceneInputs::default()).unwrap();
        advance(&program, &mut a, &mut b, 0.125);
    }
}
#[test]
fn quota_boundaries_and_denied_step_advance_lifetime_without_packet_backlog() {
    for quota in [1, WIDTH - 1, WIDTH, WIDTH + 1, WIDTH * 2] {
        let program = fixture(WIDTH * 2 + 1);
        let (mut a, mut b) = pair(&program, WIDTH * 2 + 1);
        a.limits.work_per_update = quota;
        b.limits.work_per_update = quota;
        advance(&program, &mut a, &mut b, 0.125);
        assert_eq!(a.stats.omitted_executions, (WIDTH * 2 + 1 - quota) as u64);
        let packet = b.packet.as_ref().unwrap();
        assert_eq!(packet.committed, (quota / WIDTH) * WIDTH);
        assert_eq!(packet.fallbacks, 0);
        assert_eq!(
            a.advance_with_step_budget(&program, 0.125, SceneInputs::default(), || false),
            b.advance_with_step_budget(&program, 0.125, SceneInputs::default(), || false)
        );
        equivalent(&a, &b, &program);
        advance(&program, &mut a, &mut b, 0.125);
    }
}
#[test]
fn interleaved_layers_and_controller_boundary_do_not_gather_across_active_order() {
    for interleaved in [true, false] {
        let mut program = fixture(WIDTH * 2 + 1);
        let graph = Arc::get_mut(&mut program.inner).unwrap();
        let mut controller = graph.layers[0].clone();
        controller.evolve = None;
        controller.spawner = true;
        let visible = graph.layers[0].clone();
        graph.layers = vec![visible, controller].into_boxed_slice();
        graph.events[graph.entries[0].event].targets = if interleaved {
            (0..WIDTH * 2 + 1)
                .map(|index| EffectLayerId((index % 2) as u32))
                .collect()
        } else {
            vec![EffectLayerId(0); WIDTH]
                .into_iter()
                .chain([EffectLayerId(1)])
                .chain(vec![EffectLayerId(0); WIDTH])
                .collect()
        };
        let (mut a, mut b) = pair(&program, WIDTH * 2 + 1);
        advance(&program, &mut a, &mut b, 0.125);
        assert_eq!(
            b.packet.as_ref().unwrap().attempts,
            if interleaved { 0 } else { 2 }
        );
        assert_eq!(
            b.packet.as_ref().unwrap().committed,
            if interleaved { 0 } else { 2 * WIDTH }
        );
        a.stop();
        b.stop();
        advance(&program, &mut a, &mut b, 0.125);
    }
}
#[test]
fn newborn_remaining_interval_stays_scalar_after_existing_tiles() {
    let program = fixture(WIDTH * 2);
    let (mut a, mut b) = pair(&program, WIDTH * 2);
    for instance in [&mut a, &mut b] {
        instance.enqueue(Spawn {
            time: 0.03,
            layer: EffectLayerId(0),
            parent: None,
            seed: 17,
            order: 0,
            payload: None,
            payload_event: 0,
            payload_target: 0,
            payload_fraction: 0.,
            payload_ordinal: 0,
        });
    }
    advance(&program, &mut a, &mut b, 0.125);
    assert_eq!(b.packet.as_ref().unwrap().attempts, 2);
    assert_eq!(b.packet.as_ref().unwrap().committed, 2 * WIDTH);
    assert_eq!(b.packet.as_ref().unwrap().fallbacks, 0);
    let newborn = a.active.last().copied().unwrap();
    assert_eq!(a.particles[newborn].birth, 0.03);
    assert_eq!(a.particles[newborn].cursor, 0.125);
    assert_eq!(
        f32::from_bits(a.fields.read(3, newborn)[0]),
        0.125f32 - 0.095f32
    );
}
#[test]
fn optional_packet_respects_exact_scalar_storage_budget_and_ineligible_graph_has_no_planes() {
    let program = fixture(WIDTH);
    let mut low = 0;
    let mut high = 128 * 1024 * 1024;
    while low < high {
        let middle = low + (high - low) / 2;
        if program
            .instantiate(
                EffectLimits {
                    storage_bytes: middle,
                    ..Default::default()
                },
                0,
            )
            .is_ok()
        {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    let scalar = program
        .instantiate(
            EffectLimits {
                storage_bytes: low,
                ..Default::default()
            },
            0,
        )
        .unwrap();
    assert!(scalar.packet.is_none());
    let extra = PacketScratch::bytes(PacketScratch::shape(&program.inner.layers).unwrap()).unwrap();
    assert!(program
        .instantiate(
            EffectLimits {
                storage_bytes: low + extra - 1,
                ..Default::default()
            },
            0
        )
        .unwrap()
        .packet
        .is_none());
    assert!(program
        .instantiate(
            EffectLimits {
                storage_bytes: low + extra,
                ..Default::default()
            },
            0
        )
        .unwrap()
        .packet
        .is_some());
    let mut program = fixture(WIDTH);
    Arc::get_mut(&mut program.inner).unwrap().layers[0].pure_evolution = None;
    assert!(program
        .instantiate(EffectLimits::default(), 0)
        .unwrap()
        .packet
        .is_none());
}
#[test]
#[ignore = "requires retained native HD Lich; original scalar-versus-packet full runtime differential"]
fn native_lich_selected_physics_fixed_and_varying_packets_keep_every_frame_exact() {
    let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/dependencies/files/war3.w3mod/_hd.w3mod/sharedfx/lich/lichattack--f9ecf5d6868f.pkb");
    let bytes = read(root).unwrap();
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
        assert!(PacketScratch::shape(&program.inner.layers).is_some());
        let multiplier = program
            .attributes()
            .iter()
            .position(|attribute| {
                attribute.name.as_ref() == b"Game.EmissionRateMultiplier"
                    || attribute.name.as_ref() == b"__a_Game.EmissionRateMultiplier"
            })
            .unwrap();
        let limits = EffectLimits {
            particles: 1024,
            step_seconds: 1. / 64.,
            ..Default::default()
        };
        let mut a = program.instantiate(limits, 42).unwrap();
        let mut b = program.instantiate(limits, 42).unwrap();
        a.packet = None;
        a.set_attribute(EffectAttributeId(multiplier as u32), Value::from(64f32))
            .unwrap();
        b.set_attribute(EffectAttributeId(multiplier as u32), Value::from(64f32))
            .unwrap();
        a.start(&program, 0, SceneInputs::default()).unwrap();
        b.start(&program, 0, SceneInputs::default()).unwrap();
        for _ in 0..384 {
            advance(&program, &mut a, &mut b, 1. / 64.);
        }
        assert!(b.packet.as_ref().unwrap().committed > 0);
    }
}

#[test]
fn alias_broadcasts_and_nonfinite_float_words_keep_scalar_normalization_and_order() {
    for alias in [false, true] {
        let mut program = fixture(WIDTH);
        let graph = Arc::get_mut(&mut program.inner).unwrap();
        let bound = graph.layers[0].evolve.as_mut().unwrap();
        let r = Destination {
            scope: 1,
            slot: 0,
            kind: ValueType::Float4,
        };
        let mut operations = vec![
            Instruction::Load {
                destination: r,
                external: 3,
            },
            Instruction::Binary {
                destination: r,
                left: Source {
                    address: Address::Slot { scope: 1, slot: 0 },
                    kind: ValueType::Float4,
                },
                right: Source {
                    address: Address::Constant([
                        (-0.0f32).to_bits(),
                        f32::INFINITY.to_bits(),
                        f32::NAN.to_bits(),
                        1,
                    ]),
                    kind: ValueType::Float4,
                },
                operation: BinaryOperation::Multiply,
            },
        ];
        if alias {
            operations.push(Instruction::Binary {
                destination: r,
                left: Source {
                    address: Address::Slot { scope: 1, slot: 0 },
                    kind: ValueType::Float,
                },
                right: Source {
                    address: Address::Constant([
                        1f32.to_bits(),
                        2f32.to_bits(),
                        3f32.to_bits(),
                        4f32.to_bits(),
                    ]),
                    kind: ValueType::Float4,
                },
                operation: BinaryOperation::Add,
            });
        }
        operations.push(Instruction::Store {
            source: Source {
                address: Address::Slot { scope: 1, slot: 0 },
                kind: ValueType::Float4,
            },
            external: 3,
        });
        bound.program.operations = operations
            .into_iter()
            .enumerate()
            .map(|(offset, instruction)| Operation {
                offset,
                instruction,
            })
            .collect();
        prepare(&mut graph.layers, &graph.samplers);
        let (mut a, mut b) = pair(&program, WIDTH);
        for &slot in &a.active {
            let words = [
                0f32.to_bits(),
                (-0.0f32).to_bits(),
                f32::NEG_INFINITY.to_bits(),
                1,
            ];
            a.fields.write(5, slot, words);
            b.fields.write(5, slot, words);
        }
        advance(&program, &mut a, &mut b, 0.125);
        assert!(b.packet.as_ref().unwrap().committed > 0);
        let words = a.fields.read(5, a.active[0]);
        if alias {
            assert_eq!(
                words,
                [
                    1f32.to_bits(),
                    2f32.to_bits(),
                    3f32.to_bits(),
                    4f32.to_bits()
                ]
            );
        } else {
            assert_eq!(words[0], (-0.0f32).to_bits());
            assert!(f32::from_bits(words[1]).is_nan());
            assert!(f32::from_bits(words[2]).is_nan());
        }
    }
}
#[test]
fn instant_lifetime_and_evolve_on_death_remain_scalar_boundaries() {
    for on_death in [false, true] {
        let mut program = fixture(WIDTH * 2 + 1);
        Arc::get_mut(&mut program.inner).unwrap().layers[0].evolve_on_death = on_death;
        let (mut a, mut b) = pair(&program, WIDTH * 2 + 1);
        let slot = a.active[WIDTH / 2];
        a.fields.write(1, slot, [f32::INFINITY.to_bits(), 0, 0, 0]);
        b.fields.write(1, slot, [f32::INFINITY.to_bits(), 0, 0, 0]);
        advance(&program, &mut a, &mut b, 0.125);
        assert!(!a.active.contains(&slot));
    }
}

#[test]
fn canonical_life_load_observes_advanced_life_before_pure_evolution() {
    let mut program = fixture(WIDTH);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let life = graph.layers[0].life;
    graph.layers[0].evolve.as_mut().unwrap().externals[0] = External::Field(life);
    prepare(&mut graph.layers, &graph.samplers);
    let (mut a, mut b) = pair(&program, WIDTH);
    advance(&program, &mut a, &mut b, 0.125);
    let stale = sample(&program.inner.samplers[0], 0.125)
        .unwrap()
        .words()
        .map(|word| (f32::from_bits(word) * 1.5).to_bits());
    assert_ne!(a.fields.read(5, a.active[0]), stale);
    assert!(b.packet.as_ref().unwrap().committed > 0);
}

#[test]
fn persistent_packets_keep_scalar_storage_and_reuse_only_evolution_dead_words() {
    use crate::model::scene::popcorn::packet::tests::persistent_fixture;
    let program = persistent_fixture(WIDTH * 2 + 1);
    let (mut a, mut b) = pair(&program, WIDTH * 2 + 1);
    let baseline_state =
        State::persistent_for_programs(program.inner.layers.iter().flat_map(|layer| {
            once(&layer.init.program).chain(layer.evolve.iter().map(|bound| &bound.program))
        }));
    assert!(a.particles[0]
        .state
        .same_observable(&baseline_state, |_, _| false));
    assert!(b.particles[0]
        .state
        .same_observable(&baseline_state, |_, _| false));
    advance(&program, &mut a, &mut b, 0.125);
    assert!(b.packet.as_ref().unwrap().committed > 0);
    assert_ne!(
        format!("{:?}", a.particles[a.active[0]].state),
        format!("{:?}", b.particles[b.active[0]].state),
        "successful packets must actually avoid scalar persistent writes"
    );
    // Force one scalar failure after prior packet work, then prove restart
    // clears the pool before an initializer's incoming persistent read.
    let slot = a.active[WIDTH / 2];
    a.fields.write(2, slot, [f32::NAN.to_bits(), 0, 0, 0]);
    b.fields.write(2, slot, [f32::NAN.to_bits(), 0, 0, 0]);
    advance(&program, &mut a, &mut b, 0.125);
    assert!(a.faulted);
    assert!(b.packet.as_ref().unwrap().fallbacks > 0);
    a.start(&program, 0, SceneInputs::default()).unwrap();
    b.start(&program, 0, SceneInputs::default()).unwrap();
    for &slot in &a.active {
        assert_eq!(a.fields.read(3, slot), [0; 4]);
        assert_eq!(b.fields.read(3, slot), [0; 4]);
    }
    equivalent(&a, &b, &program);
    advance(&program, &mut a, &mut b, 0.125);
}
