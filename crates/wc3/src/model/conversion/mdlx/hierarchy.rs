//! Stable native identities and transform hierarchy updates.
use super::*;
pub(super) fn patch_node<F: native::scene::NodeFlagInterpretation>(
    node: &mut native::Node<'static, F>,
    updates: &BTreeMap<u32, native::Node<'static>>,
) {
    if let Some(update) = updates.get(&node.object_id) {
        *node = update.clone().cast_flags();
    }
}

pub(super) fn patch_hierarchy<D: ModelDialect>(
    model: &mut native::Model<'static, D>,
    updates: &BTreeMap<u32, native::Node<'static>>,
) {
    for chunk in &mut model.chunks {
        match chunk {
            ModelChunk::Bones(chunk) => {
                for record in &mut chunk.records {
                    patch_node(&mut record.node, updates);
                }
            }
            ModelChunk::Helpers(chunk) => {
                for node in &mut chunk.records {
                    patch_node(node, updates);
                }
            }
            ModelChunk::Attachments(chunk) => {
                for record in &mut chunk.records {
                    patch_node(&mut record.node, updates);
                }
            }
            ModelChunk::Lights(chunk) => {
                for record in &mut chunk.records {
                    patch_node(&mut record.node, updates);
                }
            }
            ModelChunk::EventObjects(chunk) => {
                for record in &mut chunk.records {
                    patch_node(&mut record.node, updates);
                }
            }
            ModelChunk::CollisionShapes(chunk) => {
                for record in &mut chunk.records {
                    patch_node(&mut record.node, updates);
                }
            }
            ModelChunk::ParticleEmitters(chunk) => {
                for record in &mut chunk.records {
                    patch_node(&mut record.node, updates);
                }
            }
            ModelChunk::ParticleEmitters2(chunk) => {
                for record in &mut chunk.records {
                    patch_node(&mut record.node, updates);
                }
            }
            ModelChunk::RibbonEmitters(chunk) => {
                for record in &mut chunk.records {
                    patch_node(&mut record.node, updates);
                }
            }
            ModelChunk::PopcornEmitters(chunk) => {
                for record in &mut chunk.records {
                    patch_node(&mut record.node, updates);
                }
            }
            _ => {}
        }
    }
}

pub(super) fn native_ids(
    nodes: &[wr::Node<wr::V19>],
    raw: &RawDocument<'_>,
) -> Result<Vec<u32>, Error> {
    let mut identities = Vec::new();
    let mut reserved = Vec::new();
    for node in nodes {
        let values = metadata(raw, node.native.clone())?;
        let identity = values
            .get("mdlx.object_id")
            .map(|(value, _)| {
                u32::try_from(*value).map_err(|error| failure("nodes.object_id", error))
            })
            .transpose()?;
        if let Some(identity) = identity {
            if reserved.contains(&identity) {
                return Err(Error::new("nodes.object_id", "duplicate node identity"));
            }
            reserved.push(identity);
        }
        identities.push(identity);
    }
    let mut next = 0u32;
    let mut output = Vec::new();
    for identity in identities {
        if let Some(identity) = identity {
            output.push(identity);
        } else {
            while reserved.contains(&next) {
                next = next
                    .checked_add(1)
                    .ok_or_else(|| Error::new("nodes.object_id", "identity space exhausted"))?;
            }
            output.push(next);
            reserved.push(next);
            next = next
                .checked_add(1)
                .ok_or_else(|| Error::new("nodes.object_id", "identity space exhausted"))?;
        }
    }
    Ok(output)
}
pub(super) fn common_flags(flags: native::scene::NodeFlags) -> wr::NodeFlags {
    let mut out = wr::NodeFlags::default();
    out.set_dont_inherit_translation(flags.dont_inherit_translation());
    out.set_dont_inherit_rotation(flags.dont_inherit_rotation());
    out.set_dont_inherit_scale(flags.dont_inherit_scaling());
    out.set_billboarded(flags.billboarded());
    out.set_billboard_lock_x(flags.billboard_lock_x());
    out.set_billboard_lock_y(flags.billboard_lock_y());
    out.set_billboard_lock_z(flags.billboard_lock_z());
    out.set_model_space(flags.model_space());
    out
}
pub(super) fn native_flags(flags: wr::NodeFlags, original: u32) -> native::scene::NodeFlags {
    let mut out = native::scene::NodeFlags(original);
    out.set_dont_inherit_translation(flags.dont_inherit_translation());
    out.set_dont_inherit_rotation(flags.dont_inherit_rotation());
    out.set_dont_inherit_scaling(flags.dont_inherit_scale());
    out.set_billboarded(flags.billboarded());
    out.set_billboard_lock_x(flags.billboard_lock_x());
    out.set_billboard_lock_y(flags.billboard_lock_y());
    out.set_billboard_lock_z(flags.billboard_lock_z());
    out.set_model_space(flags.model_space());
    out
}
