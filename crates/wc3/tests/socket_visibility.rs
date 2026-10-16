use wc3::model::profiles::{m2, Universal, M2};
use wc3::model::{
    Animated, AnimationProgram, AnimationSample, Attachment, Channel, Clip, Clock, Curve, Light,
    LightActivation, Model, Node, NodeFeature, ScalarBinding, Socket, SocketIdentity, Time, Track,
};

fn time(value: f64) -> Time {
    Time::seconds(value).unwrap()
}

#[test]
fn boolean_sources_remain_shared_across_socket_and_light_bindings() {
    let mut builder = Model::<M2>::builder("shared boolean sources");
    let node = builder.add_node(Node::named("carrier")).unwrap();
    let point = builder.reserve_feature(node);
    let light = builder.reserve_feature(node);
    let socket_enabled = m2::properties::attachment::ENABLED.bind(point);
    let light_enabled = m2::properties::light::ENABLED.bind(light);
    let expected_source = light_enabled.clone().widen::<Universal>();
    builder
        .set_feature(
            point,
            m2::ActorKind::Attachment(m2::Attachment {
                identity: 7,
                visibility: Animated::new(true, light_enabled.clone()),
                control: (),
            }),
        )
        .unwrap();
    builder
        .set_feature(
            light,
            m2::ActorKind::Light(m2::Light {
                activation: Animated::new(false, socket_enabled.clone()),
                ..Default::default()
            }),
        )
        .unwrap();
    let clip = builder
        .add_clip(Clip::named("toggle").with_duration(time(1.)))
        .unwrap();
    for (target, keys) in [
        (socket_enabled, [false, true]),
        (light_enabled, [true, false]),
    ] {
        builder
            .add_channel(
                clip,
                Channel::Boolean(Track {
                    target,
                    curve: Curve::step([time(0.), time(0.5)], keys).unwrap(),
                    clock: Clock::Clip,
                }),
            )
            .unwrap();
    }
    let universal = builder.finish().unwrap().widen::<Universal>();
    universal.validate().unwrap();
    let NodeFeature::Attachment(Attachment::Socket(socket)) =
        &universal.feature(point).unwrap().payload
    else {
        panic!()
    };
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
    assert_eq!(
        socket.visibility.binding.as_ref().unwrap(),
        &expected_source
    );
}

#[test]
fn boolean_node_source_samples_through_scalar_binding() {
    let mut builder = Model::<Universal>::builder("boolean node source");
    let node = builder.add_node(Node::named("carrier")).unwrap();
    let source = node.visibility::<Universal>();
    let visibility = Animated {
        default: 1.,
        binding: Some(ScalarBinding::FromBoolean(source.clone())),
    };
    builder
        .add_timeline_channel(Channel::Boolean(Track {
            target: source,
            curve: Curve::step([time(0.), time(1.)], [true, false]).unwrap(),
            clock: Clock::Timeline,
        }))
        .unwrap();
    let model = builder.finish().unwrap();
    let animation = AnimationProgram::compile(&model);
    let sample = animation
        .sample(AnimationSample {
            timeline_time: time(1.5),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(sample.animated(&visibility), 0.);
    assert_eq!(
        sample
            .at_times(time(0.), time(0.5), time(0.))
            .animated(&visibility),
        1.
    );
    assert_eq!(visibility.clone(), visibility);
    let empty = Model::builder("missing source").finish().unwrap();
    let fallback = AnimationProgram::compile(&empty);
    assert_eq!(
        fallback
            .sample(AnimationSample::default())
            .unwrap()
            .animated(&visibility),
        1.
    );
}

#[test]
fn native_scalar_export_rejects_converted_boolean_bindings_in_strict_mode() {
    use std::borrow::Cow;
    use wc3::model::{
        conversion::ExportOptions,
        formats::mdlx::V800,
        io::mdlx,
        profiles::{mdlx as profile, Mdlx},
        SocketDefinition,
    };
    let mut builder = Model::<Mdlx>::builder("converted native scalar");
    let node = builder.add_node(Node::named("carrier")).unwrap();
    let source = node.visibility::<Mdlx>();
    builder
        .add_feature(
            node,
            profile::Node::Attachment(profile::Attachment {
                resource: Cow::Borrowed("child.mdx"),
                socket: SocketDefinition {
                    identity: 0,
                    visibility: Animated {
                        default: 1.,
                        binding: Some(ScalarBinding::FromBoolean(source.clone().widen())),
                    },
                    control: (),
                },
                appearance: (),
            }),
        )
        .unwrap();
    let model = builder.finish().unwrap();
    let error = mdlx::plan::<V800>(&model, &ExportOptions::default())
        .err()
        .unwrap();
    assert!(
        error.to_string().contains("converted boolean binding"),
        "{error}"
    );
}

#[test]
fn exact_m2_narrowing_accepts_binary_steps_and_rejects_fractional_visibility() {
    use wc3::model::socket;
    let mut builder = Model::<M2>::builder("binary visibility");
    let node = builder.add_node(Node::named("carrier")).unwrap();
    let feature = builder
        .add_feature(
            node,
            m2::ActorKind::Attachment(m2::Attachment {
                identity: 7,
                visibility: Animated::constant(true),
                control: (),
            }),
        )
        .unwrap();
    let clip = builder
        .add_clip(Clip::named("toggle").with_duration(time(1.)))
        .unwrap();
    let mut model = builder.finish().unwrap().widen::<Universal>();
    let property = socket::VISIBILITY.bind(feature);
    model
        .edit(|builder| {
            builder.set_feature(
                feature,
                NodeFeature::Attachment(Attachment::Socket(Socket {
                    identity: SocketIdentity::Role(7),
                    visibility: Animated::new(1., property.clone()),
                    control: None,
                })),
            )?;
            builder.add_channel(
                clip,
                Channel::Scalar(Track {
                    target: property.clone(),
                    curve: Curve::step([time(0.), time(0.5)], [1., 0.])?,
                    clock: Clock::Clip,
                }),
            )
        })
        .unwrap();
    assert!(model.clone().try_narrow::<M2>().is_ok());
    let mut fade = Clip::named("fade").with_duration(time(1.));
    fade.controls = model.clip(clip).unwrap().controls.clone();
    model
        .edit(|builder| {
            builder.set_clip(clip, fade)?;
            builder.add_channel(
                clip,
                Channel::Scalar(Track {
                    target: property,
                    curve: Curve::linear([time(0.), time(0.5)], [1., 0.])?,
                    clock: Clock::Clip,
                }),
            )
        })
        .unwrap();
    assert!(model.try_narrow::<M2>().is_err());
}
