use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::scene::popcorn::{
    EffectLayerId, ExecutionBackend, ExecutionOptions, ExecutionPreference, NumericPolicy,
    SpecializationReason, SpecializationRejection, SpecializationStatus,
};

fn options(numeric_policy: NumericPolicy) -> ExecutionOptions {
    ExecutionOptions {
        preference: ExecutionPreference::ImmutableAge,
        numeric_policy,
    }
}

#[test]
fn original_native_reports_are_owned_policy_specific_and_distinct_from_routes() {
    let native = Document::read(include_bytes!("fixtures/pkb/immutable-128.pkb")).unwrap();
    let program = native
        .compile_effect(1, &EffectCompileOptions::default())
        .unwrap();
    let child = EffectLayerId(1);
    let interpreted = program.specialization_report(child).unwrap();
    assert_eq!(interpreted.pure_packet, SpecializationStatus::Accepted);
    assert_eq!(
        interpreted.immutable_expressions,
        SpecializationStatus::NotExamined
    );
    let graphics = program
        .clone()
        .with_execution(options(NumericPolicy::Graphics));
    let accepted = graphics.specialization_report(child).unwrap();
    assert_eq!(
        accepted.immutable_expressions,
        SpecializationStatus::Accepted
    );
    assert_eq!(graphics.immutable_recipe(child).unwrap().phase_count(), 0);
    assert!(graphics
        .execution_reports()
        .iter()
        .any(|report| report.layer == child && report.backend == ExecutionBackend::ImmutableAge));
    let strict = program.with_execution(options(NumericPolicy::Strict));
    assert_eq!(
        strict
            .specialization_report(child)
            .unwrap()
            .immutable_expressions,
        SpecializationStatus::Rejected(SpecializationRejection {
            reason: SpecializationReason::NumericPolicy,
            offset: None,
            field: None
        })
    );
    assert_eq!(
        strict.specialization_report(child).unwrap().pure_packet,
        accepted.pure_packet
    );
    assert_eq!(
        interpreted.immutable_expressions,
        SpecializationStatus::NotExamined,
        "previously returned snapshots cannot change with another wrapper"
    );
    assert!(graphics
        .specialization_report(EffectLayerId(u32::MAX))
        .is_none());
    assert_eq!(
        graphics.specialization_reports().len(),
        graphics.layers().len()
    );
}
