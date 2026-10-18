use wc3::model::profiles::{Mdlx, Universal};
use wc3::model::{
    AnimationProgram, AnimationSample, Channel, Clip, Curve, Model, PoseInputs, PropertyValue,
    PropertyValues, Quaternion, Time, TimeSource, Track,
};

fn time(seconds: f64) -> Time {
    Time::seconds(seconds).unwrap()
}

#[test]
fn types_select_independent_subtables_and_copies_share_slots() {
    let mut values = PropertyValues::default();
    let scalar = values.add(2f32);
    let rotation = values.add(Quaternion::IDENTITY);
    let shared = scalar;
    values.set(shared, 7.).unwrap();
    assert_eq!(values[scalar], 7.);
    assert_eq!(values[rotation], Quaternion::IDENTITY);
    assert_eq!(f32::table(&values).len(), 1);
    assert_eq!(Quaternion::table(&values).len(), 1);
    let foreign = PropertyValues::default().add(3f32);
    assert!(values.get(foreign).is_none());
    assert!(values.set(foreign, 4.).is_err());
}

#[test]
fn two_nodes_can_consume_the_same_animated_translation() {
    let mut draft = Model::<Universal>::builder("shared translation");
    let first = draft.add_named_node("first").unwrap();
    let second = draft.add_named_node("second").unwrap();
    let translation = draft.model().node(first).unwrap().transform.translation;
    let mut node = draft.model().node(second).unwrap().clone();
    node.transform.translation = translation;
    draft.set_node(second, node).unwrap();
    draft
        .add_global_channel(Channel::Vector3(Track {
            target: translation,
            curve: Curve::linear([time(0.), time(1.)], [[0.; 3], [4.; 3]]).unwrap(),
            clock: TimeSource::Animation,
        }))
        .unwrap();
    let model = draft.finish().unwrap();
    let layout = model.prepare_pose().unwrap();
    let sample = model
        .animation_sample(AnimationSample {
            clip_time: time(0.5),
            ..Default::default()
        })
        .unwrap();
    let pose = layout.evaluate(&sample, PoseInputs::default()).unwrap();
    assert_eq!(pose.nodes[layout.index(first).unwrap()].matrix.w_axis.x, 2.);
    assert_eq!(
        pose.nodes[layout.index(second).unwrap()].matrix.w_axis.x,
        2.
    );
}

#[test]
fn instance_tables_are_independent_and_each_evaluation_resets_constants() {
    let mut draft = Model::<Universal>::builder("instances");
    let value = draft.add_value(3f32);
    let clip = draft
        .add_clip(Clip::named("animated").with_duration(time(2.)))
        .unwrap();
    draft
        .add_channel(
            clip,
            Channel::Scalar(Track {
                target: value,
                curve: Curve::linear([time(0.), time(2.)], [3., 7.]).unwrap(),
                clock: TimeSource::Animation,
            }),
        )
        .unwrap();
    let model = draft.finish().unwrap();
    let program = AnimationProgram::compile(&model);
    let early = program
        .sample(AnimationSample {
            clip: Some(clip),
            clip_time: time(0.5),
            ..Default::default()
        })
        .unwrap()
        .evaluate();
    let mut late = program
        .sample(AnimationSample {
            clip: Some(clip),
            clip_time: time(1.5),
            ..Default::default()
        })
        .unwrap()
        .evaluate();
    assert_eq!(early[value], 4.);
    assert_eq!(late[value], 6.);
    program
        .sample(AnimationSample::default())
        .unwrap()
        .evaluate_into(&mut late);
    assert_eq!(late[value], 3.);
    assert_eq!(early[value], 4.);
    assert_eq!(model.values()[value], 3.);
}

#[test]
fn foreign_and_removed_consumer_slots_are_rejected() {
    let mut draft = Model::<Universal>::builder("invalid reference");
    let node = draft.add_named_node("node").unwrap();
    let mut definition = draft.model().node(node).unwrap().clone();
    definition.transform.translation = PropertyValues::default().add([0f32; 3]);
    draft.set_node(node, definition).unwrap();
    assert!(draft.finish().is_err());
    let mut draft = Model::<Universal>::builder("removed reference");
    let node = draft.add_named_node("node").unwrap();
    let visibility = draft.model().node(node).unwrap().visible;
    draft.values_mut().remove(visibility);
    assert!(draft.finish().is_err());
}

