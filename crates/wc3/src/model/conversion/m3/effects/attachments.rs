//! Named attachment points linked to an existing hierarchy bone.
use super::{metadata, native, nr, read_error, revision, sem, wem, wr, Envelope, Error, Ref};
use crate::model::conversion::m3::{append, append_string};
use std::collections::BTreeMap;

pub(super) fn attach<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::AttachmentPoint>,
    doc: &mut wem::RawDocument<'static>,
    nodes: &mut Vec<wr::Node<wr::V19>>,
) -> Result<(), Error> {
    if reference.entries == 0 {
        return Ok(());
    }
    macro_rules! read {
        ($v:ty) => {
            for record in source
                .resolve::<nr::AttachmentPoint<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let record = record.map_err(read_error)?;
                if record.bone_index as usize >= nodes.len() {
                    return Err(Error::new(
                        "m3.attachment.bone",
                        "attachment bone is out of range",
                    ));
                }
                let name = source
                    .string(record.name)
                    .map_err(read_error)?
                    .unwrap_or_default();
                let mut values = BTreeMap::new();
                values.insert("m3.attachment.unknown".into(), i64::from(record.unknown));
                values.insert("m3.effect.bone".into(), i64::from(record.bone_index));
                let bag = metadata::extend(doc, &wr::NativeBag::default(), &values)?;
                nodes.push(wr::Node::<wr::V19> {
                    name: metadata::text(doc, name)?,
                    parent: record.bone_index,
                    kind: wr::NodeKind::Attachment,
                    local: wr::Transform {
                        rotation: [0., 0., 0., 1.],
                        scale: [1.; 3],
                        ..Default::default()
                    },
                    native: bag,
                    payload: wr::NodePayload::Attachment(Box::new(wr::AttachmentPayload {
                        model: u32::MAX,
                        asset: wr::AssetKey {
                            id: u32::MAX,
                            ..Default::default()
                        },
                    })),
                    ..Default::default()
                });
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        _ => {
            return Err(Error::new(
                "m3.attachments",
                "unsupported attachment revision",
            ));
        }
    }
    Ok(())
}

pub(super) fn export<E: Envelope>(
    model: &sem::Model<'_>,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
) -> Result<Ref<nr::targets::AttachmentPoint>, Error> {
    let Some(native) = &model.native else {
        return Ok(Ref::default());
    };
    let source = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let record = source
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let mut records = Vec::new();
    for (index, node) in source
        .resolve::<wr::Node<wr::V19>>(record.nodes.nodes.cast())?
        .iter()
        .enumerate()
    {
        let node = node?;
        let wr::NodePayload::Attachment(payload) = node.payload else {
            continue;
        };
        if payload.model != u32::MAX
            || payload.asset.path.entries != 0
            || payload.asset.id != u32::MAX
        {
            return Err(Error::new(
                format!("nodes[{index}].attachment.asset"),
                "M3 attachment points do not name an attached external model",
            ));
        }
        let values = metadata::read(source, &node.native)?;
        // Collapsed identity wrappers map to their parent. Transformed attachment nodes
        // receive their own exported bone through the same hierarchy map.
        let bone = *bone_map
            .get(index)
            .ok_or_else(|| Error::new("m3.attachment.bone", "missing bone mapping"))?;
        records.push(nr::AttachmentPoint::<nr::V1> {
            unknown: values.get("m3.attachment.unknown").copied().unwrap_or(0) as u32,
            name: append_string(doc, source.string(node.name)?.unwrap_or_default())?,
            bone_index: bone,
        });
    }
    Ok(append(doc, &records)?.cast())
}
