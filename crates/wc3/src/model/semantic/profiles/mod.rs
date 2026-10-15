//! Compile-time vocabulary of logical payloads; unrelated to native file revisions.
use crate::model::semantic::OwnerId;
use crate::model::semantic::{
    AppearanceSchema, BindingSchema, Bounds, Cardinality, Error, FeatureKind, GeometrySchema, Link,
    MaterialSurface, Model, ModelControls, Multiple, NodeFeature, PbrSurface, Resources, SkinId,
    SystemPayload, ValidateLinks,
};
use crate::model::semantic::{
    ClipControls, DrawControls, GeometryData, InstanceControls, Material, MeshControls,
    NodeControls, System, TextureControls, UniversalAppearance, UniversalBinding,
};
use crate::model::semantic::{
    PropertyIdentityKey, PropertyKeyRef, PropertyPayload, PropertyTarget,
};
use std::any::TypeId;
use std::fmt::Debug;
use std::marker::PhantomData;

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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
    /// Structural animation-property vocabulary accepted by this profile.
    type Property: PropertyPayload;
    /// Model cardinality represented by a native asset in this vocabulary.
    type Models: Cardinality;
    /// Metadata shared by the models in an asset.
    type AssetMetadata<'a>: ControlsPayload + Default;
    /// Complete material-assignment and appearance vocabulary.
    type Appearance<'a>: AppearanceSchema<'a>;
    /// Joint binding interpretations supported by this vocabulary.
    type Binding<'a>: BindingSchema<'a>;
    /// Complete geometry vocabulary, including supported streams and connectivity.
    type Geometry<'a>: GeometrySchema<'a>;
    /// Genuinely model-wide authoring controls.
    type Model<'a>: ControlsPayload + Default;
    /// Independently authored scene systems.
    type System<'a>: SystemPayload;
    /// Complete surface payload, with no unsupported family variants.
    type Surface<'a>: SurfacePayload;
    /// Complete attached behavior payload.
    type Feature<'a>: FeaturePayload;
    /// Additional logical hierarchy behavior.
    type Node<'a>: ControlsPayload + Default;
    /// Additional texture behavior, which may borrow its authoring data.
    type Texture<'a>: ControlsPayload + Default;
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
    /// Checks asset-wide authoring domains after model and standard references are valid.
    fn validate_asset<'a>(
        metadata: &Self::AssetMetadata<'a>,
        models: &[Model<'a, Self>],
        library: &Resources<'a, Self>,
    ) -> Result<(), Error> {
        let _ = (metadata, models, library);
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
/// Union of two vocabularies, preserving the original payloads unchanged.
#[derive(Clone, Debug, Default)]
pub struct Union<A, B>(PhantomData<(A, B)>);
impl<A: Profile, B: Profile> Profile for Union<A, B> {
    type Property = Either<A::Property, B::Property>;
    type Models = Multiple;
    type AssetMetadata<'a> = Either<A::AssetMetadata<'a>, B::AssetMetadata<'a>>;
    type Appearance<'a> = Either<A::Appearance<'a>, B::Appearance<'a>>;
    type Binding<'a> = Either<A::Binding<'a>, B::Binding<'a>>;
    type Geometry<'a> = Either<A::Geometry<'a>, B::Geometry<'a>>;
    type Draw = Either<A::Draw, B::Draw>;
    type Model<'a> = Either<A::Model<'a>, B::Model<'a>>;
    type System<'a> = Either<A::System<'a>, B::System<'a>>;
    type Surface<'a> = Either<A::Surface<'a>, B::Surface<'a>>;
    type Feature<'a> = Either<A::Feature<'a>, B::Feature<'a>>;
    type Node<'a> = Either<A::Node<'a>, B::Node<'a>>;
    type Texture<'a> = Either<A::Texture<'a>, B::Texture<'a>>;
    type Mesh<'a> = Either<A::Mesh<'a>, B::Mesh<'a>>;
    type Instance<'a> = Either<A::Instance<'a>, B::Instance<'a>>;
    type Clip<'a> = Either<A::Clip<'a>, B::Clip<'a>>;
    type Controller<'a> = Either<A::Controller<'a>, B::Controller<'a>>;
    fn mesh_skin(controls: &Self::Mesh<'_>) -> Option<SkinId> {
        match controls {
            Either::Left(value) => A::mesh_skin(value),
            Either::Right(value) => B::mesh_skin(value),
        }
    }
}
/// All built-in logical feature families. Custom families can be added with `Union`.
#[derive(Clone, Debug, Default)]
pub struct Universal;
impl Profile for Universal {
    type Property = UniversalProperty;
    type Models = Multiple;
    type AssetMetadata<'a> = NoControls;
    type Appearance<'a> = UniversalAppearance<'a>;
    type Binding<'a> = UniversalBinding<'a>;
    type Geometry<'a> = GeometryData<'a>;
    type Draw = Option<DrawControls>;
    type Model<'a> = Option<ModelControls<'a>>;
    type System<'a> = System<'a>;
    type Surface<'a> = MaterialSurface<'a>;
    type Feature<'a> = NodeFeature<'a>;
    type Node<'a> = Option<NodeControls<'a>>;
    type Texture<'a> = Option<TextureControls<'a>>;
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
    /// Embeds structural property selectors without changing their identity.
    fn property(value: Source::Property) -> Self::Property;
    /// Embeds asset-wide metadata without changing its meaning.
    fn asset_metadata<'a>(value: Source::AssetMetadata<'a>) -> Self::AssetMetadata<'a>;
    /// Embeds the complete appearance vocabulary while preserving identities.
    fn appearance<'a>(value: Source::Appearance<'a>) -> Self::Appearance<'a>;
    /// Embeds the authored joint binding without changing its interpretation.
    fn binding<'a>(value: Source::Binding<'a>) -> Self::Binding<'a>;
    /// Embeds complete geometry without changing its logical values.
    fn geometry<'a>(value: Source::Geometry<'a>) -> Self::Geometry<'a>;
    /// Embeds model-wide controls without changing their meaning.
    fn model<'a>(value: Source::Model<'a>) -> Self::Model<'a>;
    /// Embeds an independent scene system without changing its meaning.
    fn system<'a>(value: Source::System<'a>) -> Self::System<'a>;
    /// Embeds a surface without changing its meaning.
    fn surface<'a>(value: Source::Surface<'a>) -> Self::Surface<'a>;
    /// Embeds an attached feature without changing its meaning.
    fn feature<'a>(value: Source::Feature<'a>) -> Self::Feature<'a>;
    /// Embeds node controls without changing meaning.
    fn node<'a>(value: Source::Node<'a>) -> Self::Node<'a>;
    /// Embeds texture controls without changing meaning.
    fn texture<'a>(value: Source::Texture<'a>) -> Self::Texture<'a>;
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
    fn property(value: P::Property) -> Self::Property {
        value
    }
    fn asset_metadata<'a>(value: <P as Profile>::AssetMetadata<'a>) -> Self::AssetMetadata<'a> {
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
    fn model<'a>(value: <P as Profile>::Model<'a>) -> Self::Model<'a> {
        value
    }
    fn system<'a>(value: <P as Profile>::System<'a>) -> Self::System<'a> {
        value
    }
    fn surface<'a>(value: P::Surface<'a>) -> Self::Surface<'a> {
        value
    }
    fn feature<'a>(value: P::Feature<'a>) -> Self::Feature<'a> {
        value
    }
    fn node<'a>(value: <P as Profile>::Node<'a>) -> Self::Node<'a> {
        value
    }
    fn texture<'a>(value: <P as Profile>::Texture<'a>) -> Self::Texture<'a> {
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
impl ValidateLinks for MaterialSurface<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::D4(value) => value.visit_links(visitor),
            Self::D3(value) => value.visit_links(visitor),
            Self::Generic(value) => value.visit_links(visitor),
            Self::Mdlx(value) => value.visit_links(visitor),
            Self::M2(value) => value.visit_links(visitor),
            Self::M3(value) => value.visit_links(visitor),
            Self::Wmo(value) => value.visit_links(visitor),
        }
    }
}
impl ValidateLinks for PbrSurface {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for input in &self.inputs {
            input.texture.visit_links(visitor);
        }
    }
}
impl SurfacePayload for PbrSurface {
    fn validate(&self) -> Result<(), Error> {
        Material {
            name: "".into(),
            surface: MaterialSurface::Generic(Box::new(self.clone())),
        }
        .validate_portable()
    }
}
impl SurfacePayload for MaterialSurface<'_> {
    fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        match self {
            Self::Mdlx(v) => v.supports_property(key),
            Self::M2(v) => v.supports_property(key),
            Self::M3(v) => v.supports_property(key),

            Self::D3(v) => v.supports_property(key),
            Self::D4(v) => v.supports_property(key),
            Self::Wmo(v) => v.supports_property(key),
            Self::Generic(_) => false,
        }
    }

    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::D4(value) => value.validate(),
            Self::D3(value) => value.validate(),
            Self::Generic(value) => value.validate(),

            _ => Ok(()),
        }
    }
}
impl FeaturePayload for NodeFeature<'_> {
    fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        match self {
            Self::Mdlx(v) => v.supports_property(key),
            Self::M2(v) => v.supports_property(key),
            Self::M3(v) => v.supports_property(key),

            Self::Wmo(v) => v.supports_property(key),
            Self::Generic(_) => false,
        }
    }

    fn kind(&self) -> FeatureKind {
        self.kind()
    }
}

