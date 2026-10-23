//! Runtime ownership/lifecycle parity, not a generated-code microbenchmark.
use super::*;
#[cfg(not(feature = "popcorn-jit"))]
use crate::model::scene::popcorn::execution_policy::ExecutionFallback;
use crate::model::scene::popcorn::execution_policy::{
    ExecutionBackend, ExecutionOptions, ExecutionPreference, NumericPolicy,
};
use crate::model::scene::popcorn::packet::tests::fixture;
use std::sync::Arc;

fn options(preference: ExecutionPreference) -> ExecutionOptions {
    ExecutionOptions {
        preference,
        numeric_policy: NumericPolicy::Strict,
    }
}
fn wrappers(count: usize) -> (EffectProgram, EffectProgram) {
    let program = fixture(count);
    let interpreted = program
        .clone()
        .with_execution(options(ExecutionPreference::Interpreter));
    let automatic = program.with_execution(options(ExecutionPreference::ScalarJit));
    assert!(interpreted.same_program(&automatic));
    (interpreted, automatic)
}
fn structural(a: &EffectInstance, b: &EffectInstance, program: &EffectProgram) {
    assert_eq!(a.attributes, b.attributes);
    assert_eq!(a.seed, b.seed);
    assert_eq!(
        a.fields, b.fields,
        "all canonical planes, not merely renderer fields"
    );
    assert_eq!(a.active, b.active, "retire/swap order must match");
    assert_eq!(a.free, b.free);
    assert_eq!(format!("{:?}", a.pending), format!("{:?}", b.pending));
    let stats = |x: &EffectStats| {
        (
            x.spawned,
            x.retired,
            x.omitted_births,
            x.omitted_executions,
            x.suppressed_births,
            x.skipped_seconds.to_bits(),
            x.omitted_spatial_writes,
            x.omitted_spatial_queries,
        )
    };
    assert_eq!(stats(&a.stats), stats(&b.stats));
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
        assert_eq!(
            (
                left.layer,
                left.generation,
                left.birth.to_bits(),
                left.serial,
                left.cursor.to_bits(),
                left.random.seed,
                left.random.counter,
                left.parent,
                left.payload_frame
            ),
            (
                right.layer,
                right.generation,
                right.birth.to_bits(),
                right.serial,
                right.cursor.to_bits(),
                right.random.seed,
                right.random.counter,
                right.parent,
                right.payload_frame
            )
        );
        let layer = &program.inner.layers[left.layer.0 as usize];
        for field in 0..layer.payload.fields.len() {
            assert_eq!(a.payload.read(slot, field), b.payload.read(slot, field));
        }
        // The interpreter's established pure packet path need not materialize
        // proven dead persistent temporaries. All other private history remains
        // compared; native code must preserve the raw scalar state contract.
        let plan = layer.pure_evolution.as_deref();
        assert!(
            left.state.same_observable(&right.state, |scope, index| plan
                .is_some_and(|plan| plan.persistent_writes.contains(&(scope, index)))),
            "private persistent history mismatch in slot{slot}"
        );
    }
}
fn instances(
    interpreted: &EffectProgram,
    automatic: &EffectProgram,
    count: usize,
) -> (EffectInstance, EffectInstance) {
    let limits = EffectLimits {
        particles: count + 4,
        step_seconds: 0.125,
        ..Default::default()
    };
    let mut a = interpreted.instantiate(limits, 42).unwrap();
    let mut b = automatic.instantiate(limits, 42).unwrap();
    a.start(interpreted, 0, SceneInputs::default()).unwrap();
    b.start(automatic, 0, SceneInputs::default()).unwrap();
    assert_eq!(a.active.len(), count);
    assert_eq!(b.active.len(), count);
    structural(&a, &b, interpreted);
    (a, b)
}
fn advance(
    interpreted: &EffectProgram,
    automatic: &EffectProgram,
    a: &mut EffectInstance,
    b: &mut EffectInstance,
    delta: f64,
) {
    assert_eq!(
        a.advance(interpreted, delta, SceneInputs::default()),
        b.advance(automatic, delta, SceneInputs::default())
    );
    structural(a, b, interpreted);
}
#[test]
fn instances_capture_execution_plan_across_wrapper_reconfiguration_drop_and_restart() {
    let (interpreted, automatic) = wrappers(8);
    let limits = EffectLimits {
        particles: 12,
        step_seconds: 0.125,
        ..Default::default()
    };
    let mut a = interpreted.instantiate(limits, 42).unwrap();
    let mut b = automatic.instantiate(limits, 42).unwrap();
    let interpreted_reports = a.execution_reports().to_vec();
    let automatic_reports = b.execution_reports().to_vec();
    assert!(Arc::ptr_eq(&a.execution, &interpreted.execution));
    assert!(Arc::ptr_eq(&b.execution, &automatic.execution));
    // Opposite wrappers retain graph/attribute identity but cannot replace the
    // plan captured at instantiation, even when restart is requested.
    a.start(&automatic, 0, SceneInputs::default()).unwrap();
    b.start(&interpreted, 0, SceneInputs::default()).unwrap();
    assert_eq!(a.execution_reports(), interpreted_reports);
    assert_eq!(b.execution_reports(), automatic_reports);
    structural(&a, &b, &interpreted);
    drop(automatic);
    for delta in [0., 0.125, 0.25] {
        advance(&interpreted, &interpreted, &mut a, &mut b, delta);
    }
    assert_eq!(a.execution_stats().scalar_jit_invocations, 0);
    #[cfg(all(
        feature = "popcorn-jit",
        any(unix, windows),
        any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "riscv64",
            target_arch = "s390x"
        )
    ))]
    assert!(
        b.execution_stats().scalar_jit_invocations > 0,
        "captured executable owner must remain alive after wrapper drop"
    );
    a.start(&interpreted, 0, SceneInputs::default()).unwrap();
    b.start(&interpreted, 0, SceneInputs::default()).unwrap();
    advance(&interpreted, &interpreted, &mut a, &mut b, 0.125);
    assert_eq!(a.execution_reports(), interpreted_reports);
    assert_eq!(b.execution_reports(), automatic_reports);
    let foreign = fixture(8);
    let before = (
        b.fields.clone(),
        b.active.clone(),
        b.time.to_bits(),
        b.execution_stats(),
    );
    assert!(b.advance(&foreign, 0.125, SceneInputs::default()).is_err());
    assert_eq!(
        before,
        (
            b.fields.clone(),
            b.active.clone(),
            b.time.to_bits(),
            b.execution_stats()
        ),
        "foreign identity must reject before native entry or canonical mutation"
    );
}
#[cfg(not(feature = "popcorn-jit"))]
#[test]
fn automatic_without_feature_is_explicit_interpreter_fallback_with_ordinary_results() {
    let (interpreted, automatic) = wrappers(8);
    for report in automatic.execution_reports() {
        assert_eq!(report.backend, ExecutionBackend::Interpreter);
        assert_eq!(report.fallback, Some(ExecutionFallback::FeatureDisabled));
    }
    for report in interpreted.execution_reports() {
        assert_eq!(report.backend, ExecutionBackend::Interpreter);
        assert_eq!(report.fallback, None);
    }
    let (mut a, mut b) = instances(&interpreted, &automatic, 8);
    for delta in [0., 0.125, 0.25, 5.] {
        advance(&interpreted, &automatic, &mut a, &mut b, delta);
    }
    assert_eq!(a.execution_stats().scalar_jit_invocations, 0);
    assert_eq!(b.execution_stats().scalar_jit_invocations, 0);
}
#[cfg(all(
    feature = "popcorn-jit",
    any(unix, windows),
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64",
        target_arch = "s390x"
    )
))]
mod supported {
    use super::*;
    fn native(automatic: &EffectProgram) {
        assert!(!automatic.execution_reports().is_empty());
        for report in automatic.execution_reports() {
            assert_eq!(
                report.backend,
                ExecutionBackend::ScalarJit,
                "native must actually be prepared: {report:?}"
            );
            assert_eq!(report.fallback, None);
        }
    }
    #[test]
    fn scalar_jit_matches_every_frame_at_small_and_dense_populations_through_reuse_and_stop() {
        for count in [1, 8, 128, 1024] {
            let (interpreted, automatic) = wrappers(count);
            native(&automatic);
            let (mut a, mut b) = instances(&interpreted, &automatic, count);
            let birth_entries = b.execution_stats().scalar_jit_invocations;
            assert_eq!(
                birth_entries, count as u64,
                "initializer entries must be real"
            );
            let before_pause = b.execution_stats();
            advance(&interpreted, &automatic, &mut a, &mut b, 0.);
            assert_eq!(b.execution_stats(), before_pause, "hold executes nothing");
            for frame in 0..40 {
                advance(
                    &interpreted,
                    &automatic,
                    &mut a,
                    &mut b,
                    if frame % 3 == 0 { 0.25 } else { 0.125 },
                );
            }
            let interpreted_commits = a.packet.as_ref().map_or(0, |scratch| scratch.committed);
            if count >= 128 {
                assert!(
                    interpreted_commits > 0,
                    "dense Interpreter must commit real packet lanes"
                );
            } else {
                assert_eq!(
                    interpreted_commits, 0,
                    "short populations retain scalar execution"
                );
            }
            assert_eq!(
                b.packet.as_ref().map_or(0, |scratch| scratch.committed),
                0,
                "ScalarJit must not substitute the Rust packet route"
            );
            assert!(a.active.is_empty());
            assert!(b.execution_stats().scalar_jit_invocations > birth_entries);
            assert_eq!(a.execution_stats().scalar_jit_invocations, 0);
            a.start(&interpreted, 0, SceneInputs::default()).unwrap();
            b.start(&automatic, 0, SceneInputs::default()).unwrap();
            structural(&a, &b, &interpreted);
            advance(&interpreted, &automatic, &mut a, &mut b, 0.125);
            a.stop();
            b.stop();
            for delta in [0., 0.125, 5.] {
                advance(&interpreted, &automatic, &mut a, &mut b, delta);
            }
        }
    }
    #[test]
    fn partial_work_and_denied_steps_preserve_canonical_outputs_omissions_and_entry_counts() {
        for count in [1, 8, 128, 1024] {
            let (interpreted, automatic) = wrappers(count);
            native(&automatic);
            for quota in [1, count.div_ceil(2), count] {
                let (mut a, mut b) = instances(&interpreted, &automatic, count);
                a.limits.work_per_update = quota;
                b.limits.work_per_update = quota;
                let before = b.execution_stats().scalar_jit_invocations;
                advance(&interpreted, &automatic, &mut a, &mut b, 0.125);
                assert_eq!(
                    b.execution_stats().scalar_jit_invocations - before,
                    quota.min(count) as u64
                );
                assert_eq!(
                    b.stats.omitted_executions,
                    (count - quota.min(count)) as u64
                );
                let before = b.execution_stats();
                assert_eq!(
                    a.advance_with_step_budget(&interpreted, 0.25, SceneInputs::default(), || {
                        false
                    }),
                    b.advance_with_step_budget(&automatic, 0.25, SceneInputs::default(), || false)
                );
                structural(&a, &b, &interpreted);
                assert_eq!(
                    b.execution_stats(),
                    before,
                    "denied steps cannot enter native code"
                );
                assert!(b.stats.skipped_seconds >= 0.25);
                a.limits.work_per_update = EffectLimits::default().work_per_update;
                b.limits.work_per_update = EffectLimits::default().work_per_update;
                advance(&interpreted, &automatic, &mut a, &mut b, 0.125);
            }
        }
    }
    #[test]
    fn birth_omissions_and_mixed_terminal_swap_remove_use_only_authorized_native_entries() {
        for count in [1usize, 8, 128, 1024] {
            let (interpreted, automatic) = wrappers(count);
            native(&automatic);
            let quota = count.div_ceil(2);
            let limits = EffectLimits {
                particles: count + 4,
                work_per_update: quota,
                step_seconds: 0.125,
                ..Default::default()
            };
            let mut a = interpreted.instantiate(limits, 42).unwrap();
            let mut b = automatic.instantiate(limits, 42).unwrap();
            a.start(&interpreted, 0, SceneInputs::default()).unwrap();
            b.start(&automatic, 0, SceneInputs::default()).unwrap();
            structural(&a, &b, &interpreted);
            assert_eq!(b.active.len(), quota);
            assert_eq!(b.stats.omitted_births, (count - quota) as u64);
            assert_eq!(b.execution_stats().scalar_jit_invocations, quota as u64);
        }
        for count in [8, 128] {
            let (interpreted, automatic) = wrappers(count);
            native(&automatic);
            let (mut a, mut b) = instances(&interpreted, &automatic, count);
            for index in [0, count / 2, count - 1] {
                let slot = a.active[index];
                a.fields.write(0, slot, [0.99f32.to_bits(), 0, 0, 0]);
                b.fields.write(0, slot, [0.99f32.to_bits(), 0, 0, 0]);
            }
            let before = b.execution_stats().scalar_jit_invocations;
            advance(&interpreted, &automatic, &mut a, &mut b, 0.125);
            assert_eq!(b.stats.retired, 3);
            assert_eq!(
                b.execution_stats().scalar_jit_invocations - before,
                (count - 3) as u64
            );
            advance(&interpreted, &automatic, &mut a, &mut b, 0.125);
        }
    }
    #[test]
    fn native_callback_fault_matches_exact_store_prefix_layer_offset_and_restart() {
        let count = 128;
        let (interpreted, automatic) = wrappers(count);
        native(&automatic);
        for index in [0, 7, 16, 127] {
            let (mut a, mut b) = instances(&interpreted, &automatic, count);
            let slot = a.active[index];
            a.fields.write(2, slot, [f32::NAN.to_bits(), 0, 0, 0]);
            b.fields.write(2, slot, [f32::NAN.to_bits(), 0, 0, 0]);
            let before = b.execution_stats().scalar_jit_invocations;
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
            assert_eq!(
                b.execution_stats().scalar_jit_invocations - before,
                (index + 1) as u64,
                "includes faulting entry and excludes future lanes"
            );
            let before = b.execution_stats();
            assert_eq!(
                a.advance(&interpreted, 0.125, SceneInputs::default()),
                b.advance(&automatic, 0.125, SceneInputs::default())
            );
            assert_eq!(
                b.execution_stats(),
                before,
                "fault blocks future execution until explicit restart"
            );
            a.start(&interpreted, 0, SceneInputs::default()).unwrap();
            b.start(&automatic, 0, SceneInputs::default()).unwrap();
            advance(&interpreted, &automatic, &mut a, &mut b, 0.125);
        }
    }
}

#[cfg(all(
    feature = "popcorn-jit",
    any(unix, windows),
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
#[path = "simd_runtime_tests.rs"]
mod simd_tests;
