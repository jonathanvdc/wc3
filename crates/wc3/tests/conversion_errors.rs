//! Checked conversion boundaries retain causes; accepted loss remains a report.
use std::cell::Cell;
use std::error::Error as _;
use std::str::Utf8Error;
use wc3::model::common::ZeroVec;
use wc3::model::conversion::{
    ApproximationPolicy, Context, Error, ImportOptions, IssueKind, LossPolicy,
};
use wc3::model::error::{Location, Segment};
use wc3::model::formats::m2::{self, companions::SkinProfile, Bundle, Document, Flat, V256};
use wc3::model::io;

#[test]
fn rejected_native_validation_keeps_the_concrete_codec_failure() {
    let mut document = Document::<Flat<V256>>::new();
    document.core_mut().skins.push(SkinProfile {
        indices: ZeroVec::alloc_from_slice(&[0, 1]),
        ..Default::default()
    });
    let source = Bundle::from_model(document);
    let expected = source.validate_complete().unwrap_err();
    let error = io::m2::import(&source, &ImportOptions::default()).unwrap_err();
    assert_eq!(
        error.source().unwrap().downcast_ref::<m2::ReadError>(),
        Some(&expected)
    );
    assert!(matches!(error.location().segments(), [Segment::Field(field)] if field == "m2"));
    let cloned = error.clone();
    assert!(std::ptr::eq(
        error.source().unwrap(),
        cloned.source().unwrap()
    ));
}

#[test]
fn invalid_native_name_keeps_utf8_evidence_and_structured_field_context() {
    let mut document = Document::<Flat<V256>>::new();
    document.core_mut().name = ZeroVec::alloc_from_slice(&[0xff, 0]);
    let source = Bundle::from_model(document);
    let error = io::m2::import(&source, &ImportOptions::default()).unwrap_err();
    let cause = error.source().unwrap().downcast_ref::<Utf8Error>().unwrap();
    assert_eq!(cause.valid_up_to(), 0);
    assert_eq!(cause.error_len(), Some(1));
    assert_eq!(error.location(), &Location::new().field("m2").field("name"));
}

#[test]
fn unsupported_operation_retains_rejected_cause_and_reports_only_accepted_loss() {
    let location = Location::new().field("layers").index(2).field("shader");
    let native = m2::ReadError::invalid(27, "unsupported shader record");
    let mut strict = Context::import(&ImportOptions::default());
    let error = strict
        .unsupported_source(location.clone(), native.clone())
        .unwrap_err();
    assert!(matches!(error, Error::Source { .. }));
    assert_eq!(error.location(), &location);
    assert_eq!(
        error.source().unwrap().downcast_ref::<m2::ReadError>(),
        Some(&native)
    );
    assert!(strict.report.issues.is_empty());

    let mut lossy = Context::import(&ImportOptions {
        loss_policy: LossPolicy::Drop,
        ..Default::default()
    });
    lossy.unsupported_source(location, native.clone()).unwrap();
    assert_eq!(lossy.report.issues.len(), 1);
    let issue = &lossy.report.issues[0];
    assert_eq!(issue.path, "layers[2].shader");
    assert_eq!(issue.kind, IssueKind::Dropped);
    assert_eq!(issue.description, native.to_string());
}

#[test]
fn exact_conversion_omits_context_and_approximation_builds_it_once() {
    let calls = Cell::new(0);
    let location = || {
        calls.set(calls.get() + 1);
        Location::new().field("rotation")
    };
    let mut strict = Context::import(&ImportOptions::default());
    strict.approximate(location, 0., "exact").unwrap();
    assert_eq!(calls.get(), 0);
    assert!(strict.approximate(location, 0.25, "quantized").is_err());
    assert_eq!(calls.get(), 1);
    let mut accepted = Context::import(&ImportOptions {
        approximation: ApproximationPolicy::Allow { tolerance: 0.5 },
        ..Default::default()
    });
    accepted.approximate(location, 0.25, "quantized").unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(accepted.report.issues[0].path, "rotation");
    assert_eq!(accepted.report.issues[0].kind, IssueKind::Approximated);
}
