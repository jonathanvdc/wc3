use wc3::model::profiles::{Mdlx, Universal};
use wc3::model::{
    Animatable, AnimationProgram, AnimationSample, Channel, Clip, Clock, Curve, Model, PoseInputs,
    Quaternion, Time, Track, Values,
};

fn time(seconds: f64) -> Time {
    Time::seconds(seconds).unwrap()
}

#[test]
fn types_select_independent_subtables_and_copies_share_slots() {
    let mut values = Values::default();
    let scalar = values.add(2f32);
    let rotation = values.add(Quaternion::IDENTITY);
    let shared = scalar;
    values.set(shared, 7.).unwrap();
    assert_eq!(values[scalar], 7.);
    assert_eq!(values[rotation], Quaternion::IDENTITY);
    assert_eq!(f32::table(&values).len(), 1);
    assert_eq!(Quaternion::table(&values).len(), 1);
    let foreign = Values::default().add(3f32);
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
        .add_timeline_channel(Channel::Vector3(Track {
            target: translation,
            curve: Curve::linear([time(0.), time(1.)], [[0.; 3], [4.; 3]]).unwrap(),
            clock: Clock::Timeline,
        }))
        .unwrap();
    let model = draft.finish().unwrap();
    let layout = model.prepare_pose().unwrap();
    let sample = model
        .animation_sample(AnimationSample {
            timeline_time: time(0.5),
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
                clock: Clock::Clip,
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
    definition.transform.translation = Values::default().add([0f32; 3]);
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
        .add_timeline_channel(Channel::Boolean(Track {
            target: boolean,
            curve: Curve::step([time(0.), time(1.)], [false, true]).unwrap(),
            clock: Clock::Timeline,
        }))
        .unwrap();
    let model = draft.finish().unwrap();
    let values = model
        .animation_sample(AnimationSample {
            timeline_time: time(1.),
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
        .add_timeline_channel(Channel::Scalar(Track {
            target: output,
            clock: Clock::Timeline,
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
    use wc3::model::{Animated, Construct};
    #[derive(Construct)]
    struct Pair(Animated<f32>, Animated<f32>);
    #[derive(Construct)]
    struct Nested {
        pair: Pair,
        flags: [Animated<bool>; 2],
    }
    let mut values = Values::default();
    let nested = Nested::construct(&mut values);
    assert_ne!(nested.pair.0, nested.pair.1);
    assert_ne!(nested.flags[0], nested.flags[1]);
    assert_eq!(values.len(), 4);
    assert_eq!(values[nested.pair.0], 0.);
}
