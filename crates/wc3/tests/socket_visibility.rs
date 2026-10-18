use wc3::model::profiles::{m2, Universal, M2};
use wc3::model::ModelMetadata;
use wc3::model::{
    AnimationProgram, AnimationSample, Attachment, Clip, Curve, DynamicTrack, Light,
    LightActivation, Model, NodeFeature, Time, TimeSource, Track,
};

fn time(value: f64) -> Time {
    Time::seconds(value).unwrap()
}

#[test]
fn boolean_sources_remain_shared_across_socket_and_light_slots() {
    let mut draft = Model::<M2>::builder("shared sources");
    let node = draft.add_named_node("carrier").unwrap();
    let socket_source = draft.add_value(false);
    let light_source = draft.add_value(true);
    let point = draft
        .add_feature(
            node,
            m2::ActorKind::Attachment(m2::Attachment {
                identity: 7,
                visibility: light_source,
                control: (),
            }),
        )
        .unwrap();
    let mut light: m2::Light = draft.construct();
    light.activation = socket_source;
    let light = draft
        .add_feature(node, m2::ActorKind::Light(light))
        .unwrap();
    let clip = draft
        .add_clip(Clip::named("toggle").with_duration(time(1.)))
        .unwrap();
    for (target, keys) in [
        (socket_source, [false, true]),
        (light_source, [true, false]),
    ] {
        draft
            .add_track(
                clip,
                DynamicTrack::Boolean(Track {
                    target,
                    curve: Curve::step([time(0.), time(0.5)], keys).unwrap(),
                    clock: TimeSource::Animation,
                }),
            )
            .unwrap();
    }
    let universal = draft.finish().unwrap().widen::<Universal>();
    let NodeFeature::Attachment(Attachment::Socket(socket)) =
        &universal.feature(point).unwrap().payload
    else {
        panic!()
    };
    assert_eq!(
        universal.values().boolean_source(socket.visibility),
        Some(light_source)
    );
    let NodeFeature::Light(Light::AmbientDiffuse(light)) =
        &universal.feature(light).unwrap().payload
    else {
        panic!()
    };
    let LightActivation::Enabled(enabled) = &light.activation else {
        panic!()
    };
    let animation = AnimationProgram::compile(&universal);
    for (t, visibility, enabled_value) in [(0.25, 1., false), (0.75, 0., true)] {
        let sample = animation
            .sample(AnimationSample {
                clip: Some(clip),
                clip_time: time(t),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(sample.animated(&socket.visibility), visibility);
        assert_eq!(sample.animated(enabled), enabled_value);
    }
    let native = universal.try_narrow::<M2>().unwrap();
    let m2::ActorKind::Attachment(socket) = &native.feature(point).unwrap().payload else {
        panic!()
    };
    assert_eq!(socket.visibility, light_source);
}

#[test]
fn explicit_boolean_conversion_samples_at_retimed_clocks() {
    let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let source = draft.add_value(true);
    let visibility = draft.values_mut().scalar_from_boolean(source).unwrap();
    draft
        .add_global_track(DynamicTrack::Boolean(Track {
            target: source,
            curve: Curve::step([time(0.), time(1.)], [true, false]).unwrap(),
            clock: TimeSource::Animation,
        }))
        .unwrap();
    let model = draft.finish().unwrap();
    let animation = AnimationProgram::compile(&model);
    let sample = animation
        .sample(AnimationSample {
            clip_time: time(1.5),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(sample.animated(&visibility), 0.);
    assert_eq!(
        sample.at_times(time(0.), time(0.)).animated(&visibility),
        1.
    );
    assert!(
        Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()))
            .finish()
            .unwrap()
            .values()
            .get(visibility)
            .is_none()
    );
}
