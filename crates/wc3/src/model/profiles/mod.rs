//! Compile-time vocabulary of logical payloads; unrelated to native file revisions.
use crate::model::EffectResourceId;
use crate::model::PropertyValues;
use crate::model::ResourceLocator;
use crate::model::{
    AppearanceSchema, BindingSchema, Bounds, Error, FeatureKind, FeatureProperty, GeometrySchema,
    Link, MaterialSurface, Model, ModelMetadata, NodeFeature, SkinId, SystemPayload, ValidateLinks,
};
use crate::model::{
    Attachment, EventSource, FeatureSet, FeatureView, FeatureVocabulary, ImageResource, Light,
    ParticleEmitter, RibbonEmitter,
};
use crate::model::{Camera, CameraConvention, FeatureSchema, OwnerId};
use crate::model::{
    ClipControls, DrawControls, GeometryData, InstanceControls, MeshControls, NodeControls, System,
    UniversalAppearance, UniversalBinding,
};
use crate::model::{PropertyIdentityKey, PropertyKeyRef, PropertyPayload, PropertyTarget};
use std::any::TypeId;
use std::fmt::Debug;

/// A checked surface supplied by a semantic profile.
pub trait SurfacePayload: Clone + Debug + Send + Sync + ValidateLinks {
    /// Checks that a structural property selects an existing field in this surface.
    fn supports_property(&self, _key: &PropertyIdentityKey) -> bool {
        true
    }

    /// Checks local invariants independent of graph references.
    fn validate(&self) -> Result<(), Error>;
}
/// A checked attached behavior supplied by a semantic profile.
pub trait FeaturePayload: Clone + Debug + Send + Sync + ValidateLinks {
    /// Checks the selected field family and any owner-local element ordinal.
    fn supports_property(&self, _key: &PropertyIdentityKey) -> bool {
        true
    }