#[test]
fn explicit_boolean_reads_evaluate_the_source_slot() {
    let mut draft = Model::<Universal>::builder("conversion");
    let boolean = draft.add_value(false);
    let scalar = draft.values_mut().scalar_from_boolean(boolean).unwrap();
    draft
        .add_global_channel(Channel::Boolean(Track {
            target: boolean,
            curve: Curve::step([time(0.), time(1.)], [false, true]).unwrap(),
            clock: TimeSource::Animation,
        }))
        .unwrap();
    let model = draft.finish().unwrap();
    let values = model
        .animation_sample(AnimationSample {
            clip_time: time(1.),
            ..Default::default()
        })
        .unwrap()
        .evaluate();
    assert_eq!(values[scalar], 1.);
    assert!(values[boolean]);
}

#[test]
fn profile_mapping_preserves_slot_identities_and_constants() {
    let mut draft = Model::<Mdlx>::builder("mapping");
    let node = draft.add_named_node("bone").unwrap();
    let translation = draft.model().node(node).unwrap().transform.translation;
    draft.values_mut().set(translation, [2.; 3]).unwrap();
    let universal = draft.finish().unwrap().widen::<Universal>();
    assert_eq!(
        universal.node(node).unwrap().transform.translation,
        translation
    );
    assert_eq!(universal.values()[translation], [2.; 3]);
    let native = universal.try_narrow::<Mdlx>().unwrap();
    assert_eq!(
        native.node(node).unwrap().transform.translation,
        translation
    );
    assert_eq!(native.values()[translation], [2.; 3]);
}

#[test]
fn importing_conflicting_slots_is_atomic() {
    let mut draft = Model::<Universal>::builder("import");
    let shared = draft.add_value(1f32);
    let mut source = draft.model().values().clone();
    source.set(shared, 2.).unwrap();
    let extra = source.add([3f32; 3]);
    assert!(draft.import_values(&source).is_err());
    assert_eq!(draft.model().values()[shared], 1.);
    assert!(draft.model().values().get(extra).is_none());
}

#[test]
fn boolean_defaults_and_direct_scalar_writers_have_explicit_precedence() {
    let mut draft = Model::<Universal>::builder("converted output");
    let source = draft.add_value(false);
    let output = draft.values_mut().scalar_from_boolean(source).unwrap();
    draft.values_mut().set(source, true).unwrap();
    assert_eq!(draft.model().values()[output], 1.);
    draft
        .add_global_channel(Channel::Scalar(Track {
            target: output,
            clock: TimeSource::Animation,
            curve: Curve::step([time(0.)], [0.25]).unwrap(),
        }))
        .unwrap();
    let model = draft.finish().unwrap();
    let snapshot = model
        .animation_sample(AnimationSample::default())
        .unwrap()
        .evaluate();
    assert_eq!(snapshot[output], 0.25);
    assert!(snapshot[source]);
}

#[test]
fn prepared_nodes_read_static_slots_from_the_supplied_instance_table() {
    let mut draft = Model::<Universal>::builder("static instance");
    let id = draft.add_named_node("node").unwrap();
    let node = draft.model().node(id).unwrap().clone();
    let model = draft.finish().unwrap();
    let layout = model.prepare_pose().unwrap();
    let mut instance = model.values().clone();
    instance
        .set(node.transform.translation, [4., 5., 6.])
        .unwrap();
    instance.set(node.visible, false).unwrap();
    let prepared = &layout.nodes()[layout.index(id).unwrap()];
    assert_eq!(prepared.sample(&instance).translation, [4., 5., 6.]);
    assert!(!prepared.sample_visibility(&instance));
    assert_eq!(model.values()[node.transform.translation], [0.; 3]);
}

