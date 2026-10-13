//! Regions, hierarchy bindings, and editable mesh extraction.
use super::super::{revision, string};
use super::packed_vertices;
use crate::model::m3::{self as native, records as nr};
use crate::model::wem::{self, records as wr, semantic as sem};
use native::{Envelope, Ref};
use std::borrow::Cow;
#[derive(Clone, Copy, Debug)]
struct Region {
    first_vertex: u32,
    vertex_count: u32,
    first_index: u32,
    index_count: u32,
    first_bone_lookup: u16,
    bone_lookup_count: u16,
    uv_scale: f32,
    uv_offset: f32,
}
fn regions<E: Envelope>(
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
                            first_vertex: u32::from(r.first_vertex),
                            vertex_count: u32::from(r.vertex_count),
                            first_index: 0,
                            index_count: face_count as u32,
                            first_bone_lookup: r.first_bone_lookup,
                            bone_lookup_count: r.bone_lookup_count,
                            uv_scale: 1.0,
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
        2 => read!(nr::V2, |_| 1.0, |_| 0.0),
        3 => read!(nr::V3, |_| 1.0, |_| 0.0),
        4 => read!(nr::V4, |_| 1.0, |_| 0.0),
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
pub(in crate::model::conversion::m3) fn divisions<E: Envelope>(
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

pub(in crate::model::conversion::m3) fn semantic<E: Envelope>(
    model: &native::Model<'_, E>,
) -> Result<sem::Model<'static>, native::ReadError> {
    let root = model.root_record()?;
    let document = model.document();
    let mut output = sem::Model {
        space: wem::records::CoordSpace::Sc2,
        name: Cow::Owned(string(document, root.name())?),
        profile: wem::records::ProfileId::Sc2,
        ..Default::default()
    };
    let (division_ref, palette_ref, initial_ref, material_ref) =
        native::visit_m3_root!(root, |r| {
            output.bounds = wem::records::Extent {
                minimum: r.bounds.minimum,
                maximum: r.bounds.maximum,
                sphere_radius: r.bounds.radius,
            };
            (
                r.divisions,
                r.bone_lookup,
                r.initial_reference,
                r.material_maps,
            )
        });
    let inverse_bind = document
        .resolve::<nr::InitialReference>(initial_ref.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    macro_rules! bones {
        ($v:ty) => {
            for (index, row) in document
                .resolve::<nr::Bone<$v>>(root.bones().cast())?
                .iter()
                .enumerate()
            {
                let bone = row?;
                output.nodes.push(sem::Node {
                    name: Cow::Owned(string(document, bone.name)?),
                    parent: (bone.parent_index != u16::MAX).then_some(u32::from(bone.parent_index)),
                    translation: bone.position.initial_value,
                    rotation: bone.rotation.initial_value,
                    scale: bone.scale.initial_value,
                    flags: wr::NodeFlags(super::super::hierarchy::common_flags(
                        (bone.flags).bits(),
                    )),
                    inverse_bind: inverse_bind.get(index).map(|r| r.matrix),
                    bone: true,
                    ..Default::default()
                });
            }
        };
    }
    if root.bones().entries != 0 {
        match revision(document, root.bones())? {
            0 => bones!(nr::V0),
            1 => bones!(nr::V1),
            _ => return Err(native::ReadError::invalid(0, "unsupported bone revision")),
        }
    }
    let vertices = packed_vertices(model)?;
    let palette = document
        .resolve::<nr::U16>(palette_ref.cast())?
        .iter()
        .map(|row| row.map(|r| r.value))
        .collect::<Result<Vec<_>, _>>()?;
    let material_maps = document
        .resolve::<nr::MaterialMap>(material_ref.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    for (index, _) in material_maps.iter().enumerate() {
        output.materials.push(sem::Material {
            name: Cow::Owned(format!("Material {index}")),
            ..Default::default()
        });
    }
    for (lod, division) in divisions(document, division_ref)?.into_iter().enumerate() {
        let faces = document
            .resolve::<nr::U16>(division.faces.cast())?
            .iter()
            .map(|row| row.map(|r| r.value))
            .collect::<Result<Vec<_>, _>>()?;
        let regions = regions(document, division.regions, faces.len())?;
        let mut batches = Vec::new();
        macro_rules! batches {
            ($v:ty) => {
                for row in document
                    .resolve::<nr::Batch<$v>>(division.batches.cast())?
                    .iter()
                {
                    let b = row?;
                    batches.push((usize::from(b.region_index), u32::from(b.material_index)));
                }
            };
        }
        if division.batches.entries != 0 {
            match revision(document, division.batches)? {
                0 => batches!(nr::V0),
                1 => batches!(nr::V1),
                _ => return Err(native::ReadError::invalid(0, "unsupported batch revision")),
            }
        }
        if batches.is_empty() {
            batches.extend((0..regions.len()).map(|i| (i, 0)));
        }
        for (region_index, material) in batches {
            let region = regions
                .get(region_index)
                .ok_or_else(|| native::ReadError::invalid(0, "batch region index"))?;
            let first = region.first_vertex as usize;
            let end = first
                .checked_add(region.vertex_count as usize)
                .ok_or_else(|| native::ReadError::invalid(0, "region vertex range overflow"))?;
            let rows = vertices
                .get(first..end)
                .ok_or_else(|| native::ReadError::invalid(0, "region exceeds packed vertices"))?;
            let first_index = region.first_index as usize;
            let end_index = first_index
                .checked_add(region.index_count as usize)
                .ok_or_else(|| native::ReadError::invalid(0, "region face range overflow"))?;
            let indices = faces
                .get(first_index..end_index)
                .ok_or_else(|| native::ReadError::invalid(0, "region exceeds face indices"))?;
            if indices.len() % 3 != 0 {
                return Err(native::ReadError::invalid(
                    0,
                    "non-triangular native region",
                ));
            }
            let mut mesh = sem::Mesh {
                name: Cow::Owned(format!("LOD {lod} region {region_index}")),
                positions: Cow::Owned(rows.iter().map(|r| r.position).collect()),
                normals: Cow::Owned(rows.iter().map(|r| r.normal).collect()),
                tangents: Cow::Owned(rows.iter().map(|r| r.tangent).collect()),
                material_slot: material,
                lod: lod as u32,
                ..Default::default()
            };
            mesh.triangles = Cow::Owned(
                indices
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .map(|t| [u32::from(t[0]), u32::from(t[1]), u32::from(t[2])])
                    .collect(),
            );
            if mesh
                .triangles
                .iter()
                .flatten()
                .any(|i| *i >= region.vertex_count)
            {
                return Err(native::ReadError::invalid(
                    0,
                    "face index exceeds region vertex range",
                ));
            }
            for channel in 0..5 {
                if rows.first().is_some_and(|r| r.uv[channel].is_some()) {
                    while mesh.uv_sets.len() < channel {
                        mesh.uv_sets.push(Cow::Owned(vec![[0.; 2]; rows.len()]));
                    }
                    mesh.uv_sets.push(Cow::Owned(
                        rows.iter()
                            .map(|r| {
                                r.uv[channel]
                                    .unwrap()
                                    .map(|v| v * 2048.0 * region.uv_scale + region.uv_offset)
                            })
                            .collect(),
                    ));
                }
            }
            if rows.first().is_some_and(|r| r.color.is_some()) {
                mesh.colors = Cow::Owned(rows.iter().map(|r| r.color.unwrap()).collect());
            }
            let start = usize::from(region.first_bone_lookup);
            let end = start + usize::from(region.bone_lookup_count);
            let lookup = palette
                .get(start..end)
                .ok_or_else(|| native::ReadError::invalid(0, "region exceeds bone palette"))?;
            for vertex in rows {
                let mut influences = Vec::new();
                for (bone, weight) in vertex.bones.into_iter().zip(vertex.weights) {
                    if weight != 0 {
                        let node = *lookup.get(usize::from(bone)).ok_or_else(|| {
                            native::ReadError::invalid(0, "packed bone index exceeds palette")
                        })?;
                        if usize::from(node) >= output.nodes.len() {
                            return Err(native::ReadError::invalid(
                                0,
                                "palette bone exceeds skeleton",
                            ));
                        }
                        influences.push(sem::Influence {
                            node: u32::from(node),
                            weight: f32::from(weight) / 255.0,
                        });
                    }
                }
                mesh.influences.push(influences);
            }
            output.meshes.push(mesh);
        }
    }
    Ok(output)
}
