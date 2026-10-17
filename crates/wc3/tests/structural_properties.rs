use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use wc3::model::profiles::m2::properties::particle as m2_particle;
use wc3::model::profiles::mdlx::properties::{light, particle2, Layer as LayerProperties};
use wc3::model::profiles::mdlx::{
    Attachment, Layer as MdlxLayer, Light, Material as MdlxMaterial, Node as MdlxFeature,
};
use wc3::model::profiles::{FeaturePayload, Mdlx, Profile, Universal, M2};
use wc3::model::{
    Animated, Channel, Clock, Curve, FeatureId, FeatureKind, FeatureSchema, FeatureView,
    LightEmission, Link, Material, Model, Node, OwnerId, Property, PropertyDescriptor,
    PropertyPayload, PropertyTarget, SocketDefinition, Time, Track, ValidateLinks,
};

fn light_model<'a>(times: &'a [Time], values: &'a [f32]) -> Model<'a, Mdlx> {
    let mut builder = Model::<Mdlx>::builder("");
    let node = builder.add_node(Node::named("light")).unwrap();
    let feature = builder.reserve_feature(node);
    let property = light::INTENSITY.bind(feature);
    builder
        .set_feature(
            feature,
            MdlxFeature::Light(Light {
                diffuse: LightEmission {
                    intensity: Animated::new(1., property.clone()),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .unwrap();
    builder
        .add_timeline_channel(Channel::Scalar(Track {
            target: property,
            curve: Curve::linear(times, values).unwrap(),
            clock: Clock::Timeline,
        }))
        .unwrap();
    builder.finish().unwrap()
}

#[test]
fn containing_profiles_preserve_structural_identity_hashes_and_curve_values() {
    let times = [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()];
    let values = [1f32, 2.];
    let model = light_model(&times, &values);
    let original = model.timeline_channels()[0].target().clone();
    let original_hash = target_hash(&original);
    let broader = model.widen::<Universal>();
    let target = broader.timeline_channels()[0].target();
    assert_eq!(&original, target);
    assert_eq!(target_hash(target), original_hash);
    let narrowed = broader.try_narrow::<Mdlx>().unwrap();
    let Channel::Scalar(track) = &narrowed.timeline_channels()[0] else {
        panic!("wrong curve type")
    };
    assert_eq!(track.target.target(), &original);
    assert_eq!(track.curve.values().to_vec(), values);
    assert_eq!(track.curve.times().to_vec(), times);
}

fn target_hash<P: Profile>(target: &PropertyTarget<P>) -> u64 {
    let mut state = DefaultHasher::new();
    target.hash(&mut state);
    state.finish()
}

#[test]
fn emission_rate_selectors_in_distinct_game_vocabularies_do_not_alias() {
    let mut builder = Model::<Universal>::builder("selectors");
    let node = builder.add_node(Node::named("owner")).unwrap();
    let feature = builder.reserve_feature(node);
    let wc3 = particle2::EMISSION_RATE.bind(feature);
    let wow = m2_particle::RATE.bind(feature);
    assert_ne!(wc3, wow);
    assert_ne!(wc3.target().identity(), wow.target().identity());
}

#[test]
fn selectors_require_the_actual_feature_family_even_when_declared_by_a_field() {
    let mut builder = Model::<Mdlx>::builder("");
    let node = builder.add_node(Node::named("attachment")).unwrap();
    let feature = builder.reserve_feature(node);
    let property = light::INTENSITY.bind(feature);
    let resource = builder.add_model_resource(wc3::model::PathResource::Path(
        b"child.mdx".as_slice().into(),
    ));
    builder
        .set_feature(
            feature,
            MdlxFeature::Attachment(Attachment {
                resource,
                appearance: (),
                socket: SocketDefinition {
                    visibility: Animated::new(1., property.clone()),
                    ..Default::default()
                },
            }),
        )
        .unwrap();
    builder
        .add_timeline_channel(Channel::Scalar(Track {
            target: property,
            curve: Curve::step([Time::default()], [1f32]).unwrap(),
            clock: Clock::Timeline,
        }))
        .unwrap();
    assert!(builder.finish().is_err());
}

#[test]
fn layer_and_texture_slot_ordinals_must_exist_in_the_selected_material() {
    for slot in [false, true] {
        let mut builder = Model::<Mdlx>::builder("");
        let mut surface = MdlxMaterial {
            layers: vec![MdlxLayer::default()],
            ..Default::default()
        };
        let material = builder
            .add_material(Material {
                name: "material".into(),
                surface: surface.clone(),
            })
            .unwrap();
        if slot {
            surface.layers[0].binding.image =
                Animated::new(None, LayerProperties(0).texture_slot(2).bind(material));
        } else {
            surface.layers[0].opacity =
                Animated::new(1., LayerProperties(2).opacity().bind(material));
        }
        builder
            .set_material(
                material,
                Material {
                    name: "material".into(),
                    surface,
                },
            )
            .unwrap();
        assert!(builder.finish().is_err());
    }
}

#[test]
fn exact_property_narrowing_accepts_shared_fields_and_rejects_different_semantics() {
    let times = [Time::default()];
    let values = [1f32];
    let model = light_model(&times, &values);
    let target = model.timeline_channels()[0]
        .target()
        .clone()
        .widen::<Universal>();
    assert!(target.try_narrow::<M2>().is_ok());
    let feature = model.features().iter().next().unwrap().0;
    assert!(light::VISIBILITY
        .bind(feature)
        .widen::<Universal>()
        .try_narrow::<M2>()
        .is_err());
}

#[derive(Clone, Debug)]
struct Application;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ApplicationProperty {
    Weight,
}
impl PropertyPayload for ApplicationProperty {
    fn value_type(&self) -> Option<TypeId> {
        Some(TypeId::of::<f32>())
    }
}
#[derive(Clone, Debug)]
struct ApplicationFeature {
    weight: Animated<f32, Application>,
}
impl ValidateLinks for ApplicationFeature {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.weight.visit_links(visitor);
    }
}
impl<'data> FeatureSchema<'data> for ApplicationFeature {
    fn feature(&self) -> FeatureView<'_, 'data> {
        FeatureView::Custom(self)
    }
}
impl FeaturePayload for ApplicationFeature {
    fn kind(&self) -> FeatureKind {
        FeatureKind::Other
    }
}
impl Profile for Application {
    type ImageResource<'a> = wc3::model::PathResource<'a>;
    type ModelResource<'a> = wc3::model::profiles::Unavailable;
    type EffectResource<'a> = wc3::model::profiles::Unavailable;
    type PhysicsResource<'a> = wc3::model::profiles::Unavailable;
    type BoneOverrideResource<'a> = wc3::model::profiles::Unavailable;

    type Property = ApplicationProperty;
    type Appearance<'a> = <Universal as Profile>::Appearance<'a>;
    type Binding<'a> = <Universal as Profile>::Binding<'a>;
    type Geometry<'a> = <Universal as Profile>::Geometry<'a>;
    type ModelMetadata<'a> = <Universal as Profile>::ModelMetadata<'a>;
    type System<'a> = <Universal as Profile>::System<'a>;
    type Surface<'a> = <Universal as Profile>::Surface<'a>;
    type Feature<'a> = ApplicationFeature;
    type Node<'a> = <Universal as Profile>::Node<'a>;
    type Mesh<'a> = <Universal as Profile>::Mesh<'a>;
    type Instance<'a> = <Universal as Profile>::Instance<'a>;
    type Draw = <Universal as Profile>::Draw;
    type Clip<'a> = <Universal as Profile>::Clip<'a>;
    type Controller<'a> = <Universal as Profile>::Controller<'a>;
}
struct Weight;
impl PropertyDescriptor for Weight {
    type Profile = Application;
    type Owner = FeatureId;
    type Value = f32;
    fn key(self) -> ApplicationProperty {
        ApplicationProperty::Weight
    }
}

#[test]
fn custom_profile_enums_remain_typed_and_preserve_identity() {
    let times = [Time::default()];
    let values = [0.5f32];
    let mut builder = Model::<Application>::builder("custom");
    let node = builder.add_node(Node::named("owner")).unwrap();
    let feature = builder.reserve_feature(node);
    let property = Property::profile(feature, Weight);
    let original = property.target().clone();
    builder
        .set_feature(
            feature,
            ApplicationFeature {
                weight: Animated::new(1., property.clone()),
            },
        )
        .unwrap();
    builder
        .add_timeline_channel(Channel::Scalar(Track {
            target: property,
            curve: Curve::linear(&times[..], &values[..]).unwrap(),
            clock: Clock::Timeline,
        }))
        .unwrap();
    let model = builder.finish().unwrap();
    assert!(matches!(
        model.timeline_channels()[0].target(),
        PropertyTarget::Profile {
            key: ApplicationProperty::Weight,
            ..
        }
    ));
    assert_eq!(model.timeline_channels()[0].target(), &original);
    let Channel::Scalar(track) = &model.timeline_channels()[0] else {
        panic!("wrong curve type")
    };
    assert_eq!(track.curve.values().to_vec(), values);
}

#[test]
fn model_owner_addresses_include_their_scoped_model_identity() {
    let first = Model::<Application>::builder("first");
    let second = Model::<Application>::builder("second");
    let first_target = PropertyTarget::<Application>::Profile {
        owner: OwnerId::Model(first.id()),
        key: ApplicationProperty::Weight,
    };
    let second_target = PropertyTarget::<Application>::Profile {
        owner: OwnerId::Model(second.id()),
        key: ApplicationProperty::Weight,
    };
    assert_ne!(first_target, second_target);
}
