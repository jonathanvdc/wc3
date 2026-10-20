use bevy_wc3::{
    compile_model, RenderDiagnosticKind, RenderDiagnosticSeverity, RenderDiagnosticSource,
};
use wc3::model::{conversion::ImportOptions, io, profiles::Universal};

#[test]
fn applications_can_filter_effect_budgets_and_missing_resources_by_semantic_owner() {
    let source = io::mdlx::decode_mdl(
        include_str!("fixtures/ribbon_capture.mdl"),
        &ImportOptions::default(),
    )
    .unwrap()
    .value
    .widen::<Universal>();
    let compiled = compile_model(&source, |_| None).unwrap();
    assert!(
        compiled.diagnostics().iter().all(|diagnostic| {
            diagnostic.kind != RenderDiagnosticKind::Unsupported
                || !matches!(diagnostic.source, RenderDiagnosticSource::Feature(_))
        }),
        "implemented native ribbons must not also report a missing visual interpreter"
    );
    let budgets: Vec<_> = compiled
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.kind == RenderDiagnosticKind::Budget)
        .collect();
    assert!(!budgets.is_empty());
    for diagnostic in budgets {
        assert_eq!(diagnostic.severity, RenderDiagnosticSeverity::Info);
        let RenderDiagnosticSource::Feature(id) = diagnostic.source else {
            panic!("effect limits must identify their authored emitter");
        };
        assert!(source.feature(id).is_some());
    }
    let resources: Vec<_> = compiled
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.kind == RenderDiagnosticKind::Resource)
        .collect();
    assert!(!resources.is_empty());
    assert!(resources.iter().all(|diagnostic| {
        diagnostic.severity == RenderDiagnosticSeverity::Warning
            && match diagnostic.source {
                RenderDiagnosticSource::Feature(id) => source.feature(id).is_some(),
                RenderDiagnosticSource::Material(id) => source.material(id).is_some(),
                _ => false,
            }
    }));
}