    /// Relationship category used by typed feature references.
    fn kind(&self) -> FeatureKind;
}
/// An owner for which this profile has no additional controls.
///
/// This is one zero-sized value, rather than optional presence of an empty payload.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct NoControls;
impl ValidateLinks for NoControls {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
/// A vocabulary with no possible authored payload.
///
/// Use this for object families that a profile cannot describe. An owner with
/// optional controls can still exist, but its controls must remain absent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Unavailable {}
impl ValidateLinks for Unavailable {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {
        match *self {}
    }
}
impl FeaturePayload for Unavailable {
    fn kind(&self) -> FeatureKind {
        match *self {}
    }
}
impl SurfacePayload for Unavailable {
    fn validate(&self) -> Result<(), Error> {
        match *self {}
    }
}
/// Owner-local payloads expose their relationships to graph validation.
///
/// `ValidateLinks` implementations must visit every referenced graph object;
/// they must report invalid authored values with `Link::InvalidValue`.
pub trait ControlsPayload: Clone + Debug + Send + Sync + ValidateLinks {}
impl<T: Clone + Debug + Send + Sync + ValidateLinks> ControlsPayload for T {}
/// Selects the complete payload vocabulary allowed in a model graph.
///
/// This parameter describes authoring meaning, rather than an on-disk version.
/// For example, `Mdlx` describes Warcraft III behavior while native MDLX dialects
/// select which file revisions can encode that behavior.
/// Surface and feature payloads define complete objects; other owner payloads
/// appear directly in their corresponding object's `controls` field. Profiles
/// select required, optional, or absent controls independently for each owner.
///
/// Implementations may use their own payload types. Implementing relationship traversal
/// is mandatory: graph validation checks custom references exactly like built-in ones.
pub trait Profile: Clone + Debug + 'static {
    /// External image resource declarations admitted by this vocabulary.
    type ImageResource<'a>: ControlsPayload;
    /// External model resource declarations admitted by this vocabulary.
    type ModelResource<'a>: ControlsPayload;
    /// External effect resource declarations admitted by this vocabulary.
    type EffectResource<'a>: ControlsPayload;
    /// External physics resource declarations admitted by this vocabulary.
    type PhysicsResource<'a>: ControlsPayload;
    /// External boneoverride resource declarations admitted by this vocabulary.
    type BoneOverrideResource<'a>: ControlsPayload;
    /// Structural animation-property vocabulary accepted by this profile.
    type Property: PropertyPayload;
    /// Complete material-assignment and appearance vocabulary.
    type Appearance<'a>: AppearanceSchema<'a>;
    /// Joint binding interpretations supported by this vocabulary.
    type Binding<'a>: BindingSchema<'a>;
    /// Complete geometry vocabulary, including supported streams and connectivity.
    type Geometry<'a>: GeometrySchema<'a>;
    /// Model-wide authoring metadata.
    type ModelMetadata<'a>: ControlsPayload + Default;
    /// Independently authored scene systems.
    type System<'a>: SystemPayload;
    /// Complete surface payload, with no unsupported family variants.
    type Surface<'a>: SurfacePayload;
    /// Complete attached behavior payload.
    type Feature<'a>: FeaturePayload + FeatureSchema<'a>;
    /// Additional logical hierarchy behavior.
    type Node<'a>: ControlsPayload + Default;
    /// Additional geometry behavior.
    type Mesh<'a>: ControlsPayload + Default;
    /// Additional geometry placement behavior.
    type Instance<'a>: ControlsPayload + Default;
    /// Additional draw-pass behavior.
    type Draw: ControlsPayload + Default;
    /// Additional playback selection and transition behavior.
    type Clip<'a>: ControlsPayload + Default;
    /// Additional layered-controller behavior.
    type Controller<'a>: ControlsPayload + Default;
    /// Checks a structural selector against its actual owner payload.
    fn validate_property(
        model: &Model<'_, Self>,
        target: &PropertyTarget<Self>,
    ) -> Result<(), Error> {
        if let PropertyTarget::Profile { owner, key } = target {
            let identity = key.identity().to_owned();
            let supported = match owner {
                OwnerId::Feature(id) => model
                    .feature(*id)
                    .is_some_and(|v| v.payload.supports_property(&identity)),
                OwnerId::Material(id) => model
                    .material(*id)
                    .is_some_and(|v| v.surface.supports_property(&identity)),
                OwnerId::Model(id) => *id == model.id(),
                _ => true,
            };
            if !supported {
                return Err(Error::new(
                    "animation.target",
                    "structural selector does not select a field or element of its actual owner",
                ));
            }
        }
        Ok(())
    }
    /// Initial bounding volume for a new model draft.
    fn default_bounds() -> Option<Bounds> {
        None
    }
    /// Returns the binding owned by shared geometry, when the vocabulary supports one.
    fn mesh_skin(controls: &Self::Mesh<'_>) -> Option<SkinId> {
        let _ = controls;
        None
    }
    /// Checks profile-specific resource constraints on the complete model.
    fn validate_resources(model: &Model<'_, Self>) -> Result<(), Error> {
        let _ = model;
        Ok(())
    }
    /// Checks relationships within additional authoring domains owned by this profile.
    ///
    /// Standard graph references have already been checked. Implementations can
    /// inspect the complete model to validate profile-specific authoring domains.
    fn validate_model(model: &Model<'_, Self>) -> Result<(), Error> {
        let _ = model;
        Ok(())
    }
}
/// Either vocabulary is accepted, without conversion between their meanings.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Either<A, B> {
    /// Payload belonging to the first vocabulary.
    Left(A),
    /// Payload belonging to the second vocabulary.
    Right(B),
}
impl<A: ValidateLinks, B: ValidateLinks> ValidateLinks for Either<A, B> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Left(value) => value.visit_links(visitor),
            Self::Right(value) => value.visit_links(visitor),
        }
    }
}
impl<A: SurfacePayload, B: SurfacePayload> SurfacePayload for Either<A, B> {
    fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        match self {
            Self::Left(v) => v.supports_property(key),
            Self::Right(v) => v.supports_property(key),
        }
    }

    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Left(value) => value.validate(),
            Self::Right(value) => value.validate(),
        }
    }
}
impl<A: FeaturePayload, B: FeaturePayload> FeaturePayload for Either<A, B> {
    fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        match self {
            Self::Left(v) => v.supports_property(key),
            Self::Right(v) => v.supports_property(key),
        }
    }

    fn kind(&self) -> FeatureKind {
        match self {
            Self::Left(value) => value.kind(),
            Self::Right(value) => value.kind(),
        }
    }
}
/// All built-in logical feature families. Applications can define custom profiles.
#[derive(Clone, Debug, Default)]
pub struct Universal;
impl Profile for Universal {
    type ImageResource<'a> = ImageResource<'a>;
    type ModelResource<'a> = ResourceLocator<'a>;
    type EffectResource<'a> = ResourceLocator<'a>;
    type PhysicsResource<'a> = ResourceLocator<'a>;
    type BoneOverrideResource<'a> = ResourceLocator<'a>;

