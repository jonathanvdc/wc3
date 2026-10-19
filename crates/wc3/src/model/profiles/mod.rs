//! Compile-time vocabulary of logical payloads; unrelated to native file revisions.
use crate::model::EffectResourceId;
use crate::model::PropertyValues;
use crate::model::ResourceLocator;
use crate::model::{
    AppearanceSchema, BindingSchema, Error, FeatureKind, GeometrySchema, Link, MaterialSurface,
    Model, ModelMetadata, NodeFeature, SkinId, SystemPayload, ValidateLinks,
};
use crate::model::{
    Attachment, EventSource, FeatureSet, FeatureView, FeatureVocabulary, ImageResource, Light,
    ParticleEmitter, RibbonEmitter,
};
use crate::model::{Camera, FeatureSchema};
use crate::model::{
    ClipProperties, DrawProperties, GeometryData, InstanceProperties, MeshProperties,
    NodeProperties, System, UniversalAppearance, UniversalBinding,
};
use crate::model::{EventSourceSchema, TimeSource};
use std::fmt::Debug;

/// A checked surface supplied by a semantic profile.
pub trait SurfacePayload: Clone + Debug + Send + Sync + ValidateLinks {
    /// Checks local invariants independent of graph references.
    fn validate(&self) -> Result<(), Error>;
}
/// A checked attached behavior supplied by a semantic profile.
pub trait FeaturePayload: Clone + Debug + Send + Sync + ValidateLinks {
    /// Relationship category used by typed feature references.
    fn kind(&self) -> FeatureKind;
}
/// An owner for which this profile has no additional properties.
///
/// This is one zero-sized value, rather than optional presence of an empty payload.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct NoProperties;
impl ValidateLinks for NoProperties {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
/// A vocabulary with no possible authored payload.
///
/// Use this for object families that a profile cannot describe. An owner with
/// optional properties can still exist, but its properties must remain absent.
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
pub trait PropertiesPayload: Clone + Debug + Send + Sync + ValidateLinks {}
impl<T: Clone + Debug + Send + Sync + ValidateLinks> PropertiesPayload for T {}
/// Selects the complete payload vocabulary allowed in a model graph.
///
/// This parameter describes authoring meaning, rather than an on-disk version.
/// For example, `Mdlx` describes Warcraft III behavior while native MDLX dialects
/// select which file revisions can encode that behavior.
/// Surface and feature payloads define complete objects; other owner payloads
/// appear directly in their corresponding object's `properties` field. Profiles
/// select required, optional, or absent properties independently for each owner.
///
/// Implementations may use their own payload types. Implementing relationship traversal
/// is mandatory: graph validation checks custom references exactly like built-in ones.
pub trait Profile: Clone + Debug + 'static {
    /// External image resource declarations admitted by this vocabulary.
    type ImageResource<'a>: PropertiesPayload;
    /// External model resource declarations admitted by this vocabulary.
    type ModelResource<'a>: PropertiesPayload;
    /// External effect resource declarations admitted by this vocabulary.
    type EffectResource<'a>: PropertiesPayload;
    /// External physics resource declarations admitted by this vocabulary.
    type PhysicsResource<'a>: PropertiesPayload;
    /// External boneoverride resource declarations admitted by this vocabulary.
    type BoneOverrideResource<'a>: PropertiesPayload;
    /// Complete material-assignment and appearance vocabulary.
    type Appearance<'a>: AppearanceSchema<'a>;
    /// Joint binding interpretations supported by this vocabulary.
    type Binding<'a>: BindingSchema<'a>;
    /// Complete geometry vocabulary, including supported streams and connectivity.
    type Geometry<'a>: GeometrySchema<'a>;
    /// Model-wide authoring metadata.
    type ModelMetadata<'a>: PropertiesPayload;
    /// Complete event-source definitions admitted by this vocabulary.
    type EventSource<'a>: EventSourceSchema;
    /// Independently authored scene systems.
    type System<'a>: SystemPayload;
    /// Complete surface payload, with no unsupported family variants.
    type Surface<'a>: SurfacePayload;
    /// Complete attached behavior payload.
    type Feature<'a>: FeaturePayload + FeatureSchema<'a>;
    /// Additional logical hierarchy behavior.
    type NodeProperties<'a>: PropertiesPayload + Default;
    /// Additional geometry behavior.
    type MeshProperties<'a>: PropertiesPayload + Default;
    /// Additional geometry placement behavior.
    type InstanceProperties<'a>: PropertiesPayload + Default;
    /// Additional draw-pass behavior.
    type DrawProperties: PropertiesPayload + Default;
    /// Additional playback selection and transition behavior.
    type ClipProperties<'a>: PropertiesPayload + Default;

    /// Returns the binding owned by shared geometry, when the vocabulary supports one.
    fn mesh_skin(controls: &Self::MeshProperties<'_>) -> Option<SkinId> {
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
impl<A: EventSourceSchema, B: EventSourceSchema> EventSourceSchema for Either<A, B> {
    fn clock(&self) -> TimeSource {
        match self {
            Self::Left(value) => value.clock(),
            Self::Right(value) => value.clock(),
        }
    }
}
impl<A: SurfacePayload, B: SurfacePayload> SurfacePayload for Either<A, B> {
    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Left(value) => value.validate(),
            Self::Right(value) => value.validate(),
        }
    }
}
impl<A: FeaturePayload, B: FeaturePayload> FeaturePayload for Either<A, B> {
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

