//! Editable native draw controls attached to each WEM section.
use super::super::{effects::metadata, read_error, revision, Error};
use super::import::divisions;
use crate::model::m3::{self as native, records as nr};
use crate::model::wem::{self, records as wr, semantic as sem};
use native::{Envelope, Ref};
use std::collections::BTreeMap;

type Fields = BTreeMap<String, i64>;
fn regions<E: Envelope>(
    doc: &native::Document<'_, E>,
    reference: Ref<nr::targets::Region>,
) -> Result<Vec<Fields>, Error> {
    let mut out = Vec::new();
    if reference.entries == 0 {
        return Ok(out);
    }
    macro_rules! rows {
        ($v:ty, $extra:expr) => {
            for row in doc
                .resolve::<nr::Region<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = row.map_err(read_error)?;
                let mut v = Fields::new();
                for (key, value) in [
                    ("index", r.index),
                    ("bone_count", u32::from(r.bone_count)),
                    ("padding", u32::from(r.padding)),
                    ("bone_weight_pairs", u32::from(r.bone_weight_pairs)),
                    ("unknown_byte", u32::from(r.unknown_byte)),
                    ("root_bone", u32::from(r.root_bone)),
                ] {
                    v.insert(format!("m3.region.{key}"), i64::from(value));
                }
                v.insert("m3.region.uv_scale".into(), i64::from(16f32.to_bits()));
                v.insert("m3.region.uv_offset".into(), 0);
                $extra(&r, &mut v);
                out.push(v);
            }
        };
    }
    match revision(doc, reference).map_err(read_error)? {
        0 => rows!(nr::V0, |_: &nr::Region<nr::V0>, _: &mut Fields| {}),
        1 => rows!(nr::V1, |_: &nr::Region<nr::V1>, _: &mut Fields| {}),
        2 => rows!(nr::V2, |_: &nr::Region<nr::V2>, _: &mut Fields| {}),
        3 => rows!(nr::V3, |r: &nr::Region<nr::V3>, v: &mut Fields| {
            v.insert("m3.region.unknown".into(), i64::from(r.unknown));
        }),
        4 => rows!(nr::V4, |r: &nr::Region<nr::V4>, v: &mut Fields| {
            v.insert("m3.region.unknown".into(), i64::from(r.unknown));
            v.insert("m3.region.flags".into(), i64::from(r.flags));
        }),
        5 => rows!(nr::V5, |r: &nr::Region<nr::V5>, v: &mut Fields| {
            v.insert("m3.region.unknown".into(), i64::from(r.unknown));
            v.insert("m3.region.flags".into(), i64::from(r.flags));
            v.insert("m3.region.uv_scale".into(), i64::from(r.uv_scale.to_bits()));
            v.insert(
                "m3.region.uv_offset".into(),
                i64::from(r.uv_offset.to_bits()),
            );
        }),
        _ => {
            return Err(Error::new(
                "regions",
                "unsupported region controls revision",
            ));
        }
    }
    Ok(out)
}
fn batches<E: Envelope>(
    doc: &native::Document<'_, E>,
    reference: Ref<nr::targets::Batch>,
) -> Result<Vec<(usize, Fields)>, Error> {
    let mut out = Vec::new();
    if reference.entries == 0 {
        return Ok(out);
    }
    macro_rules! rows {
        ($v:ty) => {
            for row in doc
                .resolve::<nr::Batch<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = row.map_err(read_error)?;
                let mut v = Fields::new();
                for (key, value) in [
                    ("flags", r.flags),
                    ("priority_plane", r.priority_plane),
                    ("bounds_index", r.bounds_index),
                    ("color_index", r.color_index),
                    ("visibility_bone", r.visibility_bone),
                ] {
                    v.insert(format!("m3.batch.{key}"), i64::from(value));
                }
                out.push((usize::from(r.region_index), v));
            }
        };
    }
    match revision(doc, reference).map_err(read_error)? {
        0 => rows!(nr::V0),
        1 => rows!(nr::V1),
        _ => return Err(Error::new("batches", "unsupported batch controls revision")),
    }
    Ok(out)
}
pub(in crate::model::conversion::m3) fn attach<E: Envelope>(
    source: &native::Model<'_, E>,
    document: wem::Document<'static>,
) -> Result<wem::Document<'static>, Error> {
    let source_doc = source.document();
    let reference = native::visit_m3_root!(source.root_record().map_err(read_error)?, |r| {
        r.divisions
    });
    let mut fields = Vec::new();
    for division in divisions(source_doc, reference).map_err(read_error)? {
        let regions = regions(source_doc, division.regions)?;
        let mut batches = batches(source_doc, division.batches)?;
        if batches.is_empty() {
            batches.extend((0..regions.len()).map(|i| (i, Fields::new())));
        }
        for (index, mut v) in batches {
            v.extend(
                regions
                    .get(index)
                    .ok_or_else(|| Error::new("batches.region", "missing region"))?
                    .clone(),
            );
            v.insert(
                "m3.division.instances".into(),
                i64::from(division.instances),
            );
            fields.push(v);
        }
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
    let meshes = doc
        .resolve::<wr::Mesh<wr::V2>>(model.meshes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    if fields.len() != meshes.len() {
        return Err(Error::new("meshes", "draw control cardinality mismatch"));
    }
    for (mesh, fields) in meshes.iter().zip(fields) {
        let mut sections = doc
            .resolve::<wr::MeshSection>(mesh.sections.cast())?
            .iter()
            .collect::<Result<Vec<_>, _>>()?;
        let section = sections
            .first_mut()
            .ok_or_else(|| Error::new("mesh.sections", "missing section"))?;
        section.native = metadata::extend(&mut doc, &section.native, &fields)?;
        let id = doc
            .section_id(mesh.sections.index)
            .ok_or_else(|| Error::new("mesh.sections", "missing section storage"))?;
        doc.replace(id, &sections)?;
    }
    Ok(wem::Document::from_raw(doc)?)
}
pub(super) fn values(model: &sem::Model<'_>, index: usize) -> Result<Fields, Error> {
    let Some(native) = &model.native else {
        return Ok(Fields::new());
    };
    let doc = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let record = doc
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let projection = native.mesh_projections.get(index);
    let source = projection.map_or(index, |p| p.mesh as usize);
    let section = projection.map_or(0, |p| p.section as usize);
    macro_rules! sections {
        ($v:ty) => {
            doc.resolve::<wr::Mesh<$v>>(record.meshes.cast())?
                .get(source)
                .ok_or_else(|| Error::new("wem.mesh", "missing source mesh"))??
                .sections
        };
    }
    let sections = match doc.referenced_section(record.meshes)?.map(|s| s.revision()) {
        Some(1) => sections!(wr::V1),
        Some(2) => sections!(wr::V2),
        _ => return Err(Error::new("wem.mesh", "unsupported source mesh revision")),
    };
    let section = doc
        .resolve::<wr::MeshSection>(sections.cast())?
        .get(section)
        .ok_or_else(|| Error::new("wem.section", "missing draw section"))??;
    metadata::read(doc, &section.native)
}
pub(super) fn uint(fields: &Fields, key: &str, default: u32) -> u32 {
    fields.get(key).map_or(default, |v| *v as u32)
}
pub(super) fn float(fields: &Fields, key: &str, default: f32) -> f32 {
    fields
        .get(key)
        .map_or(default, |v| f32::from_bits(*v as u32))
}
pub(super) fn bone(fields: &Fields, key: &str, default: u16, map: &[u32]) -> Result<u16, Error> {
    if !fields.contains_key(key) {
        return Ok(default);
    }
    let source = uint(fields, key, u32::from(default));
    if source == u32::from(u16::MAX) {
        return Ok(u16::MAX);
    }
    u16::try_from(
        *map.get(source as usize)
            .ok_or_else(|| Error::new(key, "missing draw-control bone"))?,
    )
    .map_err(|_| Error::new(key, "draw-control bone cannot map to native skeleton"))
}