    type Property = UniversalProperty;
    type Appearance<'a> = UniversalAppearance<'a>;
    type Binding<'a> = UniversalBinding<'a>;
    type Geometry<'a> = GeometryData<'a>;
    type Draw = Option<DrawControls>;
    type ModelMetadata<'a> = Option<ModelMetadata<'a>>;
    type System<'a> = System<'a>;
    type Surface<'a> = MaterialSurface<'a>;
    type Feature<'a> = NodeFeature<'a>;
    type Node<'a> = Option<NodeControls<'a>>;
    type Mesh<'a> = Option<MeshControls>;
    type Instance<'a> = Option<InstanceControls<'a>>;
    type Clip<'a> = Option<ClipControls<'a>>;
    type Controller<'a> = Option<m3::ControllerControls<'a>>;
    fn mesh_skin(controls: &Self::Mesh<'_>) -> Option<SkinId> {
        match controls {
            Some(MeshControls::M3(value)) => value.skin,
            _ => None,
        }
    }
}
/// Proof that a target vocabulary can accept every source payload unchanged.
///
/// This is separate from approximate translation. Implementations must preserve meaning
/// and borrowed storage, and must not discard fields or relationships.
pub trait Extends<Source: Profile>: Profile {
    /// Embeds image dependency declarations.
    fn image_resource<'a>(value: Source::ImageResource<'a>) -> Self::ImageResource<'a>;
    /// Embeds model dependency declarations.
    fn model_resource<'a>(value: Source::ModelResource<'a>) -> Self::ModelResource<'a>;
    /// Embeds effect dependency declarations.
    fn effect_resource<'a>(value: Source::EffectResource<'a>) -> Self::EffectResource<'a>;
    /// Embeds physics dependency declarations.
    fn physics_resource<'a>(value: Source::PhysicsResource<'a>) -> Self::PhysicsResource<'a>;
    /// Embeds boneoverride dependency declarations.
    fn bone_override_resource<'a>(
        value: Source::BoneOverrideResource<'a>,
    ) -> Self::BoneOverrideResource<'a>;

    /// Embeds structural property selectors without changing their identity.
    fn property(value: Source::Property) -> Self::Property;
    /// Embeds the complete appearance vocabulary while preserving identities.
    fn appearance<'a>(value: Source::Appearance<'a>) -> Self::Appearance<'a>;
    /// Embeds the authored joint binding without changing its interpretation.
    fn binding<'a>(value: Source::Binding<'a>) -> Self::Binding<'a>;
    /// Embeds complete geometry without changing its logical values.
    fn geometry<'a>(value: Source::Geometry<'a>) -> Self::Geometry<'a>;
    /// Embeds model-wide metadata without changing their meaning.
    fn model_metadata<'a>(value: Source::ModelMetadata<'a>) -> Self::ModelMetadata<'a>;
    /// Embeds an independent scene system without changing its meaning.
    fn system<'a>(value: Source::System<'a>) -> Self::System<'a>;
    /// Embeds a surface without changing its meaning.
    fn surface<'a>(value: Source::Surface<'a>) -> Self::Surface<'a>;
    /// Embeds an attached feature without changing its meaning.
    fn feature<'a>(value: Source::Feature<'a>, values: &mut PropertyValues) -> Self::Feature<'a>;
    /// Embeds node controls without changing meaning.
    fn node<'a>(value: Source::Node<'a>) -> Self::Node<'a>;
    /// Embeds mesh controls without changing meaning.
    fn mesh<'a>(value: Source::Mesh<'a>) -> Self::Mesh<'a>;
    /// Embeds instance controls without changing meaning.
    fn instance<'a>(value: Source::Instance<'a>) -> Self::Instance<'a>;
    /// Embeds draw-pass behavior without changing meaning.
    fn draw(value: Source::Draw) -> Self::Draw;
    /// Embeds clip controls without changing meaning.
    fn clip<'a>(value: Source::Clip<'a>) -> Self::Clip<'a>;
    /// Embeds controller controls without changing meaning.
    fn controller<'a>(value: Source::Controller<'a>) -> Self::Controller<'a>;
}
impl<P: Profile> Extends<P> for P {
    fn image_resource<'a>(value: P::ImageResource<'a>) -> Self::ImageResource<'a> {
        value
    }
    fn model_resource<'a>(value: P::ModelResource<'a>) -> Self::ModelResource<'a> {
        value
    }
    fn effect_resource<'a>(value: P::EffectResource<'a>) -> Self::EffectResource<'a> {
        value
    }
    fn physics_resource<'a>(value: P::PhysicsResource<'a>) -> Self::PhysicsResource<'a> {
        value
    }
    fn bone_override_resource<'a>(
        value: P::BoneOverrideResource<'a>,
    ) -> Self::BoneOverrideResource<'a> {
        value
    }

    fn property(value: P::Property) -> Self::Property {
        value
    }
    fn appearance<'a>(value: <P as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        value
    }
    fn binding<'a>(value: <P as Profile>::Binding<'a>) -> Self::Binding<'a> {
        value
    }
    fn geometry<'a>(value: <P as Profile>::Geometry<'a>) -> Self::Geometry<'a> {
        value
    }
    fn draw(value: <P as Profile>::Draw) -> Self::Draw {
        value
    }
    fn model_metadata<'a>(value: <P as Profile>::ModelMetadata<'a>) -> Self::ModelMetadata<'a> {
        value
    }
    fn system<'a>(value: <P as Profile>::System<'a>) -> Self::System<'a> {
        value
    }
    fn surface<'a>(value: P::Surface<'a>) -> Self::Surface<'a> {
        value
    }
    fn feature<'a>(value: P::Feature<'a>, _values: &mut PropertyValues) -> Self::Feature<'a> {
        value
    }
    fn node<'a>(value: <P as Profile>::Node<'a>) -> Self::Node<'a> {
        value
    }
    fn mesh<'a>(value: <P as Profile>::Mesh<'a>) -> Self::Mesh<'a> {
        value
    }
    fn instance<'a>(value: <P as Profile>::Instance<'a>) -> Self::Instance<'a> {
        value
    }
    fn clip<'a>(value: <P as Profile>::Clip<'a>) -> Self::Clip<'a> {
        value
    }
    fn controller<'a>(value: <P as Profile>::Controller<'a>) -> Self::Controller<'a> {
        value
    }
}
impl NodeFeature<'_> {
    pub(crate) fn profile_supports_property(&self, key: &PropertyIdentityKey) -> bool {
        if let Some(supported) = self.supports_shared_feature_property(key) {
            return supported;
        }
        if matches!(
            key.downcast_ref::<FeatureProperty>(),
            Some(FeatureProperty::CameraVerticalFieldOfView)
        ) {
            return matches!(self, NodeFeature::Camera(Camera::Aimed(camera)) if matches!(camera.convention, CameraConvention::Vertical(_)));
        }
        mdlx::properties::Target::from_identity(key)
            .is_some_and(|target| target.supports_universal_feature(self))
            || m2::properties::Target::from_identity(key)
                .is_some_and(|target| target.supports_universal_feature(self))
            || m3::properties::Target::from_identity(key)
                .is_some_and(|target| target.supports_universal_feature(self))
    }
}

