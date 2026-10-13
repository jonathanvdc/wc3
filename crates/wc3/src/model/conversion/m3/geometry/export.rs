//! Packed vertex quantization and independently encoded LOD divisions.
use super::super::{append, quantize, signed_unorm, Context, Error};
use super::controls;
use crate::model::m3::{self as native, records as nr};
use crate::model::wem::semantic as sem;
use native::{Envelope, Ref};
use std::collections::BTreeSet;
pub(in crate::model::conversion::m3) struct EncodedGeometry {
    pub(in crate::model::conversion::m3) vertices: Ref<nr::targets::Byte>,
    pub(in crate::model::conversion::m3) palette: Ref<nr::targets::U16>,
    pub(in crate::model::conversion::m3) divisions: Ref<nr::targets::MeshDivision>,
    pub(in crate::model::conversion::m3) flags: u32,
}
pub(in crate::model::conversion::m3) fn encode<E: Envelope>(
    model: &sem::Model<'_>,
    document: &mut native::Document<'static, E>,
    context: &mut Context,
) -> Result<EncodedGeometry, Error> {
    let node_map = super::super::hierarchy::mapping(model)?;
    let uv_count = model
        .meshes
        .iter()
        .map(|m| m.uv_sets.len())
        .max()
        .unwrap_or(0);
    if uv_count > 5 {
        return Err(Error::new(
            "meshes.uv_sets",
            "M3 supports at most five packed UV channels",
        ));
    }
    let has_color = model.meshes.iter().any(|m| !m.colors.is_empty());
    let stride = 28 + uv_count * 4 + usize::from(has_color) * 4;
    let mut vertices = Vec::new();
    let mut region_rows = Vec::new();
    let mut batches = Vec::new();
    let mut faces = Vec::new();
    let mut palettes = Vec::new();
    for (index, mesh) in model.meshes.iter().enumerate() {
        if mesh.positions.len() > 65536 {
            return Err(Error::new(
                format!("meshes[{index}]"),
                "region exceeds 16-bit face index capacity",
            ));
        }
        let controls = controls::values(model, index)?;
        let uv_scale = controls::float(&controls, "m3.region.uv_scale", 16.);
        let uv_offset = controls::float(&controls, "m3.region.uv_offset", 0.);
        if !uv_scale.is_finite() || !uv_offset.is_finite() {
            return Err(Error::new(
                format!("meshes[{index}].uv_transform"),
                "native UV scale and offset must be finite",
            ));
        }
        let first_vertex = vertices.len() / stride;
        let first_index = faces.len();
        let mut palette = BTreeSet::new();
        for influences in &mesh.influences {
            for influence in influences {
                palette.insert(node_map[influence.node as usize]);
            }
        }
        if palette.len() > 256 {
            return Err(Error::new(
                format!("meshes[{index}].influences"),
                "region exceeds byte-sized palette capacity",
            ));
        }
        let palette: Vec<_> = palette.into_iter().collect();
        let first_palette = palettes.len();
        for node in &palette {
            if *node as usize >= model.nodes.len() {
                return Err(Error::new(
                    format!("meshes[{index}].influences"),
                    "missing skeleton node",
                ));
            }
            palettes.push(nr::U16 {
                value: *node as u16,
            });
        }
        if palettes.len() > 65535 {
            return Err(Error::new(
                "meshes.influences",
                "M3 palette table capacity exceeded",
            ));
        }
        for (vertex_index, position) in mesh.positions.iter().enumerate() {
            let path = format!("meshes[{index}].vertices[{vertex_index}]");
            let mut row = vec![0u8; stride];
            if position.iter().any(|v| !v.is_finite()) {
                return Err(Error::new(&path, "non-finite position"));
            }
            for (component, value) in position.iter().enumerate() {
                row[component * 4..component * 4 + 4].copy_from_slice(&value.to_le_bytes());
            }
            let influences = mesh
                .influences
                .get(vertex_index)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            if influences.len() > 4 {
                return Err(Error::new(
                    format!("{path}.influences"),
                    "M3 packed vertices support four influences",
                ));
            }
            for (lane, influence) in influences.iter().enumerate() {
                let q = quantize(
                    influence.weight,
                    255.,
                    0.,
                    255.,
                    &format!("{path}.weights[{lane}]"),
                    context,
                )?;
                row[12 + lane] = (q * 255.).round() as u8;
                row[16 + lane] = palette
                    .binary_search(&node_map[influence.node as usize])
                    .unwrap() as u8;
            }
            let normal = mesh
                .normals
                .get(vertex_index)
                .copied()
                .unwrap_or([0., 0., 1.]);
            for (component, value) in normal.iter().enumerate() {
                let q = signed_unorm(*value, &format!("{path}.normal[{component}]"), context)?;
                row[20 + component] = (q * 255.).round() as u8;
            }
            row[23] = 255;
            if has_color {
                let color = mesh.colors.get(vertex_index).copied().unwrap_or([1.; 4]);
                for (component, value) in color.iter().enumerate() {
                    let q = quantize(
                        *value,
                        255.,
                        0.,
                        255.,
                        &format!("{path}.color[{component}]"),
                        context,
                    )?;
                    let at = [26, 25, 24, 27][component];
                    row[at] = (q * 255.).round() as u8;
                }
            }
            let mut at = 24 + usize::from(has_color) * 4;
            for channel in 0..uv_count {
                let uv = mesh
                    .uv_sets
                    .get(channel)
                    .and_then(|set| set.get(vertex_index))
                    .copied()
                    .unwrap_or([0.; 2]);
                for (component, value) in uv.iter().enumerate() {
                    let packed_uv = if uv_scale == 0. {
                        if *value != uv_offset {
                            return Err(Error::new(
                                format!("{path}.uv[{channel}][{component}]"),
                                "edited UV cannot be represented by a zero native scale",
                            ));
                        }
                        0.
                    } else {
                        (*value - uv_offset) / (uv_scale / 16.)
                    };
                    let q = quantize(
                        packed_uv,
                        2048.,
                        f32::from(i16::MIN),
                        f32::from(i16::MAX),
                        &format!("{path}.uv[{channel}][{component}]"),
                        context,
                    )?;
                    row[at + component * 2..at + component * 2 + 2]
                        .copy_from_slice(&((q * 2048.).round() as i16).to_le_bytes());
                }
                at += 4;
            }
            let tangent = mesh
                .tangents
                .get(vertex_index)
                .copied()
                .unwrap_or([1., 0., 0., 1.]);
            for (component, value) in tangent.iter().enumerate() {
                let q = signed_unorm(*value, &format!("{path}.tangent[{component}]"), context)?;
                row[stride - 4 + component] = (q * 255.).round() as u8;
            }
            vertices.extend(row);
        }
        for triangle in mesh.triangles.iter() {
            for vertex in triangle {
                if *vertex as usize >= mesh.positions.len() {
                    return Err(Error::new(
                        format!("meshes[{index}].triangles"),
                        "triangle references a missing vertex",
                    ));
                }
                faces.push(nr::U16 {
                    value: *vertex as u16,
                });
            }
        }
        region_rows.push(nr::Region::<nr::V5> {
            index: controls::uint(&controls, "m3.region.index", index as u32),
            unknown: controls::uint(&controls, "m3.region.unknown", 0),
            bone_count: u16::try_from(controls::uint(
                &controls,
                "m3.region.bone_count",
                palette.len() as u32,
            ))
            .map_err(|_| Error::new("region.bone_count", "bone count capacity exceeded"))?,
            padding: u16::try_from(controls::uint(&controls, "m3.region.padding", 0))
                .map_err(|_| Error::new("region.padding", "padding capacity exceeded"))?,
            first_vertex: first_vertex as u32,
            vertex_count: mesh.positions.len() as u32,
            first_index: first_index as u32,
            index_count: (faces.len() - first_index) as u32,
            first_bone_lookup: first_palette as u16,
            bone_lookup_count: palette.len() as u16,
            bone_weight_pairs: u8::try_from(controls::uint(
                &controls,
                "m3.region.bone_weight_pairs",
                4,
            ))
            .map_err(|_| Error::new("region.bone_weight_pairs", "weight pair capacity exceeded"))?,
            unknown_byte: u8::try_from(controls::uint(&controls, "m3.region.unknown_byte", 1))
                .map_err(|_| Error::new("region.unknown_byte", "byte capacity exceeded"))?,
            root_bone: controls::bone(
                &controls,
                "m3.region.root_bone",
                palette.first().copied().unwrap_or(u32::from(u16::MAX)) as u16,
                &node_map,
            )?,
            flags: controls::uint(&controls, "m3.region.flags", 0),
            uv_scale,
            uv_offset,
        });
        batches.push(nr::Batch::<nr::V1> {
            region_index: index as u16,
            material_index: u16::try_from(mesh.material_slot).map_err(|_| {
                Error::new(
                    format!("meshes[{index}].material"),
                    "material index capacity exceeded",
                )
            })?,
            flags: u16::try_from(controls::uint(&controls, "m3.batch.flags", 0))
                .map_err(|_| Error::new("batch.flags", "flags capacity exceeded"))?,
            priority_plane: u16::try_from(controls::uint(&controls, "m3.batch.priority_plane", 0))
                .map_err(|_| Error::new("batch.priority_plane", "priority capacity exceeded"))?,
            bounds_index: u16::try_from(controls::uint(&controls, "m3.batch.bounds_index", 0))
                .map_err(|_| Error::new("batch.bounds_index", "bounds capacity exceeded"))?,
            color_index: u16::try_from(controls::uint(&controls, "m3.batch.color_index", 0))
                .map_err(|_| Error::new("batch.color_index", "color capacity exceeded"))?,
            visibility_bone: controls::bone(
                &controls,
                "m3.batch.visibility_bone",
                u16::MAX,
                &node_map,
            )?,
        });
    }
    let maximum_lod = model.meshes.iter().map(|m| m.lod).max().unwrap_or(0);
    if maximum_lod > 255 {
        return Err(Error::new(
            "meshes.lod",
            "M3 conversion supports at most 256 explicit division identities",
        ));
    }
    let mut divisions = Vec::new();
    if !model.meshes.is_empty() {
        for lod in 0..=maximum_lod {
            let mut instances = None;
            for (index, _) in model
                .meshes
                .iter()
                .enumerate()
                .filter(|(_, m)| m.lod == lod)
            {
                let value =
                    controls::uint(&controls::values(model, index)?, "m3.division.instances", 1);
                if instances.is_some_and(|old| old != value) {
                    return Err(Error::new(
                        "division.instances",
                        "conflicting instance counts within one division",
                    ));
                }
                instances = Some(value);
            }
            let mut selected_faces = Vec::new();
            let mut selected_regions = Vec::new();
            let mut selected_batches = Vec::new();
            for ((mesh, region), batch) in model.meshes.iter().zip(&region_rows).zip(&batches) {
                if mesh.lod != lod {
                    continue;
                }
                let start = region.first_index as usize;
                let count = region.index_count as usize;
                let mut region = *region;
                region.first_index = selected_faces.len() as u32;
                selected_faces.extend_from_slice(&faces[start..start + count]);
                let mut batch = *batch;
                batch.region_index = selected_regions.len() as u16;
                selected_regions.push(region);
                selected_batches.push(batch);
            }
            divisions.push(nr::MeshDivision::<nr::V2> {
                faces: append(document, &selected_faces)?.cast(),
                regions: append(document, &selected_regions)?.cast(),
                batches: append(document, &selected_batches)?.cast(),
                instances: instances.unwrap_or(1),
                ..Default::default()
            });
        }
    }
    let division_ref = append(document, &divisions)?.cast();
    let palette = append(document, &palettes)?.cast();
    let vertices = append(
        document,
        &vertices
            .into_iter()
            .map(|value| nr::Byte { value })
            .collect::<Vec<_>>(),
    )?
    .cast();
    let mask = (1u32 << uv_count) - 1;
    let flags =
        0x0180_0061 | ((mask & 15) << 17) | ((mask & 16) << 26) | if has_color { 0x200 } else { 0 };
    Ok(EncodedGeometry {
        vertices,
        palette,
        divisions: division_ref,
        flags,
    })
}
