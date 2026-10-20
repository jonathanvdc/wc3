use super::*;
use crate::model::scene::popcorn::graph::Callback;
fn random_vector_program(shell: bool) -> EffectProgram {
    let mut program = sprite();
    let init = &mut Arc::get_mut(&mut program.inner).unwrap().layers[0].init;
    init.externals = vec![External::RandomContext, External::Field(2)].into();
    init.callbacks = vec![Callback::VRand].into();
    let context = Source {
        address: Address::Slot { scope: 1, slot: 0 },
        kind: ValueType::Context,
    };
    let mut arguments = Vec::new();
    if shell {
        for radius in [1f32, 0.337] {
            arguments.push(Source {
                address: Address::Constant([radius.to_bits(), 0, 0, 0]),
                kind: ValueType::Float,
            });
        }
    }
    arguments.push(context);
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
        Operation {
            offset: 1,
            instruction: Instruction::Call {
                destination: Some(Destination {
                    scope: 1,
                    slot: 1,
                    kind: ValueType::Float3,
                }),
                result_kind: Some(ValueType::Float3),
                receiver: None,
                function: 0,
                arguments,
                controls: vec![],
                flags: 0,
            },
        },
        Operation {
            offset: 2,
            instruction: Instruction::Store {
                source: Source {
                    address: Address::Slot { scope: 1, slot: 1 },
                    kind: ValueType::Float3,
                },
                external: 1,
            },
        },
    ];
    program
}
#[test]
fn vector_random_consumes_only_its_authored_two_or_three_draws_and_restarts_reproducibly() {
    for shell in [false, true] {
        let program = random_vector_program(shell);
        let mut instance = program.instantiate(Default::default(), 42).unwrap();
        instance.start(&program, 0, inputs()).unwrap();
        let first = instance
            .particles(EffectLayerId(0))
            .next()
            .unwrap()
            .field(EffectFieldId(2))
            .unwrap();
        let slot = instance.active[0];
        assert_eq!(
            instance.particles[slot].random.counter,
            if shell { 3 } else { 2 }
        );
        instance.start(&program, 0, inputs()).unwrap();
        assert_eq!(
            instance
                .particles(EffectLayerId(0))
                .next()
                .unwrap()
                .field(EffectFieldId(2)),
            Some(first)
        );
    }
}
