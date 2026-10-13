//! Draw-section geometry, face flags, and collision ownership.
use super::*;
#[derive(Clone, Debug)]
pub(super) struct MeshSource {
    pub(super) group: GroupKey,
    pub(super) triangle_flags: Vec<u32>,
    pub(super) triangle_materials: Vec<u32>,
    pub(super) group_fields: Vec<(String, i64, String)>,
}

mod export;
mod import;
mod merge;
pub(super) use export::export;
pub(super) use import::import;
pub(super) use merge::merge;
