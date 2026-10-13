//! PHYS companion graphs installed as WEM bodies, shapes, joints, and WOW rigs.
use super::*;
pub(in super::super) fn import_physics<D: Dialect>(
    document: wem::Document<'static>,
    source: &Bundle<'_, D>,
    context: &mut Context,
    options: &ImportOptions,
    opaque: &mut Vec<OpaquePayload>,
) -> Result<wem::Document<'static>, Error> {
    let mut sources: Vec<_> = source
        .files()
        .filter_map(|(key, file)| {
            if let File::Physics(file) = file {
                Some((key.clone(), file))
            } else {
                None
            }
        })
        .collect();
    for chunk in source.model.extensions() {
        if let Chunk::Physics(file) = chunk {
            sources.push((source.key.clone(), file));
        }
    }
    if sources.is_empty() {
        return Ok(document);
    }
    if sources.len() != 1 {
        return Err(Error::new(
            "physics",
            "multiple independent native physics documents require explicit rig selection",
        ));
    }
    let (key, file) = &sources[0];
    file.validate_with_bone_count(source.bones().map_err(failure)?.1.len())
        .map_err(failure)?;
    let root = document.root::<wr::DocumentRecord<wr::V1>>()?;
    let mut model = document
        .raw()
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let mut nodes: Vec<_> = document
        .raw()
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let mut existing_bodies: Vec<_> = document
        .raw()
        .resolve::<wr::PhysicsBody<wr::V3>>(model.physics.bodies.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let mut raw = document.into_raw();
    let shape_rows = shapes(file, &mut raw)?;
    let shape_section = put(&mut raw, &shape_rows)?;
    let native_bodies = bodies(file);
    let first_id = existing_bodies.iter().map(|b| b.id).max().unwrap_or(0) + 1;
    for (index, body) in native_bodies.iter().enumerate() {
        let first = usize::try_from(body.first)
            .map_err(|_| Error::new("physics.body.shapes", "negative shape start"))?;
        let count = usize::try_from(body.count)
            .map_err(|_| Error::new("physics.body.shapes", "negative shape count"))?;
        let end = first
            .checked_add(count)
            .ok_or_else(|| Error::new("physics.body.shapes", "shape range overflow"))?;
        let selected = shape_rows
            .get(first..end)
            .ok_or_else(|| Error::new("physics.body.shapes", "missing body shape window"))?;
        let shapes = if first == 0 {
            wem::Ref::<wr::targets::PhysicsShape>::from_section(count as u32, shape_section.index)
        } else {
            put(&mut raw, selected)?.cast()
        };
        let node = nodes.len() as u32;
        let name = format!("physics-body-{index}");
        let chars: Vec<_> = name.bytes().map(|value| wr::Character { value }).collect();
        let mut fields = vec![
            ("m2.physics.host".to_owned(), 1),
            (
                "m2.physics.body_type".to_owned(),
                i64::from(body.body_type.raw()),
            ),
            ("m2.physics.reserved".to_owned(), i64::from(body.reserved)),
        ];
        let definitions: Vec<_> = file
            .chunks
            .iter()
            .flat_map(|chunk| match chunk {
                Pc::Shapes(rows) => rows
                    .iter()
                    .map(|shape| native::companions::PhysicsShape2 {
                        shape: shape.clone(),
                        ..Default::default()
                    })
                    .collect(),
                Pc::Shapes2(rows) => rows.clone(),
                _ => Vec::new(),
            })
            .collect();
        for (local, shape) in definitions[first..end].iter().enumerate() {
            for (name, value) in [
                ("reserved", i64::from(shape.shape.reserved)),
                ("unused0", i64::from(shape.unused[0].to_bits())),
                ("unused1", i64::from(shape.unused[1].to_bits())),
                ("unused1c", i64::from(shape.unused1c)),
                ("padding", i64::from(shape.reserved)),
            ] {
                fields.push((format!("m2.physics.shape.{local}.{name}"), value));
            }
        }
        let fields: Vec<_> = fields
            .iter()
            .map(|(key, value)| (key.as_str(), *value))
            .collect();
        let helper = wr::Node::<wr::V19> {
            name: put(&mut raw, &chars)?.cast(),
            parent: body.bone,
            kind: wr::NodeKind::Helper,
            flags: wr::NodeFlags(
                wr::NodeFlags::DONT_INHERIT_ROTATION.0 | wr::NodeFlags::DONT_INHERIT_SCALE.0,
            ),
            local: wr::Transform {
                translation: body.position,
                rotation: [0., 0., 0., 1.],
                scale: [1.; 3],
            },
            native: bag(&mut raw, &fields)?,
            ..Default::default()
        };
        nodes.push(helper);
        let mut record = body_record(body, first_id + index as u32, node, shapes);
        if let Some(wow) = record.wow.value.as_mut() {
            if wow.parent != u32::MAX {
                wow.parent += first_id - 1;
            }
        }
        existing_bodies.push(record);
    }
    let mut joints = joints(file, first_id + native_bodies.len() as u32)?;
    for joint in &mut joints {
        joint.body_a += first_id - 1;
        joint.body_b += first_id - 1;
    }
    let body_ids: Vec<_> = existing_bodies
        .iter()
        .filter(|body| body.node != u32::MAX)
        .map(|body| wr::U32 { value: body.id })
        .collect();
    let kind = file
        .chunks
        .iter()
        .find_map(|chunk| {
            if let Pc::Kind(kind) = chunk {
                Some(*kind)
            } else {
                None
            }
        })
        .unwrap_or_default();
    let vegetation = file
        .chunks
        .iter()
        .find_map(|chunk| {
            if let Pc::Vegetation(v) = chunk {
                v.first()
            } else {
                None
            }
        })
        .map(|v| wr::WowVegetation {
            pos_max_push: v.position_max_push,
            pos_push_amt: v.position_push_amount,
            pos_relax_speed: v.position_relax_speed,
            vel_max_push: v.velocity_max_push,
            vel_speed: v.velocity_speed,
            min_push_dist: v.minimum_push_distance_squared,
        });
    let allow = file
        .chunks
        .iter()
        .find_map(|chunk| {
            if let Pc::AllowList { keys, .. } = chunk {
                Some(keys)
            } else {
                None
            }
        })
        .map(|keys| {
            keys.iter()
                .map(|value| wr::U32 { value })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let rig =
        wr::PhysicsRig::<wr::V8> {
            id: first_id + native_bodies.len() as u32 + joints.len() as u32,
            bodies: put(&mut raw, &body_ids)?.cast(),
            wow: wem::InlineOption::from(Some(wr::WowRigExtension {
                kind: wr::WowPhysicsKind::from_raw(u8::try_from(kind.raw()).map_err(|_| {
                    Error::new("physics.kind", "rig category exceeds WEM byte field")
                })?),
                vegetation: wem::InlineOption::from(vegetation),
                allow_list: put(&mut raw, &allow)?.cast(),
            })),
            ..Default::default()
        };
    model.physics.next_id = rig.id + 1;
    model.physics.rigs = put(&mut raw, &[rig])?.cast();
    model.physics.bodies = put(&mut raw, &existing_bodies)?.cast();
    model.physics.joints = put(&mut raw, &joints)?.cast();
    let nodes_id = raw
        .section_id(model.nodes.nodes.index)
        .ok_or_else(|| Error::new("wem.nodes", "missing node table"))?;
    if model.nodes.nodes.entries == 0 {
        model.nodes.nodes = put(&mut raw, &nodes)?.cast();
    } else {
        raw.replace(nodes_id, &nodes)?;
        model.nodes.nodes.entries = nodes.len() as u32;
    }
    let model_id = raw
        .section_id(root.models.index)
        .ok_or_else(|| Error::new("wem.model", "missing model table"))?;
    raw.replace(model_id, &[model.clone()])?;
    let mut profile = raw
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.profile", "missing profile"))??;
    let version = file
        .chunks
        .iter()
        .find_map(|chunk| {
            if let Pc::Header(v) = chunk {
                Some(*v)
            } else {
                None
            }
        })
        .unwrap_or(6);
    append_bag(
        &mut raw,
        &mut profile.native,
        &[("m2.physics.version", i64::from(version))],
    )?;
    let links: Vec<_> = file
        .chunks
        .iter()
        .filter_map(|chunk| {
            if let Pc::Joints(rows) = chunk {
                Some(rows)
            } else {
                None
            }
        })
        .flatten()
        .collect();
    for (joint, source) in joints.iter().zip(links) {
        append_bag(
            &mut raw,
            &mut profile.native,
            &[(
                format!("m2.physics.joint.{}.reserved", joint.id).as_str(),
                i64::from(source.reserved),
            )],
        )?;
    }
    match key {
        native::FileKey::FileId(id) => append_bag(
            &mut raw,
            &mut profile.native,
            &[("m2.physics.id", i64::from(*id))],
        )?,
        native::FileKey::Path(path) => add_texts(
            &mut raw,
            &mut profile.native,
            &[("m2.physics.path", path.clone())],
        )?,
        _ => {}
    }
    let profile_id = raw
        .section_id(model.profile_sets.index)
        .ok_or_else(|| Error::new("wem.profile", "missing profile table"))?;
    raw.replace(profile_id, &[profile])?;
    for chunk in &file.chunks {
        if let Pc::Unknown { tag, data } = chunk {
            let path = format!("physics.{tag:?}");
            match options.unknown {
                UnknownPolicy::Reject => {
                    return Err(Error::new(
                        path,
                        "unknown physics extension requires an explicit policy",
                    ));
                }
                UnknownPolicy::Drop => {
                    context.issue(path, IssueKind::Dropped, "unknown physics payload excluded")
                }
                UnknownPolicy::Archive => {
                    opaque.push(OpaquePayload {
                        format: Format::M2,
                        file: key.clone(),
                        tag: *tag,
                        revision: None,
                        count: None,
                        bytes: Cow::Owned(data.to_vec()),
                    });
                    context.issue(
                        path,
                        IssueKind::Archived,
                        "unknown physics payload retained separately",
                    );
                }
            }
        }
    }
    Ok(wem::Document::from_raw(raw)?)
}
