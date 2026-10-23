//! Public preparation policy, graph identity and fixed-instance route contracts.
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::scene::popcorn::{
    EffectAttributeId, EffectFieldId, EffectInstance, EffectLayerId, EffectLimits, EffectProgram,
    ExecutionBackend, ExecutionFallback, ExecutionOptions, ExecutionPreference, NumericPolicy,
    SceneInputs, Value,
};

const NATIVE_SUPPORTED: bool = cfg!(all(
    feature = "popcorn-jit",
    any(unix, windows),
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64",
        target_arch = "s390x"
    )
));

fn controls() -> EffectProgram {
    Document::read(include_bytes!("fixtures/pkb/controls.pkb"))
        .unwrap()
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap()
}
fn options(preference: ExecutionPreference) -> ExecutionOptions {
    ExecutionOptions {
        preference,
        numeric_policy: NumericPolicy::Strict,
    }
}
fn custom_attribute(program: &EffectProgram) -> EffectAttributeId {
    EffectAttributeId(
        program
            .attributes()
            .iter()
            .position(|attribute| attribute.name.as_ref() == b"Custom.SizeMultiplier")
            .expect("original fixture declares this exposed control") as u32,
    )
}
fn size(instance: &EffectInstance) -> f32 {
    f32::from_bits(
        instance
            .particles(EffectLayerId(0))
            .next()
            .unwrap()
            .field(EffectFieldId(3))
            .unwrap()
            .words()[0],
    )
}

#[test]
fn ordinary_compile_and_explicit_interpreter_do_not_prepare_native_routes() {
    let ordinary = controls();
    assert_eq!(
        ordinary.execution_options().preference,
        ExecutionPreference::Interpreter
    );
    let explicit = ordinary
        .clone()
        .with_execution(options(ExecutionPreference::Interpreter));
    for program in [&ordinary, &explicit] {
        assert!(!program.execution_reports().is_empty());
        assert!(program
            .execution_reports()
            .iter()
            .all(|report| report.backend == ExecutionBackend::Interpreter
                && report.fallback.is_none()));
        let mut instance = program.instantiate(EffectLimits::default(), 42).unwrap();
        instance.start(program, 0, SceneInputs::default()).unwrap();
        instance
            .advance(program, 0.25, SceneInputs::default())
            .unwrap();
        assert_eq!(instance.execution_stats().scalar_jit_invocations, 0);
        assert_eq!(size(&instance), 2.);
    }
}

#[test]
fn automatic_preparation_preserves_identity_and_existing_instance_routes() {
    let ordinary = controls();
    let custom = custom_attribute(&ordinary);
    let mut old_instance = ordinary.instantiate(EffectLimits::default(), 42).unwrap();
    let automatic = ordinary
        .clone()
        .with_execution(options(ExecutionPreference::Automatic));
    assert!(ordinary.same_program(&automatic));
    assert!(
        !automatic.same_program(&controls()),
        "independent equal graphs are distinct"
    );
    assert_eq!(custom_attribute(&automatic), custom);
    assert_eq!(ordinary.attributes().len(), automatic.attributes().len());
    for (before, after) in ordinary.attributes().iter().zip(automatic.attributes()) {
        assert_eq!(before.name, after.name);
        assert_eq!(before.default, after.default);
    }
    assert!(!automatic.execution_reports().is_empty());
    for report in automatic.execution_reports() {
        if NATIVE_SUPPORTED {
            assert_eq!(report.backend, ExecutionBackend::ScalarJit, "{report:?}");
            assert!(report.fallback.is_none(), "{report:?}");
        } else {
            assert_eq!(report.backend, ExecutionBackend::Interpreter);
            if cfg!(feature = "popcorn-jit") {
                assert_eq!(
                    report.fallback,
                    Some(ExecutionFallback::UnsupportedPlatform)
                );
            } else {
                assert_eq!(report.fallback, Some(ExecutionFallback::FeatureDisabled));
            }
        }
    }
    // The old instance accepts the same graph wrapper without adopting its route.
    old_instance.set_attribute(custom, Value::from(7.)).unwrap();
    old_instance
        .start(&automatic, 0, SceneInputs::default())
        .unwrap();
    old_instance
        .advance(&automatic, 0.25, SceneInputs::default())
        .unwrap();
    assert_eq!(
        old_instance.execution_reports(),
        ordinary.execution_reports()
    );
    assert_eq!(old_instance.execution_stats().scalar_jit_invocations, 0);
    assert_eq!(size(&old_instance), 7.);

    // Conversely the new instance keeps Automatic even when called with the old wrapper.
    let mut new_instance = automatic.instantiate(EffectLimits::default(), 42).unwrap();
    new_instance.set_attribute(custom, Value::from(5.)).unwrap();
    new_instance
        .start(&ordinary, 0, SceneInputs::default())
        .unwrap();
    let started = new_instance.execution_stats().scalar_jit_invocations;
    assert_eq!(started > 0, NATIVE_SUPPORTED);
    new_instance
        .advance(&ordinary, 0.25, SceneInputs::default())
        .unwrap();
    assert_eq!(
        new_instance.execution_reports(),
        automatic.execution_reports()
    );
    assert_eq!(size(&new_instance), 5.);
    let advanced = new_instance.execution_stats().scalar_jit_invocations;
    assert_eq!(advanced > started, NATIVE_SUPPORTED);
    assert_eq!(
        size(&old_instance),
        7.,
        "shared graph does not share instance controls"
    );

    // start resets simulation counters/time, while execution diagnostics are lifetime cumulative.
    new_instance.stop();
    new_instance
        .start(&ordinary, 0, SceneInputs::default())
        .unwrap();
    assert_eq!(new_instance.time(), 0.);
    assert_eq!(new_instance.stats().spawned, 1);
    assert_eq!(
        new_instance.execution_stats().scalar_jit_invocations,
        advanced + u64::from(NATIVE_SUPPORTED)
    );
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
fn native_fault_counts_once_and_explicit_restart_retains_diagnostic_history() {
    let program = controls().with_execution(options(ExecutionPreference::Automatic));
    assert!(program
        .execution_reports()
        .iter()
        .all(|report| report.backend == ExecutionBackend::ScalarJit && report.fallback.is_none()));
    let mut instance = program.instantiate(EffectLimits::default(), 0).unwrap();
    instance.start(&program, 0, SceneInputs::default()).unwrap();
    let before = instance.execution_stats().scalar_jit_invocations;
    instance
        .set_attribute(custom_attribute(&program), Value::from(f32::NAN))
        .unwrap();
    assert!(instance
        .advance(&program, 0.25, SceneInputs::default())
        .is_err());
    let faulted = instance.execution_stats().scalar_jit_invocations;
    assert_eq!(faulted, before + 1);
    assert!(instance
        .advance(&program, 0.25, SceneInputs::default())
        .is_err());
    assert_eq!(
        instance.execution_stats().scalar_jit_invocations,
        faulted,
        "a faulted instance rejects execution rather than entering native code again"
    );
    instance
        .set_attribute(custom_attribute(&program), Value::from(3.))
        .unwrap();
    instance.start(&program, 0, SceneInputs::default()).unwrap();
    assert_eq!(
        instance.execution_stats().scalar_jit_invocations,
        faulted + 1
    );
    assert_eq!(size(&instance), 3.);
    instance
        .advance(&program, 0.25, SceneInputs::default())
        .unwrap();
    assert_eq!(size(&instance), 3.);
}