#[test]
fn construction_allocates_nested_fields_and_array_elements_independently() {
    use wc3::model::{Construct, Property};
    #[derive(Construct)]
    struct Pair(Property<f32>, Property<f32>);
    #[derive(Construct)]
    struct Nested {
        pair: Pair,
        flags: [Property<bool>; 2],
    }
    let mut values = PropertyValues::default();
    let nested = Nested::construct(&mut values);
    assert_ne!(nested.pair.0, nested.pair.1);
    assert_ne!(nested.flags[0], nested.flags[1]);
    assert_eq!(values.len(), 4);
    assert_eq!(values[nested.pair.0], 0.);
}

#[test]
fn reusable_programs_keep_clip_mapping_and_defaults_after_compilation() {
    use wc3::model::Animation;
    let mut draft = Model::<Universal>::builder("reusable animation");
    let property = draft.add_value(7f32);
    let constant = draft.add_value(false);
    let animation = draft.add_animation(Animation {
        channels: vec![Channel::Scalar(Track {
            target: property,
            clock: TimeSource::Animation,
            curve: Curve::linear(
                [time(9.), time(10.), time(12.), time(13.)],
                [900., 2., 6., 1300.],
            )
            .unwrap(),
        })],
        ..Default::default()
    });
    let first = draft
        .add_clip(
            Clip::named("once")
                .with_animation(animation)
                .with_source_start(time(10.))
                .with_duration(time(2.))
                .once(),
        )
        .unwrap();
    let second = draft
        .add_clip(
            Clip::named("repeat")
                .with_animation(animation)
                .with_source_start(time(10.))
                .with_duration(time(2.)),
        )
        .unwrap();
    let model = draft.finish().unwrap();
    assert_eq!(model.animations().len(), 1);
    let compiled = AnimationProgram::compile(&model);
    for (clip, expected) in [(first, 6.), (second, 4.)] {
        let input = AnimationSample {
            clip: Some(clip),
            clip_time: time(3.),
            ..Default::default()
        };
        let direct = model.animation_sample(input).unwrap();
        let prepared = compiled.sample(input).unwrap();
        assert_eq!(direct.animated(&property), expected);
        assert_eq!(prepared.animated(&property), expected);
        assert!(!prepared.animated(&constant));
        assert_eq!(
            prepared.track(&property).unwrap().interval,
            Some((time(10.), time(12.)))
        );
    }
    drop(model);
    assert_eq!(
        compiled
            .sample(AnimationSample {
                clip: Some(second),
                clip_time: time(1.),
                ..Default::default()
            })
            .unwrap()
            .animated(&property),
        4.
    );
}

#[test]
fn animation_selection_rejects_foreign_removed_and_duplicate_identities() {
    use wc3::model::Animation;
    let mut foreign = Model::<Universal>::builder("foreign");
    let foreign_animation = foreign.add_animation(Animation::default());
    let mut draft = Model::<Universal>::builder("local");
    draft
        .add_clip(Clip::named("invalid").with_animation(foreign_animation))
        .unwrap();
    assert!(draft.finish().is_err());

    let mut draft = Model::<Universal>::builder("duplicate");
    let animation = draft.add_animation(Animation::default());
    draft
        .add_clip(
            Clip::named("duplicate")
                .with_animation(animation)
                .with_animation(animation),
        )
        .unwrap();
    assert!(draft.finish().is_err());

    let mut draft = Model::<Universal>::builder("removed");
    let animation = draft.add_animation(Animation::default());
    draft.add_global_animation(animation).unwrap();
    let mut model = draft.finish().unwrap();
    assert!(model
        .edit(|draft| {
            draft.remove_animation(animation);
            Ok(())
        })
        .is_err());
    assert!(model.animation(animation).is_some());
}

