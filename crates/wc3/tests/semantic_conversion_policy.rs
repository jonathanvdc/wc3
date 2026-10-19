use wc3::model::conversion::{DropUnsupportedFeatures, Error, IssueKind, Policy, Report};
use wc3::model::profiles::{Mdlx, Universal};
use wc3::model::{FeatureId, Model, ModelBuilder, NodeFeature, ParticleEmitter, ResourceLocator};

fn source() -> (Model<'static>, FeatureId) {
    let model = Model::<Mdlx>::empty("policy test").unwrap();
    let mut model = model.widen::<Universal>();
    let feature = model
        .edit(|draft| {
            let node = draft.add_named_node("effect")?;
            let resource =
                draft.add_effect_resource(ResourceLocator::Path(b"effect.mdx".as_slice().into()));
            draft.add_feature(
                node,
                NodeFeature::Particle(ParticleEmitter::Resource(resource)),
            )
        })
        .unwrap();
    (model, feature)
}

#[test]
fn strict_policy_rejects_unsupported_behavior_without_editing_source() {
    let (source, feature) = source();
    assert!(Policy::<Mdlx>::new().plan(&source).is_err());
    assert!(source.feature(feature).is_some());
}

#[test]
fn selected_drop_prepares_a_valid_target_and_reports_its_consequence() {
    let (source, feature) = source();
    let plan = Policy::<Mdlx>::new()
        .with_rule(DropUnsupportedFeatures::new([feature]))
        .plan(&source)
        .unwrap();
    assert!(source.feature(feature).is_some());
    assert!(plan.draft().feature(feature).is_none());
    assert_eq!(plan.report().issues.len(), 1);
    assert_eq!(plan.report().issues[0].kind, IssueKind::Dropped);
    assert!(plan.report().issues[0].description.contains("behavior"));
    let converted = plan.execute().unwrap();
    converted.value.validate().unwrap();
    assert_eq!(converted.report.issues.len(), 1);
}

#[test]
fn dropping_one_selected_feature_does_not_authorize_other_losses() {
    let (mut source, first) = source();
    let second = source.feature(first).unwrap().clone();
    source
        .edit(|draft| {
            draft.add_feature(second.node, second.payload)?;
            Ok(())
        })
        .unwrap();
    assert!(Policy::<Mdlx>::new()
        .with_rule(DropUnsupportedFeatures::new([first]))
        .plan(&source)
        .is_err());
    assert_eq!(source.features().len(), 2);
}

#[test]
fn application_rules_observe_previous_rewrites_and_cannot_publish_invalid_graphs() {
    let (source, feature) = source();
    let node = source.feature(feature).unwrap().node;
    let valid = Policy::<Mdlx>::new()
        .with_rule(DropUnsupportedFeatures::new([feature]))
        .with_rule(move |draft: &mut ModelBuilder<'static>, _: &mut Report| {
            assert!(draft.model().feature(feature).is_none());
            Ok(())
        });
    assert!(valid.plan(&source).is_ok());
    let invalid = Policy::<Mdlx>::new().with_rule(
        move |draft: &mut ModelBuilder<'static>, _: &mut Report| -> Result<(), Error> {
            draft.remove_node(node);
            Ok(())
        },
    );
    assert!(invalid.plan(&source).is_err());
    assert!(source.node(node).is_some());
    assert!(source.feature(feature).is_some());
}

fn switched_light(animated: bool) -> (Model<'static>, FeatureId) {
    use wc3::model::profiles::mdlx;
    use wc3::model::{Curve, DynamicTrack, Light, LightActivation, Time, TimeSource, Track};
    let mut draft = Model::<Mdlx>::builder("light policy");
    let light: mdlx::Light = draft.construct();
    let node = draft.add_named_node("light").unwrap();
    let feature = draft.add_feature(node, mdlx::Node::Light(light)).unwrap();
    let mut model = draft.finish().unwrap().widen::<Universal>();
    model
        .edit(|draft| {
            let enabled = draft.add_value(true);
            let mut payload = draft.model().feature(feature).unwrap().payload.clone();
            let NodeFeature::Light(Light::AmbientDiffuse(light)) = &mut payload else {
                panic!("expected light");
            };
            light.activation = LightActivation::Enabled(enabled);
            draft.set_feature(feature, payload)?;
            if animated {
                draft.add_global_track(DynamicTrack::Boolean(Track {
                    target: enabled,
                    curve: Curve::step(
                        [Time::default(), Time::seconds(1.).unwrap()],
                        [true, false],
                    )?,
                    clock: TimeSource::Animation,
                }))?;
            }
            Ok(())
        })
        .unwrap();
    (model, feature)
}

