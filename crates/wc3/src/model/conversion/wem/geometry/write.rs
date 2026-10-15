//! Rebuild polygon and authoring tables from their independently editable logical owners.
use super::super::{attributes, metadata, text};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::{LayerData, Wem};
use crate::model::semantic::{
    AttributeDomain, AttributeSemantic, AttributeValues, Bounds, MaterialSlotId, Mesh, NodeId,
    Topology,
};
use crate::model::wem::{records, RawDocument, Record, RecordLayout, Ref, Write};
use std::collections::HashMap;

fn error(value: crate::model::wem::WriteError) -> Error {
    Error::new("wem.geometry", value.to_string())
}
fn append<T: Record + RecordLayout + Write>(
    document: &mut RawDocument<'_>,
    rows: &[T],
) -> Result<Ref<T>, Error> {
    if rows.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(rows.len())
        .map_err(|_| Error::new("wem.geometry", "table exceeds native count"))?;
    let section = document.append(rows).map_err(error)?;
    Ok(Ref::from_section(count, section.index()))
}
fn words(
    document: &mut RawDocument<'_>,
    values: impl IntoIterator<Item = u32>,
) -> Result<Ref<records::targets::U32>, Error> {
    Ok(append(
        document,
        &values
            .into_iter()
            .map(|value| records::U32 { value })
            .collect::<Vec<_>>(),
    )?
    .cast())
}
fn bounds(value: Bounds) -> records::Extent {
    records::Extent {
        minimum: value.minimum,
        maximum: value.maximum,
        sphere_radius: value.sphere_radius,
    }
}
fn layer<'view, 'data: 'view>(
    document: &mut RawDocument<'view>,
    name: &str,
    domain: records::Domain,
    values: &AttributeValues<'data>,
) -> Result<records::AttrLayer, Error> {
    let (type_, data) = attributes::write(document, values)?;
    Ok(records::AttrLayer {
        name: text::write_bytes(document, name.as_bytes())?,
        domain,
        type_,
        data,
        storage: Default::default(),
    })
}
fn semantic(value: &AttributeSemantic<'_>) -> String {
    match value {
        AttributeSemantic::LightmapCoordinates => "lightmap_coordinates".into(),
        AttributeSemantic::SurfaceControls => "surface_controls".into(),
        AttributeSemantic::Normal => "normal".into(),
        AttributeSemantic::Tangent => "tangent".into(),
        AttributeSemantic::TangentVector => "tangent_vector".into(),
        AttributeSemantic::Binormal => "binormal".into(),
        AttributeSemantic::ClothVertex => "cloth_vertex".into(),
        AttributeSemantic::CollisionTriangleControl => "collision_triangle_control".into(),
        AttributeSemantic::Color(index) => {
            if *index == 0 {
                "color".into()
            } else {
                format!("color{index}")
            }
        }
        AttributeSemantic::TexCoord(index) => format!("uv{index}"),
        AttributeSemantic::Material => "material".into(),
        AttributeSemantic::Named(name) => name.to_string(),
    }
}
pub(in crate::model::conversion::wem) fn write<'view, 'data: 'view>(
    document: &mut RawDocument<'view>,
    mesh: &Mesh<'data, Wem>,
    nodes: &HashMap<NodeId, u32>,
    slots: &HashMap<MaterialSlotId, u32>,
) -> Result<records::Mesh<records::V2>, Error> {
    let geometry = &mesh.geometry;
    let (valences, corners) = match &geometry.topology {
        Topology::Polygons { offsets, corners } => (
            offsets
                .iter()
                .zip(offsets.iter().skip(1))
                .map(|(a, b)| b - a)
                .collect::<Vec<_>>(),
            corners.iter().collect::<Vec<_>>(),
        ),
        Topology::Triangles(triangles) => (
            vec![3; triangles.len()],
            triangles.iter().flatten().collect::<Vec<_>>(),
        ),
        _ => {
            return Err(Error::new(
                "wem.geometry.topology",
                "WEM stores polygon loops",
            ));
        }
    };
    let mut layers = Vec::new();
    layers.push(layer(
        document,
        "position",
        records::Domain::Vertex,
        &AttributeValues::Vector3(geometry.positions.clone()),
    )?);
    if !geometry.normals.is_empty() {
        layers.push(layer(
            document,
            "normal",
            records::Domain::Vertex,
            &AttributeValues::Vector3(geometry.normals.clone()),
        )?);
    }
    if !geometry.tangents.is_empty() {
        layers.push(layer(
            document,
            "tangent",
            records::Domain::Vertex,
            &AttributeValues::Vector4(geometry.tangents.clone()),
        )?);
    }
    if !geometry.colors.is_empty() {
        layers.push(layer(
            document,
            "color",
            records::Domain::Vertex,
            &AttributeValues::Vector4(geometry.colors.clone()),
        )?);
    }
    for (index, values) in geometry.texture_coordinates.iter().enumerate() {
        if !values.is_empty() {
            layers.push(layer(
                document,
                &format!("uv{index}"),
                records::Domain::Vertex,
                &AttributeValues::Vector2(values.clone()),
            )?);
        }
    }
    for value in geometry.layers() {
        match &value.data {
            LayerData::Standard(attribute) => {
                let domain = match attribute.domain {
                    AttributeDomain::Vertex => records::Domain::Vertex,
                    AttributeDomain::Corner => records::Domain::Halfedge,
                    AttributeDomain::Face => records::Domain::Face,
                    AttributeDomain::Mesh => records::Domain::Mesh,
                    AttributeDomain::ConnectivityFace(_)
                    | AttributeDomain::ConnectivityCorner(_)
                    | AttributeDomain::Pool => {
                        return Err(Error::new(
                            "wem.geometry.attributes",
                            "indexed attribute pools require explicit translation",
                        ))
                    }
                };
                layers.push(layer(
                    document,
                    &semantic(&attribute.semantic),
                    domain,
                    &attribute.values,
                )?);
            }
            LayerData::Edge(attribute) => layers.push(layer(
                document,
                &attribute.name,
                records::Domain::Edge,
                &attribute.values,
            )?),
        }
    }
    let sections = geometry
        .sections()
        .enumerate()
        .map(|(index, (id, _))| (id, index as u32))
        .collect::<HashMap<_, _>>();
    if !sections.is_empty() {
        let section_indices = sections.clone();
        let assignments = geometry
            .face_sections()
            .clone()
            .map(move |id| section_indices[&id]);
        layers.push(layer(
            document,
            "section",
            records::Domain::Face,
            &AttributeValues::Unsigned(assignments),
        )?);
    }
    let mut section_rows = Vec::new();
    for (_, section) in geometry.sections() {
        let rigid_node = section
            .rigid_node
            .map(|id| {
                nodes
                    .get(&id)
                    .copied()
                    .ok_or_else(|| Error::new("wem.geometry.rigid_node", "node is missing"))
            })
            .transpose()?;
        section_rows.push(records::MeshSection {
            name: text::write_bytes(document, section.name.as_bytes())?,
            material_slot: section
                .slot
                .map(|id| {
                    slots.get(&id).copied().ok_or_else(|| {
                        Error::new("wem.geometry.slot", "appearance slot is missing")
                    })
                })
                .transpose()?
                .unwrap_or(u32::MAX),
            profiles: records::ProfileMask::from(section.availability.bits()),
            rigid_node: rigid_node.into(),
            selection_group: section.selection_group,
            flags: records::SectionFlags::from_bits_retain(section.flags.bits()),
            bounds: bounds(section.bounds),
            native: metadata::write(document, &section.metadata)?,
        });
    }
    let mut influence_offsets = Vec::new();
    let mut influences = Vec::new();
    if !geometry.influences.is_empty() {
        influence_offsets.push(0);
        for row in geometry.influences.iter() {
            for influence in row {
                influences.push(records::Influence {
                    bone: *nodes
                        .get(&influence.node)
                        .ok_or_else(|| Error::new("wem.geometry.influence", "node is missing"))?,
                    weight: influence.weight,
                });
            }
            influence_offsets.push(u32::try_from(influences.len()).map_err(|_| {
                Error::new(
                    "wem.geometry.influence",
                    "influence count exceeds native domain",
                )
            })?);
        }
    }
    let splits = geometry
        .repair
        .splits
        .iter()
        .map(|value| records::VertexSplit {
            original: value.original,
            created: value.created,
        })
        .collect::<Vec<_>>();
    let mut removed = Vec::new();
    for face in &geometry.repair.removed_faces {
        removed.push(records::FaceRecord {
            corners: words(document, face.corners.iter())?,
            section: face
                .section
                .map(|id| {
                    sections.get(&id).copied().ok_or_else(|| {
                        Error::new("wem.geometry.repair", "original section is missing")
                    })
                })
                .transpose()?
                .unwrap_or(u32::MAX),
            index: face.original_order,
        });
    }
    let mut triangle_offsets = Vec::new();
    let mut triangles = Vec::new();
    if !geometry.triangulation.is_empty() {
        triangle_offsets.push(0);
        for face in geometry.triangulation.iter() {
            triangles.extend(face.into_iter().flatten());
            triangle_offsets.push(u32::try_from(triangles.len()).map_err(|_| {
                Error::new(
                    "wem.geometry.triangulation",
                    "decomposition exceeds native count",
                )
            })?);
        }
    }
    Ok(records::Mesh {
        name: text::write_bytes(document, mesh.name.as_bytes())?,
        lod_level: mesh.controls.level_of_detail.unwrap_or(u32::MAX),
        face_set: records::FaceSet {
            vertex_count: u32::try_from(geometry.positions.len()).map_err(|_| {
                Error::new(
                    "wem.geometry.vertices",
                    "vertex count exceeds native domain",
                )
            })?,
            face_valence: words(document, valences)?,
            corner_vertex: words(document, corners.clone())?,
        },
        attributes: records::AttributeSet {
            domain_counts: [
                geometry.positions.len() as u32,
                corners.len() as u32,
                geometry.edge_count as u32,
                match &geometry.topology {
                    Topology::Polygons { offsets, .. } => offsets.len().saturating_sub(1) as u32,
                    Topology::Triangles(values) => values.len() as u32,
                    _ => 0,
                },
                1,
            ],
            layers: append(document, &layers)?.cast(),
        },
        skin: records::SkinBinding {
            offsets: words(document, influence_offsets)?,
            influences: append(document, &influences)?.cast(),
        },
        sections: append(document, &section_rows)?.cast(),
        repair_log: records::RepairLog {
            splits: append(document, &splits)?.cast(),
            dropped_faces: append(document, &removed)?.cast(),
        },
        bounds: bounds(mesh.bounds),
        triangulation: records::FaceTriangulation {
            offsets: words(document, triangle_offsets)?,
            vertices: words(document, triangles)?,
        },
    })
}
