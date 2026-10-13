//! Corner/face attributes become explicit target vertices without losing source addresses.
use super::*;
use std::collections::BTreeMap;
fn values<const N: usize>(
    doc: &RawDocument<'_>,
    layer: &r::AttrLayer,
    map: &[VertexSource],
    expected: usize,
) -> Result<Cow<'static, [[f32; N]]>, ReadError> {
    let values = float_values::<N>(doc, layer)?;
    if values.len() != expected {
        return Err(ReadError::invalid(0, "attribute domain element count"));
    }
    Ok(Cow::Owned(
        map.iter()
            .map(|source| {
                values
                    .get(domain_index(layer.domain, *source)?)
                    .copied()
                    .ok_or_else(|| ReadError::invalid(0, "attribute domain address"))
            })
            .collect::<Result<Vec<_>, _>>()?,
    ))
}
pub(in crate::model::wem::semantic) fn read_mesh<'a>(
    doc: &'a RawDocument<'_>,
    mesh: r::Mesh<r::V2>,
    index: u32,
) -> Result<(Vec<Mesh<'a>>, Vec<MeshProjection>), ReadError> {
    let layers = rows::<r::AttrLayer>(doc, mesh.attributes.layers.cast())?;
    let vertices = mesh.face_set.vertex_count as usize;
    let valences = rows::<r::U32>(doc, mesh.face_set.face_valence.cast())?
        .into_iter()
        .map(|row| row.value as usize)
        .collect::<Vec<_>>();
    let corners = rows::<r::U32>(doc, mesh.face_set.corner_vertex.cast())?
        .into_iter()
        .map(|row| row.value)
        .collect::<Vec<_>>();
    let corner_count = valences
        .iter()
        .try_fold(0usize, |sum, valence| sum.checked_add(*valence))
        .ok_or_else(|| ReadError::invalid(0, "face corner overflow"))?;
    if corners.len() != corner_count || corners.iter().any(|v| *v as usize >= vertices) {
        return Err(ReadError::invalid(0, "face corner topology"));
    }
    let sections = rows::<r::MeshSection>(doc, mesh.sections.cast())?;
    let section_count = sections.len().max(1);
    let mut assignments = None;
    for layer in &layers {
        let name = doc.string(layer.name)?.unwrap_or_default();
        if name == "section" {
            if layer.domain != r::Domain::FACE {
                return Err(ReadError::invalid(
                    0,
                    "section assignment must use the face domain",
                ));
            }
            assignments = Some(scalar_sections(doc, layer)?);
        }
    }
    let assignments = assignments.unwrap_or_else(|| vec![0; valences.len()]);
    if assignments.len() != valences.len()
        || assignments.iter().any(|v| *v as usize >= section_count)
    {
        return Err(ReadError::invalid(0, "face section assignment"));
    }
    if section_count > 1
        && !layers
            .iter()
            .any(|layer| doc.string(layer.name).ok().flatten() == Some("section"))
    {
        return Err(ReadError::invalid(
            0,
            "multi-section mesh needs a face section assignment",
        ));
    }
    let expand = layers.iter().any(|layer| {
        layer.domain != r::Domain::VERTEX
            && layer.domain != r::Domain::MESH
            && doc.string(layer.name).ok().flatten() != Some("section")
    });
    let offsets = rows::<r::U32>(doc, mesh.triangulation.offsets.cast())?;
    let explicit = rows::<r::U32>(doc, mesh.triangulation.vertices.cast())?;
    if !offsets.is_empty()
        && (offsets.len() != valences.len() + 1
            || offsets[0].value != 0
            || offsets.last().unwrap().value as usize != explicit.len()
            || offsets.windows(2).any(|v| v[0].value > v[1].value))
    {
        return Err(ReadError::invalid(0, "triangulation boundaries"));
    }
    let position = layers
        .iter()
        .find(|layer| doc.string(layer.name).ok().flatten() == Some("position"));
    let positions = position
        .map(|layer| float_values::<3>(doc, layer))
        .transpose()?
        .unwrap_or_default();
    if vertices != 0 && position.is_none() {
        return Err(ReadError::invalid(0, "missing position attribute"));
    }
    let mut face_triangles = Vec::new();
    let mut corner = 0usize;
    for (face, &valence) in valences.iter().enumerate() {
        let polygon = &corners[corner..corner + valence];
        let cached = if offsets.is_empty() {
            &[][..]
        } else {
            &explicit[offsets[face].value as usize..offsets[face + 1].value as usize]
        };
        let triangle_vertices = if !cached.is_empty() {
            if !cached.len().is_multiple_of(3) || cached.iter().any(|v| !polygon.contains(&v.value))
            {
                return Err(ReadError::invalid(0, "explicit face triangulation"));
            }
            cached
                .as_chunks::<3>()
                .0
                .iter()
                .map(|v| [v[0].value, v[1].value, v[2].value])
                .collect::<Vec<_>>()
        } else if valence == 3 {
            vec![[polygon[0], polygon[1], polygon[2]]]
        } else {
            let layer =
                position.ok_or_else(|| ReadError::invalid(0, "missing polygon positions"))?;
            let local = polygon
                .iter()
                .enumerate()
                .map(|(local, vertex)| {
                    positions
                        .get(domain_index(
                            layer.domain,
                            VertexSource {
                                vertex: *vertex,
                                corner: (corner + local) as u32,
                                face: face as u32,
                            },
                        )?)
                        .copied()
                        .ok_or_else(|| ReadError::invalid(0, "polygon position address"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let ids = (0..valence as u32).collect::<Vec<_>>();
            triangulate(&ids, &local)?
                .into_iter()
                .map(|v| v.map(|i| polygon[i as usize]))
                .collect()
        };
        let triangles = triangle_vertices
            .into_iter()
            .map(|triangle| {
                triangle.map(|vertex| {
                    let local = polygon
                        .iter()
                        .position(|candidate| *candidate == vertex)
                        .unwrap();
                    VertexSource {
                        vertex,
                        corner: (corner + local) as u32,
                        face: face as u32,
                    }
                })
            })
            .collect::<Vec<_>>();
        face_triangles.push(triangles);
        corner += valence;
    }
    let mut skin = Vec::new();
    if mesh.skin.offsets.entries != 0 {
        let offsets = rows::<r::U32>(doc, mesh.skin.offsets.cast())?
            .into_iter()
            .map(|row| row.value as usize)
            .collect::<Vec<_>>();
        let influences = rows::<r::Influence>(doc, mesh.skin.influences.cast())?;
        if offsets.len() != vertices + 1
            || offsets.first() != Some(&0)
            || offsets.last() != Some(&influences.len())
            || offsets.windows(2).any(|v| v[0] > v[1])
        {
            return Err(ReadError::invalid(0, "skin boundaries"));
        }
        skin = offsets
            .windows(2)
            .map(|range| {
                influences[range[0]..range[1]]
                    .iter()
                    .map(|row| Influence {
                        node: row.bone,
                        weight: row.weight,
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
    }
    let expected = |domain| match domain {
        r::Domain::VERTEX => Ok(vertices),
        r::Domain::HALFEDGE => Ok(corner_count),
        r::Domain::FACE => Ok(valences.len()),
        r::Domain::MESH => Ok(1),
        _ => Err(ReadError::unsupported(0, "edge attribute domain")),
    };
    let mut result = Vec::new();
    let mut provenance = Vec::new();
    for section_index in 0..section_count {
        let mut map = if expand {
            Vec::new()
        } else {
            (0..vertices as u32)
                .map(|vertex| VertexSource {
                    vertex,
                    corner: u32::MAX,
                    face: u32::MAX,
                })
                .collect()
        };
        let mut corner_vertices = BTreeMap::new();
        let mut triangles = Vec::new();
        for (face, rows) in face_triangles.iter().enumerate() {
            if assignments[face] as usize != section_index {
                continue;
            }
            for triangle in rows {
                let mut output = [0; 3];
                for (i, source) in triangle.iter().enumerate() {
                    output[i] = if expand {
                        *corner_vertices.entry(source.corner).or_insert_with(|| {
                            let index = map.len() as u32;
                            map.push(*source);
                            index
                        })
                    } else {
                        source.vertex
                    };
                }
                triangles.push(output);
            }
        }
        let section = sections.get(section_index);
        let mut value = Mesh {
            name: text_view(doc, mesh.name)?,
            lod: mesh.lod_level,
            bounds: mesh.bounds.clone(),
            material_slot: section.map(|v| v.material_slot).unwrap_or(u32::MAX),
            selection_group: section.map(|v| u32::from(v.selection_group)).unwrap_or(0),
            triangles: Cow::Owned(triangles),
            ..Default::default()
        };
        for layer in &layers {
            let name = doc.string(layer.name)?.unwrap_or_default();
            if name == "section" {
                continue;
            }
            let count = expected(layer.domain)?;
            match name {
                "position" => value.positions = values(doc, layer, &map, count)?,
                "normal" => value.normals = values(doc, layer, &map, count)?,
                "tangent" => value.tangents = values(doc, layer, &map, count)?,
                "color" => value.colors = values(doc, layer, &map, count)?,
                name if name.starts_with("uv") => {
                    let uv = name[2..]
                        .parse::<usize>()
                        .map_err(|_| ReadError::invalid(0, "UV set name"))?;
                    if uv > 31 {
                        return Err(ReadError::invalid(0, "UV set count"));
                    }
                    value.uv_sets.resize_with(uv + 1, || Cow::Borrowed(&[]));
                    value.uv_sets[uv] = values(doc, layer, &map, count)?;
                }
                _ => return Err(ReadError::unsupported(0, "attribute semantic")),
            }
        }
        if !skin.is_empty() {
            value.influences = map
                .iter()
                .map(|source| skin[source.vertex as usize].clone())
                .collect();
        }
        if let Some(section) = section {
            if let Some(node) = section.rigid_node.value {
                value.influences = vec![vec![Influence { node, weight: 1. }]; map.len()];
            }
        }
        result.push(value);
        provenance.push(MeshProjection {
            mesh: index,
            section: section_index as u32,
            source_vertex_count: mesh.face_set.vertex_count,
            vertices: map,
        });
    }
    Ok((result, provenance))
}
