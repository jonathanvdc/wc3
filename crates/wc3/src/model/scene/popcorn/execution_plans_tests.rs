use super::*;
use crate::model::scene::popcorn::packet::tests::fixture;
use crate::model::scene::popcorn::{EffectLimits, ExecutionPreference, NumericPolicy};
use std::sync::Barrier;
use std::thread;

fn interpreted(policy: NumericPolicy) -> ExecutionOptions {
    ExecutionOptions {
        preference: ExecutionPreference::Interpreter,
        numeric_policy: policy,
    }
}

#[test]
fn exact_policy_live_plan_is_shared_without_changing_graph_or_instance_ownership() {
    let source = fixture(16);
    let default = source.clone().with_execution(source.execution_options());
    assert!(Arc::ptr_eq(&source.execution, &default.execution));
    let strict = source
        .clone()
        .with_execution(interpreted(NumericPolicy::Strict));
    let repeated = source
        .clone()
        .with_execution(interpreted(NumericPolicy::Strict));
    assert!(Arc::ptr_eq(&strict.execution, &repeated.execution));
    assert!(!Arc::ptr_eq(&source.execution, &strict.execution));
    assert!(source.same_program(&strict));
    let owners = Arc::strong_count(&strict.execution);
    let instance = strict.instantiate(EffectLimits::default(), 42).unwrap();
    assert_eq!(Arc::strong_count(&strict.execution), owners + 1);
    drop(instance);
    assert_eq!(Arc::strong_count(&strict.execution), owners);
    let foreign = fixture(16).with_execution(interpreted(NumericPolicy::Strict));
    assert!(!source.same_program(&foreign));
    assert!(!Arc::ptr_eq(&strict.execution, &foreign.execution));
}

#[test]
fn cache_does_not_retain_executable_plans_and_instances_keep_them_alive() {
    let source = fixture(16);
    let strict = source
        .clone()
        .with_execution(interpreted(NumericPolicy::Strict));
    let weak = Arc::downgrade(&strict.execution);
    let instance = strict.instantiate(EffectLimits::default(), 42).unwrap();
    drop(strict);
    assert!(weak.upgrade().is_some());
    let reused = source
        .clone()
        .with_execution(interpreted(NumericPolicy::Strict));
    assert!(Arc::ptr_eq(&weak.upgrade().unwrap(), &reused.execution));
    drop(reused);
    drop(instance);
    assert!(weak.upgrade().is_none(), "cache must own only a Weak plan");
    let rebuilt = source
        .clone()
        .with_execution(interpreted(NumericPolicy::Strict));
    assert!(rebuilt.same_program(&source));
    assert_eq!(source.plans.entries.lock().unwrap().len(), 2);
}

#[test]
fn concurrent_same_policy_requests_share_one_prepared_plan() {
    let source = fixture(16);
    let barrier = Arc::new(Barrier::new(4));
    let jobs: Vec<_> = (0..4)
        .map(|_| {
            let source = source.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                source.with_execution(interpreted(NumericPolicy::Strict))
            })
        })
        .collect();
    let wrappers: Vec<_> = jobs.into_iter().map(|job| job.join().unwrap()).collect();
    assert!(wrappers
        .iter()
        .all(|wrapper| Arc::ptr_eq(&wrappers[0].execution, &wrapper.execution,)));
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
#[test]
fn shared_native_entry_survives_all_native_wrapper_drops() {
    use crate::model::scene::popcorn::SceneInputs;
    let source = fixture(16);
    let options = ExecutionOptions {
        preference: ExecutionPreference::ScalarJit,
        numeric_policy: NumericPolicy::Strict,
    };
    let native = source.clone().with_execution(options);
    let repeated = source.clone().with_execution(options);
    assert!(Arc::ptr_eq(&native.execution, &repeated.execution));
    let mut instance = native.instantiate(EffectLimits::default(), 42).unwrap();
    drop(native);
    drop(repeated);
    instance
        .start(&source, source.default_entry(), SceneInputs::default())
        .unwrap();
    instance
        .advance(&source, 0.125, SceneInputs::default())
        .unwrap();
    assert!(instance.execution_stats().scalar_jit_invocations > 0);
}
