//! Packed animation addresses reserve their constrained identity space before unconstrained objects.
use super::codec::allocate;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::physics::{ClothId, Physics, SoftBodyId, SoftEffectId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(in crate::model::conversion::wem) struct PackedIdentities {
    pub cloths: BTreeSet<ClothId>,
    pub soft_bodies: BTreeSet<SoftBodyId>,
    pub soft_effects: BTreeSet<(SoftBodyId, SoftEffectId)>,
}
pub(super) struct Reserved {
    pub cloths: BTreeMap<ClothId, u32>,
    pub soft_bodies: BTreeMap<SoftBodyId, u32>,
    pub soft_effects: BTreeMap<(SoftBodyId, SoftEffectId), u32>,
}
impl PackedIdentities {
    pub(super) fn reserve(&self, physics: &Physics<'_>, next: &mut u32) -> Result<Reserved, Error> {
        let mut cloths = BTreeMap::new();
        let mut soft_bodies = BTreeMap::new();
        let mut soft_effects = BTreeMap::new();
        for id in &self.cloths {
            if physics.cloth(*id).is_none() {
                return Err(missing());
            }
            cloths.insert(*id, packed(next)?);
        }
        let bodies = self
            .soft_bodies
            .iter()
            .copied()
            .chain(self.soft_effects.iter().map(|(body, _)| *body))
            .collect::<BTreeSet<_>>();
        for id in bodies {
            if physics.soft_body(id).is_none() {
                return Err(missing());
            }
            soft_bodies.insert(id, packed(next)?);
            let mut effect_next = 1;
            for (_, effect) in self.soft_effects.iter().filter(|(body, _)| *body == id) {
                if physics.soft_body(id).unwrap().effect(*effect).is_none() {
                    return Err(missing());
                }
                soft_effects.insert((id, *effect), packed(&mut effect_next)?);
            }
        }
        Ok(Reserved {
            cloths,
            soft_bodies,
            soft_effects,
        })
    }
}
fn packed(next: &mut u32) -> Result<u32, Error> {
    let value = allocate(next)?;
    u16::try_from(value).map_err(|_| {
        Error::new(
            "wem.physics.identity",
            "packed animation destinations exceed their 16-bit identity space",
        )
    })?;
    Ok(value)
}
fn missing() -> Error {
    Error::new(
        "wem.physics.identity",
        "packed destination belongs to another simulation system",
    )
}
