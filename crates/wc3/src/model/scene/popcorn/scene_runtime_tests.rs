use super::*;
use crate::model::scene::popcorn::graph::Callback;
use glam::Vec3;

fn camera_program(index: i32, context: External) -> EffectProgram {
    let mut program = sprite();
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let init = &mut graph.layers[0].init;
    init.externals = vec![context, External::Field(2)].into();
    init.callbacks = vec![Callback::ViewPosition].into();
    init.program.counts = [0, 2, 0, 0];
    init.program.max_arguments = 2;
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
                arguments: vec![
                    Source {
                        address: Address::Constant([index as u32, 0, 0, 0]),
                        kind: ValueType::Int,
                    },
                    Source {
                        address: Address::Slot { scope: 1, slot: 0 },
                        kind: ValueType::Context,
                    },
                ],
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
    graph.layers[0].evolve = Some(init.clone());
    program
}
fn position(instance: &EffectInstance) -> Vec3 {
    let value = instance
        .particles(EffectLayerId(0))
        .next()
        .unwrap()
        .field(EffectFieldId(2))
        .unwrap();
    let words: [u32; 3] = value.words()[..3].try_into().unwrap();
    Vec3::from_array(words.map(f32::from_bits))
}
#[test]
fn camera_snapshot_updates_without_restarting_or_consuming_randomness() {
    let program = camera_program(0, External::SceneContext);
    let mut instance = program.instantiate(Default::default(), 19).unwrap();
    let snapshot = |position| SceneInputs {
        // A scene camera is already in caller-world coordinates. The emitter
        // frame must not transform this position a second time.
        world_from_effect: Mat4::from_translation(Vec3::splat(1000.)),
        scene: SceneContext::with_camera(position).unwrap(),
    };
    instance
        .start(&program, 0, snapshot(Vec3::new(3., -4., 9.)))
        .unwrap();
    let first = instance.particles(EffectLayerId(0)).next().unwrap().id;
    assert_eq!(position(&instance), Vec3::new(3., -4., 9.));
    instance
        .advance(&program, 0.01, snapshot(Vec3::new(-2., 8., 5.)))
        .unwrap();
    assert_eq!(position(&instance), Vec3::new(-2., 8., 5.));
    assert_eq!(
        instance.particles(EffectLayerId(0)).next().unwrap().id,
        first
    );
    assert_eq!(instance.particles[instance.active[0]].random.counter, 0);
}
#[test]
fn missing_and_unsupported_camera_indices_are_unavailable_not_fatal() {
    for (index, scene) in [
        (0, SceneContext::default()),
        (1, SceneContext::with_camera(Vec3::ONE).unwrap()),
        (-1, SceneContext::with_camera(Vec3::ONE).unwrap()),
    ] {
        let program = camera_program(index, External::SceneContext);
        let mut instance = program.instantiate(Default::default(), 0).unwrap();
        let inputs = SceneInputs {
            scene,
            ..Default::default()
        };
        instance.start(&program, 0, inputs).unwrap();
        instance.advance(&program, 0.01, inputs).unwrap();
        assert_eq!(position(&instance), Vec3::splat(f32::INFINITY));
        assert_eq!(
            instance.active_count(),
            1,
            "unavailable geometry must not stop its effect"
        );
    }
    assert!(SceneContext::with_camera(Vec3::splat(f32::NAN)).is_none());
    assert!(SceneContext::with_camera(Vec3::splat(f32::INFINITY)).is_none());
}
#[test]
fn scene_callback_rejects_other_context_owners_and_forged_zero_capability() {
    for owner in [
        External::ParticleContext,
        External::EffectContext,
        External::RandomContext,
    ] {
        let program = camera_program(0, owner);
        let mut instance = program.instantiate(Default::default(), 0).unwrap();
        assert!(instance
            .start(&program, 0, inputs())
            .unwrap_err()
            .message
            .contains("ownership kind"));
    }
    let mut program = camera_program(0, External::SceneContext);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let Instruction::Call { arguments, .. } =
        &mut graph.layers[0].init.program.operations[1].instruction
    else {
        panic!()
    };
    arguments[1].address = Address::Constant([0; 4]);
    let mut instance = program.instantiate(Default::default(), 0).unwrap();
    assert!(instance
        .start(&program, 0, inputs())
        .unwrap_err()
        .message
        .contains("invalid scene context"));
}

fn turbulence_program(context: External, query: Vec3) -> EffectProgram {
    use crate::model::scene::popcorn::graph::EffectSampler;
    use crate::model::scene::popcorn::turbulence::{Parameters, Turbulence};
    let mut program = camera_program(0, context);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    graph.samplers = vec![EffectSampler::Turbulence(
        Turbulence::new(Parameters {
            time_scale: 0.2,
            ..Default::default()
        })
        .unwrap(),
    )]
    .into();
    let init = &mut graph.layers[0].init;
    init.externals = vec![context, External::Field(2), External::Sampler(0)].into();
    init.callbacks = vec![Callback::TurbulenceSample].into();
    let Instruction::Call {
        arguments,
        receiver,
        ..
    } = &mut init.program.operations[1].instruction
    else {
        panic!()
    };
    arguments[0] = Source {
        address: Address::Constant([query.x.to_bits(), query.y.to_bits(), query.z.to_bits(), 0]),
        kind: ValueType::Float3,
    };
    *receiver = Some(2);
    graph.layers[0].evolve = Some(init.clone());
    program
}
#[test]
fn turbulence_host_uses_effect_time_not_particle_age_and_never_consumes_randomness() {
    use crate::model::scene::popcorn::graph::EffectSampler;
    use crate::model::scene::popcorn::turbulence::TemporalCache;
    let query = Vec3::new(0.1, -0.2, 0.3);
    let program = turbulence_program(External::SceneContext, query);
    let mut instance = program.instantiate(Default::default(), 17).unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    let slot = instance.active[0];
    instance.particles[slot].birth = -0.25;
    instance.advance(&program, 0.1, inputs()).unwrap();
    let EffectSampler::Turbulence(resource) = &program.inner.samplers[0] else {
        panic!()
    };
    let mut cache = TemporalCache::default();
    let expected = resource
        .sample(query, cache.phase(0, resource, 0.1).unwrap())
        .unwrap();
    assert_eq!(position(&instance), expected);
    assert_eq!(instance.particles[slot].random.counter, 0);
    instance.advance(&program, 0., inputs()).unwrap();
    assert_eq!(position(&instance), expected);
}
#[test]
fn turbulence_host_checks_scene_ownership_before_propagating_unavailability() {
    for owner in [
        External::ParticleContext,
        External::RandomContext,
        External::EffectContext,
    ] {
        let program = turbulence_program(owner, Vec3::splat(f32::INFINITY));
        let mut instance = program.instantiate(Default::default(), 0).unwrap();
        assert!(instance
            .start(&program, 0, inputs())
            .unwrap_err()
            .message
            .contains("ownership kind"));
    }
    let program = turbulence_program(External::SceneContext, Vec3::splat(f32::INFINITY));
    let mut instance = program.instantiate(Default::default(), 0).unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    assert_eq!(position(&instance), Vec3::splat(f32::INFINITY));
    assert_eq!(instance.active_count(), 1);
}
