//! Built-in WMO authoring vocabulary and its lossless embeddings.
use crate::model::profiles::{wmo, Extends, NoControls, Profile, Unavailable, Universal};
use crate::model::resources::{WorldImageResource, WorldModelResource};
use crate::model::Link;
use crate::model::Model;
use crate::model::PropertyValues;
use crate::model::{
    DrawControls, Error, FeatureKind, InstanceControls, MaterialSurface, ModelMetadata, SingleLook,
    System, UniversalAppearance, WmoGeometry,
};
use crate::model::{FeatureSet, FeatureView, FeatureVocabulary};
/// Vocabulary that maps mechanically to Wmo logical features.
#[derive(Clone, Debug, Default)]
pub struct Wmo;
impl Profile for Wmo {
    type ImageResource<'a> = WorldImageResource<'a>;
    type ModelResource<'a> = WorldModelResource<'a>;
    type EffectResource<'a> = Unavailable;
    type PhysicsResource<'a> = Unavailable;
    type BoneOverrideResource<'a> = Unavailable;

    type Appearance<'a> = SingleLook<'a>;
    type Binding<'a> = Unavailable;
    type Geometry<'a> = WmoGeometry<'a>;
    type Draw = Option<wmo::DrawControls>;
    type ModelMetadata<'a> = wmo::ModelMetadata;
    type System<'a> = wmo::WorldSystem<'a>;
    type Surface<'a> = wmo::Material<'a>;
    type Feature<'a> = wmo::Node<'a>;
    type Node<'a> = NoControls;
    type Mesh<'a> = NoControls;
    type Instance<'a> = wmo::RegionControls<'a>;
    type Clip<'a> = NoControls;
    type Controller<'a> = NoControls;
    fn validate_resources(model: &Model<'_, Self>) -> Result<(), Error> {
        super::validation::resources(model)
    }
    fn validate_model(model: &Model<'_, Self>) -> Result<(), Error> {
        super::validation::model(model)
    }
}
impl wmo::Node<'_> {
    pub(crate) fn profile_payload_kind(&self) -> FeatureKind {
        match self {
            Self::Light(_) => FeatureKind::WmoLight,
            Self::PointLight(_) => FeatureKind::WmoPointLight,
            Self::LightProgram(_) => FeatureKind::WmoPointProgram,
            Self::Attachment(_) => FeatureKind::WmoDoodad,

            Self::Camera(unavailable)
            | Self::Particle(unavailable)
            | Self::Ribbon(unavailable)
            | Self::Event(unavailable)
            | Self::GeometryAnimation(unavailable)
            | Self::CollisionShape(unavailable)
            | Self::ParticleProgram(unavailable)
            | Self::FaceActor(unavailable)
            | Self::Extent(unavailable)
            | Self::ParticleCopy(unavailable)
            | Self::Projector(unavailable)
            | Self::Force(unavailable)
            | Self::Warp(unavailable)
            | Self::HitTest(unavailable)
            | Self::AttachmentVolume(unavailable)
            | Self::Billboard(unavailable)
            | Self::Turret(unavailable)
            | Self::IkJoint(unavailable)
            | Self::IkTwoJoint(unavailable)
            | Self::IkCcd(unavailable)
            | Self::OneBoneSolver(unavailable)
            | Self::ShadowBox(unavailable)
            | Self::ViewVolume(unavailable)
            | Self::AtlasParticles(unavailable) => match *unavailable {},
        }
    }
}
impl Extends<Wmo> for Universal {
    fn image_resource<'a>(value: <Wmo as Profile>::ImageResource<'a>) -> Self::ImageResource<'a> {
        value.into()
    }
    fn model_resource<'a>(value: <Wmo as Profile>::ModelResource<'a>) -> Self::ModelResource<'a> {
        value.into()
    }
    fn effect_resource<'a>(
        value: <Wmo as Profile>::EffectResource<'a>,
    ) -> Self::EffectResource<'a> {
        match value {}
    }
    fn physics_resource<'a>(
        value: <Wmo as Profile>::PhysicsResource<'a>,
    ) -> Self::PhysicsResource<'a> {
        match value {}
    }
    fn bone_override_resource<'a>(
        value: <Wmo as Profile>::BoneOverrideResource<'a>,
    ) -> Self::BoneOverrideResource<'a> {
        match value {}
    }

    fn appearance<'a>(value: <Wmo as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        UniversalAppearance::Single(value)
    }
    fn binding<'a>(value: <Wmo as Profile>::Binding<'a>) -> Self::Binding<'a> {
        match value {}
    }
    fn geometry<'a>(value: <Wmo as Profile>::Geometry<'a>) -> Self::Geometry<'a> {
        value.into()
    }
    fn draw(value: <Wmo as Profile>::Draw) -> Self::Draw {
        value.map(DrawControls::Wmo)
    }
    fn model_metadata<'a>(value: <Wmo as Profile>::ModelMetadata<'a>) -> Self::ModelMetadata<'a> {
        ModelMetadata::Wmo(value)
    }
    fn system<'a>(value: <Wmo as Profile>::System<'a>) -> Self::System<'a> {
        System::Wmo(value)
    }
    fn surface<'a>(value: <Wmo as Profile>::Surface<'a>) -> Self::Surface<'a> {
        MaterialSurface::from(value)
    }
    fn feature<'a>(
        value: <Wmo as Profile>::Feature<'a>,
        _values: &mut PropertyValues,
    ) -> Self::Feature<'a> {
        value.into()
    }
    fn node<'a>(value: <Wmo as Profile>::Node<'a>) -> Self::Node<'a> {
        let _ = value;
        None
    }
    fn mesh<'a>(value: <Wmo as Profile>::Mesh<'a>) -> Self::Mesh<'a> {
        let _ = value;
        None
    }
    fn instance<'a>(value: <Wmo as Profile>::Instance<'a>) -> Self::Instance<'a> {
        Some(InstanceControls::Wmo(Box::new(value)))
    }
    fn clip<'a>(value: <Wmo as Profile>::Clip<'a>) -> Self::Clip<'a> {
        let _ = value;
        None
    }
    fn controller<'a>(value: <Wmo as Profile>::Controller<'a>) -> Self::Controller<'a> {
        let _ = value;
        None
    }
}

impl<'a> FeatureVocabulary<'a> for Wmo {
    type Light = Box<wmo::Light>;
    type Camera = Unavailable;
    type Attachment = wmo::Doodad;
    type Particle = Unavailable;
    type Ribbon = Unavailable;
    type Event = Unavailable;
    type GeometryAnimation = Unavailable;
    type CollisionShape = Unavailable;
    type ParticleProgram = Unavailable;
    type FaceActor = Unavailable;
    type LightProgram = wmo::PointProgram;
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
    type PointLight = wmo::PointLight;
    fn visit_links(value: &FeatureSet<'a, Self>, visitor: &mut dyn FnMut(Link)) {
        value.visit_profile_links(visitor);
    }
    fn feature<'view>(value: &'view FeatureSet<'a, Self>) -> FeatureView<'view, 'a> {
        value.profile_feature()
    }

    fn kind(value: &FeatureSet<'a, Self>) -> FeatureKind {
        value.profile_payload_kind()
    }
}