#[test]
fn a_constant_light_switch_can_be_reexpressed_without_approximating_animation() {
    use wc3::model::conversion::ConstantLightVisibility;
    use wc3::model::profiles::mdlx;
    let (source, feature) = switched_light(false);
    assert!(Policy::<Mdlx>::new().plan(&source).is_err());
    let plan = Policy::<Mdlx>::new()
        .with_rule(ConstantLightVisibility::new([feature]))
        .plan(&source)
        .unwrap();
    let mdlx::Node::Light(light) = &plan.draft().feature(feature).unwrap().payload else {
        panic!("expected light");
    };
    assert_eq!(*plan.draft().values().get(light.activation).unwrap(), 1.);
    assert_eq!(plan.report().issues[0].kind, IssueKind::Normalized);
    let (animated, feature) = switched_light(true);
    let error = Policy::<Mdlx>::new()
        .with_rule(ConstantLightVisibility::new([feature]))
        .plan(&animated)
        .unwrap_err();
    assert!(error.description.contains("animated"));
}

#[test]
fn selected_supported_features_are_preserved_without_loss_reports() {
    use wc3::model::conversion::ConstantLightVisibility;
    let (source, feature) = switched_light(false);
    let supported = Policy::<Mdlx>::new()
        .with_rule(ConstantLightVisibility::new([feature]))
        .plan(&source)
        .unwrap()
        .execute()
        .unwrap()
        .value
        .widen::<Universal>();
    let plan = Policy::<Mdlx>::new()
        .with_rule(DropUnsupportedFeatures::new([feature]))
        .plan(&supported)
        .unwrap();
    assert!(plan.draft().feature(feature).is_some());
    assert!(plan.report().issues.is_empty());
}

fn fractional_light(visibility: f32, animated: bool) -> (Model<'static>, FeatureId) {
    use wc3::model::profiles::{m2, M2};
    use wc3::model::{Curve, DynamicTrack, Light, LightActivation, Time, TimeSource, Track};
    let mut draft = Model::<M2>::builder("fractional light");
    let light: m2::Light = draft.construct();
    let node = draft.add_named_node("light").unwrap();
    let feature = draft
        .add_feature(node, m2::ActorKind::Light(light))
        .unwrap();
    let mut model = draft.finish().unwrap().widen::<Universal>();
    model
        .edit(|draft| {
            let visibility = draft.add_value(visibility);
            let mut payload = draft.model().feature(feature).unwrap().payload.clone();
            let NodeFeature::Light(Light::AmbientDiffuse(light)) = &mut payload else {
                panic!("expected light");
            };
            light.activation = LightActivation::Visibility(visibility);
            draft.set_feature(feature, payload)?;
            if animated {
                draft.add_global_track(DynamicTrack::Scalar(Track {
                    target: visibility,
                    curve: Curve::step(
                        [Time::default(), Time::seconds(1.).unwrap()],
                        [0.75, 0.25],
                    )?,
                    clock: TimeSource::Animation,
                }))?;
            }
            Ok(())
        })
        .unwrap();
    (model, feature)
}

