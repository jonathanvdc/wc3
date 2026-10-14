//! Native inline simulation sets rebuild from independent authored bodies and systems.
mod bodies;
mod cloth;
mod codec;
mod colliders;
mod identities;
mod joints;
pub(in crate::model::conversion::wem) use identities::PackedIdentities;
mod rig;
mod shapes;
mod soft;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::{
    physics::{BodyId, ClothId, Physics, RigId},
    SectionId,
};
use crate::model::semantic::{MaterialSlotId, MeshId, NodeId};
use crate::model::wem::{records, RawDocument, Ref};
use codec::{allocate, Scalar};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::model::conversion::wem) struct Decoded<'a> {
    pub physics: Physics<'a>,
    pub bodies: BTreeMap<u32, BodyId>,
    pub rigs: BTreeMap<u32, RigId>,
    pub cloths: BTreeMap<u32, ClothId>,
    pub soft: soft::ReadMaps,
}
pub(in crate::model::conversion::wem) struct Encoded {
    pub value: records::PhysicsSet<records::V9>,
    pub bodies: BTreeMap<BodyId, u32>,
    pub rigs: BTreeMap<RigId, u32>,
    pub cloths: BTreeMap<ClothId, u32>,
    pub soft: soft::WriteMaps,
}
pub(in crate::model::conversion::wem) fn read<'a, D: records::PhysicsSetDialect>(
    document: &'a RawDocument<'_>,
    source: &records::PhysicsSet<D>,
    nodes: &[NodeId],
    meshes: &[MeshId],
    sections: &[Vec<SectionId>],
    slots: &[MaterialSlotId],
) -> Result<Decoded<'a>, Error>
where
    D::Soft: Scalar<Ref<records::targets::SoftBody>>,
{
    let mut physics = Physics::default();
    let bodies = bodies::read(document, source.bodies, nodes, &mut physics)?;
    let joints = joints::read(document, source.joints, &mut physics, &bodies)?;
    let colliders = colliders::read(document, source.colliders, nodes, &mut physics, &bodies)?;
    let cloths = cloth::read(
        document,
        source.cloths,
        &mut physics,
        &colliders,
        nodes,
        meshes,
        sections,
    )?;
    let rigs = rig::read(
        document,
        source.rigs,
        &mut physics,
        &bodies,
        nodes,
        meshes,
        slots,
    )?;
    let soft =
        if let Some(reference) = Scalar::<Ref<records::targets::SoftBody>>::scalar(&source.soft) {
            soft::read(
                document,
                reference,
                &mut physics,
                &colliders,
                nodes,
                meshes,
                slots,
            )?
        } else {
            soft::ReadMaps::default()
        };
    validate_identities(
        bodies
            .keys()
            .chain(joints.keys())
            .chain(colliders.keys())
            .chain(cloths.keys())
            .chain(rigs.keys())
            .chain(soft.bodies.keys())
            .copied(),
    )?;
    physics.validate()?;
    Ok(Decoded {
        physics,
        bodies,
        rigs,
        cloths,
        soft,
    })
}
pub(in crate::model::conversion::wem) fn write_packed(
    document: &mut RawDocument<'_>,
    physics: &Physics<'_>,
    nodes: &BTreeMap<NodeId, u32>,
    meshes: &BTreeMap<MeshId, u32>,
    sections: &BTreeMap<SectionId, u32>,
    slots: &BTreeMap<MaterialSlotId, u32>,
    packed: &PackedIdentities,
) -> Result<Encoded, Error> {
    physics.validate()?;
    let mut next_id = 1u32;
    let reserved = packed.reserve(physics, &mut next_id)?;
    let bodies = physics
        .bodies()
        .map(|(id, _)| Ok((id, allocate(&mut next_id)?)))
        .collect::<Result<BTreeMap<BodyId, u32>, Error>>()?;
    let native_bodies = bodies::write(document, physics, &bodies, nodes)?;
    let (native_joints, _) = joints::write(document, physics, &bodies, &mut next_id)?;
    let (native_colliders, colliders) =
        colliders::write(document, physics, &bodies, nodes, &mut next_id)?;
    let (native_cloths, cloths) = cloth::write(
        document,
        physics,
        &cloth::WriteContext {
            colliders: &colliders,
            nodes,
            meshes,
            sections,
            reserved: &reserved.cloths,
        },
        &mut next_id,
    )?;
    let (native_rigs, rigs) = rig::write(
        document,
        physics,
        &bodies,
        nodes,
        meshes,
        slots,
        &mut next_id,
    )?;
    let soft = soft::write(
        document,
        physics,
        soft::Inputs {
            colliders: &colliders,
            nodes,
            meshes,
            slots,
            reserved_bodies: &reserved.soft_bodies,
            reserved_effects: &reserved.soft_effects,
        },
        &mut next_id,
    )?;
    Ok(Encoded {
        value: records::PhysicsSet {
            bodies: native_bodies,
            joints: native_joints,
            colliders: native_colliders,
            cloths: native_cloths,
            rigs: native_rigs,
            next_id,
            soft: soft.reference,
        },
        bodies,
        rigs,
        cloths,
        soft,
    })
}

fn validate_identities(ids: impl IntoIterator<Item = u32>) -> Result<(), Error> {
    let mut identities = BTreeSet::new();
    for id in ids {
        if id == 0 || !identities.insert(id) {
            return Err(Error::new(
                "wem.physics.identity",
                "physics object identities must be nonzero and unique across all object categories",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_identities;

    #[test]
    fn producer_identity_namespace_is_shared_across_object_categories() {
        assert!(validate_identities([1, 9, 77]).is_ok());
        assert!(validate_identities([1, 9, 1]).is_err());
        assert!(validate_identities([0]).is_err());
    }
}
