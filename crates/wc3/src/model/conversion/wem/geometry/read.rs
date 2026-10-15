//! Polygon loops, node weights, groups, and repair history become their actual logical owners.
use super::super::{attributes, metadata, text};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::{
    Availability, EdgeAttribute, Geometry, LayerData, RemovedFace, Section, SectionFlags,
    VertexSplit, Wem,
};
use crate::model::semantic::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, Bounds, GeometryParts,
    GeometrySchema, MaterialSlotId, Mesh, NodeId, NodeInfluence, Topology, Values,
};
use crate::model::wem::{records, RawDocument, Ref};

pub(in crate::model::conversion::wem) trait TriangulationField {
    fn references(&self) -> Option<&records::FaceTriangulation>;
}
impl TriangulationField for () {
    fn references(&self) -> Option<&records::FaceTriangulation> {
        None
    }
}
impl TriangulationField for records::FaceTriangulation {
    fn references(&self) -> Option<&records::FaceTriangulation> {
        Some(self)
    }
}

fn error(value: crate::model::wem::ReadError) -> Error {
    Error::new("wem.geometry", value.to_string())
}
fn words<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::U32>,
) -> Result<Values<'a, u32>, Error> {
    let rows = document
        .resolve::<records::U32>(reference.cast())
        .map_err(error)?;
    for row in rows.iter() {
        row.map_err(error)?;
    }
    Ok(Values::view(rows.len(), move |index| {
        rows.get(index).unwrap().unwrap().value
    }))
}
fn bounds(value: records::Extent) -> Bounds {
    Bounds {
        minimum: value.minimum,
        maximum: value.maximum,
        sphere_radius: value.sphere_radius,
    }
}