pub use m2::M2;
pub use m3::M3;
pub use mdlx::Mdlx;
pub use wmo::Wmo;

// Logical authoring features specific to each model family.

pub mod m2;
pub mod m3;
pub mod mdlx;
pub mod wmo;

mod narrowing;
pub use narrowing::Narrows;

/// Diablo game authoring vocabularies independent of native document layouts.
pub mod diablo;

pub use diablo::d3::D3;

pub use diablo::d4::D4;

impl PropertyPayload for Unavailable {
    fn identity(&self) -> PropertyKeyRef<'_> {
        match *self {}
    }
}
impl<A: PropertyPayload, B: PropertyPayload> PropertyPayload for Either<A, B> {
    fn identity(&self) -> PropertyKeyRef<'_> {
        match self {
            Self::Left(v) => v.identity(),
            Self::Right(v) => v.identity(),
        }
    }
    fn from_identity(key: &PropertyIdentityKey) -> Option<Self> {
        A::from_identity(key)
            .map(Self::Left)
            .or_else(|| B::from_identity(key).map(Self::Right))
    }
    fn value_type(&self) -> Option<TypeId> {
        match self {
            Self::Left(v) => v.value_type(),
            Self::Right(v) => v.value_type(),
        }
    }
    fn accepts_owner(&self, owner: OwnerId) -> bool {
        match self {
            Self::Left(v) => v.accepts_owner(owner),
            Self::Right(v) => v.accepts_owner(owner),
        }
    }
}
/// Structural property selectors from all built-in semantic vocabularies.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UniversalProperty {
    /// Shared behavior destination with format-independent semantics.
    Feature(FeatureProperty),
    /// Warcraft III destination.
    Mdlx(mdlx::properties::Target),
    /// World of Warcraft destination.
    M2(m2::properties::Target),
    /// StarCraft II destination.
    M3(m3::properties::Target),
}
impl PropertyPayload for UniversalProperty {
    fn identity(&self) -> PropertyKeyRef<'_> {
        match self {
            Self::Feature(v) => v.identity(),
            Self::Mdlx(v) => v.identity(),
            Self::M2(v) => v.identity(),
            Self::M3(v) => v.identity(),
        }
    }
    fn from_identity(key: &PropertyIdentityKey) -> Option<Self> {
        FeatureProperty::from_identity(key)
            .map(Self::Feature)
            .or_else(|| mdlx::properties::Target::from_identity(key).map(Self::Mdlx))
            .or_else(|| m2::properties::Target::from_identity(key).map(Self::M2))
            .or_else(|| m3::properties::Target::from_identity(key).map(Self::M3))
    }
    fn value_type(&self) -> Option<TypeId> {
        match self {
            Self::Feature(v) => v.value_type(),
            Self::Mdlx(v) => v.value_type(),
            Self::M2(v) => v.value_type(),
            Self::M3(v) => v.value_type(),
        }
    }
    fn accepts_owner(&self, owner: OwnerId) -> bool {
        match self {
            Self::Feature(v) => v.accepts_owner(owner),
            Self::Mdlx(v) => v.accepts_owner(owner),
            Self::M2(v) => v.accepts_owner(owner),
            Self::M3(v) => v.accepts_owner(owner),
        }
    }
}

