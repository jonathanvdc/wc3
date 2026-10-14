//! Temporary decoded controls attached to their logical owners before publication.
use crate::model::semantic::profiles::mdlx::{
    Bone, Geometry, ModelControls, Sequence, TextureControls,
};
use crate::model::semantic::{ClipId, MeshId, NodeId, TextureId};
#[derive(Default)]
pub(super) struct ImportState<'a> {
    pub controls: ModelControls<'a>,
    pub textures: Vec<(TextureId, TextureControls<'a>)>,
    pub sequences: Vec<(ClipId, Sequence)>,
    pub geometry: Vec<(MeshId, Geometry)>,
    pub bones: Vec<(NodeId, Bone)>,
    pub node_controls: Vec<(NodeId, u32)>,
    pub bind_poses: Vec<(NodeId, [f32; 12])>,
    pub gliders: Vec<MeshId>,
}
