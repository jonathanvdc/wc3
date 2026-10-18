//! Typed object handles distinguish object families and owning models.
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT_OBJECT: AtomicU64 = AtomicU64::new(1);
static NEXT_GRAPH: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub(crate) struct Scope {
    identity: u64,
}
impl Default for Scope {
    fn default() -> Self {
        Self {
            identity: NEXT_GRAPH.fetch_add(1, Ordering::Relaxed),
        }
    }
}
impl Scope {
    pub(crate) fn allocate<K>(&self) -> Id<K> {
        Id {
            scope: self.identity,
            key: NEXT_OBJECT.fetch_add(1, Ordering::Relaxed),
            marker: PhantomData,
        }
    }
    pub(crate) fn contains<K>(&self, id: Id<K>) -> bool {
        id.scope == self.identity
    }
}
/// Stable scoped identity. Different object families and unrelated domains cannot be interchanged.
pub struct Id<K> {
    scope: u64,
    key: u64,
    marker: PhantomData<fn() -> K>,
}
impl<K> Copy for Id<K> {}
impl<K> Clone for Id<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K> PartialEq for Id<K> {
    fn eq(&self, other: &Self) -> bool {
        self.scope == other.scope && self.key == other.key
    }
}
impl<K> Eq for Id<K> {}
impl<K> PartialOrd for Id<K> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<K> Ord for Id<K> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.scope, self.key).cmp(&(other.scope, other.key))
    }
}
impl<K> Hash for Id<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.scope.hash(state);
        self.key.hash(state);
    }
}
impl<K> Debug for Id<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("Id")
            .field(&self.scope)
            .field(&self.key)
            .finish()
    }
}
/// Node identity marker.
pub enum NodeObject {}
/// Geometry identity marker.
pub enum MeshObject {}
/// Material identity marker.
pub enum MaterialObject {}
/// Clip identity marker.
pub enum ClipObject {}
/// Mesh placement identity marker.
pub enum InstanceObject {}
/// Skin identity marker.
pub enum SkinObject {}
/// Shared animation clock identity marker.
pub enum ClockObject {}
/// Programme event-occurrence identity marker.
pub enum EventObject {}
/// Programme event-occurrence handle.
pub type EventId = Id<EventObject>;
/// Animation controller identity marker.
pub enum ControllerObject {}
/// Animation controller handle.
pub type ControllerId = Id<ControllerObject>;
/// Node handle.
pub type NodeId = Id<NodeObject>;
/// Geometry handle.
pub type MeshId = Id<MeshObject>;
/// Material handle.
pub type MaterialId = Id<MaterialObject>;
/// Clip handle.
pub type ClipId = Id<ClipObject>;
/// Mesh placement handle.
pub type InstanceId = Id<InstanceObject>;
/// Skin handle.
pub type SkinId = Id<SkinObject>;
/// Shared clock handle.
pub type ClockId = Id<ClockObject>;
/// Material look identity marker.
pub enum LookObject {}
/// Material look handle.
pub type LookId = Id<LookObject>;
/// Attached feature identity marker.
pub enum FeatureObject {}
/// Identity of an attached effect or behavior, independent of its hierarchy anchor.
pub type FeatureId = Id<FeatureObject>;

/// Independently authored scene system identity marker.
pub enum SystemObject {}
/// Graph-scoped scene system handle.
pub type SystemId = Id<SystemObject>;

/// Draw binding identity marker.
pub enum DrawObject {}
/// Stable identity of a draw binding, resolved within its owning instance.
pub type DrawId = Id<DrawObject>;

use zerovec::ule::AsULE;
impl<K> AsULE for Id<K> {
    type ULE = <[u64; 2] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        [self.scope, self.key].to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let [scope, key] = <[u64; 2]>::from_unaligned(value);
        Self {
            scope,
            key,
            marker: PhantomData,
        }
    }
}

/// External image dependency identity marker.
pub enum ImageResourceObject {}
/// Handle to a model-owned image dependency declaration.
///
/// Model dependencies cannot be used as image dependencies:
/// ```compile_fail
/// use wc3::model::{Model, ModelMetadata, ResourceLocator, Texture};
/// use wc3::model::profiles::Universal;
/// let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
/// let model = draft.add_model_resource(ResourceLocator::FileDataId(1));
/// let texture = Texture::new(model);
/// ```
pub type ImageResourceId = Id<ImageResourceObject>;

/// External model dependency identity marker.
pub enum ModelResourceObject {}
/// Handle to a model-owned model dependency declaration.
pub type ModelResourceId = Id<ModelResourceObject>;

/// External effect dependency identity marker.
pub enum EffectResourceObject {}
/// Handle to a model-owned effect dependency declaration.
pub type EffectResourceId = Id<EffectResourceObject>;

/// External physics dependency identity marker.
pub enum PhysicsResourceObject {}
/// Handle to a model-owned physics dependency declaration.
pub type PhysicsResourceId = Id<PhysicsResourceObject>;

/// External boneoverride dependency identity marker.
pub enum BoneOverrideResourceObject {}
/// Handle to a model-owned boneoverride dependency declaration.
pub type BoneOverrideResourceId = Id<BoneOverrideResourceObject>;

/// Reusable animation identity marker.
pub enum AnimationObject {}
/// Handle to a reusable model-owned curve and event program.
pub type AnimationId = Id<AnimationObject>;
