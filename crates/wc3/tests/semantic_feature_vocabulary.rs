use wc3::model::profiles::mdlx;
use wc3::model::profiles::{Mdlx, Profile, Unavailable};
use wc3::model::TimeSource;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{
    FeatureKind, FeatureSet, FeatureView, FeatureVocabulary, LightEmission, LightView, Link, Model,
    PropertyIdentityKey,
};
use wc3::model::{FeatureSchema as _, ValidateLinks as _};
#[derive(Clone, Debug)]
struct LightsOnly;
impl<'a> FeatureVocabulary<'a> for LightsOnly {
    type Light = mdlx::Light;
    type Camera = Unavailable;
    type Attachment = Unavailable;
    type Particle = Unavailable;
    type Ribbon = Unavailable;
    type Event = Unavailable;
    type GeometryAnimation = Unavailable;
    type CollisionShape = Unavailable;
    type ParticleProgram = Unavailable;
    type FaceActor = Unavailable;
    type LightProgram = Unavailable;
    type Extent = Unavailable;
    type ParticleCopy = Unavailable;
    type Projector = Unavailable;
    type Force = Unavailable;
    type Warp = Unavailable;
    type HitTest = Unavailable;
    type AttachmentVolume = Unavailable;
    type Billboard = Unavailable;
    type Turret = Unavailable;
    type IkJoint = Unavailable;
    type IkTwoJoint = Unavailable;
    type IkCcd = Unavailable;
    type OneBoneSolver = Unavailable;
    type ShadowBox = Unavailable;
    type ViewVolume = Unavailable;
    type AtlasParticles = Unavailable;
    type PointLight = Unavailable;
    fn visit_links(value: &FeatureSet<'a, Self>, visitor: &mut dyn FnMut(Link)) {
        match value {
            FeatureSet::Light(light) => light.visit_links(visitor),
            FeatureSet::Camera(unavailable)
            | FeatureSet::Attachment(unavailable)
            | FeatureSet::Particle(unavailable)
            | FeatureSet::Ribbon(unavailable)
            | FeatureSet::Event(unavailable)
            | FeatureSet::GeometryAnimation(unavailable)
            | FeatureSet::CollisionShape(unavailable)
            | FeatureSet::ParticleProgram(unavailable)
            | FeatureSet::FaceActor(unavailable)
            | FeatureSet::LightProgram(unavailable)
            | FeatureSet::Extent(unavailable)
            | FeatureSet::ParticleCopy(unavailable)
            | FeatureSet::Projector(unavailable)
            | FeatureSet::Force(unavailable)
            | FeatureSet::Warp(unavailable)
            | FeatureSet::HitTest(unavailable)
            | FeatureSet::AttachmentVolume(unavailable)
            | FeatureSet::Billboard(unavailable)
            | FeatureSet::Turret(unavailable)
            | FeatureSet::IkJoint(unavailable)
            | FeatureSet::IkTwoJoint(unavailable)
            | FeatureSet::IkCcd(unavailable)
            | FeatureSet::OneBoneSolver(unavailable)
            | FeatureSet::ShadowBox(unavailable)
            | FeatureSet::ViewVolume(unavailable)
            | FeatureSet::AtlasParticles(unavailable)
            | FeatureSet::PointLight(unavailable) => match *unavailable {},
        }
    }
    fn feature<'view>(value: &'view FeatureSet<'a, Self>) -> FeatureView<'view, 'a> {
        match value {
            FeatureSet::Light(light) => FeatureView::Light(LightView::VisibleAmbientDiffuse(light)),
            FeatureSet::Camera(unavailable)
            | FeatureSet::Attachment(unavailable)
            | FeatureSet::Particle(unavailable)
            | FeatureSet::Ribbon(unavailable)
            | FeatureSet::Event(unavailable)
            | FeatureSet::GeometryAnimation(unavailable)
            | FeatureSet::CollisionShape(unavailable)
            | FeatureSet::ParticleProgram(unavailable)
            | FeatureSet::FaceActor(unavailable)
            | FeatureSet::LightProgram(unavailable)
            | FeatureSet::Extent(unavailable)
            | FeatureSet::ParticleCopy(unavailable)
            | FeatureSet::Projector(unavailable)
            | FeatureSet::Force(unavailable)
            | FeatureSet::Warp(unavailable)
            | FeatureSet::HitTest(unavailable)
            | FeatureSet::AttachmentVolume(unavailable)
            | FeatureSet::Billboard(unavailable)
            | FeatureSet::Turret(unavailable)
            | FeatureSet::IkJoint(unavailable)
            | FeatureSet::IkTwoJoint(unavailable)
            | FeatureSet::IkCcd(unavailable)
            | FeatureSet::OneBoneSolver(unavailable)
            | FeatureSet::ShadowBox(unavailable)
            | FeatureSet::ViewVolume(unavailable)
            | FeatureSet::AtlasParticles(unavailable)
            | FeatureSet::PointLight(unavailable) => match *unavailable {},
        }
    }
    fn supports_property(_: &FeatureSet<'a, Self>, _: &PropertyIdentityKey) -> bool {
        false
    }
    fn kind(_: &FeatureSet<'a, Self>) -> FeatureKind {
        FeatureKind::Other
    }
}
impl Profile for LightsOnly {
    type ImageResource<'a> = wc3::model::profiles::Unavailable;
    type ModelResource<'a> = wc3::model::profiles::Unavailable;
    type EffectResource<'a> = wc3::model::profiles::Unavailable;
    type PhysicsResource<'a> = wc3::model::profiles::Unavailable;
    type BoneOverrideResource<'a> = wc3::model::profiles::Unavailable;

