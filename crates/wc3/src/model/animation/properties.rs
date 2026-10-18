//! Structural destinations and their transient, profile-independent validation identities.
use crate::model::profiles::{Extends, Narrows, Profile, Universal};
use crate::model::{
    Error, FeatureId, Link, MaterialId, MeshId, ModelId, NodeId, Quaternion, SystemId, Texture,
    ValidateLinks,
};
use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

/// An object whose properties may be animated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerId {
    /// Independently authored scene system.
    System(SystemId),
    /// One attached logical feature.
    Feature(FeatureId),
    /// Model-level property.
    Model(ModelId),
    /// Hierarchy node.
    Node(NodeId),
    /// Material property.
    Material(MaterialId),
    /// Geometry property.
    Mesh(MeshId),
}
impl ValidateLinks for OwnerId {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Model(id) => id.visit_links(visitor),
            Self::System(id) => id.visit_links(visitor),
            Self::Feature(id) => id.visit_links(visitor),
            Self::Node(id) => id.visit_links(visitor),
            Self::Material(id) => id.visit_links(visitor),
            Self::Mesh(id) => id.visit_links(visitor),
        }
    }
}
trait KeyValue: Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn equal(&self, other: &dyn KeyValue) -> bool;
    fn hash_value(&self, state: &mut dyn Hasher);
    fn owned(&self) -> Arc<dyn KeyValue>;
}
impl<T: Clone + Debug + Eq + Hash + Send + Sync + 'static> KeyValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn equal(&self, other: &dyn KeyValue) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .is_some_and(|v| self == v)
    }
    fn hash_value(&self, mut state: &mut dyn Hasher) {
        TypeId::of::<T>().hash(&mut state);
        self.hash(&mut state);
    }
    fn owned(&self) -> Arc<dyn KeyValue> {
        Arc::new(self.clone())
    }
}
/// A borrowed structural selector identity used for equality and relationship checking.
/// Containing enum wrappers delegate to their underlying selector; no labels enter identity.
pub struct PropertyKeyRef<'a>(&'a dyn KeyValue);
impl<'a> PropertyKeyRef<'a> {
    pub(crate) fn new<T: PropertyPayload>(value: &'a T) -> Self {
        Self(value)
    }

