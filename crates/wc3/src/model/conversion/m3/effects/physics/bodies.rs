//! Body motion, physical parameters, force masks, and native control words.
use super::*;
use crate::model::conversion::IssueKind;

pub(super) fn import<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::RigidBody>,
    doc: &mut wem::RawDocument<'static>,
    nodes: &mut [wr::Node<wr::V19>],
    context: &mut Context,
) -> Result<Vec<wr::PhysicsBody<wr::V3>>, Error> {
    let mut output = Vec::new();
    macro_rules! modern {
        ($v:ty, $extra:expr) => {
            for (index, row) in source
                .resolve::<nr::RigidBody<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
                .enumerate()
            {
                let row = row.map_err(read_error)?;
                let mut values = BTreeMap::new();
                let prefix = format!("physics[{index}]");
                values.insert(format!("m3.{prefix}.priority"), i64::from(row.priority));
                values.insert(format!("m3.{prefix}.mass"), i64::from(row.mass.to_bits()));
                values.insert(format!("m3.{prefix}.simulation_type"), i64::from(row.simulation_type));
                $extra(&row, &mut values, &prefix);
                let node = nodes
                    .get_mut(row.parent_bone_index as usize)
                    .ok_or_else(|| Error::new("m3.physics.bone", "body bone is out of range"))?;
                shape_controls::import(source, row.rigid_body_shape, &mut values, &format!("physics[{index}]"))?;
                node.native = metadata::extend(doc, &node.native, &values)?;
                context.issue(format!("physics[{index}].motion"), IssueKind::Initialized, "initialized common dynamic motion; the uninterpreted native simulation-type control remains editable separately");
                if !row.mass.is_finite() || row.mass < 0. {
                    return Err(Error::new(
                        format!("physics[{index}].mass"),
                        "body mass must be finite and nonnegative",
                    ));
                }
                context.issue(
                    format!("physics[{index}].density"),
                    IssueKind::Initialized,
                    "initialized shape density; native body mass remains an editable named control",
                );
                let material = wr::PhysicsMaterial {
                    density: 1.,
                    friction: row.friction,
                    restitution: row.restitution,
                };
                let shapes = shapes::import(source, row.rigid_body_shape, doc, material, context)?;
                output.push(wr::PhysicsBody::<wr::V3> {
                    id: index as u32,
                    node: u32::from(row.parent_bone_index),
                    motion: wr::BodyMotion::DYNAMIC,
                    simulates: wem::Bool::from(row.flags & 0x80 == 0),
                    shapes,
                    linear_damping: row.linear_damping,
                    angular_damping: row.angular_damping,
                    inertia_scale: 1.,
                    gravity_scale: row.gravity_factor,
                    force_channels: u32::from(row.local_forces)
                        | (u32::from(row.world_forces) << 16),
                    sc2: Some(wr::Sc2BodyExtension {
                        physics_material: row.physics_type,
                        collidable: wem::Bool::from(row.flags & 1 != 0),
                        walkable: wem::Bool::from(row.flags & 2 != 0),
                        stackable: wem::Bool::from(row.flags & 4 != 0),
                        simulate_collision: wem::Bool::from(row.flags & 8 != 0),
                        untraced_flags: row.flags & !0x8f,
                        ..Default::default()
                    })
                    .into(),
                    ..Default::default()
                });
            }
        };
    }
    macro_rules! legacy {
        ($v:ty, $controls:expr) => {
            for (index, row) in source
                .resolve::<nr::RigidBody<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
                .enumerate()
            {
                let row = row.map_err(read_error)?;
                if row
                    .reserved_0
                    .0
                    .iter()
                    .chain(row.reserved_1.0.iter())
                    .any(|&b| b != 0)
                {
                    context.unsupported(
                        format!("physics[{index}].legacy_parameters"),
                        "legacy rigid-body parameter bytes have no established interpretation",
                    )?;
                }
                let (flags, local, world, priority): (u32, u16, u16, u32) = $controls(&row);
                let node = nodes
                    .get_mut(row.parent_bone_index as usize)
                    .ok_or_else(|| Error::new("m3.physics.bone", "body bone is out of range"))?;
                let mut values = BTreeMap::new();
                values.insert(format!("m3.physics[{index}].priority"), i64::from(priority));
                shape_controls::import(
                    source,
                    row.rigid_body_shape,
                    &mut values,
                    &format!("physics[{index}]"),
                )?;
                node.native = metadata::extend(doc, &node.native, &values)?;
                let shapes = shapes::import(
                    source,
                    row.rigid_body_shape,
                    doc,
                    wr::PhysicsMaterial::default(),
                    context,
                )?;
                context.issue(
                    format!("physics[{index}].parameters"),
                    IssueKind::Initialized,
                    "initialized explicit body parameters absent from the legacy layout",
                );
                output.push(wr::PhysicsBody::<wr::V3> {
                    id: index as u32,
                    node: u32::from(row.parent_bone_index),
                    motion: wr::BodyMotion::DYNAMIC,
                    simulates: wem::Bool::from(flags & 0x80 == 0),
                    shapes,
                    gravity_scale: 1.,
                    inertia_scale: 1.,
                    force_channels: u32::from(local) | (u32::from(world) << 16),
                    sc2: Some(wr::Sc2BodyExtension {
                        collidable: wem::Bool::from(flags & 1 != 0),
                        walkable: wem::Bool::from(flags & 2 != 0),
                        stackable: wem::Bool::from(flags & 4 != 0),
                        simulate_collision: wem::Bool::from(flags & 8 != 0),
                        untraced_flags: flags & !0x8f,
                        ..Default::default()
                    })
                    .into(),
                    ..Default::default()
                });
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        3 => modern!(nr::V3, |_: &nr::RigidBody<nr::V3>,
                              _: &mut BTreeMap<String, i64>,
                              _: &str| {}),
        4 => modern!(nr::V4, |r: &nr::RigidBody<nr::V4>,
                              values: &mut BTreeMap<String, i64>,
                              prefix: &str| {
            animated(values, &format!("{prefix}.dynamic_state"), r.dynamic_state);
            values.insert(
                format!("m3.{prefix}.dynamic_blend_out"),
                i64::from(r.dynamic_blend_out.to_bits()),
            );
        }),
        0 => legacy!(nr::V0, |_: &nr::RigidBody<nr::V0>| (0, 0, 0, 0)),
        1 => legacy!(nr::V1, |_: &nr::RigidBody<nr::V1>| (0, 0, 0, 0)),
        2 => legacy!(nr::V2, |r: &nr::RigidBody<nr::V2>| (
            r.flags,
            r.local_forces,
            r.world_forces,
            r.priority
        )),
        _ => return Err(Error::new("m3.physics.bodies", "unsupported body revision")),
    }
    Ok(output)
}

pub(super) fn export<E: Envelope>(
    source: &wem::RawDocument<'_>,
    model: wr::ModelRecord<wr::V9>,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
    context: &mut Context,
) -> Result<Ref<nr::targets::RigidBody>, Error> {
    let nodes = source.resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?;
    let mut output = Vec::new();
    for (index, body) in source
        .resolve::<wr::PhysicsBody<wr::V3>>(model.physics.bodies.cast())?
        .iter()
        .enumerate()
    {
        let body = body?;
        if body.node == u32::MAX || scene_shape(source, &model, body.id)? {
            continue;
        }
        if body.inertia_scale != 1.
            || body.wow.value.is_some()
            || body.inherit_dynamic.get()
            || body.exempt_from_ragdoll.get()
        {
            context.unsupported(
                format!("physics[{index}].drive"),
                "body drive controls have no established M3 mapping",
            )?;
        }
        let bone = *bone_map
            .get(body.node as usize)
            .ok_or_else(|| Error::new("wem.physics.node", "missing body bone mapping"))?;
        let node = nodes
            .get(body.node as usize)
            .ok_or_else(|| Error::new("wem.physics.node", "missing body node"))??;
        let values = metadata::read(source, &node.native)?;
        let prefix = format!("physics[{}]", body.id);
        let extension = body.sc2.value.unwrap_or_default();
        if body.motion != wr::BodyMotion::DYNAMIC {
            context.unsupported(
                format!("physics[{index}].motion"),
                "no verified mapping exists from common body motion to M3 simulation type",
            )?;
        }
        if extension.keeps_bone_driven.get() {
            context.unsupported(
                format!("physics[{index}].keeps_bone_driven"),
                "the bone-drive setting has no verified M3 flag mapping",
            )?;
        }
        let flags = extension.untraced_flags
            | u32::from(extension.collidable.get())
            | (u32::from(extension.walkable.get()) << 1)
            | (u32::from(extension.stackable.get()) << 2)
            | (u32::from(extension.simulate_collision.get()) << 3)
            | (u32::from(!body.simulates.get()) << 7);
        let (shapes, material) =
            shapes::export(source, body.shapes, doc, context, &values, &prefix)?;
        if material.density != 1. {
            context.unsupported(format!("physics[{index}].density"), "M3 stores body mass; translating edited shape density requires a verified solid volume")?;
        }
        let mass = values.get(&format!("m3.{prefix}.mass")).copied();
        if mass.is_none() {
            context.issue(format!("physics[{index}].mass"), IssueKind::Initialized, "initialized body mass to one; shape density does not determine mass without a verified solid volume");
        }
        let mass = f32::from_bits(mass.unwrap_or(i64::from(1f32.to_bits())) as u32);
        if !mass.is_finite() || mass < 0. {
            return Err(Error::new(
                format!("physics[{index}].mass"),
                "body mass must be finite and nonnegative",
            ));
        }
        output.push(nr::RigidBody::<nr::V4> {
            parent_bone_index: u16::try_from(bone)
                .map_err(|_| Error::new("wem.physics.node", "body bone exceeds 16 bits"))?,
            simulation_type: u16::try_from(
                values
                    .get(&format!("m3.{prefix}.simulation_type"))
                    .copied()
                    .unwrap_or(1),
            )
            .map_err(|_| {
                Error::new(
                    format!("physics[{index}].simulation_type"),
                    "native simulation type exceeds 16 bits",
                )
            })?,
            physics_type: extension.physics_material,
            mass,
            friction: material.friction,
            restitution: material.restitution,
            linear_damping: body.linear_damping,
            angular_damping: body.angular_damping,
            gravity_factor: body.gravity_scale,
            flags,
            local_forces: body.force_channels as u16,
            world_forces: (body.force_channels >> 16) as u16,
            rigid_body_shape: shapes,
            priority: values
                .get(&format!("m3.{prefix}.priority"))
                .copied()
                .unwrap_or(0) as u32,
            dynamic_state: restore(
                &values,
                &format!("{prefix}.dynamic_state"),
                u32::from(body.simulates.get()),
            ),
            dynamic_blend_out: f32::from_bits(
                values
                    .get(&format!("m3.{prefix}.dynamic_blend_out"))
                    .copied()
                    .unwrap_or(0) as u32,
            ),
            ..Default::default()
        });
    }
    Ok(append(doc, &output)?.cast())
}

fn scene_shape(
    source: &wem::RawDocument<'_>,
    model: &wr::ModelRecord<wr::V9>,
    id: u32,
) -> Result<bool, Error> {
    for profile in source
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .iter()
    {
        let profile = profile?;
        let values = metadata::read(source, &profile.native)?;
        if values.contains_key(&format!("m3.collision.body[{id}].role")) {
            return Ok(true);
        }
    }
    Ok(false)
}