impl<'a> FeatureVocabulary<'a> for Universal {
    type Light = Light;
    type Camera = Camera<'a>;
    type Attachment = Attachment<'a>;
    type Particle = ParticleEmitter<'a>;
    type Ribbon = RibbonEmitter<'a>;
    type Event = EventSource<'a>;
    type GeometryAnimation = mdlx::GeometryAnimation;
    type CollisionShape = mdlx::Collision;
    type ParticleProgram = Box<mdlx::Popcorn<'a>>;
    type FaceActor = EffectResourceId;
    type LightProgram = wmo::PointProgram;
    type Extent = Box<m3::ExtentBinding>;
    type ParticleCopy = Box<m3::ParticleCopy>;
    type Projector = Box<m3::Projector>;
    type Force = Box<m3::Force>;
    type Warp = Box<m3::Warp>;
    type HitTest = Box<m3::HitTestShape>;
    type AttachmentVolume = Box<m3::AttachmentVolume>;
    type Billboard = Box<m3::BillboardBehavior>;
    type Turret = Box<m3::TurretBehavior>;
    type IkJoint = Box<m3::IkJoint>;
    type IkTwoJoint = Box<m3::IkTwoJoint>;
    type IkCcd = Box<m3::IkCcd>;
    type OneBoneSolver = Box<m3::OneBoneSolver>;
    type ShadowBox = Box<m3::ShadowBox>;
    type ViewVolume = Box<m3::ViewVolume>;
    type AtlasParticles = Unavailable;
    type PointLight = Unavailable;
    fn visit_links(value: &FeatureSet<'a, Self>, visitor: &mut dyn FnMut(Link)) {
        value.visit_feature_links(visitor);
    }
    fn feature<'view>(value: &'view FeatureSet<'a, Self>) -> FeatureView<'view, 'a> {
        value.profile_feature()
    }
    fn supports_property(value: &FeatureSet<'a, Self>, key: &PropertyIdentityKey) -> bool {
        value.profile_supports_property(key)
    }
    fn kind(value: &FeatureSet<'a, Self>) -> FeatureKind {
        value.profile_kind()
    }
}
