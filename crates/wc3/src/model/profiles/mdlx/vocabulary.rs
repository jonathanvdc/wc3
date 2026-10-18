//! Built-in MDLX authoring vocabulary and its lossless embeddings.
use super::properties::Target as PropertySelector;
use crate::model::profiles::UniversalProperty;
use crate::model::profiles::{mdlx, Extends, NoControls, Profile, Unavailable, Universal};
use crate::model::resources::{MdlxImageResource, PathResource};
use crate::model::Link;
use crate::model::Model;
use crate::model::PropertyIdentityKey;
use crate::model::PropertyValues;
use crate::model::{
    ClipControls, Error, FeatureKind, IdentityBinding, MaterialSurface, MdlxGeometry, MeshControls,
    ModelMetadata, NodeControls, PropertyPayload, SingleLook, UniversalAppearance,
    UniversalBinding,
};
use crate::model::{FeatureSet, FeatureView, FeatureVocabulary};
/// Vocabulary that maps mechanically to Warcraft III logical features.
///
/// The graph cannot accept surfaces outside its vocabulary:
/// ```compile_fail
/// use wc3::model::Material;
/// use wc3::model::materials::m2::MaterialStep as M2MaterialStep;
/// use wc3::model::profiles::Mdlx;
/// let material = Material::<Mdlx> {
///     name: "invalid surface family".into(),
///     surface: M2MaterialStep::default().into(),
/// };
/// ```
///
/// Image addresses are constrained by the profile as well:
/// ```compile_fail
/// use wc3::model::{Model, ImageResource};
/// use wc3::model::profiles::Mdlx;
/// let mut draft = Model::<Mdlx>::builder("invalid resource");
/// draft.add_image_resource(ImageResource::FileDataId(42));
/// ```
#[derive(Clone, Debug, Default)]
pub struct Mdlx;
impl Profile for Mdlx {
    type ImageResource<'a> = MdlxImageResource<'a>;
    type ModelResource<'a> = PathResource<'a>;
    type EffectResource<'a> = PathResource<'a>;
    type PhysicsResource<'a> = Unavailable;
    type BoneOverrideResource<'a> = Unavailable;

