//! Private child of native_runtime_tests: reuse its full structural oracle.
use super::*;
use crate::model::scene::popcorn::graph::EffectSampler;
use crate::model::scene::popcorn::packet::{
    prepare, simd, tests::persistent_fixture, PARTICLE_BATCH_SIZE,
};
use crate::model::scene::popcorn::{ExecutionStage, ExecutionStats, Instruction};
use std::array::from_fn;

fn programs(base: EffectProgram) -> (EffectProgram, EffectProgram) {
    let interpreted = base
        .clone()
        .with_execution(options(ExecutionPreference::Interpreter));
    let automatic = base.with_execution(options(ExecutionPreference::Automatic));
    assert!(interpreted.same_program(&automatic));
    (interpreted, automatic)
}
fn vector_reports(program: &EffectProgram, available: bool) {
    let mut evolution = false;
    for report in program.execution_reports() {
        let expected =
            if report.layer == EffectLayerId(0) && report.stage == ExecutionStage::Evolution {
                evolution = true;
                if available {
                    ExecutionBackend::SimdJit
                } else {
                    ExecutionBackend::ScalarJit
                }
            } else {
                ExecutionBackend::ScalarJit
            };
        assert_eq!(
            report.backend, expected,
            "actual compiled tile availability: {report:?}"
        );
        assert_eq!(report.fallback, None);
    }
    assert!(evolution);
}
fn work(stats: ExecutionStats) -> [u64; 4] {
    [
        stats.scalar_jit_invocations,
        stats.simd_attempted_batches,
        stats.simd_committed_lanes,
        stats.rust_packet_committed_lanes,
    ]
}
fn delta(current: ExecutionStats, previous: ExecutionStats) -> [u64; 4] {
    let current = work(current);
    let previous = work(previous);
    from_fn(|i| {
        current[i]
            .checked_sub(previous[i])
            .expect("cumulative route counter regressed")
    })
}
fn check_step(
    interpreted: &EffectProgram,
    automatic: &EffectProgram,
    a: &mut EffectInstance,
    b: &mut EffectInstance,
    seconds: f64,
) {
    if let Some(scratch) = &mut b.packet {
        scratch.poison();
    }
    advance(interpreted, automatic, a, b, seconds);
}
#[test]
fn automatic_simd_matches_all_frames_at_sparse_and_dense_populations_and_persistent_banks() {
    for count in [1, 8, 128, 1024] {
        for base in [fixture(count), persistent_fixture(count)] {
            let (interpreted, automatic) = programs(base);
            vector_reports(&automatic, true);
            let (mut a, mut b) = instances(&interpreted, &automatic, count);
            assert_eq!(
                b.execution_stats().scalar_jit_invocations,
                count as u64,
                "initializers remain actual scalar entries"
            );
            let held = b.execution_stats();
            check_step(&interpreted, &automatic, &mut a, &mut b, 0.);
            assert_eq!(b.execution_stats(), held);
            for _ in 0..8 {
                let before = b.execution_stats();
                let rust = a.execution_stats();
                check_step(&interpreted, &automatic, &mut a, &mut b, 0.125);
                let expected = if count >= PARTICLE_BATCH_SIZE {
                    [0, (count / PARTICLE_BATCH_SIZE) as u64, count as u64, 0]
                } else {
                    [count as u64, 0, 0, 0]
                };
                assert_eq!(
                    delta(b.execution_stats(), before),
                    expected,
                    "sparse/dense Automatic dispatch must be real"
                );
                assert_eq!(
                    delta(a.execution_stats(), rust),
                    [
                        0,
                        0,
                        0,
                        if count >= PARTICLE_BATCH_SIZE {
                            count as u64
                        } else {
                            0
                        }
                    ]
                );
            }
            for _ in 0..32 {
                check_step(&interpreted, &automatic, &mut a, &mut b, 0.125);
            }
            assert!(b.active.is_empty());
            a.start(&interpreted, 0, SceneInputs::default()).unwrap();
            b.start(&automatic, 0, SceneInputs::default()).unwrap();
            structural(&a, &b, &interpreted);
            check_step(&interpreted, &automatic, &mut a, &mut b, 0.125);
        }
    }
}
#[test]
fn quota_partial_tiles_and_denied_steps_keep_exact_work_and_no_catchup() {
    for count in [1, 8, 128, 1024] {
        let (interpreted, automatic) = programs(fixture(count));
        vector_reports(&automatic, true);
        let mut quotas = vec![
            1,
            count.min(PARTICLE_BATCH_SIZE - 1),
            count.min(PARTICLE_BATCH_SIZE),
            count.min(PARTICLE_BATCH_SIZE + 1),
            count,
        ];
        quotas.sort_unstable();
        quotas.dedup();
        for quota in quotas {
            let (mut a, mut b) = instances(&interpreted, &automatic, count);
            a.limits.work_per_update = quota;
            b.limits.work_per_update = quota;
            let before = b.execution_stats();
            check_step(&interpreted, &automatic, &mut a, &mut b, 0.125);
            let lanes = quota / PARTICLE_BATCH_SIZE * PARTICLE_BATCH_SIZE;
            assert_eq!(
                delta(b.execution_stats(), before),
                [
                    (quota - lanes) as u64,
                    (lanes / PARTICLE_BATCH_SIZE) as u64,
                    lanes as u64,
                    0
                ]
            );
            assert_eq!(b.stats.omitted_executions, (count - quota) as u64);
            let before = b.execution_stats();
            assert_eq!(
                a.advance_with_step_budget(&interpreted, 0.25, SceneInputs::default(), || false),
                b.advance_with_step_budget(&automatic, 0.25, SceneInputs::default(), || false)
            );
            structural(&a, &b, &interpreted);
            assert_eq!(b.execution_stats(), before);
            assert!(b.stats.skipped_seconds >= 0.25);
            a.limits.work_per_update = EffectLimits::default().work_per_update;
            b.limits.work_per_update = EffectLimits::default().work_per_update;
            let before = b.execution_stats();
            check_step(&interpreted, &automatic, &mut a, &mut b, 0.125);
            assert_eq!(
                delta(b.execution_stats(), before),
                if count >= PARTICLE_BATCH_SIZE {
                    [0, (count / PARTICLE_BATCH_SIZE) as u64, count as u64, 0]
                } else {
                    [count as u64, 0, 0, 0]
                }
            );
        }
    }
}
#[test]
fn terminal_swap_removal_stays_scalar_at_boundaries_then_simd_resumes() {
    let count = 128;
    let (interpreted, automatic) = programs(fixture(count));
    let (mut a, mut b) = instances(&interpreted, &automatic, count);
    for index in [0, 17, 127] {
        let slot = a.active[index];
        a.fields.write(0, slot, [0.99f32.to_bits(), 0, 0, 0]);
        b.fields.write(0, slot, [0.99f32.to_bits(), 0, 0, 0]);
    }
    let before = b.execution_stats();
    check_step(&interpreted, &automatic, &mut a, &mut b, 0.125);
    assert_eq!(b.stats.retired, 3);
    assert_eq!(b.active.len(), 125);
    let actual = delta(b.execution_stats(), before);
    assert!(actual[1] > 0 && actual[2] > 0);
    assert_eq!(
        actual[0] + actual[2],
        125,
        "each survivor executes once and terminal lanes execute no programme"
    );
    assert_eq!(actual[3], 0);
    check_step(&interpreted, &automatic, &mut a, &mut b, 0.125);
}
#[test]
fn failed_simd_tile_replays_once_with_exact_scalar_prefix_and_no_future_lanes() {
    let count = 128;
    let (interpreted, automatic) = programs(fixture(count));
    vector_reports(&automatic, true);
    for index in [0, PARTICLE_BATCH_SIZE / 2 - 1, PARTICLE_BATCH_SIZE, 63, 127] {
        let (mut a, mut b) = instances(&interpreted, &automatic, count);
        let slot = a.active[index];
        a.fields.write(2, slot, [f32::NAN.to_bits(), 0, 0, 0]);
        b.fields.write(2, slot, [f32::NAN.to_bits(), 0, 0, 0]);
        let before = b.execution_stats();
        let left = a
            .advance(&interpreted, 0.125, SceneInputs::default())
            .unwrap_err();
        let right = b
            .advance(&automatic, 0.125, SceneInputs::default())
            .unwrap_err();
        assert_eq!(left, right);
        assert_eq!(right.layer, Some(EffectLayerId(0)));
        assert_eq!(right.offset, Some(35));
        structural(&a, &b, &interpreted);
        assert_eq!(delta(b.execution_stats(),before),[(index%PARTICLE_BATCH_SIZE+1)as u64,(index/PARTICLE_BATCH_SIZE+1)as u64,(index/PARTICLE_BATCH_SIZE*PARTICLE_BATCH_SIZE)as u64,0],"only earlier whole tiles commit; failing tile is one native attempt followed by exact scalar prefix");
        let before = b.execution_stats();
        assert_eq!(
            a.advance(&interpreted, 0.125, SceneInputs::default()),
            b.advance(&automatic, 0.125, SceneInputs::default())
        );
        assert_eq!(b.execution_stats(), before);
        a.start(&interpreted, 0, SceneInputs::default()).unwrap();
        b.start(&automatic, 0, SceneInputs::default()).unwrap();
        let before = b.execution_stats();
        check_step(&interpreted, &automatic, &mut a, &mut b, 0.125);
        assert_eq!(
            delta(b.execution_stats(), before),
            [0, (128 / PARTICLE_BATCH_SIZE) as u64, 128, 0]
        );
    }
}
#[test]
fn captured_automatic_plan_survives_automatic_wrapper_drop_restart_and_opposite_callers() {
    let base = fixture(128);
    let (interpreted, automatic) = programs(base);
    vector_reports(&automatic, true);
    let scalar = interpreted
        .clone()
        .with_execution(options(ExecutionPreference::ScalarJit));
    let limits = EffectLimits {
        particles: 132,
        step_seconds: 0.125,
        ..Default::default()
    };
    let mut a = interpreted.instantiate(limits, 42).unwrap();
    let mut b = automatic.instantiate(limits, 42).unwrap();
    let mut c = scalar.instantiate(limits, 42).unwrap();
    a.start(&automatic, 0, SceneInputs::default()).unwrap();
    b.start(&scalar, 0, SceneInputs::default()).unwrap();
    c.start(&automatic, 0, SceneInputs::default()).unwrap();
    let reports = b.execution_reports().to_vec();
    drop(automatic);
    for _ in 0..2 {
        let simd_before = b.execution_stats();
        let scalar_before = c.execution_stats();
        check_step(&interpreted, &scalar, &mut a, &mut b, 0.125);
        c.advance(&interpreted, 0.125, SceneInputs::default())
            .unwrap();
        structural(&a, &c, &interpreted);
        assert_eq!(
            delta(b.execution_stats(), simd_before),
            [0, (128 / PARTICLE_BATCH_SIZE) as u64, 128, 0]
        );
        assert_eq!(delta(c.execution_stats(), scalar_before), [128, 0, 0, 0]);
        assert_eq!(b.execution_reports(), reports);
    }
    a.start(&scalar, 0, SceneInputs::default()).unwrap();
    b.start(&scalar, 0, SceneInputs::default()).unwrap();
    check_step(&interpreted, &scalar, &mut a, &mut b, 0.125);
    assert_eq!(b.execution_reports(), reports);
}
fn replace_curve(program: &mut EffectProgram, keys: usize, duplicate: bool) {
    let graph = Arc::get_mut(&mut program.inner).unwrap();
    graph.samplers[0] = EffectSampler::Curve {
        kind: ValueType::Float4,
        times: (0..keys).map(|i| i as f32 / (keys - 1) as f32).collect(),
        values: (0..keys)
            .map(|i| [i as f32 / (keys - 1) as f32; 4])
            .collect(),
        tangents: vec![[0.1; 4]; keys * 2].into_boxed_slice(),
    };
    if duplicate {
        let operations = &mut graph.layers[0].evolve.as_mut().unwrap().program.operations;
        let position = operations
            .iter()
            .position(|operation| matches!(operation.instruction, Instruction::Call { .. }))
            .unwrap();
        let original = operations[position].clone();
        let copies = (1usize..=4).map(|extra| {
            let mut copy = original.clone();
            copy.offset += extra;
            copy
        });
        operations.splice(position + 1..position + 1, copies);
    }
    prepare(&mut graph.layers, &graph.samplers);
}
#[test]
fn valid_large_curve_keeps_rust_packet_when_native_scalar_code_is_available() {
    let mut base = fixture(128);
    replace_curve(&mut base, 17, false);
    let (interpreted, automatic) = programs(base);
    vector_reports(&automatic, false);
    let (mut a, mut b) = instances(&interpreted, &automatic, 128);
    for _ in 0..4 {
        let before = b.execution_stats();
        check_step(&interpreted, &automatic, &mut a, &mut b, 0.125);
        assert_eq!(delta(b.execution_stats(), before), [0, 0, 0, 128]);
    }
}
#[test]
fn cold_simd_expansion_budget_keeps_same_valid_scalar_and_rust_execution() {
    let mut base = fixture(128);
    replace_curve(&mut base, 16, true);
    let plan = base.inner.layers[0].pure_evolution.as_ref().unwrap();
    assert!(simd::eligible(plan, &base.inner.samplers));
    assert!(simd::cost(plan,&base.inner.samplers).unwrap()>65_536,"control must exceed the documented cold expansion allowance, not fail ordinary curve eligibility");
    let (interpreted, automatic) = programs(base);
    vector_reports(&automatic, false);
    let (mut a, mut b) = instances(&interpreted, &automatic, 128);
    let before = b.execution_stats();
    check_step(&interpreted, &automatic, &mut a, &mut b, 0.125);
    assert_eq!(delta(b.execution_stats(), before), [0, 0, 0, 128]);
}

