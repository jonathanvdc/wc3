//! Per-bone property headers and fallback values independent of clip key data.
use super::effects::metadata;
use super::{append, native, nr, read_error, revision, wem, Error};
use crate::model::wem::{records as wr, semantic as sem};
use native::{Envelope, Ref};
use std::collections::BTreeMap;
pub(super) fn attach<E: Envelope>(
    source: &native::Model<'_, E>,
    document: wem::Document<'static>,
) -> Result<wem::Document<'static>, Error> {
    let reference = source.root_record().map_err(read_error)?.bones();
    if reference.entries == 0 {
        return Ok(document);
    }
    let mut doc = document.into_raw();
    let root = doc
        .resolve::<wr::DocumentRecord<wr::V1>>(wem::Ref::from_section(
            doc.root_reference().entries,
            doc.root_reference().index,
        ))?
        .get(0)
        .ok_or_else(|| Error::new("wem.root", "missing root"))??;
    let model = doc
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let mut nodes = doc
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    macro_rules! bones {
        ($v:ty,$unknown:expr) => {
            for (index, row) in source
                .document()
                .resolve::<nr::Bone<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
                .enumerate()
            {
                let r = row.map_err(read_error)?;
                let node = nodes
                    .get_mut(index)
                    .ok_or_else(|| Error::new("bone.node", "missing bone node"))?;
                let mut values = BTreeMap::new();
                values.insert("m3.bone.flags".into(), i64::from((r.flags).bits()));
                metadata::animated(&mut values, "bone.position", r.position);
                metadata::animated(&mut values, "bone.rotation", r.rotation);
                metadata::animated(&mut values, "bone.scale", r.scale);
                metadata::animated(&mut values, "bone.batching", r.batching);
                values.insert(
                    "m3.bone.batching.initial".into(),
                    i64::from(r.batching.initial_value),
                );
                values.insert("m3.bone.padding".into(), i64::from(r.padding));
                values.insert(
                    "m3.bone.source_version".into(),
                    i64::from(<nr::Bone<$v> as native::Record>::VERSION),
                );
                $unknown(&r, &mut values);
                node.native = metadata::extend(&mut doc, &node.native, &values)?;
            }
        };
    }
    match revision(source.document(), reference).map_err(read_error)? {
        0 => bones!(
            nr::V0,
            |r: &nr::Bone<nr::V0>, values: &mut BTreeMap<String, i64>| {
                values.insert("m3.bone.unknown".into(), i64::from(r.id));
            }
        ),
        1 => bones!(
            nr::V1,
            |r: &nr::Bone<nr::V1>, v: &mut BTreeMap<String, i64>| {
                v.insert("m3.bone.id".into(), i64::from(r.id));
            }
        ),
        _ => return Err(Error::new("bone.revision", "unsupported bone revision")),
    }
    let id = doc
        .section_id(model.nodes.nodes.index)
        .ok_or_else(|| Error::new("wem.nodes", "missing node section"))?;
    doc.replace(id, &nodes)?;
    Ok(wem::Document::from_raw(doc)?)
}
pub(super) fn values(model: &sem::Model<'_>, index: usize) -> Result<BTreeMap<String, i64>, Error> {
    let Some(native) = &model.native else {
        return Ok(BTreeMap::new());
    };
    let doc = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let model = doc
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let node = doc
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .get(index)
        .ok_or_else(|| Error::new("wem.node", "missing node"))??;
    metadata::read(doc, &node.native)
}
pub(super) fn append_bones<E: Envelope>(
    model: &sem::Model<'_>,
    doc: &mut native::Document<'static, E>,
    bones: &[nr::Bone<nr::V1>],
) -> Result<Ref<nr::targets::Bone>, Error> {
    let map = super::hierarchy::mapping(model)?;
    let mut metadata = vec![BTreeMap::new(); bones.len()];
    for (index, mapped) in map.into_iter().enumerate() {
        let values = values(model, index)?;
        if mapped != u32::MAX && values.contains_key("m3.bone.source_version") {
            metadata[mapped as usize] = values;
        }
    }
    if metadata
        .iter()
        .any(|v| v.get("m3.bone.source_version") == Some(&0))
    {
        let rows = bones
            .iter()
            .zip(metadata)
            .map(|(r, v)| nr::Bone::<nr::V0> {
                id: v.get("m3.bone.unknown").copied().unwrap_or(0) as u32,
                name: r.name,
                flags: r.flags,
                parent_index: r.parent_index,
                padding: r.padding,
                position: r.position,
                rotation: r.rotation,
                scale: r.scale,
                batching: r.batching,
            })
            .collect::<Vec<_>>();
        Ok(append(doc, &rows)?.cast())
    } else {
        Ok(append(doc, bones)?.cast())
    }
}