pub(in crate::model::conversion::wem) fn read<'a, D: records::MeshDialect>(
    document: &'a RawDocument<'_>,
    source: &records::Mesh<D>,
    nodes: &[NodeId],
    slots: &[MaterialSlotId],
) -> Result<Mesh<'a, Wem>, Error>
where
    D::Triangulation: TriangulationField,
{
    let valences = words(document, source.face_set.face_valence)?;
    let corners = words(document, source.face_set.corner_vertex)?;
    let mut offsets = Vec::with_capacity(valences.len() + 1);
    offsets.push(0u32);
    for count in valences.iter() {
        offsets.push(
            offsets
                .last()
                .unwrap()
                .checked_add(count)
                .ok_or_else(|| Error::new("wem.geometry.faces", "corner count overflows"))?,
        );
    }
    if offsets.last().copied().unwrap_or(0) as usize != corners.len() {
        return Err(Error::new(
            "wem.geometry.faces",
            "valence sum differs from corner count",
        ));
    }
    let counts = source.attributes.domain_counts.map(|v| v as usize);
    if counts[0] != source.face_set.vertex_count as usize
        || counts[1] != corners.len()
        || counts[3] != valences.len()
        || counts[4] != 1
    {
        return Err(Error::new(
            "wem.geometry.domains",
            "domain sizes disagree with polygon topology",
        ));
    }
    let topology = Topology::Polygons {
        offsets: Values::owned(offsets),
        corners,
    };
    let mut data = GeometryParts {
        topology,
        ..Default::default()
    };
    let mut extra = Vec::new();
    let mut edge = Vec::new();
    let mut assignments = None;
    let mut primary = std::collections::BTreeSet::new();
    for layer in document
        .resolve::<records::AttrLayer>(source.attributes.layers.cast())
        .map_err(error)?
        .iter()
    {
        let layer = layer.map_err(error)?;
        let name = text::read(document, layer.name)?;
        let (domain, count) = match layer.domain {
            records::Domain::Vertex => (Some(AttributeDomain::Vertex), counts[0]),
            records::Domain::Halfedge => (Some(AttributeDomain::Corner), counts[1]),
            records::Domain::Edge => (None, counts[2]),
            records::Domain::Face => (Some(AttributeDomain::Face), counts[3]),
            records::Domain::Mesh => (Some(AttributeDomain::Mesh), counts[4]),
            _ => {
                return Err(Error::new(
                    "wem.geometry.domain",
                    "unknown attribute domain",
                ));
            }
        };
        let values = attributes::read(document, &layer, count)?;
        if domain == Some(AttributeDomain::Face) && name == "section" {
            let AttributeValues::Unsigned(values) = values else {
                return Err(Error::new(
                    "wem.geometry.section",
                    "section assignments require unsigned integers",
                ));
            };
            if assignments.replace(values).is_some() {
                return Err(Error::new(
                    "wem.geometry.section",
                    "multiple section-assignment authorities",
                ));
            }
            continue;
        }
        let mut consumed = false;
        if domain == Some(AttributeDomain::Vertex) {
            match (name.as_ref(), &values) {
                ("position", AttributeValues::Vector3(v)) => {
                    data.positions = v.clone();
                    consumed = true;
                }
                ("normal", AttributeValues::Vector3(v)) => {
                    data.normals = v.clone();
                    consumed = true;
                }
                ("tangent", AttributeValues::Vector4(v)) => {
                    data.tangents = v.clone();
                    consumed = true;
                }
                ("color", AttributeValues::Vector4(v)) => {
                    data.colors = v.clone();
                    consumed = true;
                }
                (name, AttributeValues::Vector2(v)) if name.starts_with("uv") => {
                    if let Ok(index) = name[2..].parse::<usize>() {
                        if index > u16::MAX as usize {
                            return Err(Error::new(
                                "wem.geometry.uv",
                                "UV slot identity is too large",
                            ));
                        }
                        data.texture_coordinates
                            .resize_with(index + 1, Values::default);
                        data.texture_coordinates[index] = v.clone();
                        consumed = true;
                    }
                }
                _ => {}
            }
        }
        if consumed {
            if !primary.insert(name.into_owned()) {
                return Err(Error::new(
                    "wem.geometry.attribute",
                    "multiple primary stream authorities",
                ));
            }
        } else if let Some(domain) = domain {
            extra.push(Attribute {
                semantic: AttributeSemantic::Named(name),
                domain,
                values,
            });
        } else {
            edge.push(LayerData::Edge(EdgeAttribute { name, values }));
        }
    }
    data.attributes = extra;
    let mut geometry = Geometry::new(data.into_data()?)?;
    geometry.edge_count = counts[2];
    for layer in edge {
        geometry.add_layer(layer);
    }
    let mut sections = Vec::new();
    for section in document
        .resolve::<records::MeshSection>(source.sections.cast())
        .map_err(error)?
        .iter()
    {
        let section = section.map_err(error)?;
        let slot = if section.material_slot == u32::MAX {
            None
        } else {
            Some(*slots.get(section.material_slot as usize).ok_or_else(|| {
                Error::new(
                    "wem.geometry.slot",
                    "section refers to missing appearance slot",
                )
            })?)
        };
        let rigid_node = section
            .rigid_node
            .value
            .map(|index| {
                nodes
                    .get(index as usize)
                    .copied()
                    .ok_or_else(|| Error::new("wem.geometry.rigid_node", "missing node"))
            })
            .transpose()?;
        sections.push(geometry.add_section(Section {
            name: text::read(document, section.name)?,
            availability: Availability::from_bits_retain(section.profiles.bits()),
            slot,
            rigid_node,
            selection_group: section.selection_group,
            flags: SectionFlags::from_bits_retain(section.flags.bits()),
            bounds: bounds(section.bounds),
            metadata: metadata::read(document, &section.native)?,
        })?);
    }
    if let Some(assignments) = assignments {
        for index in assignments.iter() {
            if index as usize >= sections.len() {
                return Err(Error::new(
                    "wem.geometry.section",
                    "face refers to missing polygon group",
                ));
            }
        }
        geometry.set_face_sections(assignments.map(move |index| sections[index as usize]))?;
    } else if !sections.is_empty() && !valences.is_empty() {
        return Err(Error::new(
            "wem.geometry.section",
            "polygon groups lack section assignment",
        ));
    }
    let offsets = words(document, source.skin.offsets)?;
    let influences = document
        .resolve::<records::Influence>(source.skin.influences.cast())
        .map_err(error)?;
    for influence in influences.iter() {
        let influence = influence.map_err(error)?;
        if nodes.get(influence.bone as usize).is_none() {
            return Err(Error::new("wem.geometry.influence", "missing node"));
        }
    }
    if !offsets.is_empty() {
        if offsets.len() != counts[0] + 1
            || offsets.get(0) != Some(0)
            || offsets.get(offsets.len() - 1) != Some(influences.len() as u32)
            || offsets
                .iter()
                .zip(offsets.iter().skip(1))
                .any(|(a, b)| a > b)
        {
            return Err(Error::new(
                "wem.geometry.skin",
                "invalid vertex influence boundaries",
            ));
        }
        let nodes = nodes.to_vec();
        geometry.influences = Values::view(counts[0], move |index| {
            (offsets.get(index).unwrap()..offsets.get(index + 1).unwrap())
                .map(|row| {
                    let value = influences.get(row as usize).unwrap().unwrap();
                    NodeInfluence {
                        node: nodes[value.bone as usize],
                        weight: value.weight,
                    }
                })
                .collect()
        });
    } else if !influences.is_empty() {
        return Err(Error::new(
            "wem.geometry.skin",
            "influences lack vertex boundaries",
        ));
    }
    let splits = document
        .resolve::<records::VertexSplit>(source.repair_log.splits.cast())
        .map_err(error)?;
    for value in splits.iter() {
        value.map_err(error)?;
    }
    geometry.repair.splits = Values::view(splits.len(), move |index| {
        let value = splits.get(index).unwrap().unwrap();
        VertexSplit {
            original: value.original,
            created: value.created,
        }
    });
    for face in document
        .resolve::<records::FaceRecord>(source.repair_log.dropped_faces.cast())
        .map_err(error)?
        .iter()
    {
        let face = face.map_err(error)?;
        let section = if face.section == u32::MAX {
            None
        } else {
            Some(
                geometry
                    .sections()
                    .nth(face.section as usize)
                    .ok_or_else(|| Error::new("wem.geometry.repair", "missing original section"))?
                    .0,
            )
        };
        geometry.repair.removed_faces.push(RemovedFace {
            corners: words(document, face.corners)?,
            original_order: face.index,
            section,
        });
    }
    let (offsets, triangles) = match source.triangulation.references() {
        Some(value) => (
            words(document, value.offsets)?,
            words(document, value.vertices)?,
        ),
        None => (Values::owned(Vec::new()), Values::owned(Vec::new())),
    };
    if !offsets.is_empty() {
        if offsets.len() != valences.len() + 1
            || offsets.get(0) != Some(0)
            || offsets.get(offsets.len() - 1) != Some(triangles.len() as u32)
            || offsets
                .iter()
                .zip(offsets.iter().skip(1))
                .any(|(a, b)| a > b)
            || (0..valences.len())
                .any(|i| (offsets.get(i + 1).unwrap() - offsets.get(i).unwrap()) % 3 != 0)
        {
            return Err(Error::new(
                "wem.geometry.triangulation",
                "invalid decomposition boundaries",
            ));
        }
        geometry.triangulation = Values::view(valences.len(), move |index| {
            (offsets.get(index).unwrap()..offsets.get(index + 1).unwrap())
                .step_by(3)
                .map(|row| std::array::from_fn(|c| triangles.get(row as usize + c).unwrap()))
                .collect()
        });
    } else if !triangles.is_empty() {
        return Err(Error::new(
            "wem.geometry.triangulation",
            "decompositions lack polygon boundaries",
        ));
    }
    geometry.validate()?;
    let mut mesh = Mesh::<Wem>::new(geometry);
    mesh.name = text::read(document, source.name)?;
    mesh.bounds = bounds(source.bounds.clone());
    mesh.controls.level_of_detail = (source.lod_level != u32::MAX).then_some(source.lod_level);
    Ok(mesh)
}