#[test]
fn light_switch_approximation_obeys_its_explicit_error_bound() {
    use wc3::model::conversion::ConstantLightSwitch;
    use wc3::model::profiles::{m2, M2};
    let (source, feature) = fractional_light(0.75, false);
    let plan = Policy::<M2>::new()
        .with_rule(ConstantLightSwitch::new([feature], 0.5, 0.25).unwrap())
        .plan(&source)
        .unwrap();
    let m2::ActorKind::Light(light) = &plan.draft().feature(feature).unwrap().payload else {
        panic!("expected light");
    };
    assert!(*plan.draft().values().get(light.activation).unwrap());
    assert_eq!(plan.report().issues[0].kind, IssueKind::Approximated);
    assert!(plan.report().issues[0].description.contains("error 0.25"));
    assert!(Policy::<M2>::new()
        .with_rule(ConstantLightSwitch::new([feature], 0.5, 0.2).unwrap())
        .plan(&source)
        .is_err());
    // The same rule can choose the other switch state with an explicit bound.
    let plan = Policy::<M2>::new()
        .with_rule(ConstantLightSwitch::new([feature], 0.8, 0.75).unwrap())
        .plan(&source)
        .unwrap();
    let m2::ActorKind::Light(light) = &plan.draft().feature(feature).unwrap().payload else {
        panic!("expected light");
    };
    assert!(!*plan.draft().values().get(light.activation).unwrap());
    assert!(Policy::<M2>::new().plan(&source).is_err());
}

#[test]
fn light_switch_rule_rejects_invalid_parameters_and_unbounded_inputs() {
    use wc3::model::conversion::ConstantLightSwitch;
    use wc3::model::profiles::M2;
    for threshold in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
        assert!(ConstantLightSwitch::new([], threshold, 0.5).is_err());
    }
    for tolerance in [f32::NAN, f32::INFINITY, -0.1] {
        assert!(ConstantLightSwitch::new([], 0.5, tolerance).is_err());
    }
    let (source, feature) = fractional_light(1.5, false);
    assert!(Policy::<M2>::new()
        .with_rule(ConstantLightSwitch::new([feature], 0.5, 1.).unwrap())
        .plan(&source)
        .is_err());
    let (source, feature) = fractional_light(0.75, true);
    let error = Policy::<M2>::new()
        .with_rule(ConstantLightSwitch::new([feature], 0.5, 1.).unwrap())
        .plan(&source)
        .unwrap_err();
    assert!(error.description.contains("animated"));
    let (source, feature) = fractional_light(1., false);
    let plan = Policy::<M2>::new()
        .with_rule(ConstantLightSwitch::new([feature], 0.5, 0.).unwrap())
        .plan(&source)
        .unwrap();
    assert_eq!(plan.report().issues[0].kind, IssueKind::Normalized);
}

#[test]
fn selected_drop_checks_graph_dependent_feature_constraints() {
    use wc3::model::profiles::mdlx;
    use wc3::model::{Camera, Curve, DynamicTrack, Light, Time, TimeSource, Track};
    let mut draft = Model::<Mdlx>::builder("animated camera");
    let camera: mdlx::Camera = draft.construct();
    let node = draft.add_named_node("camera").unwrap();
    let feature = draft.add_feature(node, mdlx::Node::Camera(camera)).unwrap();
    let mut source = draft.finish().unwrap().widen::<Universal>();
    source
        .edit(|draft| {
            let NodeFeature::Camera(Camera::LookAt(camera)) =
                &draft.model().feature(feature).unwrap().payload
            else {
                panic!("expected camera");
            };
            let target = camera.projection.field_of_view;
            // Keep the shared channel meaningful after removing the camera.
            let mut light: mdlx::Light = draft.construct();
            light.diffuse.intensity = target;
            draft.add_feature(
                node,
                NodeFeature::Light(Light::AmbientDiffuse(Box::new(light.into()))),
            )?;
            draft.add_global_track(DynamicTrack::Scalar(Track {
                target,
                curve: Curve::linear([Time::default(), Time::seconds(1.).unwrap()], [0.5, 1.])?,
                clock: TimeSource::Animation,
            }))?;
            Ok(())
        })
        .unwrap();
    assert!(Policy::<Mdlx>::new().plan(&source).is_err());
    let plan = Policy::<Mdlx>::new()
        .with_rule(DropUnsupportedFeatures::new([feature]))
        .plan(&source)
        .unwrap();
    assert!(plan.draft().feature(feature).is_none());
    let light = plan.draft().features().values().next().unwrap();
    let mdlx::Node::Light(light) = &light.payload else {
        panic!("expected retained light");
    };
    assert!(plan.draft().is_animated(light.diffuse.intensity));
    assert_eq!(plan.report().issues[0].kind, IssueKind::Dropped);
    assert!(plan.report().issues[0]
        .description
        .contains("constant field of view"));
    assert!(source.feature(feature).is_some());
}