    type Property = PropertySelector;
    fn default_bounds() -> Option<crate::model::Bounds> {
        Some(Default::default())
    }
    type Appearance<'a> = SingleLook<'a>;
    type Binding<'a> = IdentityBinding;
    type Geometry<'a> = MdlxGeometry<'a>;
    type Draw = NoControls;
    type ModelMetadata<'a> = mdlx::ModelMetadata<'a>;
    type System<'a> = Unavailable;
    type Surface<'a> = mdlx::Material<'a>;
    type Feature<'a> = mdlx::Node<'a>;
    type Node<'a> = mdlx::NodeControls;
    type Mesh<'a> = mdlx::Geometry;
    type Instance<'a> = NoControls;
    type Clip<'a> = mdlx::Sequence;
    type Controller<'a> = NoControls;
    fn validate_resources(model: &Model<'_, Self>) -> Result<(), Error> {
        super::validation::resources(model)
    }
    fn validate_model(model: &Model<'_, Self>) -> Result<(), Error> {
        super::validation::model(model)
    }
}
impl mdlx::Node<'_> {
    pub(crate) fn profile_supports_property(&self, key: &PropertyIdentityKey) -> bool {
        super::properties::Target::from_identity(key).is_some_and(|key| key.supports_feature(self))
    }

    pub(crate) fn profile_payload_kind(&self) -> FeatureKind {
        match self {
            mdlx::Node::GeometryAnimation(_) => FeatureKind::MdlxGeometryAnimation,
            _ => FeatureKind::Other,
        }
    }
}
impl Extends<Mdlx> for Universal {
    fn image_resource<'a>(value: <Mdlx as Profile>::ImageResource<'a>) -> Self::ImageResource<'a> {
        value.into()
    }
    fn model_resource<'a>(value: <Mdlx as Profile>::ModelResource<'a>) -> Self::ModelResource<'a> {
        value.into()
    }
    fn effect_resource<'a>(
        value: <Mdlx as Profile>::EffectResource<'a>,
    ) -> Self::EffectResource<'a> {
        value.into()
    }
    fn physics_resource<'a>(
        value: <Mdlx as Profile>::PhysicsResource<'a>,
    ) -> Self::PhysicsResource<'a> {
        match value {}
    }
    fn bone_override_resource<'a>(
        value: <Mdlx as Profile>::BoneOverrideResource<'a>,
    ) -> Self::BoneOverrideResource<'a> {
        match value {}
    }

    fn property(value: <Mdlx as Profile>::Property) -> Self::Property {
        UniversalProperty::Mdlx(value)
    }
    fn appearance<'a>(value: <Mdlx as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        UniversalAppearance::Single(value)
    }
    fn binding<'a>(value: <Mdlx as Profile>::Binding<'a>) -> Self::Binding<'a> {
        UniversalBinding::Identity(value)
    }
    fn geometry<'a>(value: <Mdlx as Profile>::Geometry<'a>) -> Self::Geometry<'a> {
        value.into()
    }
    fn draw(value: <Mdlx as Profile>::Draw) -> Self::Draw {
        let _ = value;
        None
    }
    fn model_metadata<'a>(value: <Mdlx as Profile>::ModelMetadata<'a>) -> Self::ModelMetadata<'a> {
        Some(ModelMetadata::Mdlx(value))
    }
    fn system<'a>(value: <Mdlx as Profile>::System<'a>) -> Self::System<'a> {
        match value {}
    }
    fn surface<'a>(value: <Mdlx as Profile>::Surface<'a>) -> Self::Surface<'a> {
        MaterialSurface::try_from(value).expect("validated MDLX surface")
    }
    fn feature<'a>(
        value: <Mdlx as Profile>::Feature<'a>,
        values: &mut PropertyValues,
    ) -> Self::Feature<'a> {
        crate::model::features::FromFeature::from_feature(value, values)
    }
    fn node<'a>(value: <Mdlx as Profile>::Node<'a>) -> Self::Node<'a> {
        Some(NodeControls::Mdlx(value))
    }
    fn mesh<'a>(value: <Mdlx as Profile>::Mesh<'a>) -> Self::Mesh<'a> {
        Some(MeshControls::Mdlx(value))
    }
    fn instance<'a>(value: <Mdlx as Profile>::Instance<'a>) -> Self::Instance<'a> {
        let _ = value;
        None
    }
    fn clip<'a>(value: <Mdlx as Profile>::Clip<'a>) -> Self::Clip<'a> {
        Some(ClipControls::Mdlx(value))
    }
    fn controller<'a>(value: <Mdlx as Profile>::Controller<'a>) -> Self::Controller<'a> {
        let _ = value;
        None
    }
}

impl<'a> FeatureVocabulary<'a> for Mdlx {
    type Light = mdlx::Light;
    type Camera = mdlx::Camera;
    type Attachment = mdlx::Attachment;
    type Particle = mdlx::Particle;
    type Ribbon = mdlx::Ribbon;
    type Event = mdlx::Events<'a>;
    type GeometryAnimation = mdlx::GeometryAnimation;
    type CollisionShape = mdlx::Collision;
    type ParticleProgram = mdlx::Popcorn<'a>;
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
    type AtlasParticles = mdlx::Particle2;
    type PointLight = Unavailable;
    fn visit_links(value: &FeatureSet<'a, Self>, visitor: &mut dyn FnMut(Link)) {
        value.visit_profile_links(visitor);
    }
    fn feature<'view>(value: &'view FeatureSet<'a, Self>) -> FeatureView<'view, 'a> {
        value.profile_feature()
    }
    fn supports_property(value: &FeatureSet<'a, Self>, key: &PropertyIdentityKey) -> bool {
        value.profile_supports_property(key)
    }
    fn kind(value: &FeatureSet<'a, Self>) -> FeatureKind {
        value.profile_payload_kind()
    }
}