    /// Copies the selector into a transient validation relationship.
    pub fn to_owned(&self) -> PropertyIdentityKey {
        PropertyIdentityKey(self.0.owned())
    }
}
impl Debug for PropertyKeyRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}
impl PartialEq for PropertyKeyRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.equal(other.0)
    }
}
impl Eq for PropertyKeyRef<'_> {}
impl Hash for PropertyKeyRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash_value(state);
    }
}
/// Owned structural selector identity used only by relationship validation.
/// Active model targets retain their concrete profile enum, not this erased representation.
#[derive(Clone, Debug)]
pub struct PropertyIdentityKey(Arc<dyn KeyValue>);
impl PropertyIdentityKey {
    /// Resolves the exact selector type without interpreting a display label.
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.0.as_any().downcast_ref()
    }
}
impl PartialEq for PropertyIdentityKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.equal(other.0.as_ref())
    }
}
impl Eq for PropertyIdentityKey {}
impl Hash for PropertyIdentityKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash_value(state);
    }
}
/// A profile's structural property vocabulary.
///
/// Custom profiles use an enum or another equality-comparable structural value.
/// The default identity is that value itself. Containing vocabularies delegate
/// identity to their contained selector so widening preserves relationships.
pub trait PropertyPayload: Clone + Debug + Eq + Hash + Send + Sync + 'static {
    /// Canonical selector identity, independent of the containing profile wrapper.
    fn identity(&self) -> PropertyKeyRef<'_> {
        PropertyKeyRef(self)
    }
    /// Projects a canonical selector into this vocabulary when it belongs here.
    fn from_identity(key: &PropertyIdentityKey) -> Option<Self> {
        key.downcast_ref::<Self>().cloned()
    }
    /// Exact value type when it is intrinsic to the selector.
    fn value_type(&self) -> Option<TypeId> {
        None
    }
    /// Whether this selector accepts the supplied owner category.
    fn accepts_owner(&self, _owner: OwnerId) -> bool {
        true
    }
}
/// Profile-independent relationship address. It is transient validation data.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PropertyIdentity {
    /// Local translation.
    Translation(NodeId),
    /// Local rotation.
    Rotation(NodeId),
    /// Local scaling.
    Scale(NodeId),
    /// Hierarchy visibility.
    Visibility(NodeId),
    /// Common material opacity.
    MaterialOpacity(MaterialId),
    /// Common material sampling slot.
    MaterialTexture(MaterialId, u32),
    /// Common geometry tint.
    MeshColor(MeshId),
    /// Common geometry opacity.
    MeshOpacity(MeshId),
    /// Actual profile selector and owner.
    Profile {
        /// Actual logical owner.
        owner: OwnerId,
        /// Canonical structural selector.
        key: PropertyIdentityKey,
    },
}
#[derive(PartialEq, Eq, Hash)]
enum TargetRef<'a> {
    Translation(NodeId),
    Rotation(NodeId),
    Scale(NodeId),
    Visibility(NodeId),
    MaterialOpacity(MaterialId),
    MaterialTexture(MaterialId, u32),
    MeshColor(MeshId),
    MeshOpacity(MeshId),
    Profile(OwnerId, PropertyKeyRef<'a>),
}
/// A semantic destination whose profile selects its structural property enum.
#[derive(Clone, Debug)]
pub enum PropertyTarget<P: Profile = Universal> {
    /// Local translation.
    Translation(NodeId),
    /// Local rotation.
    Rotation(NodeId),
    /// Local scaling.
    Scale(NodeId),
    /// Hierarchy visibility.
    Visibility(NodeId),
    /// Common material opacity.
    MaterialOpacity(MaterialId),
    /// Common material sampling slot.
    MaterialTexture(MaterialId, u32),
    /// Common geometry tint.
    MeshColor(MeshId),
    /// Common geometry opacity.
    MeshOpacity(MeshId),
    /// Profile-owned structural destination.
    Profile {
        /// Actual logical owner.
        owner: OwnerId,
        /// Selector in the profile's associated vocabulary.
        key: P::Property,
    },
}
impl<P: Profile> PropertyTarget<P> {
    fn identity_ref(&self) -> TargetRef<'_> {
        match self {
            Self::Translation(id) => TargetRef::Translation(*id),
            Self::Rotation(id) => TargetRef::Rotation(*id),
            Self::Scale(id) => TargetRef::Scale(*id),
            Self::Visibility(id) => TargetRef::Visibility(*id),
            Self::MaterialOpacity(id) => TargetRef::MaterialOpacity(*id),
            Self::MaterialTexture(id, slot) => TargetRef::MaterialTexture(*id, *slot),
            Self::MeshColor(id) => TargetRef::MeshColor(*id),
            Self::MeshOpacity(id) => TargetRef::MeshOpacity(*id),
            Self::Profile { owner, key } => TargetRef::Profile(*owner, key.identity()),
        }
    }
    /// Canonical address used when checking references from different profile payloads.
    pub fn identity(&self) -> PropertyIdentity {
        match self {
            Self::Translation(id) => PropertyIdentity::Translation(*id),
            Self::Rotation(id) => PropertyIdentity::Rotation(*id),
            Self::Scale(id) => PropertyIdentity::Scale(*id),
            Self::Visibility(id) => PropertyIdentity::Visibility(*id),
            Self::MaterialOpacity(id) => PropertyIdentity::MaterialOpacity(*id),
            Self::MaterialTexture(id, slot) => PropertyIdentity::MaterialTexture(*id, *slot),
            Self::MeshColor(id) => PropertyIdentity::MeshColor(*id),
            Self::MeshOpacity(id) => PropertyIdentity::MeshOpacity(*id),
            Self::Profile { owner, key } => PropertyIdentity::Profile {
                owner: *owner,
                key: key.identity().to_owned(),
            },
        }
    }
    /// Resolves a relationship address into this profile's permitted vocabulary.
    pub fn from_identity(identity: &PropertyIdentity) -> Result<Self, Error> {
        Ok(match identity {
            PropertyIdentity::Translation(id) => Self::Translation(*id),
            PropertyIdentity::Rotation(id) => Self::Rotation(*id),
            PropertyIdentity::Scale(id) => Self::Scale(*id),
            PropertyIdentity::Visibility(id) => Self::Visibility(*id),
            PropertyIdentity::MaterialOpacity(id) => Self::MaterialOpacity(*id),
            PropertyIdentity::MaterialTexture(id, slot) => Self::MaterialTexture(*id, *slot),
            PropertyIdentity::MeshColor(id) => Self::MeshColor(*id),
            PropertyIdentity::MeshOpacity(id) => Self::MeshOpacity(*id),
            PropertyIdentity::Profile { owner, key } => Self::Profile {
                owner: *owner,
                key: P::Property::from_identity(key).ok_or_else(|| {
                    Error::new(
                        "animation.target",
                        "property belongs to another profile vocabulary",
                    )
                })?,
            },
        })
    }
    pub(crate) fn map_profile<Q: Profile>(
        self,
        map: impl FnOnce(P::Property) -> Result<Q::Property, Error>,
    ) -> Result<PropertyTarget<Q>, Error> {
        Ok(match self {
            Self::Translation(id) => PropertyTarget::Translation(id),
            Self::Rotation(id) => PropertyTarget::Rotation(id),
            Self::Scale(id) => PropertyTarget::Scale(id),
            Self::Visibility(id) => PropertyTarget::Visibility(id),
            Self::MaterialOpacity(id) => PropertyTarget::MaterialOpacity(id),
            Self::MaterialTexture(id, slot) => PropertyTarget::MaterialTexture(id, slot),
            Self::MeshColor(id) => PropertyTarget::MeshColor(id),
            Self::MeshOpacity(id) => PropertyTarget::MeshOpacity(id),
            Self::Profile { owner, key } => PropertyTarget::Profile {
                owner,
                key: map(key)?,
            },
        })
    }
    /// Embeds a structural target into a containing profile vocabulary.
    pub fn widen<Q: Extends<P>>(self) -> PropertyTarget<Q> {
        self.map_profile(|key| Ok(Q::property(key)))
            .expect("infallible property embedding")
    }
    /// Projects a structural target into a supported subset profile.
    pub fn try_narrow<Q: Narrows<P>>(self) -> Result<PropertyTarget<Q>, Error> {
        self.map_profile(Q::property)
    }
    /// Exact value domain selected by this destination.
    pub fn value_type(&self) -> Option<TypeId> {
        Some(match self {
            Self::Translation(_) | Self::Scale(_) | Self::MeshColor(_) => TypeId::of::<[f32; 3]>(),
            Self::Rotation(_) => TypeId::of::<Quaternion>(),
            Self::Visibility(_) => TypeId::of::<bool>(),
            Self::MaterialOpacity(_) | Self::MeshOpacity(_) => TypeId::of::<f32>(),
            Self::MaterialTexture(..) => TypeId::of::<Texture>(),
            Self::Profile { key, .. } => return key.value_type(),
        })
    }
}
// Identity is canonical across containing profiles.
impl<P: Profile, Q: Profile> PartialEq<PropertyTarget<Q>> for PropertyTarget<P> {
    fn eq(&self, other: &PropertyTarget<Q>) -> bool {
        self.identity_ref() == other.identity_ref()
    }
}
impl<P: Profile> Eq for PropertyTarget<P> {}
impl<P: Profile> Hash for PropertyTarget<P> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity_ref().hash(state);
    }
}
/// A property handle with a checked value domain and profile vocabulary.
pub struct PropertyBinding<T, P: Profile = Universal> {
    target: PropertyTarget<P>,
    marker: PhantomData<fn() -> T>,
}
impl<T, P: Profile> Clone for PropertyBinding<T, P> {
    fn clone(&self) -> Self {
        Self::new(self.target.clone())
    }
}
impl<T, P: Profile> Debug for PropertyBinding<T, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.target.fmt(f)
    }
}
impl<T, P: Profile, Q: Profile> PartialEq<PropertyBinding<T, Q>> for PropertyBinding<T, P> {
    fn eq(&self, other: &PropertyBinding<T, Q>) -> bool {
        self.target == other.target
    }
}
impl<T, P: Profile> Eq for PropertyBinding<T, P> {}
impl<T, P: Profile> PropertyBinding<T, P> {
    /// Binds a descriptor whose profile, owner, and value types determine its target.
    ///
    /// A descriptor cannot change the value domain:
    /// ```compile_fail
    /// use wc3::model::{FeatureId, PropertyBinding};
    /// use wc3::model::profiles::{Mdlx, mdlx::properties::light};
    /// fn invalid(owner: FeatureId) -> PropertyBinding<bool, Mdlx> {
    ///     PropertyBinding::profile(owner, light::INTENSITY)
    /// }
    /// ```
    /// It cannot select another owner domain:
    /// ```compile_fail
    /// use wc3::model::NodeId;
    /// use wc3::model::profiles::mdlx::properties::light;
    /// fn invalid(node: NodeId) { let _ = light::INTENSITY.bind(node); }
    /// ```
    pub fn profile<D: PropertyDescriptor<Profile = P, Value = T>>(
        owner: D::Owner,
        descriptor: D,
    ) -> Self {
        Self::new(PropertyTarget::Profile {
            owner: owner.into(),
            key: descriptor.key(),
        })
    }
    /// Structural destination.
    pub fn target(&self) -> &PropertyTarget<P> {
        &self.target
    }
    pub(crate) fn new(target: PropertyTarget<P>) -> Self {
        Self {
            target,
            marker: PhantomData,
        }
    }
    /// Embeds the destination in a containing vocabulary, preserving its identity.
    pub fn widen<Q: Extends<P>>(self) -> PropertyBinding<T, Q> {
        PropertyBinding::new(
            self.target
                .map_profile::<Q>(|key| Ok(Q::property(key)))
                .expect("infallible property embedding"),
        )
    }
    /// Projects a destination without changing its value domain.
    pub fn try_narrow<Q: Narrows<P>>(self) -> Result<PropertyBinding<T, Q>, Error>
    where
        T: 'static,
    {
        let source_type = TypeId::of::<T>();
        let target = self.target.map_profile::<Q>(Q::property)?;
        if target.value_type() != Some(source_type) {
            return Err(Error::new(
                "animation.binding",
                "narrowing changes the binding source value domain",
            ));
        }
        Ok(PropertyBinding {
            target,
            marker: PhantomData,
        })
    }
}
impl NodeId {
    /// Local translation property in the selected animation vocabulary.
    pub fn translation<P: Profile>(self) -> PropertyBinding<[f32; 3], P> {
        PropertyBinding::new(PropertyTarget::Translation(self))
    }
    /// Local quaternion property.
    pub fn rotation<P: Profile>(self) -> PropertyBinding<Quaternion, P> {
        PropertyBinding::new(PropertyTarget::Rotation(self))
    }
    /// Local scale property.
    pub fn scale<P: Profile>(self) -> PropertyBinding<[f32; 3], P> {
        PropertyBinding::new(PropertyTarget::Scale(self))
    }
    /// Visibility gate.
    pub fn visibility<P: Profile>(self) -> PropertyBinding<bool, P> {
        PropertyBinding::new(PropertyTarget::Visibility(self))
    }
}
impl MeshId {
    /// Geometry-owned RGB tint, independent of its assigned surface.
    pub fn color<P: Profile>(self) -> PropertyBinding<[f32; 3], P> {
        PropertyBinding::new(PropertyTarget::MeshColor(self))
    }
    /// Geometry-owned opacity multiplier.
    pub fn opacity<P: Profile>(self) -> PropertyBinding<f32, P> {
        PropertyBinding::new(PropertyTarget::MeshOpacity(self))
    }
}
impl MaterialId {
    /// Common material opacity.
    pub fn opacity<P: Profile>(self) -> PropertyBinding<f32, P> {
        PropertyBinding::new(PropertyTarget::MaterialOpacity(self))
    }
    /// Common texture selection.
    pub fn texture<P: Profile>(self, slot: u32) -> PropertyBinding<Texture, P> {
        PropertyBinding::new(PropertyTarget::MaterialTexture(self, slot))
    }
}
/// A typed structural selector supplied by a profile.
pub trait PropertyDescriptor {
    /// Vocabulary containing this definition.
    type Profile: Profile;
    /// Owner identity accepted by this definition.
    type Owner: Into<OwnerId>;
    /// Animated value domain.
    type Value;
    /// Structural selector; display names do not participate in identity.
    fn key(self) -> <Self::Profile as Profile>::Property;
}
/// A profile's typed property definition.
#[derive(Debug)]
pub struct PropertyKey<O, T, P: Profile> {
    key: P::Property,
    marker: PhantomData<fn(O) -> T>,
}
impl<O, T, P: Profile> Clone for PropertyKey<O, T, P> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            marker: PhantomData,
        }
    }
}
impl<O: Into<OwnerId>, T, P: Profile> PropertyDescriptor for PropertyKey<O, T, P> {
    type Profile = P;
    type Owner = O;
    type Value = T;
    fn key(self) -> P::Property {
        self.key
    }
}
impl<O: Into<OwnerId>, T, P: Profile> PropertyKey<O, T, P> {
    pub(crate) const fn new(key: P::Property) -> Self {
        Self {
            key,
            marker: PhantomData,
        }
    }
    /// Binds this structural selector to an owner.
    pub fn bind(self, owner: O) -> PropertyBinding<T, P> {
        PropertyBinding::profile(owner, self)
    }
}
macro_rules! owner_identity {
    ($($id:ty => $kind:ident),* $(,)?) => {$ (
        impl From<$id> for OwnerId { fn from(id: $id) -> Self { Self::$kind(id) } }
    )*};
}
owner_identity!(ModelId => Model, NodeId => Node, MaterialId => Material, MeshId => Mesh,
    FeatureId => Feature, SystemId => System);
