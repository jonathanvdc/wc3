//! Typed object handles distinguish logical graph and shared-library identity domains.
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
/// Model identity marker, stable across edits and profile mappings.
pub enum ModelObject {}
/// Model handle identifying the owner of model-level properties.
pub type ModelId = Id<ModelObject>;
/// Node identity marker.
pub enum NodeObject {}
/// Geometry identity marker.
pub enum MeshObject {}
/// Material identity marker.
pub enum MaterialObject {}
/// Texture identity marker.
pub enum TextureObject {}
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
/// Texture handle.
pub type TextureId = Id<TextureObject>;
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

/// Shared texture-coordinate transform identity marker.
pub enum TextureTransformObject {}
/// Graph-scoped shared texture-coordinate transform handle.
pub type TextureTransformId = Id<TextureTransformObject>;
/// Independently authored scene system identity marker.
pub enum SystemObject {}
/// Graph-scoped scene system handle.
pub type SystemId = Id<SystemObject>;

/// Named material-slot identity marker.
pub enum MaterialSlotObject {}
/// Scoped identity of a slot in an appearance vocabulary.
pub type MaterialSlotId = Id<MaterialSlotObject>;

/// Draw binding identity marker.
pub enum DrawObject {}
/// Stable identity of a draw binding, resolved within its owning instance.
pub type DrawId = Id<DrawObject>;