#[test]
fn actual_native_burst_128_commits_both_real_execution_routes() {
    use crate::model::formats::pkb::{Document, EffectCompileOptions};
    let document = Document::read(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/benches/fixtures/particle_runtime/burst-128.pkb"
    )))
    .unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    let base = document
        .compile_effect(root, &EffectCompileOptions::default())
        .unwrap();
    let (interpreted, automatic) = programs(base);
    assert!(automatic
        .execution_reports()
        .iter()
        .any(|report| report.layer == EffectLayerId(1)
            && report.stage == ExecutionStage::Evolution
            && report.backend == ExecutionBackend::SimdJit));
    let limits = EffectLimits {
        particles: 129,
        pending_events: 129,
        occurrences: 129,
        ..Default::default()
    };
    let mut a = interpreted.instantiate(limits, 42).unwrap();
    let mut b = automatic.instantiate(limits, 42).unwrap();
    a.start(
        &interpreted,
        interpreted.default_entry(),
        SceneInputs::default(),
    )
    .unwrap();
    b.start(
        &automatic,
        automatic.default_entry(),
        SceneInputs::default(),
    )
    .unwrap();
    for _ in 0..2 {
        check_step(&interpreted, &automatic, &mut a, &mut b, 1. / 64.);
    }
    let left = a.execution_stats();
    let right = b.execution_stats();
    check_step(&interpreted, &automatic, &mut a, &mut b, 1. / 64.);
    assert_eq!(a.particles(EffectLayerId(1)).count(), 128);
    assert_eq!(
        a.execution_stats().rust_packet_committed_lanes - left.rust_packet_committed_lanes,
        128
    );
    assert_eq!(
        b.execution_stats().simd_committed_lanes - right.simd_committed_lanes,
        128
    );
    assert_eq!(b.execution_stats().rust_packet_committed_lanes, 0);
}
