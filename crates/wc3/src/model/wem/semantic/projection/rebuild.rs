//! Consistent projected edits replace source-domain values while polygon/native records stay active.
use super::*;
fn old_meshes(
    doc: &RawDocument<'_>,
    reference: Ref<r::targets::Mesh>,
) -> Result<Vec<r::Mesh<r::V2>>, ReadError> {
    let Some(section) = doc.referenced_section(reference)? else {
        return Ok(Vec::new());
    };
    match section.revision() {
        2 => rows(doc, reference.cast()),
        1 => Ok(rows::<r::Mesh<r::V1>>(doc, reference.cast())?
            .into_iter()
            .map(|mesh| r::Mesh::<r::V2> {
                name: mesh.name,
                lod_level: mesh.lod_level,
                face_set: mesh.face_set,
                attributes: mesh.attributes,
                skin: mesh.skin,
                sections: mesh.sections,
                repair_log: mesh.repair_log,
                bounds: mesh.bounds,
                ..Default::default()
            })
            .collect()),
        _ => Err(ReadError::unsupported(0, "mesh revision")),
    }
}
fn attribute_values<'data, const N: usize>(
    doc: &RawDocument<'_>,
    layer: &r::AttrLayer,
    changes: &[(usize, &Mesh<'data>, &MeshProjection)],
    get: impl for<'view> Fn(&'view Mesh<'data>) -> &'view [[f32; N]],
) -> Result<Vec<[f32; N]>, WriteError> {
    let mut values =
        float_values::<N>(doc, layer).map_err(|_| WriteError::invalid("source attribute"))?;
    let mut written = vec![None; values.len()];
    for (_, mesh, map) in changes {
        let input = get(mesh);
        if input.len() != map.vertices.len() {
            return Err(WriteError::invalid(
                "projected attribute cardinality changed",
            ));
        }
        for (source, value) in map.vertices.iter().zip(input) {
            let index = domain_index(layer.domain, *source)
                .map_err(|_| WriteError::invalid("source attribute domain"))?;
            let target = written
                .get_mut(index)
                .ok_or_else(|| WriteError::invalid("source attribute domain address"))?;
            if target.as_ref().is_some_and(|old| old != value) {
                return Err(WriteError::invalid(
                    "conflicting edits to repeated source-domain value",
                ));
            }
            *target = Some(*value);
            values[index] = *value;
        }
    }
    Ok(values)
}
fn packed<const N: usize>(values: Vec<[f32; N]>) -> Vec<u8> {
    values
        .into_iter()
        .flatten()
        .flat_map(f32::to_le_bytes)
        .collect()
}
fn new_attribute<'data, const N: usize>(
    doc: &mut RawDocument<'static>,
    name: &str,
    type_: r::AttrType,
    domain: r::Domain,
    count: usize,
    changes: &[(usize, &Mesh<'data>, &MeshProjection)],
    get: impl for<'view> Fn(&'view Mesh<'data>) -> &'view [[f32; N]],
) -> Result<r::AttrLayer, WriteError> {
    let mut values = vec![[0.; N]; count];
    let mut assigned = vec![false; count];
    for (_, mesh, map) in changes {
        let input = get(mesh);
        if input.len() != map.vertices.len() {
            return Err(WriteError::invalid(
                "new projected attribute must cover every draw view",
            ));
        }
        for (source, value) in map.vertices.iter().zip(input) {
            let index = domain_index(domain, *source)
                .map_err(|_| WriteError::invalid("new attribute domain"))?;
            if assigned[index] && values[index] != *value {
                return Err(WriteError::invalid(
                    "conflicting edits to repeated source-domain value",
                ));
            }
            values[index] = *value;
            assigned[index] = true;
        }
    }
    Ok(r::AttrLayer {
        name: text(doc, name)?,
        domain,
        type_,
        data: bytes(doc, &packed(values))?,
        ..Default::default()
    })
}
pub(in crate::model::wem::semantic) fn rebuild(
    model: &Model<'_>,
    doc: &mut RawDocument<'static>,
) -> Result<Vec<r::Mesh<r::V2>>, WriteError> {
    let native = model
        .native
        .as_ref()
        .ok_or_else(|| WriteError::invalid("projection provenance"))?;
    let source = native.document.raw();
    let root = native
        .document
        .root::<r::DocumentRecord<r::V1>>()
        .map_err(|_| WriteError::invalid("native root"))?;
    let graph = rows::<r::ModelRecord<r::V9>>(source, root.models.cast())
        .map_err(|_| WriteError::invalid("source model revision"))?
        .into_iter()
        .nth(native.model as usize)
        .ok_or_else(|| WriteError::invalid("native model selection"))?;
    let mut meshes =
        old_meshes(source, graph.meshes).map_err(|_| WriteError::invalid("source geometry"))?;
    for (index, mesh) in meshes.iter_mut().enumerate() {
        let (expected, expected_maps) = read_mesh(source, mesh.clone(), index as u32)
            .map_err(|_| WriteError::invalid("source projection"))?;
        let changes = model
            .meshes
            .iter()
            .zip(&native.mesh_projections)
            .enumerate()
            .filter(|(_, (_, map))| map.mesh as usize == index)
            .map(|(i, (mesh, map))| (i, mesh, map))
            .collect::<Vec<_>>();
        if changes.len() != expected.len() {
            return Err(WriteError::invalid(
                "projected draw section cardinality changed",
            ));
        }
        for ((_, value, map), (old, old_map)) in
            changes.iter().zip(expected.iter().zip(&expected_maps))
        {
            if *map != old_map
                || value.triangles.as_ref() != old.triangles.as_ref()
                || value.positions.len() != map.vertices.len()
            {
                return Err(WriteError::invalid(
                    "projected topology changed; edit polygon records or construct a generic model",
                ));
            }
        }
        let mut layers = rows::<r::AttrLayer>(source, mesh.attributes.layers.cast())
            .map_err(|_| WriteError::invalid("source attributes"))?;
        for layer in &mut layers {
            let name = source
                .string(layer.name)
                .map_err(|_| WriteError::invalid("source attribute name"))?
                .unwrap_or_default();
            let data = match name {
                "section" => continue,
                "position" => packed(attribute_values(source, layer, &changes, |m| &m.positions)?),
                "normal" => packed(attribute_values(source, layer, &changes, |m| &m.normals)?),
                "tangent" => packed(attribute_values(source, layer, &changes, |m| &m.tangents)?),
                "color" => packed(attribute_values(source, layer, &changes, |m| &m.colors)?),
                name if name.starts_with("uv") => {
                    let uv = name[2..]
                        .parse::<usize>()
                        .map_err(|_| WriteError::invalid("UV set name"))?;
                    packed(attribute_values(source, layer, &changes, |m| {
                        m.uv_sets.get(uv).map(|v| v.as_ref()).unwrap_or(&[])
                    })?)
                }
                _ => return Err(WriteError::invalid("unmapped source attribute")),
            };
            layer.data = editing::retain_flags(bytes(doc, &data)?, layer.data);
        }
        let domain = if changes
            .iter()
            .any(|(_, _, map)| map.vertices.iter().any(|source| source.corner != u32::MAX))
        {
            r::Domain::Halfedge
        } else {
            r::Domain::Vertex
        };
        let count = if domain == r::Domain::Vertex {
            mesh.face_set.vertex_count as usize
        } else {
            rows::<r::U32>(source, mesh.face_set.corner_vertex.cast())
                .map_err(|_| WriteError::invalid("source corners"))?
                .len()
        };
        for name in ["normal", "tangent", "color"] {
            if layers
                .iter()
                .any(|layer| doc.string(layer.name).ok().flatten() == Some(name))
            {
                continue;
            }
            let present = changes.iter().any(|(_, m, _)| match name {
                "normal" => !m.normals.is_empty(),
                "tangent" => !m.tangents.is_empty(),
                _ => !m.colors.is_empty(),
            });
            if !present {
                continue;
            }
            let layer = match name {
                "normal" => new_attribute(
                    doc,
                    name,
                    r::AttrType::F32x3,
                    domain,
                    count,
                    &changes,
                    |m| &m.normals,
                )?,
                "tangent" => new_attribute(
                    doc,
                    name,
                    r::AttrType::F32x4,
                    domain,
                    count,
                    &changes,
                    |m| &m.tangents,
                )?,
                _ => new_attribute(
                    doc,
                    name,
                    r::AttrType::F32x4,
                    domain,
                    count,
                    &changes,
                    |m| &m.colors,
                )?,
            };
            layers.push(layer);
        }
        let uv_count = changes
            .iter()
            .map(|(_, m, _)| m.uv_sets.len())
            .max()
            .unwrap_or(0);
        for uv in 0..uv_count {
            let name = format!("uv{uv}");
            if layers
                .iter()
                .any(|layer| doc.string(layer.name).ok().flatten() == Some(name.as_str()))
            {
                continue;
            }
            if changes
                .iter()
                .all(|(_, m, _)| m.uv_sets.get(uv).is_none_or(|values| values.is_empty()))
            {
                continue;
            }
            layers.push(new_attribute(
                doc,
                &name,
                r::AttrType::F32x2,
                domain,
                count,
                &changes,
                |m| m.uv_sets.get(uv).map(|v| v.as_ref()).unwrap_or(&[]),
            )?);
        }
        mesh.attributes.layers =
            editing::retain_flags(put(doc, &layers)?.cast(), mesh.attributes.layers);
        let mut sections = rows::<r::MeshSection>(source, mesh.sections.cast())
            .map_err(|_| WriteError::invalid("source sections"))?;
        for (semantic_index, value, map) in &changes {
            if let Some(section) = sections.get_mut(map.section as usize) {
                section.name = editing::retain_flags(text(doc, &value.name)?, section.name);
                section.material_slot =
                    editing::material_slot(model, *semantic_index, value.material_slot)?;
                section.selection_group = u16::try_from(value.selection_group)
                    .map_err(|_| WriteError::invalid("selection group"))?;
                section.bounds = value.bounds.clone();
                if section.rigid_node.value.is_some() {
                    let rigid = value
                        .influences
                        .first()
                        .and_then(|v| (v.len() == 1 && v[0].weight == 1.0).then_some(v[0].node))
                        .filter(|node| {
                            value
                                .influences
                                .iter()
                                .all(|v| v.len() == 1 && v[0].node == *node && v[0].weight == 1.0)
                        });
                    section.rigid_node.value = rigid;
                }
            } else if !sections.is_empty() {
                return Err(WriteError::invalid("source section address"));
            }
        }
        mesh.sections = editing::retain_flags(put(doc, &sections)?.cast(), mesh.sections);
        if let Some((_, first, _)) = changes.first() {
            mesh.name = editing::retain_flags(text(doc, &first.name)?, mesh.name);
            mesh.lod_level = first.lod;
            mesh.bounds = first.bounds.clone();
            if changes.iter().any(|(_, m, _)| m.lod != first.lod) {
                return Err(WriteError::invalid(
                    "conflicting projected level of detail edits",
                ));
            }
            for (_, value, _) in &changes {
                for axis in 0..3 {
                    mesh.bounds.minimum[axis] =
                        mesh.bounds.minimum[axis].min(value.bounds.minimum[axis]);
                    mesh.bounds.maximum[axis] =
                        mesh.bounds.maximum[axis].max(value.bounds.maximum[axis]);
                }
                mesh.bounds.sphere_radius =
                    mesh.bounds.sphere_radius.max(value.bounds.sphere_radius);
            }
        }
        let mut skin = vec![None; mesh.face_set.vertex_count as usize];
        let mut has_skin = false;
        for (_, value, map) in &changes {
            has_skin |= !value.influences.is_empty();
            if !value.influences.is_empty() && value.influences.len() != map.vertices.len() {
                return Err(WriteError::invalid("projected skin cardinality"));
            }
            for (projected, source) in map.vertices.iter().enumerate() {
                let influences = value.influences.get(projected).cloned().unwrap_or_default();
                let index = source.vertex as usize;
                if skin[index].as_ref().is_some_and(|old| old != &influences) {
                    return Err(WriteError::invalid(
                        "conflicting skin edits to repeated source vertex",
                    ));
                }
                skin[index] = Some(influences);
            }
        }
        if !has_skin {
            mesh.skin.offsets = editing::retain_flags(Ref::default(), mesh.skin.offsets);
            mesh.skin.influences = editing::retain_flags(Ref::default(), mesh.skin.influences);
            for section in &mut sections {
                section.rigid_node.value = None;
            }
            mesh.sections = editing::retain_flags(put(doc, &sections)?.cast(), mesh.sections);
        }
        if has_skin {
            let offsets = rows::<r::U32>(source, mesh.skin.offsets.cast())
                .map_err(|_| WriteError::invalid("source skin"))?;
            let old = rows::<r::Influence>(source, mesh.skin.influences.cast())
                .map_err(|_| WriteError::invalid("source skin"))?;
            let mut records = Vec::new();
            let mut boundaries = vec![0];
            for (vertex, value) in skin.into_iter().enumerate() {
                if let Some(value) = value {
                    records.extend(value.into_iter().map(|v| r::Influence {
                        bone: v.node,
                        weight: v.weight,
                    }));
                } else if offsets.len() == mesh.face_set.vertex_count as usize + 1 {
                    let a = offsets[vertex].value as usize;
                    let b = offsets[vertex + 1].value as usize;
                    records.extend_from_slice(
                        old.get(a..b)
                            .ok_or_else(|| WriteError::invalid("source skin boundaries"))?,
                    );
                }
                boundaries.push(records.len() as u32);
            }
            mesh.skin.offsets = editing::retain_flags(words(doc, &boundaries)?, mesh.skin.offsets);
            mesh.skin.influences =
                editing::retain_flags(put(doc, &records)?.cast(), mesh.skin.influences);
        }
    }
    Ok(meshes)
}
