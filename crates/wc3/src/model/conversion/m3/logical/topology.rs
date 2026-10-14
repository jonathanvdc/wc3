//! Private normalization of understood native region layouts.
use super::materials::{revision, Slot};
use crate::model::m3::{self as native, records as nr};
use native::{Envelope, Ref};
#[derive(Clone, Copy, Debug)]
pub(super) struct Region {
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
    pub(super) first_index: u32,
    pub(super) index_count: u32,
    pub(super) first_bone_lookup: u16,
    pub(super) bone_lookup_count: u16,
    pub(super) uv_scale: f32,
    pub(super) uv_offset: f32,
    pub(super) id: u32,
    pub(super) unknown: u32,
    pub(super) flags: nr::RegionFlags,
    pub(super) root_bone: u16,
    pub(super) unknown_byte: u8,
}
pub(super) fn regions<E: Envelope>(
    document: &native::Document<'_, E>,
    reference: Ref<nr::targets::Region>,
    face_count: usize,
) -> Result<Vec<Region>, native::ReadError> {
    if reference.entries == 0 {
        return Ok(Vec::new());
    }
    let mut values = Vec::new();
    macro_rules! read {
        ($v:ty, $scale:expr, $offset:expr) => {
            for row in document.resolve::<nr::Region<$v>>(reference.cast())?.iter() {
                let r = row?;
                values.push(Region {
                    id: r.index,
                    unknown: Slot::<u32>::value(&r.unknown),
                    flags: Slot::<nr::RegionFlags>::value(&r.flags),
                    root_bone: r.root_bone,
                    unknown_byte: r.unknown_byte,
                    first_vertex: u32::from(r.first_vertex),
                    vertex_count: u32::from(r.vertex_count),
                    first_index: r.first_index,
                    index_count: r.index_count,
                    first_bone_lookup: r.first_bone_lookup,
                    bone_lookup_count: r.bone_lookup_count,
                    uv_scale: $scale(&r),
                    uv_offset: $offset(&r),
                });
            }
        };
    }
    match revision(document, reference)? {
        0 | 1 => {
            if reference.entries != 1 {
                return Err(native::ReadError::invalid(
                    0,
                    "legacy region has no unambiguous face partition",
                ));
            }
            macro_rules! legacy {
                ($v:ty) => {
                    for row in document.resolve::<nr::Region<$v>>(reference.cast())?.iter() {
                        let r = row?;
                        values.push(Region {
                            id: r.index,
                            unknown: Slot::<u32>::value(&r.unknown),
                            flags: Slot::<nr::RegionFlags>::value(&r.flags),
                            root_bone: r.root_bone,
                            unknown_byte: r.unknown_byte,
                            first_vertex: u32::from(r.first_vertex),
                            vertex_count: u32::from(r.vertex_count),
                            first_index: 0,
                            index_count: face_count as u32,
                            first_bone_lookup: r.first_bone_lookup,
                            bone_lookup_count: r.bone_lookup_count,
                            uv_scale: 1.0 / 2048.0,
                            uv_offset: 0.0,
                        });
                    }
                };
            }
            if revision(document, reference)? == 0 {
                legacy!(nr::V0);
            } else {
                legacy!(nr::V1);
            }
        }
        2 => read!(nr::V2, |_| 1.0 / 2048.0, |_| 0.0),
        3 => read!(nr::V3, |_| 1.0 / 2048.0, |_| 0.0),
        4 => read!(nr::V4, |_| 1.0 / 2048.0, |_| 0.0),
        5 => read!(
            nr::V5,
            |r: &nr::Region<nr::V5>| r.uv_scale / 32768.,
            |r: &nr::Region<nr::V5>| r.uv_offset
        ),
        _ => {
            return Err(native::ReadError::invalid(
                0,
                "unsupported region topology revision",
            ));
        }
    }
    Ok(values)
}
pub(super) fn divisions<E: Envelope>(
    document: &native::Document<'_, E>,
    reference: Ref<nr::targets::MeshDivision>,
) -> Result<Vec<nr::MeshDivision<nr::V2>>, native::ReadError> {
    if reference.entries == 0 {
        return Ok(Vec::new());
    }
    macro_rules! read {
        ($v:ty) => {
            document
                .resolve::<nr::MeshDivision<$v>>(reference.cast())?
                .iter()
                .map(|row| {
                    row.map(|d| nr::MeshDivision::<nr::V2> {
                        faces: d.faces,
                        regions: d.regions,
                        batches: d.batches,
                        msec: d.msec,
                        instances: d.instances,
                    })
                })
                .collect()
        };
    }
    match revision(document, reference)? {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        2 => read!(nr::V2),
        _ => Err(native::ReadError::invalid(
            0,
            "unsupported mesh division revision",
        )),
    }
}
