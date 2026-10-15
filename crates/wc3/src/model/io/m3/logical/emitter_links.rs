//! Active scene identities used while decoding emitter relationships.
use super::hierarchy::Skeleton;
use crate::model::{FeatureId, MaterialId, MeshId};
pub(super) struct Links<'a, 'data> {
    pub skeleton: &'a Skeleton<'data>,
    pub materials: &'a [MaterialId],
    pub meshes: &'a [MeshId],
    pub ribbons: &'a [FeatureId],
    pub projectors: &'a [FeatureId],
}
