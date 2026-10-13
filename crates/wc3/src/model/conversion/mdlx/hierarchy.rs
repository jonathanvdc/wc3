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
pub(super) fn common_flags(flags: u32) -> u32 {
    ((flags & 0x7f) << 1)
        | if flags & (1 << 19) != 0 {
            wr::NodeFlags::MODEL_SPACE.0
        } else {
            0
        }
}
pub(super) fn native_flags(flags: u32, original: u32) -> u32 {
    let mask = 0x7f | (1 << 19);
    (original & !mask)
        | ((flags >> 1) & 0x7f)
        | if flags & wr::NodeFlags::MODEL_SPACE.0 != 0 {
            1 << 19
        } else {
            0
        }
}
