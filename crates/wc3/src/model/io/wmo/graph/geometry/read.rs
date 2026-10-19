//! World-group fields borrowed directly into the static authoring vocabulary.
use super::bounds;
use super::face;
use crate::model::common::PackedArray;
use crate::model::conversion::{Context, Error};
use crate::model::formats::wmo::group::GroupChunk;
use crate::model::formats::wmo::records::{BatchMetadata, HeaderTailMetadata, TriangleFlags2};
use crate::model::formats::wmo::{Dialect, Group, GroupKey};
use crate::model::io::wmo::graph::draw::{
    bind_draw, ImportedDrawControls, StripWindow, TangentWindow,
};
use crate::model::io::{materialize, owned};
use crate::model::profiles::wmo::{Group as LogicalGroup, Lighting, WorldSystem};
use crate::model::profiles::Wmo;
use crate::model::{MaterialId, Mesh, MeshInstance, ModelBuilder, WmoGeometry};
use glam::Vec4;
pub(in crate::model::io::wmo::graph) fn read<'a, D: Dialect>(
    source: &'a Group<'_, D>,
    group_root: (GroupKey, &'a crate::model::formats::wmo::Root<'_, D>),
    builder: &mut ModelBuilder<'a, Wmo>,
    materials: &[MaterialId],
    world: &mut WorldSystem<'a>,
    owner: crate::model::SystemId,
    context: &mut Context,
) -> Result<crate::model::InstanceId, Error> {
    let (key, root) = group_root;
    let mut positions = PackedArray::new();
    let mut normals = PackedArray::new();
    let mut indices = PackedArray::new();
    let mut coordinates = Vec::new();
    let mut colors = Vec::new();
    let mut strip_indices = PackedArray::new();
    let mut tangent_vectors = PackedArray::new();
    for chunk in source.chunks() {
        match chunk {
            GroupChunk::Positions(values) => {
                positions = PackedArray::parse_bytes(values.as_bytes())
                    .map_err(|error| Error::new("world.positions", error.to_string()))?
            }
            GroupChunk::Normals(values) => {
                normals = PackedArray::parse_bytes(values.as_bytes())
                    .map_err(|error| Error::new("world.normals", error.to_string()))?
            }
            GroupChunk::TextureCoordinates(values) => coordinates.push(
                PackedArray::parse_bytes(values.as_bytes())
                    .map_err(|error| Error::new("world.coordinates", error.to_string()))?,
            ),
            GroupChunk::VertexColors(values) | GroupChunk::VertexColors2(values) => {
                colors.push(materialize(values.len(), move |index| {
                    let [b, g, r, a] = values
                        .get(index)
                        .expect("checked color index")
                        .expect("validated color record");
                    Vec4::from_array([r, g, b, a].map(|component| f32::from(component) / 255.))
                }))
            }
            GroupChunk::StripIndices(rows) => {
                strip_indices = materialize(rows.len(), move |index| {
                    u32::from(rows.get(index).unwrap().unwrap())
                })
            }
            GroupChunk::Tangents(rows) => {
                tangent_vectors = PackedArray::parse_bytes(rows.tangents.as_bytes())
                    .map_err(|error| Error::new("world.tangents", error.to_string()))?
            }
            GroupChunk::TangentsUnresolved(bytes) => {
                let count = source
                    .chunks()
                    .iter()
                    .find_map(|chunk| {
                        if let GroupChunk::Batches(rows) = chunk {
                            Some(rows.len())
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);
                let prefix = count
                    .checked_mul(2)
                    .ok_or_else(|| Error::new("world.tangents", "batch map length overflows"))?;
                let bytes = bytes.get(prefix..).ok_or_else(|| {
                    Error::new("world.tangents", "batch map exceeds tangent payload")
                })?;
                tangent_vectors = PackedArray::parse_bytes(bytes)
                    .map_err(|error| Error::new("world.tangents", error.to_string()))?;
            }
            GroupChunk::Indices(values) => {
                indices = materialize(values.len(), move |index| {
                    u32::from(
                        values
                            .get(index)
                            .expect("checked index")
                            .expect("validated index record"),
                    )
                })
            }
            GroupChunk::Indices32(values) => {
                indices = materialize(values.len(), move |index| {
                    values
                        .get(index)
                        .expect("checked index")
                        .expect("validated index record")
                })
            }
            _ => {}
        }
    }
    if !indices.len().is_multiple_of(3) {
        return Err(Error::new(
            "world.triangles",
            "indices do not form complete triangles",
        ));
    }
    let triangles = materialize(indices.len() / 3, move |index| {
        [
            indices.get(index * 3).unwrap(),
            indices.get(index * 3 + 1).unwrap(),
            indices.get(index * 3 + 2).unwrap(),
        ]
    });
    let mut face_controls = Vec::new();
    for chunk in source.chunks() {
        match chunk {
            GroupChunk::Triangles(rows) => {
                for row in rows.iter() {
                    let row = row.map_err(|error| Error::new("world.face", error.to_string()))?;
                    face_controls.push(face(
                        TriangleFlags2(u16::from(row.flags.bits())),
                        u16::from(row.material),
                        row.material == u8::MAX,
                        materials,
                    )?);
                }
            }
            GroupChunk::Triangles2(rows) => {
                for row in rows.iter() {
                    let row = row.map_err(|error| Error::new("world.face", error.to_string()))?;
                    face_controls.push(face(
                        row.flags,
                        row.material,
                        row.material == u16::MAX,
                        materials,
                    )?);
                }
            }
            _ => {}
        }
    }
    for index in super::super::spatial_queries::faces(source.chunks().iter(), context)? {
        face_controls
            .get_mut(index)
            .ok_or_else(|| Error::new("world.query", "spatial query references absent face"))?
            .spatial_queries = true;
    }
    let query_start = source
        .chunks()
        .iter()
        .find_map(|chunk| match chunk {
            GroupChunk::QueryGeometry(query) => Some(query.first_face as usize),
            _ => None,
        })
        .unwrap_or(0);
    for chunk in source.chunks() {
        if let GroupChunk::GroundTypes(rows) = chunk {
            for (offset, value) in rows.iter().enumerate() {
                let face = face_controls.get_mut(query_start + offset).ok_or_else(|| {
                    Error::new("world.ground_types", "query range exceeds geometry")
                })?;
                face.ground_type = Some(
                    value.map_err(|error| Error::new("world.ground_types", error.to_string()))?,
                );
            }
        }
    }
    let geometry = WmoGeometry::triangles(positions, triangles)?
        .with_strip_indices(strip_indices)?
        .with_tangent_vectors(tangent_vectors)?
        .with_normals(normals)?
        .with_texture_coordinates(coordinates)?
        .with_color_sets(colors)?
        .with_triangle_controls(owned(face_controls))?;
    let header = source.header();
    let mut mesh = Mesh::<Wmo>::new(geometry);
    mesh.name = super::relationships::name(root, source.header().name_offset)?;
    mesh.bounds = bounds([header.bounds_minimum, header.bounds_maximum]);
    let mesh_id = builder.add_mesh(mesh)?;
    let node = builder.add_named_node(format!("Group {} LOD {}", key.index, key.lod))?;
    let mut draws = Vec::new();
    let mut ordinal = 0usize;
    for chunk in source.chunks() {
        if let GroupChunk::Batches(values) = chunk {
            for batch in values.iter() {
                let batch = batch.map_err(|error| Error::new("world.batch", error.to_string()))?;
                if batch.start_index % 3 != 0 || batch.index_count % 3 != 0 {
                    return Err(Error::new(
                        "world.batch",
                        "batch does not cover complete triangles",
                    ));
                }
                let material = *materials
                    .get(usize::from(batch.material_index()))
                    .ok_or_else(|| {
                        Error::new("world.material", "batch references absent material")
                    })?;
                let lighting = if ordinal < usize::from(header.transition_batch_count) {
                    Lighting::Transition
                } else if ordinal
                    < usize::from(header.transition_batch_count)
                        + usize::from(header.interior_batch_count)
                {
                    Lighting::Interior
                } else {
                    Lighting::Exterior
                };
                let mut unknown = batch.flags;
                unknown.set_large_material(false);
                unknown.set_extended_vertices(false);
                draws.push(bind_draw(
                    builder,
                    mesh_id,
                    batch.start_index / 3,
                    u32::from(batch.index_count) / 3,
                    material,
                    Some(ImportedDrawControls {
                        lighting,
                        vertex_window: Some([batch.vertex_range().0, batch.vertex_range().1]),
                        strip: source
                            .chunks()
                            .iter()
                            .find_map(|chunk| {
                                if let GroupChunk::BatchOverrides(rows) = chunk {
                                    rows.get(ordinal)
                                } else {
                                    None
                                }
                            })
                            .transpose()
                            .map_err(|error| Error::new("world.strips", error.to_string()))?
                            .map(|row| StripWindow {
                                first: row.start_index,
                                count: u32::from(row.index_count),
                                control: row.padding,
                            }),
                        tangents: tangent_start(source, ordinal)?.map(|first| {
                            let (minimum, maximum) = batch.vertex_range();
                            TangentWindow {
                                first_vertex: minimum,
                                vertex_count: maximum - minimum + 1,
                                first_tangent: u32::from(first),
                            }
                        }),
                        bounds: batch
                            .prefix
                            .culling_bounds()
                            .map(|values| bounds(values.map(|corner| corner.map(f32::from)))),
                        supplemental: batch.prefix.supplemental(),
                        unclassified: unknown.bits().into(),
                        ..Default::default()
                    }),
                )?);
                ordinal += 1;
            }
        }
    }
    let flags = header.flags;
    for chunk in source.chunks() {
        if let GroupChunk::ShadowBatches(rows) = chunk {
            for row in rows.iter() {
                let row = row.map_err(|error| Error::new("world.shadow", error.to_string()))?;
                if row.start < 0 || row.count < 0 || row.start % 3 != 0 || row.count % 3 != 0 {
                    return Err(Error::new(
                        "world.shadow",
                        "shadow pass must cover whole triangles",
                    ));
                }
                let material = if row.flags.large_material() {
                    row.large_material
                } else {
                    u16::from(row.material)
                };
                let material = *materials.get(usize::from(material)).ok_or_else(|| {
                    Error::new("world.shadow.material", "shadow pass material is absent")
                })?;
                let mut flags = row.flags;
                flags.set_large_material(false);
                draws.push(bind_draw(
                    builder,
                    mesh_id,
                    row.start as u32 / 3,
                    row.count as u32 / 3,
                    material,
                    Some(ImportedDrawControls {
                        pass: crate::model::profiles::wmo::Pass::Shadow {
                            controls: std::array::from_fn(|index| {
                                u16::from_le_bytes([
                                    row.reserved_prefix[index * 2],
                                    row.reserved_prefix[index * 2 + 1],
                                ])
                            }),
                            supplemental: std::array::from_fn(|index| {
                                u16::from_le_bytes([
                                    row.reserved_middle[index * 2],
                                    row.reserved_middle[index * 2 + 1],
                                ])
                            }),
                        },
                        unclassified: flags.bits().into(),
                        ..Default::default()
                    }),
                )?);
            }
        }
    }
    let mut region = crate::model::profiles::wmo::RegionControls {
        description: super::relationships::name(root, header.descriptive_name_offset)?,
        area_id: header.unique_id,
        batch_control: header.reserved_batch_count,
        supplemental_controls: header.tail.control().into_iter().collect(),
        options: super::super::group_flags::read(flags, header.flags2.bits()),
        ..Default::default()
    };
    for chunk in source.chunks() {
        match chunk {
            GroupChunk::TerrainCutPlanes(rows) => {
                region.terrain_cut_planes = PackedArray::from_storage(
                    (materialize(rows.len(), move |index| {
                        let row = rows
                            .get(index)
                            .expect("checked plane index")
                            .expect("validated plane");
                        [row.normal[0], row.normal[1], row.normal[2], row.distance]
                    }))
                    .into_storage(),
                )
            }
            GroupChunk::AmbientOverride(color) => {
                region.ambient_override = Some(super::rgba(*color))
            }
            GroupChunk::QueryGeometry(query) => {
                region.detail_doodad_key = query.detail_doodad_key()
            }
            _ => {}
        }
    }
    if let Some((first, next)) = header.tail.split_links() {
        let resolve = |value: i16| {
            if value == -1 {
                Ok(None)
            } else {
                u32::try_from(value)
                    .map(|index| {
                        Some(crate::model::profiles::wmo::GroupId {
                            system: owner,
                            index,
                        })
                    })
                    .map_err(|_| Error::new("world.group.split", "negative group selector"))
            }
        };
        region.parent_or_first_child = resolve(first)?;
        region.next_child = resolve(next)?;
    }
    let (draws, draw_materials): (Vec<_>, Vec<_>) = draws.into_iter().unzip();
    let instance_id = builder.add_instance_with_materials(
        MeshInstance {
            controls: region,
            node,
            mesh: mesh_id,
            skin: None,
            draws,
            lod: Some(key.lod as u32),
            selection_group: 0,
        },
        draw_materials,
    )?;
    while world.groups.len() <= key.index {
        world.groups.push(LogicalGroup::default());
    }
    world.groups[key.index].instances.push(instance_id);
    let _ = context;
    Ok(instance_id)
}

fn tangent_start<D: Dialect>(source: &Group<'_, D>, ordinal: usize) -> Result<Option<u16>, Error> {
    for chunk in source.chunks() {
        let first = match chunk {
            GroupChunk::Tangents(rows) => rows
                .first_tangent
                .get(ordinal)
                .transpose()
                .map_err(|error| Error::new("world.tangents", error.to_string()))?,
            GroupChunk::TangentsUnresolved(bytes) => {
                let offset = ordinal
                    .checked_mul(2)
                    .ok_or_else(|| Error::new("world.tangents", "batch selection overflows"))?;
                let bytes = bytes
                    .get(offset..offset + 2)
                    .ok_or_else(|| Error::new("world.tangents", "batch map is truncated"))?;
                Some(u16::from_le_bytes([bytes[0], bytes[1]]))
            }
            _ => continue,
        };
        return Ok(first.filter(|first| *first != u16::MAX));
    }
    Ok(None)
}