    type Appearance<'a> = UniversalAppearance<'a>;
    type Binding<'a> = UniversalBinding<'a>;
    type Geometry<'a> = GeometryData<'a>;
    type DrawProperties = Option<DrawProperties>;
    type ModelMetadata<'a> = ModelMetadata<'a>;
    type EventSource<'a> = EventSource<'a>;
    type System<'a> = System<'a>;
    type Surface<'a> = MaterialSurface<'a>;
    type Feature<'a> = NodeFeature<'a>;
    type NodeProperties<'a> = Option<NodeProperties>;
    type MeshProperties<'a> = Option<MeshProperties>;
    type InstanceProperties<'a> = Option<InstanceProperties<'a>>;
    type ClipProperties<'a> = Option<ClipProperties>;
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

    /// Embeds the complete appearance vocabulary while preserving identities.
    fn appearance<'a>(value: Source::Appearance<'a>) -> Self::Appearance<'a>;
    /// Embeds the authored joint binding without changing its interpretation.
    fn binding<'a>(value: Source::Binding<'a>) -> Self::Binding<'a>;
    /// Embeds complete geometry without changing its logical values.
    fn geometry<'a>(value: Source::Geometry<'a>) -> Self::Geometry<'a>;
    /// Embeds model-wide metadata without changing their meaning.
    fn model_metadata<'a>(value: Source::ModelMetadata<'a>) -> Self::ModelMetadata<'a>;
    /// Embeds a complete event source without changing its identity or meaning.
    fn event_source<'a>(value: Source::EventSource<'a>) -> Self::EventSource<'a>;
    /// Embeds an independent scene system without changing its meaning.
    fn system<'a>(value: Source::System<'a>) -> Self::System<'a>;
    /// Embeds a surface without changing its meaning.
    fn surface<'a>(value: Source::Surface<'a>) -> Self::Surface<'a>;
    /// Embeds an attached feature without changing its meaning.
    fn feature<'a>(value: Source::Feature<'a>, values: &mut PropertyValues) -> Self::Feature<'a>;
    /// Embeds node properties without changing meaning.
    fn node<'a>(value: Source::NodeProperties<'a>) -> Self::NodeProperties<'a>;
    /// Embeds mesh properties without changing meaning.
    fn mesh<'a>(value: Source::MeshProperties<'a>) -> Self::MeshProperties<'a>;
    /// Embeds instance properties without changing meaning.
    fn instance<'a>(value: Source::InstanceProperties<'a>) -> Self::InstanceProperties<'a>;
    /// Embeds draw-pass behavior without changing meaning.
    fn draw(value: Source::DrawProperties) -> Self::DrawProperties;
    /// Embeds clip properties without changing meaning.
    fn clip<'a>(value: Source::ClipProperties<'a>) -> Self::ClipProperties<'a>;
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

    fn appearance<'a>(value: <P as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        value
    }
    fn binding<'a>(value: <P as Profile>::Binding<'a>) -> Self::Binding<'a> {
        value
    }
    fn geometry<'a>(value: <P as Profile>::Geometry<'a>) -> Self::Geometry<'a> {
        value
    }
    fn draw(value: <P as Profile>::DrawProperties) -> Self::DrawProperties {
        value
    }
    fn model_metadata<'a>(value: <P as Profile>::ModelMetadata<'a>) -> Self::ModelMetadata<'a> {
        value
    }
    fn event_source<'a>(value: P::EventSource<'a>) -> Self::EventSource<'a> {
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
    fn node<'a>(value: <P as Profile>::NodeProperties<'a>) -> Self::NodeProperties<'a> {
        value
    }
    fn mesh<'a>(value: <P as Profile>::MeshProperties<'a>) -> Self::MeshProperties<'a> {
        value
    }
    fn instance<'a>(value: <P as Profile>::InstanceProperties<'a>) -> Self::InstanceProperties<'a> {
        value
    }
    fn clip<'a>(value: <P as Profile>::ClipProperties<'a>) -> Self::ClipProperties<'a> {
        value
    }
}
impl NodeFeature<'_> {}

pub use m2::M2;

pub use mdlx::Mdlx;
pub use wmo::Wmo;

// Logical authoring features specific to each model family.

pub mod m2;

pub mod mdlx;
pub mod wmo;

mod narrowing;
pub use narrowing::Narrows;

impl<'a> FeatureVocabulary<'a> for Universal {
    type Light = Light;
    type Camera = Camera;
    type Attachment = Attachment<'a>;
    type Particle = ParticleEmitter<'a>;
    type Ribbon = RibbonEmitter;
    type GeometryAnimation = mdlx::GeometryAnimation;
    type CollisionShape = mdlx::Collision;
    type ParticleProgram = Box<mdlx::Popcorn<'a>>;
    type FaceActor = EffectResourceId;
    type LightProgram = wmo::PointProgram;

    type Particle2 = Unavailable;
    type PointLight = Unavailable;
    fn visit_links(value: &FeatureSet<'a, Self>, visitor: &mut dyn FnMut(Link)) {
        value.visit_feature_links(visitor);
    }
    fn feature<'view>(value: &'view FeatureSet<'a, Self>) -> FeatureView<'view, 'a> {
        value.profile_feature()
    }

    fn kind(value: &FeatureSet<'a, Self>) -> FeatureKind {
        value.profile_kind()
    }
}