#[test]
fn global_precedence_follows_selection_order_across_time_sources() {
    use wc3::model::{Animation, SharedClock};
    let mut draft = Model::<Universal>::builder("ordered globals");
    let property = draft.add_value(0f32);
    let clock = draft
        .add_clock(SharedClock::Repeating { period: time(2.) })
        .unwrap();
    let mut animation = |clock, end| {
        draft.add_animation(Animation {
            channels: vec![Channel::Scalar(Track {
                target: property,
                clock,
                curve: Curve::linear([time(0.), time(1.)], [0., end]).unwrap(),
            })],
            ..Default::default()
        })
    };
    let shared = animation(TimeSource::Shared(clock), 20.);
    let timeline = animation(TimeSource::Animation, 4.);
    draft.set_global_animations(vec![shared, timeline]);
    let mut model = draft.finish().unwrap();
    let input = AnimationSample {
        clip_time: time(0.5),
        shared_time: time(0.5),
        ..Default::default()
    };
    for (order, expected) in [(vec![shared, timeline], 2.), (vec![timeline, shared], 10.)] {
        model
            .edit(|draft| {
                draft.set_global_animations(order);
                Ok(())
            })
            .unwrap();
        assert_eq!(
            model
                .animation_sample(input)
                .unwrap()
                .override_value(&property),
            Some(expected)
        );
        assert_eq!(
            AnimationProgram::compile(&model)
                .sample(input)
                .unwrap()
                .override_value(&property),
            Some(expected)
        );
    }
}

#[test]
fn zero_and_offset_source_intervals_use_the_same_sampling_rule() {
    use wc3::model::{Animation, SharedClock};
    let mut draft = Model::<Universal>::builder("source intervals");
    let property = draft.add_value(-1f32);
    let independent = draft.add_value(0f32);
    let clock = draft
        .add_clock(SharedClock::Repeating { period: time(2.) })
        .unwrap();
    draft
        .add_global_channel(Channel::Scalar(Track {
            target: independent,
            clock: TimeSource::Shared(clock),
            curve: Curve::linear([time(0.), time(2.)], [0., 8.]).unwrap(),
        }))
        .unwrap();
    let animation = draft.add_animation(Animation {
        channels: vec![Channel::Scalar(Track {
            target: property,
            clock: TimeSource::Animation,
            curve: Curve::linear(
                [
                    time(0.),
                    time(1.),
                    time(2.),
                    time(9.),
                    time(10.),
                    time(11.),
                    time(12.),
                ],
                [0., 2., 200., 900., 10., 14., 1200.],
            )
            .unwrap(),
        })],
        ..Default::default()
    });
    let zero = draft
        .add_clip(
            Clip::named("zero")
                .with_duration(time(1.))
                .with_animation(animation),
        )
        .unwrap();
    let offset = draft
        .add_clip(
            Clip::named("offset")
                .with_source_start(time(10.))
                .with_duration(time(1.))
                .with_animation(animation)
                .once(),
        )
        .unwrap();
    let absent = draft
        .add_clip(Clip::named("unselected").with_duration(time(1.)))
        .unwrap();
    let model = draft.finish().unwrap();
    let compiled = AnimationProgram::compile(&model);
    for (clip, elapsed, expected, source) in [
        (zero, 1.5, 1., 0.5),
        (offset, 0.5, 12., 10.5),
        (offset, 3., 14., 11.),
    ] {
        let input = AnimationSample {
            clip: Some(clip),
            clip_time: time(elapsed),
            shared_time: time(2.5),
            ..Default::default()
        };
        for sample in [
            model.animation_sample(input).unwrap(),
            compiled.sample(input).unwrap(),
        ] {
            assert_eq!(sample.animated(&property), expected);
            assert_eq!(sample.animation_time(), time(source));
            assert_eq!(sample.animated(&independent), 2.);
            let retimed = sample.at_times(time(0.25), time(1.));
            assert_eq!(
                retimed.animated(&property),
                if clip == zero { 0.5 } else { 11. }
            );
        }
    }
    for clip in [None, Some(absent)] {
        let sample = compiled
            .sample(AnimationSample {
                clip,
                clip_time: time(0.5),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(sample.animated(&property), -1.);
    }
}

#[test]
fn composition_rejects_mismatched_source_intervals() {
    let mut host = Model::<Universal>::builder("host");
    let destination = host
        .add_clip(
            Clip::named("host clip")
                .with_duration(time(1.))
                .with_source_start(time(10.)),
        )
        .unwrap();
    let mut part = Model::<Universal>::builder("part");
    let source = part
        .add_clip(Clip::named("part clip").with_duration(time(1.)))
        .unwrap();
    let host = AnimationProgram::compile(&host.finish().unwrap());
    let part = AnimationProgram::compile(&part.finish().unwrap());
    assert!(host
        .compose(&part, &[(destination, source)], |_| true)
        .is_err());
}
