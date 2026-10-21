use super::*;
use crate::model::scene::popcorn::graph::{Callback, SpatialResource};
use glam::Vec3;
const SPATIAL: &[u8] = include_bytes!("../../../../tests/fixtures/pkb/spatial.pkb");
const REORDERED: &[u8] = include_bytes!("../../../../tests/fixtures/pkb/spatial-reordered.pkb");
fn spatial_program(bytes: &[u8]) -> EffectProgram {
    Document::read(bytes)
        .unwrap()
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap()
}
fn position(instance: &EffectInstance, layer: u32) -> Vec3 {
    let value = instance
        .particles(EffectLayerId(layer))
        .next()
        .unwrap()
        .field(EffectFieldId(2))
        .unwrap();
    let words: [u32; 3] = value.words()[..3].try_into().unwrap();
    Vec3::from_array(words.map(f32::from_bits))
}
fn limits() -> EffectLimits {
    EffectLimits {
        step_seconds: 0.01,
        ..Default::default()
    }
}
#[test]
fn native_aliases_share_named_payloads_and_queries_observe_previous_step_in_any_layer_order() {
    for bytes in [SPATIAL, REORDERED] {
        let program = spatial_program(bytes);
        assert_eq!(
            program.inner.spatial.len(),
            1,
            "writer/read flags and local aliases cannot split identity"
        );
        assert_eq!(program.inner.spatial[0].payload_name.as_ref(), b"Position");
        let mut instance = program.instantiate(limits(), 0).unwrap();
        instance.start(&program, 0, inputs()).unwrap();
        for layer in [1, 2] {
            assert_eq!(position(&instance, layer), Vec3::INFINITY);
        }
        instance.advance(&program, 0.01, inputs()).unwrap();
        for layer in [1, 2] {
            assert_eq!(position(&instance, layer), Vec3::new(200., 0., 0.));
        }
        instance.advance(&program, 0., inputs()).unwrap();
        for layer in [1, 2] {
            assert_eq!(position(&instance, layer), Vec3::new(200., 0., 0.));
        }
        instance.advance(&program, 0.02, inputs()).unwrap();
        assert!(instance.particles(EffectLayerId(0)).next().is_none());
        for layer in [1, 2] {
            assert_eq!(
                position(&instance, layer),
                Vec3::INFINITY,
                "expired writers disappear after snapshot rebuild"
            );
        }
    }
}
#[test]
fn shared_query_budget_covers_all_layers_and_internal_steps_without_inventing_birth_omissions() {
    let program = spatial_program(SPATIAL);
    let mut instance = program
        .instantiate(
            EffectLimits {
                spatial_checks_per_update: 1,
                ..limits()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    instance.advance(&program, 0.02, inputs()).unwrap();
    assert_eq!(instance.stats().omitted_spatial_queries, 3);
    assert_eq!(instance.stats().omitted_spatial_writes, 0);
    assert_eq!(instance.stats().omitted_births, 0);
    for layer in [1, 2] {
        assert_eq!(position(&instance, layer), Vec3::INFINITY);
    }
    instance.advance(&program, 0.01, inputs()).unwrap();
    assert_eq!(
        instance.stats().omitted_spatial_queries,
        3,
        "empty read snapshots spend no row checks"
    );
}
#[test]
fn entry_capacity_omissions_are_separate_and_reset_reuses_cleared_snapshots() {
    let program = spatial_program(SPATIAL);
    let mut instance = program
        .instantiate(
            EffectLimits {
                spatial_entries: 0,
                ..limits()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    assert_eq!(instance.stats().omitted_spatial_writes, 1);
    instance.advance(&program, 0.01, inputs()).unwrap();
    assert_eq!(instance.stats().omitted_spatial_writes, 2);
    assert_eq!(instance.stats().omitted_spatial_queries, 0);
    assert_eq!(instance.stats().omitted_births, 0);
    instance.start(&program, 0, inputs()).unwrap();
    assert_eq!(instance.stats().omitted_spatial_writes, 1);
    assert_eq!(position(&instance, 1), Vec3::INFINITY);
}
#[test]
fn skipped_time_discards_stale_snapshots_without_backlog() {
    let program = spatial_program(SPATIAL);
    let mut instance = program
        .instantiate(
            EffectLimits {
                steps_per_update: 1,
                ..limits()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    instance.advance(&program, 0.5, inputs()).unwrap();
    assert!(instance.stats().skipped_seconds > 0.48);
    instance.advance(&program, 0.01, inputs()).unwrap();
    for layer in [1, 2] {
        assert_eq!(position(&instance, layer), Vec3::INFINITY);
    }
    assert_eq!(instance.stats().omitted_births, 0);
}
fn query_controls(program: &mut EffectProgram, argument: usize, value: u32) {
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let layer = &mut graph.layers[1];
    for bound in Some(&mut layer.init)
        .into_iter()
        .chain(layer.evolve.as_mut())
    {
        let operation = bound
            .program
            .operations
            .iter_mut()
            .find(|operation| matches!(&operation.instruction, Instruction::Call { .. }))
            .unwrap();
        let Instruction::Call { arguments, .. } = &mut operation.instruction else {
            unreachable!()
        };
        arguments[argument].address = Address::Constant([value, 0, 0, 0]);
    }
}
#[test]
fn unsupported_query_controls_and_wrong_scene_capabilities_fail_instead_of_guessing() {
    for argument in 2..6 {
        let mut program = spatial_program(SPATIAL);
        query_controls(&mut program, argument, 1);
        let mut instance = program.instantiate(limits(), 0).unwrap();
        assert!(instance
            .start(&program, 0, inputs())
            .unwrap_err()
            .message
            .contains("rank zero"));
    }
    let mut program = spatial_program(SPATIAL);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    graph.layers[1].init.externals[1] = External::ParticleContext;
    let mut instance = program.instantiate(limits(), 0).unwrap();
    assert!(instance
        .start(&program, 0, inputs())
        .unwrap_err()
        .message
        .contains("ownership kind"));
}
#[test]
fn exhausted_invocation_drafts_omit_commits_without_corrupting_the_effect() {
    let mut program = spatial_program(SPATIAL);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let init = &mut graph.layers[0].init;
    let allocation = init.program.operations[1].clone();
    assert_eq!(init.callbacks[0], Callback::SpatialAllocate);
    init.program.operations.insert(2, allocation);
    let mut instance = program
        .instantiate(
            EffectLimits {
                handles: 1,
                ..limits()
            },
            0,
        )
        .unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    assert_eq!(instance.stats().omitted_spatial_writes, 1);
    assert_eq!(instance.active_count(), 3);
    assert_eq!(instance.stats().omitted_births, 0);
}
#[test]
fn draft_handles_cannot_cross_invocations_or_receiver_resources() {
    let mut program = spatial_program(SPATIAL);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let evolution = graph.layers[0].evolve.as_mut().unwrap();
    evolution.program.operations.remove(1);
    for operation in &mut evolution.program.operations {
        if let Instruction::Call { arguments, .. } = &mut operation.instruction {
            arguments[0].address = Address::Constant([1, 0, 0, 0]);
        }
    }
    let mut instance = program.instantiate(limits(), 0).unwrap();
    instance.start(&program, 0, inputs()).unwrap();
    assert!(instance
        .advance(&program, 0.01, inputs())
        .unwrap_err()
        .message
        .contains("this invocation"));
    let mut program = spatial_program(SPATIAL);
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    let mut resources = graph.spatial.to_vec();
    resources.push(SpatialResource {
        name: b"Other".as_slice().into(),
        payload_name: b"Position".as_slice().into(),
        cell_size: 0.75,
    });
    graph.spatial = resources.into();
    let init = &mut graph.layers[0].init;
    let mut externals = init.externals.to_vec();
    externals.push(External::Spatial(1));
    init.externals = externals.into();
    let Instruction::Call { receiver, .. } = &mut init.program.operations[2].instruction else {
        panic!()
    };
    *receiver = Some(2);
    let mut instance = program.instantiate(limits(), 0).unwrap();
    assert!(instance
        .start(&program, 0, inputs())
        .unwrap_err()
        .message
        .contains("another resource"));
}

#[test]
fn spatial_storage_is_checked_before_allocating_unbounded_snapshot_rows() {
    let program = spatial_program(SPATIAL);
    let error = program
        .instantiate(
            EffectLimits {
                spatial_entries: usize::MAX,
                ..limits()
            },
            0,
        )
        .err()
        .expect("overflowing spatial storage must be rejected before allocation");
    assert!(error.message.contains("storage budget"));
}
