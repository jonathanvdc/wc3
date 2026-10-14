//! Output references are assigned from actual graph objects before any dependent payload is emitted.
use super::super::physics::PackedIdentities;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::{
    animation::{AnyDeclaration, PhysicsControl, Target},
    SectionId,
};
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{FeatureId, MeshId, Model, NodeId};
use std::collections::BTreeMap;

pub(super) struct Indices {
    pub nodes: BTreeMap<NodeId, u32>,
    pub meshes: BTreeMap<MeshId, u32>,
    pub features: BTreeMap<FeatureId, u32>,
    pub feature_nodes: BTreeMap<FeatureId, NodeId>,
    pub sections: BTreeMap<(MeshId, SectionId), u32>,
    pub section_indices: BTreeMap<SectionId, u32>,
    pub first_sections: BTreeMap<SectionId, u32>,
}
fn index(value: usize) -> Result<u32, Error> {
    u32::try_from(value).map_err(|_| Error::new("wem.references", "object count exceeds u32"))
}
pub(super) fn collect(model: &Model<'_, Wem>) -> Result<Indices, Error> {
    let nodes = model
        .nodes()
        .enumerate()
        .map(|(offset, (id, _))| Ok((id, index(offset)?)))
        .collect::<Result<BTreeMap<_, _>, Error>>()?;
    let meshes = model
        .meshes()
        .enumerate()
        .map(|(offset, (id, _))| Ok((id, index(offset)?)))
        .collect::<Result<BTreeMap<_, _>, Error>>()?;
    let mut features = BTreeMap::new();
    let mut feature_nodes = BTreeMap::new();
    for (id, value) in model.features() {
        features.insert(
            id,
            *nodes
                .get(&value.node)
                .ok_or_else(|| Error::new("wem.actor.node", "behavior owner is absent"))?,
        );
        feature_nodes.insert(id, value.node);
    }
    let mut sections = BTreeMap::new();
    let mut section_indices = BTreeMap::new();
    let mut first_sections = BTreeMap::new();
    for (mesh_index, (mesh, value)) in model.meshes().enumerate() {
        for (section_index, (id, _)) in value.geometry.sections().enumerate() {
            let offset = index(section_index)?;
            sections.insert((mesh, id), offset);
            if section_indices
                .insert(id, offset)
                .is_some_and(|old| old != offset)
            {
                return Err(Error::new(
                    "wem.sections",
                    "shared polygon-group identity has conflicting positions in separate meshes",
                ));
            }
            if mesh_index == 0 {
                first_sections.insert(id, offset);
            }
        }
    }
    Ok(Indices {
        nodes,
        meshes,
        features,
        feature_nodes,
        sections,
        section_indices,
        first_sections,
    })
}
fn packed_target<T>(target: &Target<T>, result: &mut PackedIdentities) {
    if let Target::Physics { control, .. } = target {
        match control {
            PhysicsControl::ClothDriver { cloth, .. } => {
                result.cloths.insert(*cloth);
            }
            PhysicsControl::SoftEffect { body, effect } => {
                result.soft_bodies.insert(*body);
                result.soft_effects.insert((*body, *effect));
            }
            PhysicsControl::SoftLump { body, .. } => {
                result.soft_bodies.insert(*body);
            }
            _ => {}
        }
    }
}
pub(super) fn packed(model: &Model<'_, Wem>) -> PackedIdentities {
    let mut result = PackedIdentities::default();
    for declaration in model.controls().channels.declarations() {
        match declaration {
            AnyDeclaration::Scalar(_, value) => packed_target(&value.target, &mut result),
            AnyDeclaration::Vector2(_, value) => packed_target(&value.target, &mut result),
            AnyDeclaration::Vector3(_, value) => packed_target(&value.target, &mut result),
            AnyDeclaration::Vector4(_, value) => packed_target(&value.target, &mut result),
            AnyDeclaration::Rotation(_, value) => packed_target(&value.target, &mut result),
            AnyDeclaration::Boolean(_, value) => packed_target(&value.target, &mut result),
            AnyDeclaration::Unsigned16(_, value) => packed_target(&value.target, &mut result),
            AnyDeclaration::Unsigned(_, value) => packed_target(&value.target, &mut result),
            AnyDeclaration::Signed(_, value) => packed_target(&value.target, &mut result),
            AnyDeclaration::Bytes4(_, value) => packed_target(&value.target, &mut result),
            AnyDeclaration::Unsigned4(_, value) => packed_target(&value.target, &mut result),
            AnyDeclaration::Texture(_, value) => packed_target(&value.target, &mut result),
        }
    }
    result
}
