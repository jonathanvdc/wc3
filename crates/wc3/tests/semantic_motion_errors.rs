use std::error::Error as _;
use wc3::model::error::{AnimationError, ErrorKind, Owner, Segment, TransformError};
use wc3::model::profiles::Universal;
use wc3::model::{
    AnimationProgram, AnimationSample, Axis, Basis, Clip, Curve, Handedness, LengthUnit, Mat4,
    Model, ModelMetadata, PoseLayout, SharedClock, Time, TransformSemantics, Vec3,
};

fn time(seconds: f64) -> Time {
    Time::seconds(seconds).unwrap()
}

#[test]
fn nonfinite_time_retains_the_actual_input_and_domain_source() {
    for value in [f64::from_bits(0x7ff8_0000_0000_0042), f64::INFINITY] {
        let error = Time::seconds(value).unwrap_err();
        let ErrorKind::Animation(AnimationError::NonFiniteTime { value: actual }) = error.kind()
        else {
            panic!("unexpected failure: {error:?}")
        };
        assert_eq!(actual.to_bits(), value.to_bits());
        assert!(error.source().unwrap().is::<AnimationError>());
    }
}

#[test]
fn curve_lengths_order_and_control_positions_are_inspectable() {
    let error = Curve::step([time(0.), time(1.)], [false]).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Animation(AnimationError::KeyValueCount { keys: 2, values: 1 })
    ));

    let error = Curve::linear([time(2.), time(1.)], [0f32, 1.]).unwrap_err();
    let ErrorKind::Animation(AnimationError::UnorderedKeys { previous, current }) = error.kind()
    else {
        panic!("unexpected failure: {error:?}")
    };
    assert_eq!((*previous, *current), (time(2.), time(1.)));
    assert_eq!(error.location().segments().last(), Some(&Segment::Index(1)));

    let error = Curve::bezier(
        [time(0.), time(1.)],
        [0f32, 1.],
        [0f32, 0.],
        [0f32, f32::NAN],
    )
    .unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Animation(AnimationError::InvalidControlValue)
    ));
    assert_eq!(error.location().segments().last(), Some(&Segment::Index(1)));
    assert!(error
        .location()
        .segments()
        .iter()
        .any(|segment| matches!(segment, Segment::Field(field) if field == "outgoing")));
}

#[test]
fn unknown_clips_retain_the_requested_identity_in_both_samplers() {
    let model = Model::<Universal>::empty(ModelMetadata::Mdlx(Default::default())).unwrap();
    let mut foreign = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let clip = foreign.add_clip(Clip::named("foreign")).unwrap();
    let sample = AnimationSample {
        clip: Some(clip),
        ..Default::default()
    };
    let direct = model.animation_sample(sample).err().unwrap();
    let prepared = AnimationProgram::compile(&model)
        .sample(sample)
        .err()
        .unwrap();
    for error in [direct, prepared] {
        assert!(matches!(
            error.kind(),
            ErrorKind::Animation(AnimationError::UnknownClip { clip: actual }) if *actual == clip
        ));
    }
}

#[test]
fn composition_retains_mapped_clip_ids_and_authored_starts() {
    let mut host = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let destination = host
        .add_clip(Clip::named("host").with_source_start(time(10.)))
        .unwrap();
    let mut part = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let source = part.add_clip(Clip::named("part")).unwrap();
    let host = AnimationProgram::compile(&host.finish().unwrap());
    let part = AnimationProgram::compile(&part.finish().unwrap());
    let error = host
        .compose(&part, &[(destination, source)], |_| true)
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Animation(AnimationError::ClipStartMismatch {
            destination: actual_destination,
            source: actual_source,
            destination_start,
            source_start,
        }) if *actual_destination == destination && *actual_source == source
            && *destination_start == time(10.) && *source_start == time(0.)
    ));
}

#[test]
fn composition_retains_the_conflicting_clock_and_both_periods() {
    let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let clock = draft
        .add_clock(SharedClock::Repeating { period: time(1.) })
        .unwrap();
    let host = draft.finish().unwrap();
    let mut part = host.clone();
    part.edit(|draft| draft.set_clock(clock, SharedClock::Repeating { period: time(2.) }))
        .unwrap();
    let error = AnimationProgram::compile(&host)
        .compose(&AnimationProgram::compile(&part), &[], |_| true)
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Animation(AnimationError::ClockPeriodConflict {
            clock: actual, existing, incoming,
        }) if *actual == clock && *existing == time(1.) && *incoming == time(2.)
    ));
}

#[test]
fn coordinate_failures_retain_axis_choices_and_unrepresentable_scale() {
    let error = Basis::new(Axis::PositiveX, Axis::NegativeX, Handedness::Right).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Transform(TransformError::DependentBasisAxes {
            up: Axis::PositiveX,
            forward: Axis::NegativeX,
        })
    ));
    let model = Model::<Universal>::empty(ModelMetadata::Mdlx(Default::default())).unwrap();
    let error = model
        .coordinate_transform(
            Basis::default(),
            LengthUnit::MetersPerUnit(f64::MIN_POSITIVE),
        )
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Transform(TransformError::UnrepresentableScale {
            source: LengthUnit::Unspecified,
            target: LengthUnit::MetersPerUnit(value), scale,
        }) if *value == f64::MIN_POSITIVE && *scale == 1. / f64::MIN_POSITIVE
    ));
}

#[test]
fn prepared_pose_failures_keep_source_identity_and_parent_count() {
    let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let first = draft.add_named_node("first").unwrap();
    let second = draft.add_named_node("second").unwrap();
    let model = draft.finish().unwrap();
    let layout = model.prepare_pose().unwrap();
    let mut nodes = layout.nodes().to_vec();
    nodes[1].id = first;
    let error = PoseLayout::new(nodes).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Transform(TransformError::DuplicatePoseNode { node }) if *node == first
    ));
    let mut nodes = layout.nodes().to_vec();
    nodes[1].parent = Some(42);
    let error = PoseLayout::new(nodes).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Transform(TransformError::ParentIndexOutOfBounds {
            parent: 42,
            nodes: 2
        })
    ));
    assert_eq!(
        error.location().segments().first(),
        Some(&Segment::Owner(Owner::Node(second)))
    );
}

#[test]
fn correction_failures_retain_the_actual_matrix_and_node() {
    let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let node = draft.add_named_node("corrected").unwrap();
    let model = draft.finish().unwrap();
    let mut nodes = model.prepare_pose().unwrap().nodes().to_vec();
    nodes[0].transform_semantics = TransformSemantics::Affine;
    let mut matrix = Mat4::IDENTITY;
    matrix.x_axis.w = 0.5;
    nodes[0].correction = Some(matrix);
    let error = PoseLayout::new(nodes).unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Transform(TransformError::NonAffineMatrix { last_row }) if last_row.x == 0.5 && last_row.w == 1.
    ));
    assert_eq!(
        error.location().segments().first(),
        Some(&Segment::Owner(Owner::Node(node)))
    );

    let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let mut node = draft.new_node("invalid pivot");
    node.transform.pivot = Vec3::new(f32::INFINITY, 0., 0.);
    draft.add_node(node).unwrap();
    let error = draft.finish().unwrap_err();
    assert!(matches!(
        error.kind(),
        ErrorKind::Transform(TransformError::NonFiniteVector { vector }) if vector.x == f32::INFINITY
    ));
}
