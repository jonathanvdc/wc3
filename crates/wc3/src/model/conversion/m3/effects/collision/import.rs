//! Imports static collision and native selection-volume roles.
use super::*;
pub(in crate::model::conversion::m3) fn attach<E: Envelope>(
    source: &native::Model<'_, E>,
    document: wem::Document<'static>,
    context: &mut Context,
) -> Result<wem::Document<'static>, Error> {
    let (bounds, vertices, triangles, normals, tight, fuzzy, volumes) =
        native::visit_m3_root!(source.root_record().map_err(read_error)?, |r| (
            r.collision_bounds,
            r.collision_verts,
            r.collision_faces,
            r.collision_normals,
            r.tight_hit_test_object,
            r.fuzzy_hit_test_objects,
            r.attachment_volumes
        ));
    let source_doc = source.document();
    let mut doc = document.into_raw();
    let root = doc
        .resolve::<wr::DocumentRecord<wr::V1>>(wem::Ref::from_section(
            doc.root_reference().entries,
            doc.root_reference().index,
        ))?
        .get(0)
        .ok_or_else(|| Error::new("wem.root", "missing root"))??;
    let mut model = doc
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let mut bodies = doc
        .resolve::<wr::PhysicsBody<wr::V3>>(model.physics.bodies.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let mut sets = doc
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let set = sets
        .iter_mut()
        .find(|r| r.profile == wr::ProfileId::SC2)
        .ok_or_else(|| Error::new("wem.profile", "missing SC2 profile"))?;
    let mut values = BTreeMap::new();
    let mut normals_text = None;
    let pos = points(source_doc, vertices)?;
    let tris = faces(source_doc, triangles, pos.len())?;
    let norms = points(source_doc, normals)?;
    for (i, v) in bounds
        .minimum
        .into_iter()
        .chain(bounds.maximum)
        .chain([bounds.radius])
        .enumerate()
    {
        values.insert(format!("m3.collision.bounds[{i}]"), i64::from(v.to_bits()));
    }
    if !pos.is_empty() || !tris.is_empty() || !norms.is_empty() {
        let shape = wr::PhysicsShape {
            kind: wr::PhysicsShapeKind(5),
            transform: IDENTITY,
            vertices: metadata::put(&mut doc, &pos)?.cast(),
            triangles: metadata::put(&mut doc, &tris)?.cast(),
            ..Default::default()
        };
        bodies.push(body(&mut doc, STATIC, u32::MAX, shape)?);
        normals_text = Some(
            norms
                .iter()
                .flat_map(|r| r.value)
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    if tight != nr::HitTestShape::<nr::V1>::default() {
        values.insert("m3.hit.tight.padding".into(), i64::from(tight.padding));
        values.insert("m3.hit.tight.kind".into(), i64::from(tight.shape_type));
        let shape = shape(source_doc, &mut doc, tight, context)?;
        bodies.push(body(
            &mut doc,
            TIGHT,
            if tight.bone_index == u16::MAX {
                u32::MAX
            } else {
                u32::from(tight.bone_index)
            },
            shape,
        )?);
    }
    macro_rules! fuzzy_rows {
        ($v:ty) => {
            for (index, row) in source_doc
                .resolve::<nr::HitTestShape<$v>>(fuzzy.cast())
                .map_err(read_error)?
                .iter()
                .enumerate()
            {
                let r = row.map_err(read_error)?;
                values.insert(
                    format!("m3.hit.fuzzy[{index}].kind"),
                    i64::from(r.shape_type),
                );
                values.insert(
                    format!("m3.hit.fuzzy[{index}].padding"),
                    i64::from(r.padding),
                );
                let r = nr::HitTestShape::<nr::V1> {
                    shape_type: r.shape_type,
                    bone_index: r.bone_index,
                    padding: r.padding,
                    transform: r.transform,
                    vertex_positions: r.vertex_positions,
                    face_indices: r.face_indices,
                    size_x: r.size_x,
                    size_y: r.size_y,
                    size_z: r.size_z,
                };
                let shape = shape(source_doc, &mut doc, r, context)?;
                bodies.push(body(
                    &mut doc,
                    FUZZY + index as u32,
                    if r.bone_index == u16::MAX {
                        u32::MAX
                    } else {
                        u32::from(r.bone_index)
                    },
                    shape,
                )?);
            }
        };
    }
    if fuzzy.entries != 0 {
        match revision(source_doc, fuzzy).map_err(read_error)? {
            0 => fuzzy_rows!(nr::V0),
            1 => fuzzy_rows!(nr::V1),
            _ => return Err(Error::new("hit_test.revision", "unknown hit-test revision")),
        }
    }
    for (index, row) in source_doc
        .resolve::<nr::AttachmentVolume>(volumes.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        let r = row.map_err(read_error)?;
        for (key, value) in [
            ("kind", r.shape_type),
            ("bone1", r.bone1),
            ("bone2", r.bone2),
        ] {
            values.insert(
                format!("m3.attachment_volume[{index}].{key}"),
                i64::from(value),
            );
        }
        let hit = nr::HitTestShape::<nr::V1> {
            shape_type: r.shape_type,
            bone_index: if r.bone_index == u32::MAX {
                u16::MAX
            } else {
                u16::try_from(r.bone_index).map_err(|_| {
                    Error::new(
                        "attachment_volume.node",
                        "bone index exceeds current skeleton capacity",
                    )
                })?
            },
            padding: 0,
            transform: r.transform,
            vertex_positions: r.vertex_positions,
            face_indices: r.face_indices,
            size_x: r.size_x,
            size_y: r.size_y,
            size_z: r.size_z,
        };
        let shape = shape(source_doc, &mut doc, hit, context)?;
        bodies.push(body(&mut doc, VOLUME + index as u32, r.bone_index, shape)?);
    }
    for b in bodies.iter().filter(|b| b.id & 0x8000_0000 != 0) {
        values.insert(
            format!("m3.collision.body[{}].role", b.id),
            i64::from(b.id >> 28),
        );
    }
    let addons = native::visit_m3_root!(source.root_record().map_err(read_error)?, |r| [
        r.attachment_point_addons,
        r.cameras_addons,
        OptionalArray::array(&r.attachment_volumes_addon0),
        OptionalArray::array(&r.attachment_volumes_addon1)
    ]);
    for (key, reference) in [
        "attachment_addons",
        "camera_addons",
        "volume_addon0",
        "volume_addon1",
    ]
    .into_iter()
    .zip(addons)
    {
        let rows = source_doc
            .resolve::<nr::U16>(reference.cast())
            .map_err(read_error)?
            .iter()
            .map(|r| r.map(|r| r.value.to_string()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_error)?;
        set.native =
            metadata::set_text(&mut doc, &set.native, &format!("m3.{key}"), &rows.join(" "))?;
    }
    set.native = metadata::extend(&mut doc, &set.native, &values)?;
    if let Some(text) = normals_text {
        set.native = metadata::set_text(&mut doc, &set.native, "m3.collision.normals", &text)?;
    }
    model.physics.bodies = metadata::put(&mut doc, &bodies)?.cast();
    let id = doc
        .section_id(model.profile_sets.index)
        .ok_or_else(|| Error::new("wem.profile", "missing profiles"))?;
    doc.replace(id, &sets)?;
    let id = doc
        .section_id(root.models.index)
        .ok_or_else(|| Error::new("wem.model", "missing models"))?;
    doc.replace(id, &[model])?;
    Ok(wem::Document::from_raw(doc)?)
}
