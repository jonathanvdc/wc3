//! Shape geometry and primitive dimensions, independent of rigid-body controls.
use super::*;

pub(super) fn import<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::PhysicsShape>,
    doc: &mut wem::RawDocument<'static>,
    material: wr::PhysicsMaterial,
    context: &mut Context,
) -> Result<wem::Ref<wr::targets::PhysicsShape>, Error> {
    if reference.entries == 0 {
        return Ok(wem::Ref::default());
    }
    shape_controls::preflight(source, reference, context)?;
    let mut output = Vec::new();
    macro_rules! shape {
        ($v:ty, $triangles:expr) => {
            for row in source
                .resolve::<nr::PhysicsShape<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                let triangles: Ref<nr::targets::U16> = $triangles(&row);
                let points = source
                    .resolve::<nr::Vector3>(row.source_points.cast())
                    .map_err(read_error)?
                    .iter()
                    .map(|p| {
                        p.map(|p| wr::Vector3 { value: p.value })
                            .map_err(read_error)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let faces = source
                    .resolve::<nr::U16>(triangles.cast())
                    .map_err(read_error)?
                    .iter()
                    .map(|v| {
                        v.map(|v| wr::U32 {
                            value: u32::from(v.value),
                        })
                        .map_err(read_error)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if row.shape_type > 5 {
                    return Err(Error::new("m3.physics.shape.kind", "unknown shape kind"));
                }
                if !faces.len().is_multiple_of(3)
                    || faces.iter().any(|v| v.value as usize >= points.len())
                {
                    return Err(Error::new(
                        "m3.physics.shape.triangles",
                        "invalid collision triangles",
                    ));
                }
                let kind = wr::PhysicsShapeKind(row.shape_type);
                let points = put(doc, &points)?.cast();
                output.push(wr::PhysicsShape::<wr::V2> {
                    kind,
                    transform: row.transform,
                    half_extents: row.shape_dimensions,
                    radius: row.shape_dimensions[0],
                    length: row.shape_dimensions[1],
                    points: if row.shape_type != 5 {
                        points
                    } else {
                        wem::Ref::default()
                    },
                    vertices: if row.shape_type == 5 {
                        points
                    } else {
                        wem::Ref::default()
                    },
                    triangles: put(doc, &faces)?.cast(),
                    material: material.clone(),
                    ..Default::default()
                });
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        0 => shape!(nr::V0, |_: &nr::PhysicsShape<nr::V0>| Ref::default()),
        1 => shape!(nr::V1, |r: &nr::PhysicsShape<nr::V1>| r.source_triangles),
        2 => {
            for row in source
                .resolve::<nr::PhysicsShape<nr::V2>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                if row.positions.entries != 0 || row.havok_tree.entries != 0 {
                    context.unsupported(
                        "m3.physics.shape.cooked_mesh",
                        "cooked collision trees have no WEM tree representation",
                    )?;
                }
            }
            shape!(nr::V2, |r: &nr::PhysicsShape<nr::V2>| r.source_triangles);
        }
        3 => {
            for row in source
                .resolve::<nr::PhysicsShape<nr::V3>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                if row.hull_vertices.entries != 0
                    || row.mesh_vertex_positions.entries != 0
                    || row.mesh_bvh_nodes.entries != 0
                {
                    context.unsupported("m3.physics.shape.cooked_geometry", "cooked hull topology and collision trees require reconstruction beyond source geometry")?;
                }
            }
            shape!(nr::V3, |r: &nr::PhysicsShape<nr::V3>| r.source_triangles);
        }
        _ => {
            return Err(Error::new(
                "m3.physics.shapes",
                "unsupported shape revision",
            ));
        }
    }
    Ok(put(doc, &output)?.cast())
}

pub(super) fn export<E: Envelope>(
    source: &wem::RawDocument<'_>,
    reference: wem::Ref<wr::targets::PhysicsShape>,
    doc: &mut native::Document<'static, E>,
    context: &mut Context,
    values: &BTreeMap<String, i64>,
    prefix: &str,
) -> Result<(Ref<nr::targets::PhysicsShape>, wr::PhysicsMaterial), Error> {
    let mut shapes = Vec::new();
    let mut material = None;
    let rows = source.resolve::<wr::PhysicsShape<wr::V2>>(reference.cast())?;
    for (index, row) in rows.iter().enumerate() {
        let row = row?;
        if let Some(previous) = &material {
            if previous != &row.material {
                context.unsupported(
                    "wem.physics.material",
                    "M3 assigns one physics material to all shapes in a body",
                )?;
            }
        } else {
            material = Some(row.material.clone());
        }
        if row.kind.0 > 5 {
            return Err(Error::new(
                "wem.physics.shape.kind",
                "M3 does not represent this shape kind",
            ));
        }
        if row.game_flags != 0 {
            context.unsupported(
                "wem.physics.shape.game_flags",
                "M3 has no per-shape game callback bits",
            )?;
        }
        let points = if row.kind.0 == 5 {
            row.vertices
        } else {
            row.points
        };
        let points = source
            .resolve::<wr::Vector3>(points.cast())?
            .iter()
            .map(|r| r.map(|r| nr::Vector3 { value: r.value }))
            .collect::<Result<Vec<_>, _>>()?;
        let indices = source
            .resolve::<wr::U32>(row.triangles.cast())?
            .iter()
            .map(|r| {
                let r = r?;
                Ok(nr::U16 {
                    value: u16::try_from(r.value).map_err(|_| {
                        Error::new(
                            "wem.physics.triangles",
                            "M3 source triangle index exceeds 16 bits",
                        )
                    })?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let dimensions = match row.kind.0 {
            0 => row.half_extents,
            1 => [row.radius, 0., 0.],
            2 | 3 => [row.radius, row.length, 0.],
            _ => [0.; 3],
        };
        let mut native_shape = nr::PhysicsShape::<nr::V1> {
            transform: row.transform,
            shape_type: row.kind.0,
            shape_dimensions: dimensions,
            source_points: append(doc, &points)?.cast(),
            source_triangles: append(doc, &indices)?.cast(),
            ..Default::default()
        };
        shape_controls::export(doc, &mut native_shape, values, prefix, index)?;
        shapes.push(native_shape);
    }
    Ok((append(doc, &shapes)?.cast(), material.unwrap_or_default()))
}
