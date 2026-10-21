use super::*;
use crate::model::scene::popcorn::MissingPayloadPolicy;

fn sparse_broadcast(policy: MissingPayloadPolicy) -> EffectProgram {
    let mut program = routed();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    graph.missing_payload = policy;
    graph.events[1].targets = vec![EffectLayerId(1); 2].into();
    graph.events[1].maps = vec![vec![1, 2].into_boxed_slice(); 2].into();
    let evolve = graph.layers[0].evolve.as_mut().unwrap();
    let mut second = evolve.program.operations[6].clone();
    let Instruction::Call { arguments, .. } = &mut second.instruction else {
        panic!()
    };
    arguments[0] = constant(Value::from(1i32));
    let position = evolve.program.operations[9].clone();
    let color = evolve.program.operations[10].clone();
    let kick = evolve.program.operations[11].clone();
    // The first independent packet has Position but omits Color. The second
    // appends the same nonzero prepared Color element, using the same batch.
    evolve.program.operations.remove(10);
    evolve
        .program
        .operations
        .extend([second, position, color, kick]);
    program
}
#[test]
fn sparse_broadcast_preserves_packets_supplied_values_and_explicit_zero_policy() {
    for policy in [MissingPayloadPolicy::Error, MissingPayloadPolicy::Zero] {
        let program = sparse_broadcast(policy);
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
        let result = instance.advance(&program, 0.25, inputs());
        if policy == MissingPayloadPolicy::Error {
            assert!(result
                .unwrap_err()
                .message
                .contains("not supplied by this birth"));
            continue;
        }
        result.unwrap();
        let particles: Vec<_> = instance.particles(EffectLayerId(1)).collect();
        assert_eq!(
            particles.len(),
            4,
            "both packets still broadcast to both targets"
        );
        let values: Vec<_> = particles
            .iter()
            .map(|particle| particle.field(EffectFieldId(5)).unwrap())
            .collect();
        assert_eq!(
            values
                .iter()
                .filter(|value| **value == Value::from(Vec4::ZERO))
                .count(),
            2
        );
        assert_eq!(
            values
                .iter()
                .filter(|value| **value == Value::from(Vec4::new(2., 4., 1., 1.5)))
                .count(),
            2
        );
        for particle in particles {
            assert_eq!(
                particle.field(EffectFieldId(2)).unwrap(),
                Value::from(Vec3::X * 5.)
            );
            // Defaulting an extracted value must not fabricate channel presence.
            let stored = instance.payload.read(particle.id.slot as usize, 2).unwrap();
            if particle.field(EffectFieldId(5)).unwrap() == Value::from(Vec4::ZERO) {
                assert_eq!(stored, None);
            } else {
                assert_eq!(stored, Some(Value::from(Vec4::new(2., 4., 1., 1.5))));
            }
        }
    }
}
#[test]
fn zero_policy_does_not_accept_unknown_root_slots_or_wrong_declared_layouts() {
    for root_slot in [0, 3, u32::MAX] {
        let mut program = sparse_broadcast(MissingPayloadPolicy::Zero);
        let init = &mut Arc::get_mut(&mut program.inner).unwrap().layers[1].init;
        let Instruction::Call { arguments, .. } = &mut init.program.operations[3].instruction
        else {
            panic!()
        };
        // Slot0 is declared Float, not Float4. Slot3 and MAX are undeclared.
        arguments[0] = constant(Value::from(root_slot as i32));
        let mut instance = program.instantiate(Default::default(), 0).unwrap();
        instance.start(&program, 0, inputs()).unwrap();
        let failure = instance.advance(&program, 0.25, inputs()).unwrap_err();
        assert!(failure.message.contains("root declaration"));
    }
}
#[test]
fn zero_policy_preserves_capability_batch_ownership_validation() {
    let mut program = sparse_broadcast(MissingPayloadPolicy::Zero);
    let evolve = Arc::get_mut(&mut program.inner).unwrap().layers[0]
        .evolve
        .as_mut()
        .unwrap();
    let trigger = evolve.program.operations[4].clone();
    let draft = evolve.program.operations[6].clone();
    evolve.program.operations.insert(9, draft);
    evolve.program.operations.insert(9, trigger);
    let mut instance = program.instantiate(Default::default(), 0).unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    assert!(instance
        .advance(&program, 0.25, inputs())
        .unwrap_err()
        .message
        .contains("another batch"));
}