    type Property = <Mdlx as Profile>::Property;
    type Appearance<'a> = <Mdlx as Profile>::Appearance<'a>;
    type Binding<'a> = <Mdlx as Profile>::Binding<'a>;
    type Geometry<'a> = <Mdlx as Profile>::Geometry<'a>;
    type ModelMetadata<'a> = <Mdlx as Profile>::ModelMetadata<'a>;
    type System<'a> = <Mdlx as Profile>::System<'a>;
    type Surface<'a> = <Mdlx as Profile>::Surface<'a>;
    type Node<'a> = <Mdlx as Profile>::Node<'a>;
    type Mesh<'a> = <Mdlx as Profile>::Mesh<'a>;
    type Instance<'a> = <Mdlx as Profile>::Instance<'a>;
    type Draw = <Mdlx as Profile>::Draw;
    type Clip<'a> = <Mdlx as Profile>::Clip<'a>;
    type Controller<'a> = <Mdlx as Profile>::Controller<'a>;
    type Feature<'a> = FeatureSet<'a, Self>;
}
#[test]
fn custom_profile_selects_shared_light_composition_and_inspection() {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::<LightsOnly>::builder("lights only");
    let node = builder.add_named_node("light").unwrap();
    let id = builder
        .add_feature(
            node,
            FeatureSet::Light(mdlx::Light {
                diffuse: LightEmission {
                    intensity: slot_values.add(3.5),
                    ..SlotConstruct::construct(&mut slot_values)
                },
                ..SlotConstruct::construct(&mut slot_values)
            }),
        )
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let FeatureView::Light(light) = model.feature(id).unwrap().payload.feature() else {
        panic!()
    };
    assert_eq!(
        model.values()[light.ambient_diffuse().unwrap().diffuse.intensity],
        3.5
    );
}
#[test]
fn custom_vocabulary_uses_profile_independent_slots() {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::<LightsOnly>::builder("static lights only");
    let node = builder.add_named_node("light").unwrap();
    let id = builder.reserve_feature(node);
    let intensity = slot_values.add(1f32);
    builder
        .set_feature(
            id,
            FeatureSet::Light(mdlx::Light {
                diffuse: LightEmission {
                    intensity,
                    ..SlotConstruct::construct(&mut slot_values)
                },
                ..SlotConstruct::construct(&mut slot_values)
            }),
        )
        .unwrap();
    builder.import_values(&slot_values).unwrap();
    builder
        .add_global_channel(wc3::model::Channel::Scalar(wc3::model::Track {
            target: intensity,
            clock: TimeSource::Animation,
            curve: wc3::model::Curve::step([wc3::model::Time::default()], [2.]).unwrap(),
        }))
        .unwrap();
    let model = builder.finish().unwrap();
    let sample = model
        .animation_sample(wc3::model::AnimationSample::default())
        .unwrap();
    assert_eq!(sample.animated(&intensity), 2.);
}
