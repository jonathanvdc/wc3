//! Resolves source table links into editable node identities and back.
use super::{metadata, wem, wr, BTreeMap, Error};
fn remap(value: u32, map: &BTreeMap<u32, u32>, path: &str) -> Result<u32, Error> {
    if value == u32::MAX {
        return Ok(value);
    }
    map.get(&value)
        .copied()
        .ok_or_else(|| Error::new(path, "emitter link references a missing emitter"))
}
pub(super) fn attach(
    doc: &wem::RawDocument<'_>,
    nodes: &mut [wr::Node<wr::V19>],
) -> Result<(), Error> {
    let mut particles = BTreeMap::new();
    let mut ribbons = BTreeMap::new();
    for (index, node) in nodes.iter().enumerate() {
        let values = metadata::read(doc, &node.native)?;
        if let Some(value) = values.get("m3.particle.source_index") {
            particles.insert(*value as u32, index as u32);
        }
        if let Some(value) = values.get("m3.ribbon.source_index") {
            ribbons.insert(*value as u32, index as u32);
        }
    }
    for node in nodes {
        if let wr::NodePayload::Sc2ParticleEmitter(p) = &mut node.payload {
            p.collision_spawn = remap(p.collision_spawn, &particles, "particle.collision_spawn")?;
            p.trail_link = remap(p.trail_link, &particles, "particle.trail_link")?;
            p.ribbon_link = remap(p.ribbon_link, &ribbons, "particle.ribbon_link")?;
        }
    }
    Ok(())
}
pub(super) fn export(
    particles: &mut [(wr::Sc2ParticleEmitterPayload, BTreeMap<String, i64>, u32)],
    ribbons: &[(wr::Sc2RibbonEmitterPayload, BTreeMap<String, i64>, u32)],
) -> Result<(), Error> {
    let mut pmap = BTreeMap::new();
    let mut rmap = BTreeMap::new();
    let mut index = 0;
    for (p, values, _) in particles.iter() {
        if p.copy_of == u32::MAX {
            let id = values
                .get("m3.effect.wem_index")
                .copied()
                .ok_or_else(|| Error::new("particle.node", "missing node identity"))?;
            pmap.insert(id as u32, index);
            index += 1;
        }
    }
    for (index, (_, values, _)) in ribbons.iter().enumerate() {
        let id = values
            .get("m3.effect.wem_index")
            .copied()
            .ok_or_else(|| Error::new("ribbon.node", "missing node identity"))?;
        rmap.insert(id as u32, index as u32);
    }
    for (p, _, _) in particles {
        p.collision_spawn = remap(p.collision_spawn, &pmap, "particle.collision_spawn")?;
        p.trail_link = remap(p.trail_link, &pmap, "particle.trail_link")?;
        p.ribbon_link = remap(p.ribbon_link, &rmap, "particle.ribbon_link")?;
    }
    Ok(())
}
