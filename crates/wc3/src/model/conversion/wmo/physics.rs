//! Static collision surfaces rebuilt from active WEM triangle-mesh shapes.
use super::*;
use std::slice::from_ref;
use wem::InlineOption;
const IDENTITY: [[f32; 4]; 4] = [
    [1., 0., 0., 0.],
    [0., 1., 0., 0.],
    [0., 0., 1., 0.],
    [0., 0., 0., 1.],
];
pub(super) fn decorate<D: Dialect>(
    raw: &mut RawDocument<'static>,
    graph: &mut wr::ModelRecord<wr::V9>,
    source: &Bundle<'_, D>,
) -> Result<(), Error> {
    let mut nodes = raw
        .resolve::<wr::Node<wr::V19>>(graph.nodes.nodes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let mut bodies = Vec::new();
    let mut keys = BTreeSet::new();
    for index in 0..source.root.header().group_count {
        keys.insert(GroupKey {
            index: index as usize,
            lod: 0,
        });
    }
    for dependency in source
        .dependencies()
        .map_err(|e| failure("physics.dependencies", e))?
    {
        if let native::DependencyKind::Group { index, lod } = dependency.kind {
            keys.insert(GroupKey { index, lod });
        }
    }
    for key in keys {
        let group = source
            .group(key)
            .ok_or_else(|| Error::new("collision.group", "missing group"))?;
        let faces = group
            .chunks()
            .iter()
            .filter_map(|chunk| {
                if let GroupChunk::BspFaces(rows) = chunk {
                    Some(collect(rows, "collision.faces"))
                } else {
                    None
                }
            })
            .collect::<Result<Vec<_>, _>>()?
            .concat();
        if faces.is_empty() {
            continue;
        }
        let positions = group
            .chunks()
            .iter()
            .filter_map(|chunk| {
                if let GroupChunk::Positions(rows) = chunk {
                    Some(collect(rows, "collision.vertices"))
                } else {
                    None
                }
            })
            .collect::<Result<Vec<_>, _>>()?
            .concat();
        let mut indices = Vec::new();
        for chunk in group.chunks() {
            match chunk {
                GroupChunk::Indices(rows) => indices.extend(
                    collect(rows, "collision.indices")?
                        .into_iter()
                        .map(u32::from),
                ),
                GroupChunk::Indices32(rows) => indices.extend(collect(rows, "collision.indices")?),
                _ => {}
            }
        }
        let mut selected = Vec::new();
        for face in faces.into_iter().collect::<BTreeSet<_>>() {
            let first = usize::from(face) * 3;
            selected.extend_from_slice(
                indices.get(first..first + 3).ok_or_else(|| {
                    Error::new("collision.faces", "face index is outside geometry")
                })?,
            );
        }
        let shape = wr::PhysicsShape::<wr::V2> {
            kind: wr::PhysicsShapeKind::TriangleMesh,
            transform: IDENTITY,
            vertices: metadata::append(
                raw,
                &positions
                    .iter()
                    .map(|value| wr::Vector3 { value: *value })
                    .collect::<Vec<_>>(),
            )?,
            triangles: metadata::append(
                raw,
                &selected
                    .iter()
                    .map(|value| wr::U32 { value: *value })
                    .collect::<Vec<_>>(),
            )?,
            ..Default::default()
        };
        let node = nodes.len() as u32;
        nodes.push(wr::Node::<wr::V19> {
            name: metadata::string(raw, &format!("Group {} collision", key.index))?,
            parent: u32::MAX,
            kind: wr::NodeKind::Helper,
            local: wr::Transform {
                rotation: [0., 0., 0., 1.],
                scale: [1.; 3],
                ..Default::default()
            },
            native: metadata::bag(
                raw,
                &[
                    (
                        "wmo.collision.group".into(),
                        key.index as i64,
                        String::new(),
                    ),
                    ("wmo.collision.lod".into(), key.lod as i64, String::new()),
                ],
            )?,
            ..Default::default()
        });
        bodies.push(wr::PhysicsBody::<wr::V3> {
            id: bodies.len() as u32 + 1,
            node,
            motion: wr::BodyMotion::Static,
            shapes: metadata::append(raw, &[shape])?,
            inertia_scale: 1.,
            gravity_scale: 1.,
            wow: InlineOption::default(),
            ..Default::default()
        });
    }
    if !bodies.is_empty() {
        graph.nodes.nodes = metadata::append(raw, &nodes)?;
        graph.physics.bodies = metadata::append(raw, &bodies)?;
        graph.physics.next_id = bodies.len() as u32 + 1;
        let model_id = raw
            .section_id(raw.root_reference().index)
            .ok_or_else(|| Error::new("root", "missing root"))?;
        let root = raw
            .array::<wr::DocumentRecord<wr::V1>>(model_id)?
            .get(0)
            .transpose()?
            .ok_or_else(|| Error::new("root", "missing root"))?;
        let id = raw
            .section_id(root.models.index)
            .ok_or_else(|| Error::new("model", "missing model"))?;
        raw.replace(id, from_ref(graph))?;
    }
    Ok(())
}
pub(super) fn restore<D: Dialect>(
    raw: &RawDocument<'_>,
    graph: &wr::ModelRecord<wr::V9>,
    groups: &mut BTreeMap<GroupKey, Group<'static, D>>,
    context: &mut Context,
) -> Result<(), Error> {
    let nodes = raw
        .resolve::<wr::Node<wr::V19>>(graph.nodes.nodes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    for body in raw
        .resolve::<wr::PhysicsBody<wr::V3>>(graph.physics.bodies.cast())?
        .iter()
    {
        let body = body?;
        let node = nodes
            .get(body.node as usize)
            .ok_or_else(|| Error::new("physics.node", "missing attached node"))?;
        let fields = metadata::read(raw, node.native.clone())?;
        if body.motion != wr::BodyMotion::Static {
            context.unsupported("physics.motion", "WMO collision bodies must be static")?;
            continue;
        }
        let key = GroupKey {
            index: metadata::number(&fields, "wmo.collision.group", 0)? as usize,
            lod: metadata::number(&fields, "wmo.collision.lod", 0)? as usize,
        };
        let group = groups.get_mut(&key).ok_or_else(|| {
            Error::new("physics.group", "collision requires a selected WMO group")
        })?;
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut faces = Vec::new();
        for chunk in group.chunks() {
            match chunk {
                GroupChunk::Positions(rows) => positions.extend(collect(rows, "vertices")?),
                GroupChunk::Indices(rows) => {
                    indices.extend(collect(rows, "indices")?.into_iter().map(u32::from))
                }
                GroupChunk::Indices32(rows) => indices.extend(collect(rows, "indices")?),
                GroupChunk::Triangles(rows) => faces.extend(
                    collect(rows, "triangles")?
                        .into_iter()
                        .map(|r| (u32::from(r.flags), u32::from(r.material))),
                ),
                GroupChunk::Triangles2(rows) => faces.extend(
                    collect(rows, "triangles")?
                        .into_iter()
                        .map(|r| (u32::from(r.flags), u32::from(r.material))),
                ),
                _ => {}
            }
        }
        let mut collision = Vec::new();
        for shape in raw
            .resolve::<wr::PhysicsShape<wr::V2>>(body.shapes.cast())?
            .iter()
        {
            let shape = shape?;
            if shape.kind != wr::PhysicsShapeKind::TriangleMesh {
                context.unsupported(
                    "physics.shape",
                    "WMO requires triangle-mesh collision geometry",
                )?;
                continue;
            }
            if shape.material != wr::PhysicsMaterial::default() || shape.game_flags.bits() != 0 {
                context.unsupported("physics.shape.material", "WMO collision surfaces need explicit material/contact policy for authored physics settings")?;
            }
            if shape.transform != IDENTITY
                || node.local.translation != [0.; 3]
                || node.local.rotation != [0., 0., 0., 1.]
                || node.local.scale != [1.; 3]
            {
                context.unsupported(
                    "physics.transform",
                    "transformed collision shapes require explicit geometry baking",
                )?;
                continue;
            }
            let vertices = raw
                .resolve::<wr::Vector3>(shape.vertices.cast())?
                .iter()
                .collect::<Result<Vec<_>, _>>()?;
            let source = raw
                .resolve::<wr::U32>(shape.triangles.cast())?
                .iter()
                .collect::<Result<Vec<_>, _>>()?;
            if !source.len().is_multiple_of(3) {
                return Err(Error::new("physics.triangles", "incomplete triangle"));
            }
            let vertices = vertices
                .into_iter()
                .map(|row| row.value)
                .collect::<Vec<_>>();
            let source = source.into_iter().map(|row| row.value).collect::<Vec<_>>();
            let remap = vertices
                .iter()
                .map(|vertex| {
                    if let Some(index) = positions.iter().position(|v| v == vertex) {
                        index as u32
                    } else {
                        let index = positions.len() as u32;
                        positions.push(*vertex);
                        index
                    }
                })
                .collect::<Vec<_>>();
            for triangle in source.as_chunks::<3>().0 {
                let values = triangle
                    .iter()
                    .map(|v| {
                        remap.get(*v as usize).copied().ok_or_else(|| {
                            Error::new("physics.triangles", "vertex index outside shape")
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let existing = indices
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .position(|face| face.as_slice() == values);
                let face = if let Some(face) = existing {
                    face
                } else {
                    let face = indices.len() / 3;
                    indices.extend(values);
                    faces.push((8, 255));
                    face
                };
                collision.push(u16::try_from(face).map_err(|e| failure("physics.face", e))?);
            }
        }
        for chunk in group.chunks_mut() {
            match chunk {
                GroupChunk::Normals(rows) => {
                    let mut values = collect(rows, "collision.normals")?;
                    values.resize(positions.len(), [0.; 3]);
                    *rows = array(&values)?;
                }
                GroupChunk::TextureCoordinates(rows) => {
                    let mut values = collect(rows, "collision.uv")?;
                    values.resize(positions.len(), [0.; 2]);
                    *rows = array(&values)?;
                }
                GroupChunk::VertexColors(rows) | GroupChunk::VertexColors2(rows) => {
                    let mut values = collect(rows, "collision.colors")?;
                    values.resize(positions.len(), [255; 4]);
                    *rows = array(&values)?;
                }
                _ => {}
            }
        }
        group.chunks_mut().retain(|chunk| {
            !matches!(
                chunk,
                GroupChunk::Positions(_)
                    | GroupChunk::Indices(_)
                    | GroupChunk::Indices32(_)
                    | GroupChunk::Triangles(_)
                    | GroupChunk::Triangles2(_)
                    | GroupChunk::BspNodes(_)
                    | GroupChunk::BspFaces(_)
            )
        });
        group
            .chunks_mut()
            .push(GroupChunk::Positions(array(&positions)?));
        if indices.iter().any(|v| *v > 65535) {
            group
                .chunks_mut()
                .push(GroupChunk::Indices32(array(&indices)?));
        } else {
            group.chunks_mut().push(GroupChunk::Indices(array(
                &indices.iter().map(|v| *v as u16).collect::<Vec<_>>(),
            )?));
        }
        if faces
            .iter()
            .all(|(flags, material)| *flags <= 255 && *material <= 255)
        {
            let rows = faces
                .into_iter()
                .map(|(flags, material)| nr::Triangle {
                    flags: nr::TriangleFlags(flags as u8),
                    material: material as u8,
                })
                .collect::<Vec<_>>();
            group
                .chunks_mut()
                .push(GroupChunk::Triangles(array(&rows)?));
        } else {
            let rows = faces
                .into_iter()
                .map(|(flags, material)| nr::Triangle2 {
                    flags: nr::TriangleFlags2(flags as u16),
                    material: material as u16,
                })
                .collect::<Vec<_>>();
            group
                .chunks_mut()
                .push(GroupChunk::Triangles2(array(&rows)?));
        }
        if !collision.is_empty() {
            let node = nr::BspNode {
                flags: nr::BspFlags(4),
                negative_child: -1,
                positive_child: -1,
                face_count: u16::try_from(collision.len())
                    .map_err(|e| failure("physics.faces", e))?,
                first_face: 0,
                plane_distance: 0.,
            };
            group
                .chunks_mut()
                .push(GroupChunk::BspNodes(array(&[node])?));
            group
                .chunks_mut()
                .push(GroupChunk::BspFaces(array(&collision)?));
        }
    }
    if !graph.physics.joints.is_empty()
        || !graph.physics.cloths.is_empty()
        || !graph.physics.colliders.is_empty()
        || !graph.physics.soft.is_empty()
    {
        context.unsupported(
            "physics",
            "WMO cannot represent animated constraints, cloth, or soft bodies",
        )?;
    }
    Ok(())
}
