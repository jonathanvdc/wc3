//! Temporary decoded controls attached to their logical owners before publication.
use crate::model::profiles::mdlx::{Bone, Geometry, ModelMetadata, Sequence};
use crate::model::{ClipId, MeshId, NodeId};
#[derive(Default)]
pub(super) struct ImportState<'a> {
    pub metadata: ModelMetadata<'a>,
    pub sequences: Vec<(ClipId, Sequence)>,
    pub geometry: Vec<(MeshId, Geometry)>,
    pub bones: Vec<(NodeId, Bone)>,
    pub bind_poses: Vec<(NodeId, [f32; 12])>,
    pub gliders: Vec<MeshId>,
}
