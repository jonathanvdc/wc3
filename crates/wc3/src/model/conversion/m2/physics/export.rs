//! Native PHYS companions rebuilt from the selected WEM rigid-body graph.
use super::*;
pub(in super::super) fn export_physics<D: ExportDialect>(
    model: &sm::Model<'_>,
    bundle: &mut Bundle<'static, D>,
    context: &mut Context,
) -> Result<(), Error> {
    let Some(native) = &model.native else {
        return Ok(());
    };
    let raw = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let record = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let bodies: Vec<_> = raw
        .resolve::<wr::PhysicsBody<wr::V3>>(record.physics.bodies.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let bodies: Vec<_> = bodies
        .into_iter()
        .filter(|body| body.node != u32::MAX)
        .collect();
    if bodies.is_empty() {
        return Ok(());
    }
    let nodes = native_nodes(model)?;
    let mut geometry = Geometry::default();
    let mut output_bodies = Vec::new();
    let mut id_map = BTreeMap::new();
    for (index, body) in bodies.iter().enumerate() {
        id_map.insert(body.id, index as u32);
    }
    for body in &bodies {
        let node = nodes
            .get(body.node as usize)
            .ok_or_else(|| Error::new("physics.body.node", "body references a missing node"))?;
        let shared = model
            .nodes
            .get(body.node as usize)
            .ok_or_else(|| Error::new("physics.body.node", "missing semantic node"))?;
        let data = metadata(raw, &node.native)?;
        let helper = data.get("m2.physics.host") == Some(&1);
        let (bone, position) = if helper {
            (shared.parent.unwrap_or(u32::MAX), shared.translation)
        } else {
            (body.node, [0.; 3])
        };
        let first = geometry.shapes.len();
        for (local, shape) in raw
            .resolve::<wr::PhysicsShape<wr::V2>>(body.shapes.cast())?
            .iter()
            .enumerate()
        {
            geometry.add(&shape?, raw)?;
            let saved = |name: &str| {
                data.get(&format!("m2.physics.shape.{local}.{name}"))
                    .copied()
                    .unwrap_or(0)
            };
            let row = geometry.shapes.last_mut().unwrap();
            row.shape.reserved = saved("reserved") as u16;
            row.unused = [
                f32::from_bits(saved("unused0") as u32),
                f32::from_bits(saved("unused1") as u32),
            ];
            row.unused1c = saved("unused1c") as u16;
            row.reserved = saved("padding") as u16;
        }
        let mut attachment = 0u16;
        if let Some(wow) = &body.wow.value {
            if wow.has_children.get() {
                attachment |= 0x8000;
            }
            if wow.ragdoll_root.get() {
                attachment |= 0x4000;
            }
            if wow.parent != u32::MAX {
                let index = *id_map
                    .get(&wow.parent)
                    .ok_or_else(|| Error::new("physics.body.parent", "missing kinematic parent"))?;
                if index > 0x3fff {
                    return Err(Error::new(
                        "physics.body.parent",
                        "parent index exceeds attachment bits",
                    ));
                }
                attachment |= index as u16;
            }
        }
        let body_type = match body.motion {
            wr::BodyMotion::Kinematic => PhysicsBodyKind::Kinematic,
            wr::BodyMotion::Dynamic => PhysicsBodyKind::Dynamic,
            wr::BodyMotion::Static => PhysicsBodyKind::from_raw(
                data.get("m2.physics.body_type")
                    .copied()
                    .filter(|v| *v > 1)
                    .unwrap_or(2) as u16,
            ),
            _ => return Err(Error::new("physics.body.motion", "unknown motion category")),
        };
        output_bodies.push(native::companions::PhysicsBody4 {
            body_type,
            bone_index: u16::try_from(bone)
                .map_err(|_| Error::new("physics.body.bone", "bone index exceeds BDY4 u16"))?,
            position,
            shape_index: i32::try_from(first)
                .map_err(|_| Error::new("physics.body.shapes", "shape index exceeds i32"))?,
            shape_count: (geometry.shapes.len() - first) as i32,
            gravity_scale: body.gravity_scale,
            inertia_scale: body.inertia_scale,
            linear_damping: body.linear_damping,
            angular_damping: body.angular_damping,
            follow_factor: body.wow.value.as_ref().map_or(1., |wow| wow.follow_factor),
            attachment: PhysicsAttachment::from_bits_retain(attachment),
            reserved: data.get("m2.physics.reserved").copied().unwrap_or(0) as u16,
        });
    }
    let metadata = profile_metadata(model)?;
    let mut constraints = Constraints::default();
    for joint in raw
        .resolve::<wr::PhysicsJoint<wr::V3>>(record.physics.joints.cast())?
        .iter()
    {
        let joint = joint?;
        constraints.add(&joint, &id_map, context)?;
        constraints.links.last_mut().unwrap().reserved = metadata
            .get(&format!("m2.physics.joint.{}.reserved", joint.id))
            .copied()
            .unwrap_or(0) as u32;
    }
    let mut chunks = vec![Pc::Header(6)];
    for rig in raw
        .resolve::<wr::PhysicsRig<wr::V8>>(record.physics.rigs.cast())?
        .iter()
    {
        let rig = rig?;
        if let Some(wow) = rig.wow.value {
            chunks.push(Pc::Kind(PhysicsKind::from_raw(u32::from(wow.kind.raw()))));
            if let Some(vegetation) = wow.vegetation.value {
                chunks.push(Pc::Vegetation(vec![
                    native::companions::PhysicsVegetation {
                        position_max_push: vegetation.pos_max_push,
                        position_push_amount: vegetation.pos_push_amt,
                        position_relax_speed: vegetation.pos_relax_speed,
                        velocity_max_push: vegetation.vel_max_push,
                        velocity_speed: vegetation.vel_speed,
                        minimum_push_distance_squared: vegetation.min_push_dist,
                    },
                ]));
            }
            let keys: Vec<_> = raw
                .resolve::<wr::U32>(wow.allow_list.cast())?
                .iter()
                .map(|v| v.map(|v| v.value))
                .collect::<Result<_, _>>()?;
            if !keys.is_empty() {
                chunks.push(Pc::AllowList {
                    header: 0,
                    keys: ZeroVec::alloc_from_slice(&keys),
                });
            }
        }
    }
    chunks.extend(geometry.chunks());
    chunks.push(Pc::Bodies4(output_bodies));
    chunks.extend(constraints.chunks());
    let file = PhysicsFile {
        chunks,
        padding: Cow::Owned(Vec::new()),
    };
    file.validate().map_err(failure)?;
    let key = if let Some(id) = metadata.get("m2.physics.id") {
        native::FileKey::FileId(*id as u32)
    } else {
        bundle.key.sibling(".phys")
    };
    let id = if let native::FileKey::FileId(id) = key {
        Some(id)
    } else {
        None
    };
    D::physics_id(&mut bundle.model, id, context)?;
    let key = if D::NUMERIC_COMPANIONS {
        key
    } else {
        bundle.key.sibling(".phys")
    };
    bundle.insert(key, File::Physics(file));
    context.issue(
        "physics.version",
        IssueKind::Normalized,
        "rigid-body companion rebuilt using PHYS revision six",
    );
    Ok(())
}
