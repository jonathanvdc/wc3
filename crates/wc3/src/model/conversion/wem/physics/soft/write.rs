use super::super::codec::{allocate, append, mapped, read_error, words};
use super::{leaf, settings, WriteMaps};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::physics::{
    ClothColliderId, Physics, SoftBodyId, SoftEffectId,
};
use crate::model::semantic::MaterialSlotId;
use crate::model::semantic::{MeshId, NodeId};
use crate::model::wem::{records, Bool, RawDocument};
use std::collections::{btree_map::Entry, BTreeMap};
pub(in crate::model::conversion::wem::physics) struct Inputs<'a> {
    pub colliders: &'a BTreeMap<ClothColliderId, u32>,
    pub nodes: &'a BTreeMap<NodeId, u32>,
    pub meshes: &'a BTreeMap<MeshId, u32>,
    pub slots: &'a BTreeMap<MaterialSlotId, u32>,
    pub reserved_bodies: &'a BTreeMap<SoftBodyId, u32>,
    pub reserved_effects: &'a BTreeMap<(SoftBodyId, SoftEffectId), u32>,
}
pub(in crate::model::conversion::wem::physics) fn write(
    document: &mut RawDocument<'_>,
    physics: &Physics<'_>,
    inputs: Inputs<'_>,
    next: &mut u32,
) -> Result<WriteMaps, Error> {
    let Inputs {
        colliders,
        nodes,
        meshes,
        slots,
        reserved_bodies,
        reserved_effects,
    } = inputs;
    let mut maps = WriteMaps::default();
    let mut rows = Vec::new();
    let mut context = settings::WriteContext {
        document,
        nodes,
        meshes,
        slots,
    };
    for (body_id, body) in physics.soft_bodies() {
        let native_id = match reserved_bodies.get(&body_id) {
            Some(id) => *id,
            None => allocate(next)?,
        };
        maps.bodies.insert(body_id, native_id);
        let mut riders = BTreeMap::new();
        let mut rider_rows = Vec::new();
        for (index, (id, value)) in body.riders().enumerate() {
            riders.insert(id, u32::try_from(index + 1).map_err(read_error)?);
            rider_rows.push(leaf::part(value, meshes, slots)?);
        }
        let effect_ids =
            assign_effects(body_id, body.effects().map(|(id, _)| id), reserved_effects)?;
        let mut effect_rows = Vec::new();
        for (id, value) in body.effects() {
            let native_id = effect_ids[&id];
            maps.effects.insert((body_id, id), native_id);
            let mut row = records::SoftEffect {
                id: native_id,
                kind: records::SoftEffectKind::from_raw(value.kind.raw()),
                name: leaf::write_string(context.document, &value.name)?,
                enabled: Bool::from(value.enabled),
                amount: value.amount,
                everywhere: Bool::from(value.everywhere),
                drives_jiggle: Bool::from(value.drives_jiggle),
                seed: value.seed,
                place: leaf::write_point(value.place.as_ref(), meshes)?,
                ..Default::default()
            };
            settings::write(&mut context, &value.settings, &mut row)?;
            effect_rows.push(row);
        }
        let mut lump_rows = Vec::new();
        for (index, (id, value)) in body.lumps().enumerate() {
            maps.lumps
                .insert((body_id, id), u32::try_from(index + 1).map_err(read_error)?);
            lump_rows.push(records::SoftLump {
                kind: records::SoftLumpKind::from_raw(value.kind.raw()),
                anchor: value.anchor,
                rest: value.rest,
                depth: value.depth,
                clear: value.clear,
                carrier: leaf::node_index(value.carrier, nodes)?,
                node: leaf::node_index(value.node, nodes)?,
                effect: value
                    .effect
                    .map(|id| mapped(&effect_ids, id, "soft effect"))
                    .transpose()?
                    .unwrap_or(0),
                rider: u16::try_from(
                    value
                        .rider
                        .map(|id| mapped(&riders, id, "soft rider"))
                        .transpose()?
                        .unwrap_or(0),
                )
                .map_err(read_error)?,
                swings: Bool::from(value.swings),
                rigid: Bool::from(value.rigid),
            });
        }
        let mesh_rows = body
            .meshes
            .iter()
            .map(|id| mapped(meshes, id, "soft mesh"))
            .collect::<Result<Vec<_>, _>>()?;
        let collider_rows = body
            .colliders
            .iter()
            .map(|id| mapped(colliders, *id, "soft collider"))
            .collect::<Result<Vec<_>, _>>()?;
        let inside = body
            .inside
            .iter()
            .map(|v| leaf::part(&v, meshes, slots))
            .collect::<Result<Vec<_>, _>>()?;
        rows.push(records::SoftBody {
            id: native_id,
            name: leaf::write_string(context.document, &body.name)?,
            meshes: words(context.document, mesh_rows)?,
            detail: body.detail,
            copy_all: Bool::from(body.copy_all),
            chains_as_they_are: Bool::from(body.chains_as_they_are),
            inside: append(context.document, &inside)?.cast(),
            riders: append(context.document, &rider_rows)?.cast(),
            blend: body.blend,
            influences: body.influences,
            equal_shares: Bool::from(body.equal_shares),
            lumps: append(context.document, &lump_rows)?.cast(),
            effects: append(context.document, &effect_rows)?.cast(),
            colliders: words(context.document, collider_rows)?,
            drips: body
                .drips
                .map(|id| mapped(meshes, id, "soft drip geometry"))
                .transpose()?
                .unwrap_or(u32::MAX),
            next_effect: u32::try_from(effect_rows.len() + 1).map_err(read_error)?,
        });
    }
    maps.reference = append(context.document, &rows)?.cast();
    Ok(maps)
}

fn assign_effects(
    body: SoftBodyId,
    effects: impl IntoIterator<Item = SoftEffectId>,
    reserved: &BTreeMap<(SoftBodyId, SoftEffectId), u32>,
) -> Result<BTreeMap<SoftEffectId, u32>, Error> {
    let mut ids = reserved
        .iter()
        .filter(|((owner, _), _)| *owner == body)
        .map(|((_, effect), native)| (*effect, *native))
        .collect::<BTreeMap<_, _>>();
    let mut next = ids
        .values()
        .copied()
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| read_error("effect identity capacity exhausted"))?;
    for id in effects {
        if let Entry::Vacant(entry) = ids.entry(id) {
            entry.insert(allocate(&mut next)?);
        }
    }
    Ok(ids)
}
#[cfg(test)]
mod tests {
    use super::assign_effects;
    use crate::model::semantic::Scope;
    use std::collections::BTreeMap;
    #[test]
    fn keyed_effect_identity_is_independent_of_authored_row_order() {
        let scope = Scope::default();
        let body = scope.allocate();
        let first = scope.allocate();
        let second = scope.allocate();
        let rows = [first, second];
        let ids = assign_effects(body, rows, &BTreeMap::from([((body, second), 1)])).unwrap();
        assert_eq!(rows.map(|id| ids[&id]), [2, 1]);
    }
}
