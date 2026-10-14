//! Direct authored soft deformation mappings resolve actual mesh, hierarchy, and simulation handles.
mod leaf;
mod read;
mod settings;
mod write;
use crate::model::semantic::profiles::wem::physics::soft::{SoftBodyId, SoftEffectId, SoftLumpId};
use crate::model::wem::{records::targets, Ref};
pub(super) use read::read;
use std::collections::BTreeMap;
pub(super) use write::{write, Inputs};
#[derive(Default)]
pub(in crate::model::conversion::wem) struct ReadMaps {
    pub bodies: BTreeMap<u32, SoftBodyId>,
    pub effects: BTreeMap<(u32, u32), SoftEffectId>,
    pub lumps: BTreeMap<(u32, u32), SoftLumpId>,
}
#[derive(Default)]
pub(in crate::model::conversion::wem) struct WriteMaps {
    pub reference: Ref<targets::SoftBody>,
    pub bodies: BTreeMap<SoftBodyId, u32>,
    pub effects: BTreeMap<(SoftBodyId, SoftEffectId), u32>,
    pub lumps: BTreeMap<(SoftBodyId, SoftLumpId), u32>,
}
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
