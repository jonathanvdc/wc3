//! Native skeleton transforms and explicit inverse bind matrices.
use super::effects::metadata::restore;
use super::hierarchy_metadata;
use super::{append_string, native, nr, Context, Error};
use crate::model::wem::{records as wr, semantic as sem};
use native::Envelope;
use std::collections::BTreeSet;
pub(super) fn encode<E: Envelope>(
    model: &sem::Model<'_>,
    document: &mut native::Document<'static, E>,
    context: &mut Context,
) -> Result<(Vec<nr::Bone<nr::V1>>, Vec<nr::InitialReference>), Error> {
    let mut bones = Vec::new();
    let modes = super::animation::modes(model, context)?;
    let mut initial = Vec::new();
    let (map, skipped) = mapping_details(model)?;
    for (index, node) in model.nodes.iter().enumerate() {
        if skipped[index] {
            continue;
        }
        if node.pivot != [0.; 3] {
            context.unsupported(
                format!("nodes[{index}].pivot"),
                "M3 hierarchy has no independent pivot field",
            )?;
        }
        let unrepresented = node.flags.bits()
            & !(wr::NodeFlags::DONT_INHERIT_TRANSLATION.bits()
                | wr::NodeFlags::DONT_INHERIT_ROTATION.bits()
                | wr::NodeFlags::DONT_INHERIT_SCALE.bits()
                | wr::NodeFlags::BILLBOARDED.bits());
        if unrepresented != 0 {
            context.unsupported(
                format!("nodes[{index}].flags"),
                "native bone flags cannot express this hierarchy state",
            )?;
        }
        let mut values = hierarchy_metadata::values(model, index)?;
        super::animation::reconcile_controls(&mut values, &modes);
        bones.push(nr::Bone::<nr::V1> {
            id: values.get("m3.bone.id").copied().unwrap_or(-1) as i32,
            name: append_string(document, &node.name)?,
            parent_index: node
                .parent
                .map(|p| u16::try_from(map[p as usize]))
                .transpose()
                .map_err(|_| Error::new("bone.parent", "bone parent exceeds native capacity"))?
                .unwrap_or(u16::MAX),
            flags: nr::BoneFlags(native_flags(
                (node.flags).bits(),
                values.get("m3.bone.flags").copied().unwrap_or(0) as u32,
            )),
            padding: values.get("m3.bone.padding").copied().unwrap_or(0) as u16,
            position: restore(&values, "bone.position", node.translation),
            rotation: restore(&values, "bone.rotation", node.rotation),
            scale: restore(&values, "bone.scale", node.scale),
            batching: restore(
                &values,
                "bone.batching",
                values.get("m3.bone.batching.initial").copied().unwrap_or(0) as u32,
            ),
        });
        initial.push(nr::InitialReference {
            matrix: node.inverse_bind.unwrap_or([
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [0., 0., 0., 1.],
            ]),
        });
    }
    Ok((bones, initial))
}

pub(super) fn mapping(model: &sem::Model<'_>) -> Result<Vec<u32>, Error> {
    Ok(mapping_details(model)?.0)
}
fn mapping_details(model: &sem::Model<'_>) -> Result<(Vec<u32>, Vec<bool>), Error> {
    let mut skipped = vec![false; model.nodes.len()];
    if let Some(native) = &model.native {
        let doc = native.document.raw();
        let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
        let record = doc
            .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
            .get(native.model as usize)
            .ok_or_else(|| Error::new("wem.model", "missing model"))??;
        for (index, row) in doc
            .resolve::<wr::Node<wr::V19>>(record.nodes.nodes.cast())?
            .iter()
            .enumerate()
        {
            let row = row?;
            let values = super::effects::metadata::read(doc, &row.native)?;
            let Some(node) = model.nodes.get(index) else {
                continue;
            };
            let animated=model.clips.iter().flat_map(|clip|&clip.tracks).any(|track|matches!(track.target,sem::Target::Translation(owner)|sem::Target::Rotation(owner)|sem::Target::Scale(owner)if owner as usize==index));
            let auxiliary = values.get("m3.auxiliary") == Some(&1);
            if auxiliary
                && (animated
                    || node.translation != [0.; 3]
                    || node.rotation != [0., 0., 0., 1.]
                    || node.scale != [1.; 3]
                    || node.pivot != [0.; 3])
            {
                return Err(Error::new(
                    "nodes.auxiliary",
                    "native metadata helper has no spatial transform",
                ));
            }
            skipped[index] = auxiliary
                || values.contains_key("m3.effect.bone")
                    && node.parent.is_some()
                    && node.translation == [0.; 3]
                    && node.rotation == [0., 0., 0., 1.]
                    && node.scale == [1.; 3]
                    && node.pivot == [0.; 3]
                    && !animated;
        }
    }
    let mut map = vec![u32::MAX; model.nodes.len()];
    let mut next = 0;
    for (index, skip) in skipped.iter().enumerate() {
        if !skip {
            map[index] = next;
            next += 1;
        }
    }
    for (index, skip) in skipped.iter().enumerate() {
        if *skip {
            if model.nodes[index].parent.is_none() {
                continue;
            }
            let mut ancestor = model.nodes[index]
                .parent
                .ok_or_else(|| Error::new("nodes", "effect wrapper has no parent"))?
                as usize;
            let mut visited = BTreeSet::new();
            while skipped[ancestor] {
                if !visited.insert(ancestor) {
                    return Err(Error::new("nodes", "effect owner cycle"));
                }
                ancestor = model.nodes[ancestor].parent.ok_or_else(|| {
                    Error::new("nodes", "effect owner chain has no hierarchy root")
                })? as usize;
            }
            map[index] = map[ancestor];
        }
    }
    Ok((map, skipped))
}

/// Maps the three positive native inheritance bits into WEM's negative flags.
pub(super) fn common_flags(flags: u32) -> u32 {
    let mut result = 0;
    for (source, destination) in [
        (1, wr::NodeFlags::DONT_INHERIT_TRANSLATION.bits()),
        (4, wr::NodeFlags::DONT_INHERIT_ROTATION.bits()),
        (2, wr::NodeFlags::DONT_INHERIT_SCALE.bits()),
    ] {
        if flags & source == 0 {
            result |= destination;
        }
    }
    if flags & 0x10 != 0 {
        result |= wr::NodeFlags::BILLBOARDED.bits();
    }
    result
}

fn native_flags(flags: u32, original: u32) -> u32 {
    (original & !0x17)
        | u32::from(flags & wr::NodeFlags::DONT_INHERIT_TRANSLATION.bits() == 0)
        | u32::from(flags & wr::NodeFlags::DONT_INHERIT_SCALE.bits() == 0) << 1
        | u32::from(flags & wr::NodeFlags::DONT_INHERIT_ROTATION.bits() == 0) << 2
        | u32::from(flags & wr::NodeFlags::BILLBOARDED.bits() != 0) << 4
}
