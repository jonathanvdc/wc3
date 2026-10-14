use super::super::codec::{lookup, read_error, read_words};
use super::{leaf, settings, ReadMaps};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::physics::{
    soft::{EffectKind, LumpKind, SoftBody, SoftEffect, SoftLump},
    ClothColliderId, Physics,
};
use crate::model::semantic::MaterialSlotId;
use crate::model::semantic::{MeshId, NodeId};
use crate::model::wem::{records, RawDocument, Ref};
use std::collections::BTreeMap;
pub(in crate::model::conversion::wem::physics) fn read<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::SoftBody>,
    physics: &mut Physics<'a>,
    colliders: &BTreeMap<u32, ClothColliderId>,
    nodes: &[NodeId],
    meshes: &[MeshId],
    slots: &[MaterialSlotId],
) -> Result<ReadMaps, Error> {
    let mut maps = ReadMaps::default();
    let context = settings::ReadContext {
        document,
        nodes,
        meshes,
        slots,
    };
    for source in document
        .resolve::<records::SoftBody>(reference.cast())
        .map_err(read_error)?
        .iter()
    {
        let source = source.map_err(read_error)?;
        if maps.bodies.contains_key(&source.id) {
            return Err(Error::new("wem.soft.id", "duplicate soft body identity"));
        }
        let body_id = physics.reserve_soft_body();
        maps.bodies.insert(source.id, body_id);
        let mut body = SoftBody::new(leaf::string(document, source.name)?);
        body.meshes = leaf::mesh_ids(document, source.meshes, meshes)?;
        body.detail = source.detail;
        body.copy_all = source.copy_all.get();
        body.chains_as_they_are = source.chains_as_they_are.get();
        body.inside = leaf::parts(document, source.inside, meshes, slots)?;
        body.blend = source.blend;
        body.influences = source.influences;
        body.equal_shares = source.equal_shares.get();
        body.drips = leaf::optional(meshes, source.drips, "soft drip geometry")?;
        for id in read_words(document, source.colliders)?.iter() {
            body.colliders.push(*colliders.get(&id).ok_or_else(|| {
                Error::new(
                    "wem.soft.collider",
                    "collider belongs to no declared simulation",
                )
            })?);
        }
        let riders: Vec<_> = leaf::parts(document, source.riders, meshes, slots)?
            .iter()
            .map(|part| body.add_rider(part))
            .collect();
        let mut effects = BTreeMap::new();
        for effect in document
            .resolve::<records::SoftEffect>(source.effects.cast())
            .map_err(read_error)?
            .iter()
        {
            let effect = effect.map_err(read_error)?;
            if effect.id == 0 || effects.contains_key(&effect.id) {
                return Err(Error::new(
                    "wem.soft.effect.id",
                    "effect IDs must be nonzero and unique within the body",
                ));
            }
            let id = body.add_effect(SoftEffect {
                name: leaf::string(document, effect.name)?.into(),
                kind: EffectKind::from_raw(effect.kind.raw()),
                enabled: effect.enabled.get(),
                amount: effect.amount,
                everywhere: effect.everywhere.get(),
                drives_jiggle: effect.drives_jiggle.get(),
                seed: effect.seed,
                place: leaf::point(&effect.place, meshes)?,
                settings: settings::read(&context, &effect)?,
            });
            effects.insert(effect.id, id);
            maps.effects.insert((source.id, effect.id), id);
        }
        for (index, lump) in document
            .resolve::<records::SoftLump>(source.lumps.cast())
            .map_err(read_error)?
            .iter()
            .enumerate()
        {
            let lump = lump.map_err(read_error)?;
            let effect = if lump.effect == 0 {
                None
            } else {
                Some(*effects.get(&lump.effect).ok_or_else(|| {
                    Error::new("wem.soft.lump.effect", "effect belongs to another body")
                })?)
            };
            let rider = if lump.rider == 0 {
                None
            } else {
                Some(lookup(&riders, u32::from(lump.rider) - 1, "soft rider")?)
            };
            let id = body.add_lump(SoftLump {
                kind: LumpKind::from_raw(lump.kind.raw()),
                anchor: lump.anchor,
                rest: lump.rest,
                depth: lump.depth,
                clear: lump.clear,
                carrier: leaf::node(lump.carrier, nodes)?,
                node: leaf::node(lump.node, nodes)?,
                effect,
                rider,
                swings: lump.swings.get(),
                rigid: lump.rigid.get(),
            });
            let ordinal = u32::try_from(index + 1).map_err(read_error)?;
            maps.lumps.insert((source.id, ordinal), id);
        }
        physics.set_soft_body(body_id, body)?;
    }
    Ok(maps)
}
