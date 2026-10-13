//! Named authoring controls carried by older collision-shape layouts.
use super::*;
use std::array::from_fn;

pub(super) fn import<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::PhysicsShape>,
    values: &mut BTreeMap<String, i64>,
    prefix: &str,
) -> Result<(), Error> {
    macro_rules! legacy {
        ($v:ty, $links:expr) => {
            for (index, row) in source
                .resolve::<nr::PhysicsShape<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
                .enumerate()
            {
                let row = row.map_err(read_error)?;
                let key = format!("m3.{prefix}.shape[{index}]");
                for (i, byte) in row.legacy_reserved_0.0.iter().enumerate() {
                    values.insert(format!("{key}.header[{i}]"), i64::from(*byte));
                }
                for (i, byte) in row.legacy_reserved_1.0.iter().enumerate() {
                    values.insert(format!("{key}.padding[{i}]"), i64::from(*byte));
                }
                values.insert(
                    format!("{key}.points_flags"),
                    i64::from(row.source_points.flags),
                );
                let (unused, planes, triangles): (
                    Ref<nr::targets::Byte>,
                    Ref<nr::targets::Vector4>,
                    Ref<nr::targets::U16>,
                ) = $links(&row);
                values.insert(format!("{key}.triangles_flags"), i64::from(triangles.flags));
                values.insert(format!("{key}.unused_flags"), i64::from(unused.flags));
                values.insert(format!("{key}.planes_flags"), i64::from(planes.flags));
                values.insert(format!("{key}.unused_count"), i64::from(unused.entries));
                for (i, byte) in source
                    .resolve::<nr::Byte>(unused.cast())
                    .map_err(read_error)?
                    .iter()
                    .enumerate()
                {
                    values.insert(
                        format!("{key}.unused[{i}]"),
                        i64::from(byte.map_err(read_error)?.value),
                    );
                }
                values.insert(format!("{key}.planes_count"), i64::from(planes.entries));
                for (i, plane) in source
                    .resolve::<nr::Vector4>(planes.cast())
                    .map_err(read_error)?
                    .iter()
                    .enumerate()
                {
                    for (j, component) in plane.map_err(read_error)?.value.iter().enumerate() {
                        values.insert(
                            format!("{key}.planes[{i}][{j}]"),
                            i64::from(component.to_bits()),
                        );
                    }
                }
            }
        };
    }
    if reference.entries == 0 {
        return Ok(());
    }
    match revision(source, reference).map_err(read_error)? {
        0 => legacy!(nr::V0, |_: &nr::PhysicsShape<nr::V0>| (
            Ref::default(),
            Ref::default(),
            Ref::default()
        )),
        1 => legacy!(nr::V1, |r: &nr::PhysicsShape<nr::V1>| (
            r.unused,
            r.planes,
            r.source_triangles
        )),
        2 | 3 => {
            macro_rules! modern {
                ($v:ty) => {
                    for (index, row) in source
                        .resolve::<nr::PhysicsShape<$v>>(reference.cast())
                        .map_err(read_error)?
                        .iter()
                        .enumerate()
                    {
                        let row = row.map_err(read_error)?;
                        let key = format!("m3.{prefix}.shape[{index}]");
                        for (i, byte) in row.reserved_0.0.iter().enumerate() {
                            values.insert(format!("{key}.padding[{i}]"), i64::from(*byte));
                        }
                        values.insert(
                            format!("{key}.points_flags"),
                            i64::from(row.source_points.flags),
                        );
                        values.insert(
                            format!("{key}.triangles_flags"),
                            i64::from(row.source_triangles.flags),
                        );
                    }
                };
            }
            if revision(source, reference).map_err(read_error)? == 2 {
                modern!(nr::V2);
            } else {
                modern!(nr::V3);
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn export<E: Envelope>(
    doc: &mut native::Document<'static, E>,
    row: &mut nr::PhysicsShape<nr::V1>,
    values: &BTreeMap<String, i64>,
    prefix: &str,
    index: usize,
) -> Result<(), Error> {
    let key = format!("m3.{prefix}.shape[{index}]");
    let get = |field: &str| values.get(&format!("{key}.{field}")).copied().unwrap_or(0);
    let byte = |field: &str| {
        u8::try_from(get(field))
            .map_err(|_| Error::new(format!("{key}.{field}"), "control exceeds one byte"))
    };
    for (i, value) in row.legacy_reserved_0.0.iter_mut().enumerate() {
        *value = byte(&format!("header[{i}]"))?;
    }
    for (i, value) in row.legacy_reserved_1.0.iter_mut().enumerate() {
        *value = byte(&format!("padding[{i}]"))?;
    }
    row.source_points.flags = get("points_flags") as u32;
    row.source_triangles.flags = get("triangles_flags") as u32;
    let count = |field: &str| {
        usize::try_from(get(field))
            .map_err(|_| Error::new(format!("{key}.{field}"), "invalid array count"))
    };
    let unused_count = count("unused_count")?;
    let planes_count = count("planes_count")?;
    if unused_count > values.len() || planes_count > values.len() / 4 {
        return Err(Error::new(
            &key,
            "native array count exceeds supplied controls",
        ));
    }
    let unused = (0..unused_count)
        .map(|i| {
            Ok(nr::Byte {
                value: byte(&format!("unused[{i}]"))?,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let planes = (0..planes_count)
        .map(|i| nr::Vector4 {
            value: from_fn(|j| f32::from_bits(get(&format!("planes[{i}][{j}]")) as u32)),
        })
        .collect::<Vec<_>>();
    if planes
        .iter()
        .any(|p| p.value.iter().any(|v| !v.is_finite()))
    {
        return Err(Error::new(&key, "collision planes must be finite"));
    }
    row.unused = append(doc, &unused)?.cast();
    row.unused.flags = get("unused_flags") as u32;
    row.planes = append(doc, &planes)?.cast();
    row.planes.flags = get("planes_flags") as u32;
    Ok(())
}

trait Populated {
    fn populated(&self) -> bool;
}
impl<T: Default + PartialEq> Populated for T {
    fn populated(&self) -> bool {
        self != &Self::default()
    }
}

pub(super) fn preflight<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::PhysicsShape>,
    context: &mut Context,
) -> Result<(), Error> {
    macro_rules! check {($row:ident,$index:ident,$($field:ident),* $(,)?)=>{$(if Populated::populated(&$row.$field) {
        context.unsupported(format!("m3.physics.shapes[{}].{}",$index,stringify!($field)),"cooked collision data has no active WEM counterpart")?;
    })*};}
    macro_rules! modern {
        ($v:ty) => {
            for (index, row) in source
                .resolve::<nr::PhysicsShape<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
                .enumerate()
            {
                let row = row.map_err(read_error)?;
                check!(
                    row,
                    index,
                    hull_vertices,
                    hull_planes,
                    hull_half_edges,
                    hull_face_first_edges,
                    hull_centroid,
                    hull_vertex_count,
                    hull_face_count,
                    hull_half_edge_count,
                    hull_volume,
                    hull_surface_area,
                    mesh_bvh_nodes,
                    mesh_vertex_positions,
                    mesh_face_indices16,
                    mesh_face_indices32,
                    mesh_bounds_center,
                    mesh_bounds_extent,
                    mesh_tolerance,
                    havok_tree,
                    mesh_normal_count,
                    positions,
                    triangles,
                    edges,
                    reserved_1,
                    mesh_vertex_count,
                    mesh_face_index16_count,
                    mesh_face_index32_count,
                    reserved_2,
                    mesh_unknown1,
                    mesh_reserved,
                    mesh_tree_depth,
                    mesh_collision_margin
                );
            }
        };
    }
    if reference.entries == 0 {
        return Ok(());
    }
    match revision(source, reference).map_err(read_error)? {
        2 => modern!(nr::V2),
        3 => modern!(nr::V3),
        _ => {}
    }
    Ok(())
}