impl<A: Profile, B: Profile> Extends<A> for Union<A, B> {
    fn property(value: A::Property) -> Self::Property {
        Either::Left(value)
    }
    fn asset_metadata<'a>(value: <A as Profile>::AssetMetadata<'a>) -> Self::AssetMetadata<'a> {
        Either::Left(value)
    }
    fn appearance<'a>(value: <A as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        Either::Left(value)
    }
    fn binding<'a>(value: <A as Profile>::Binding<'a>) -> Self::Binding<'a> {
        Either::Left(value)
    }
    fn geometry<'a>(value: <A as Profile>::Geometry<'a>) -> Self::Geometry<'a> {
        Either::Left(value)
    }
    fn draw(value: <A as Profile>::Draw) -> Self::Draw {
        Either::Left(value)
    }
    fn model<'a>(value: <A as Profile>::Model<'a>) -> Self::Model<'a> {
        Either::Left(value)
    }
    fn system<'a>(value: <A as Profile>::System<'a>) -> Self::System<'a> {
        Either::Left(value)
    }
    fn surface<'a>(value: A::Surface<'a>) -> Self::Surface<'a> {
        Either::Left(value)
    }
    fn feature<'a>(value: A::Feature<'a>) -> Self::Feature<'a> {
        Either::Left(value)
    }
    fn node<'a>(value: <A as Profile>::Node<'a>) -> Self::Node<'a> {
        Either::Left(value)
    }
    fn texture<'a>(value: <A as Profile>::Texture<'a>) -> Self::Texture<'a> {
        Either::Left(value)
    }
    fn mesh<'a>(value: <A as Profile>::Mesh<'a>) -> Self::Mesh<'a> {
        Either::Left(value)
    }
    fn instance<'a>(value: <A as Profile>::Instance<'a>) -> Self::Instance<'a> {
        Either::Left(value)
    }
    fn clip<'a>(value: <A as Profile>::Clip<'a>) -> Self::Clip<'a> {
        Either::Left(value)
    }
    fn controller<'a>(value: <A as Profile>::Controller<'a>) -> Self::Controller<'a> {
        Either::Left(value)
    }
}
impl<A: Profile, B: Profile> Extends<Union<A, B>> for Universal
where
    Universal: Extends<A> + Extends<B>,
{
    fn property(value: <Union<A, B> as Profile>::Property) -> Self::Property {
        match value {
            Either::Left(v) => <Self as Extends<A>>::property(v),
            Either::Right(v) => <Self as Extends<B>>::property(v),
        }
    }
    fn asset_metadata<'a>(
        value: <Union<A, B> as Profile>::AssetMetadata<'a>,
    ) -> Self::AssetMetadata<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::asset_metadata(value),
            Either::Right(value) => <Self as Extends<B>>::asset_metadata(value),
        }
    }
    fn appearance<'a>(value: <Union<A, B> as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::appearance(value),
            Either::Right(value) => <Self as Extends<B>>::appearance(value),
        }
    }
    fn binding<'a>(value: <Union<A, B> as Profile>::Binding<'a>) -> Self::Binding<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::binding(value),
            Either::Right(value) => <Self as Extends<B>>::binding(value),
        }
    }
    fn geometry<'a>(value: <Union<A, B> as Profile>::Geometry<'a>) -> Self::Geometry<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::geometry(value),
            Either::Right(value) => <Self as Extends<B>>::geometry(value),
        }
    }

    fn draw(value: <Union<A, B> as Profile>::Draw) -> Self::Draw {
        match value {
            Either::Left(value) => <Self as Extends<A>>::draw(value),
            Either::Right(value) => <Self as Extends<B>>::draw(value),
        }
    }

    fn model<'a>(value: <Union<A, B> as Profile>::Model<'a>) -> Self::Model<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::model(value),
            Either::Right(value) => <Self as Extends<B>>::model(value),
        }
    }
    fn system<'a>(value: <Union<A, B> as Profile>::System<'a>) -> Self::System<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::system(value),
            Either::Right(value) => <Self as Extends<B>>::system(value),
        }
    }

    fn surface<'a>(value: <Union<A, B> as Profile>::Surface<'a>) -> Self::Surface<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::surface(value),
            Either::Right(value) => <Self as Extends<B>>::surface(value),
        }
    }
    fn feature<'a>(value: <Union<A, B> as Profile>::Feature<'a>) -> Self::Feature<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::feature(value),
            Either::Right(value) => <Self as Extends<B>>::feature(value),
        }
    }
    fn node<'a>(value: <Union<A, B> as Profile>::Node<'a>) -> Self::Node<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::node(value),
            Either::Right(value) => <Self as Extends<B>>::node(value),
        }
    }
    fn texture<'a>(value: <Union<A, B> as Profile>::Texture<'a>) -> Self::Texture<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::texture(value),
            Either::Right(value) => <Self as Extends<B>>::texture(value),
        }
    }
    fn mesh<'a>(value: <Union<A, B> as Profile>::Mesh<'a>) -> Self::Mesh<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::mesh(value),
            Either::Right(value) => <Self as Extends<B>>::mesh(value),
        }
    }
    fn instance<'a>(value: <Union<A, B> as Profile>::Instance<'a>) -> Self::Instance<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::instance(value),
            Either::Right(value) => <Self as Extends<B>>::instance(value),
        }
    }
    fn clip<'a>(value: <Union<A, B> as Profile>::Clip<'a>) -> Self::Clip<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::clip(value),
            Either::Right(value) => <Self as Extends<B>>::clip(value),
        }
    }
    fn controller<'a>(value: <Union<A, B> as Profile>::Controller<'a>) -> Self::Controller<'a> {
        match value {
            Either::Left(value) => <Self as Extends<A>>::controller(value),
            Either::Right(value) => <Self as Extends<B>>::controller(value),
        }
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
            Self::Mdlx(v) => v.identity(),
            Self::M2(v) => v.identity(),
            Self::M3(v) => v.identity(),
        }
    }
    fn from_identity(key: &PropertyIdentityKey) -> Option<Self> {
        mdlx::properties::Target::from_identity(key)
            .map(Self::Mdlx)
            .or_else(|| m2::properties::Target::from_identity(key).map(Self::M2))
            .or_else(|| m3::properties::Target::from_identity(key).map(Self::M3))
    }
    fn value_type(&self) -> Option<TypeId> {
        match self {
            Self::Mdlx(v) => v.value_type(),
            Self::M2(v) => v.value_type(),
            Self::M3(v) => v.value_type(),
        }
    }
    fn accepts_owner(&self, owner: OwnerId) -> bool {
        match self {
            Self::Mdlx(v) => v.accepts_owner(owner),
            Self::M2(v) => v.accepts_owner(owner),
            Self::M3(v) => v.accepts_owner(owner),
        }
    }
}
