//! Native joints linked through stable WEM body identities.
use super::*;
use std::f32::consts::TAU;

pub(super) fn import<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::PhysicsJoint>,
    bodies: &[wr::PhysicsBody<wr::V3>],
    doc: &mut wem::RawDocument<'static>,
    nodes: &mut [wr::Node<wr::V19>],
    context: &mut Context,
) -> Result<Vec<wr::PhysicsJoint<wr::V3>>, Error> {
    let mut output = Vec::new();
    let body_for = |bone: u32| -> Result<&wr::PhysicsBody<wr::V3>, Error> {
        let mut matches = bodies.iter().filter(|b| b.node == bone);
        let body = matches
            .next()
            .ok_or_else(|| Error::new("m3.physics.joint.bone", "joint bone has no body"))?;
        if matches.next().is_some() {
            return Err(Error::new(
                "m3.physics.joint.bone",
                "joint bone has several bodies",
            ));
        }
        Ok(body)
    };
    for (index, row) in source
        .resolve::<nr::PhysicsJoint>(reference.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        let row = row.map_err(read_error)?;
        let kind = match row.joint_type {
            nr::JointKind::Spherical => wr::JointKind::Spherical,
            nr::JointKind::Revolute => wr::JointKind::Revolute,
            nr::JointKind::Cone => wr::JointKind::ConeTwist,
            nr::JointKind::Weld => wr::JointKind::Weld,
            nr::JointKind::Unknown(_) => {
                return Err(Error::new("m3.physics.joint.kind", "unknown joint kind"));
            }
        };
        if row.reserved_0 != [0; 3] {
            context.unsupported(
                format!("physics.joints[{index}].reserved"),
                "uninterpreted joint alignment bytes",
            )?;
        }
        let a = body_for(row.bone_index1)?;
        let b = body_for(row.bone_index2)?;
        let prefix = format!("m3.joints[{index}]");
        let mut values = BTreeMap::new();
        for (name, value) in [
            ("enable_limits", row.enable_limits),
            ("enable_friction", row.enable_friction),
            ("angular_frequency", row.angular_frequency.to_bits()),
            ("break_threshold", row.break_threshold.to_bits()),
        ] {
            values.insert(format!("{prefix}.{name}"), i64::from(value));
        }
        nodes[a.node as usize].native =
            metadata::extend(doc, &nodes[a.node as usize].native, &values)?;
        output.push(wr::PhysicsJoint::<wr::V3> {
            id: (bodies.len() + index) as u32,
            body_a: a.id,
            body_b: b.id,
            kind,
            frame_a: row.matrix_body1,
            frame_b: row.matrix_body2,
            collide_connected: wem::Bool(row.enable_shape),
            limit_enabled: wem::Bool((row.enable_limits & 255) as u8),
            lower: row.limit_min,
            upper: row.limit_max,
            cone: row.cone_angle,
            friction: if row.enable_friction & 255 != 0 {
                wr::JointFriction::Torque
            } else {
                wr::JointFriction::None
            },
            friction_amount: row.friction,
            angular_spring: wr::JointSpring {
                hz: row.angular_frequency / TAU,
                damping: row.damping_ratio,
            },
            ..Default::default()
        });
    }
    Ok(output)
}

pub(super) fn export<E: Envelope>(
    source: &wem::RawDocument<'_>,
    model: &wr::ModelRecord<wr::V9>,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
    context: &mut Context,
) -> Result<Ref<nr::targets::PhysicsJoint>, Error> {
    let bodies = source
        .resolve::<wr::PhysicsBody<wr::V3>>(model.physics.bodies.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let nodes = source.resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?;
    let body_for = |id: u32| {
        bodies
            .iter()
            .find(|b| b.id == id)
            .ok_or_else(|| Error::new("wem.physics.joint.body", "joint body identity missing"))
    };
    let mut output = Vec::new();
    for (index, row) in source
        .resolve::<wr::PhysicsJoint<wr::V3>>(model.physics.joints.cast())?
        .iter()
        .enumerate()
    {
        let row = row?;
        let a = body_for(row.body_a)?;
        let b = body_for(row.body_b)?;
        let native_kind = match row.kind {
            wr::JointKind::Spherical => Some(nr::JointKind::Spherical),
            wr::JointKind::Revolute => Some(nr::JointKind::Revolute),
            wr::JointKind::ConeTwist => Some(nr::JointKind::Cone),
            wr::JointKind::Weld => Some(nr::JointKind::Weld),
            _ => None,
        };
        if native_kind.is_none()
            || row.friction == wr::JointFriction::GravityHold
            || row.motor != wr::JointMotorMode::Off
            || row.max_motor_force != 0.
            || row.motor_speed != 0.
            || row.reference_translation != 0.
            || row.rest_length != 0.
            || row.linear_spring.hz != 0.
            || row.linear_spring.damping != 0.
            || row.break_force != 0.
            || row.break_torque != 0.
        {
            context.unsupported(
                format!("physics.joints[{index}].controls"),
                "joint controls have no native M3 equivalent",
            )?;
        }
        let Some(native_kind) = native_kind else {
            continue;
        };
        let node = nodes
            .get(a.node as usize)
            .ok_or_else(|| Error::new("wem.physics.node", "missing body node"))??;
        let values = metadata::read(source, &node.native)?;
        let prefix = format!("m3.joints[{index}]");
        let original = f32::from_bits(
            values
                .get(&format!("{prefix}.angular_frequency"))
                .copied()
                .unwrap_or(0) as u32,
        );
        let frequency = if row.angular_spring.hz == original / TAU {
            original
        } else {
            row.angular_spring.hz * TAU
        };
        output.push(nr::PhysicsJoint {
            joint_type: native_kind,
            bone_index1: *bone_map
                .get(a.node as usize)
                .ok_or_else(|| Error::new("wem.physics.node", "missing joint bone mapping"))?,
            bone_index2: *bone_map
                .get(b.node as usize)
                .ok_or_else(|| Error::new("wem.physics.node", "missing joint bone mapping"))?,
            matrix_body1: row.frame_a,
            matrix_body2: row.frame_b,
            enable_limits: (values
                .get(&format!("{prefix}.enable_limits"))
                .copied()
                .unwrap_or(0) as u32
                & !255)
                | u32::from(row.limit_enabled.0),
            limit_min: row.lower,
            limit_max: row.upper,
            cone_angle: row.cone,
            enable_friction: (values
                .get(&format!("{prefix}.enable_friction"))
                .copied()
                .unwrap_or(0) as u32
                & !255)
                | u32::from(row.friction != wr::JointFriction::None),
            friction: row.friction_amount,
            damping_ratio: row.angular_spring.damping,
            angular_frequency: frequency,
            break_threshold: f32::from_bits(
                values
                    .get(&format!("{prefix}.break_threshold"))
                    .copied()
                    .unwrap_or(0) as u32,
            ),
            enable_shape: row.collide_connected.0,
            ..Default::default()
        });
    }
    Ok(append(doc, &output)?.cast())
}
