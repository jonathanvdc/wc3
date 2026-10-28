use wc3::model::conversion::{DropUnsupportedFeatures, Error, IssueKind, Policy, Report};
use wc3::model::profiles::{Mdlx, Universal};
use wc3::model::{FeatureId, Model, ModelBuilder, NodeFeature, ParticleEmitter, ResourceLocator};

fn source() -> (Model<'static>, FeatureId) {
    let mut draft = Model::<Mdlx>::builder("policy test");
    let node = draft.add_named_node("effect").unwrap();
    let mut model = draft.finish().unwrap().widen::<Universal>();
    let feature = model
        .edit(|draft| {
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
    assert!(Policy::<Mdlx>::new().convert(&source).is_err());
    assert!(source.feature(feature).is_some());
}

#[test]
fn selected_drop_prepares_a_valid_target_and_reports_its_consequence() {
    let (source, feature) = source();
    let conversion = Policy::<Mdlx>::new()
        .with_rule(DropUnsupportedFeatures::new([feature]))
        .convert(&source)
        .unwrap();
    assert!(source.feature(feature).is_some());
    assert!(conversion.value.feature(feature).is_none());
    assert_eq!(conversion.report.issues.len(), 1);
    assert_eq!(conversion.report.issues[0].kind, IssueKind::Dropped);
    assert!(conversion.report.issues[0].description.contains("behavior"));
    conversion.value.validate().unwrap();
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
        .convert(&source)
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
    assert!(valid.convert(&source).is_ok());
    let invalid = Policy::<Mdlx>::new().with_rule(
        move |draft: &mut ModelBuilder<'static>, _: &mut Report| -> Result<(), Error> {
            draft.remove_node(node);
            Ok(())
        },
    );
    assert!(invalid.convert(&source).is_err());
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
    assert!(Policy::<Mdlx>::new().convert(&source).is_err());
    let conversion = Policy::<Mdlx>::new()
        .with_rule(ConstantLightVisibility::new([feature]))
        .convert(&source)
        .unwrap();
    let mdlx::Node::Light(light) = &conversion.value.feature(feature).unwrap().payload else {
        panic!("expected light");
    };
    assert_eq!(
        *conversion.value.values().get(light.activation).unwrap(),
        1.
    );
    assert_eq!(conversion.report.issues[0].kind, IssueKind::Normalized);
    let (animated, feature) = switched_light(true);
    let error = Policy::<Mdlx>::new()
        .with_rule(ConstantLightVisibility::new([feature]))
        .convert(&animated)
        .unwrap_err();
    assert!(
        matches!(error, wc3::model::conversion::Error::Requirement { requirement, .. } if requirement.contains("animated"))
    );
}

#[test]
fn selected_supported_features_are_preserved_without_loss_reports() {
    use wc3::model::conversion::ConstantLightVisibility;
    let (source, feature) = switched_light(false);
    let supported = Policy::<Mdlx>::new()
        .with_rule(ConstantLightVisibility::new([feature]))
        .convert(&source)
        .unwrap()
        .value
        .widen::<Universal>();
    let conversion = Policy::<Mdlx>::new()
        .with_rule(DropUnsupportedFeatures::new([feature]))
        .convert(&supported)
        .unwrap();
    assert!(conversion.value.feature(feature).is_some());
    assert!(conversion.report.issues.is_empty());
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
    let conversion = Policy::<M2>::new()
        .with_rule(ConstantLightSwitch::new([feature], 0.5, 0.25).unwrap())
        .convert(&source)
        .unwrap();
    let m2::ActorKind::Light(light) = &conversion.value.feature(feature).unwrap().payload else {
        panic!("expected light");
    };
    assert!(*conversion.value.values().get(light.activation).unwrap());
    assert_eq!(conversion.report.issues[0].kind, IssueKind::Approximated);
    assert!(conversion.report.issues[0]
        .description
        .contains("error 0.25"));
    assert!(Policy::<M2>::new()
        .with_rule(ConstantLightSwitch::new([feature], 0.5, 0.2).unwrap())
        .convert(&source)
        .is_err());
    // The same rule can choose the other switch state with an explicit bound.
    let conversion = Policy::<M2>::new()
        .with_rule(ConstantLightSwitch::new([feature], 0.8, 0.75).unwrap())
        .convert(&source)
        .unwrap();
    let m2::ActorKind::Light(light) = &conversion.value.feature(feature).unwrap().payload else {
        panic!("expected light");
    };
    assert!(!*conversion.value.values().get(light.activation).unwrap());
    assert!(Policy::<M2>::new().convert(&source).is_err());
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
        .convert(&source)
        .is_err());
    let (source, feature) = fractional_light(0.75, true);
    let error = Policy::<M2>::new()
        .with_rule(ConstantLightSwitch::new([feature], 0.5, 1.).unwrap())
        .convert(&source)
        .unwrap_err();
    assert!(
        matches!(error, wc3::model::conversion::Error::Requirement { requirement, .. } if requirement.contains("animated"))
    );
    let (source, feature) = fractional_light(1., false);
    let conversion = Policy::<M2>::new()
        .with_rule(ConstantLightSwitch::new([feature], 0.5, 0.).unwrap())
        .convert(&source)
        .unwrap();
    assert_eq!(conversion.report.issues[0].kind, IssueKind::Normalized);
}

#[test]
fn selected_drop_checks_graph_dependent_feature_constraints() {
    use wc3::model::error::{ErrorKind, Location, ProfileError, ProfileKind, ProfileViolation};
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
    let Error::Semantic(error) = Policy::<Mdlx>::new().convert(&source).unwrap_err() else {
        panic!("exact projection must preserve its semantic failure")
    };
    assert!(matches!(
        error.kind(),
        ErrorKind::Profile(ProfileError {
            target: ProfileKind::Mdlx,
            violation: ProfileViolation::AnimatedField,
        })
    ));
    assert_eq!(
        error.location(),
        &Location::for_owner(feature)
            .field("projection")
            .field("field_of_view")
    );
    let conversion = Policy::<Mdlx>::new()
        .with_rule(DropUnsupportedFeatures::new([feature]))
        .convert(&source)
        .unwrap();
    assert!(conversion.value.feature(feature).is_none());
    let light = conversion.value.features().values().next().unwrap();
    let mdlx::Node::Light(light) = &light.payload else {
        panic!("expected retained light");
    };
    assert!(conversion.value.is_animated(light.diffuse.intensity));
    assert_eq!(conversion.report.issues.len(), 1);
    assert_eq!(conversion.report.issues[0].kind, IssueKind::Dropped);
    assert_eq!(
        conversion.report.issues[0].path,
        format!("features[{feature:?}]")
    );
    assert!(source.feature(feature).is_some());
}
