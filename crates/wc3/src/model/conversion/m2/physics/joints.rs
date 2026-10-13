//! Native joint variants mapped to explicit WEM frames, limits, springs, and motors.
use super::*;
fn spring(hz: f32, damping: f32) -> wr::JointSpring {
    wr::JointSpring { hz, damping }
}
fn motor(code: u32) -> Result<wr::JointMotorMode, Error> {
    Ok(wr::JointMotorMode(u8::try_from(code).map_err(|_| {
        Error::new("physics.joint.motor", "motor code exceeds WEM's byte field")
    })?))
}
pub(super) fn joints(
    source: &PhysicsFile<'_>,
    first_id: u32,
) -> Result<Vec<wr::PhysicsJoint<wr::V3>>, Error> {
    let mut rows = Vec::new();
    let definitions = source
        .chunks
        .iter()
        .filter_map(|chunk| {
            if let Pc::Joints(rows) = chunk {
                Some(rows)
            } else {
                None
            }
        })
        .flatten();
    for (index, joint) in definitions.enumerate() {
        let key = usize::try_from(joint.joint_id)
            .map_err(|_| Error::new("physics.joint", "negative joint payload index"))?;
        let mut row = wr::PhysicsJoint::<wr::V3> {
            id: first_id + index as u32,
            body_a: joint.body_a + 1,
            body_b: joint.body_b + 1,
            frame_a: identity(),
            frame_b: identity(),
            ..Default::default()
        };
        match joint.joint_type {
            0 => {
                let value = source
                    .chunks
                    .iter()
                    .filter_map(|c| {
                        if let Pc::Sphericals(v) = c {
                            Some(v)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .nth(key)
                    .ok_or_else(|| {
                        Error::new("physics.joint.spherical", "missing constraint payload")
                    })?;
                row.kind = wr::JointKind::SPHERICAL;
                row.frame_a = translate(value.anchor_a);
                row.frame_b = translate(value.anchor_b);
                row.friction = wr::JointFriction::TORQUE;
                row.friction_amount = value.friction_torque;
            }
            1 => {
                let values: Vec<_> = source
                    .chunks
                    .iter()
                    .flat_map(|c| match c {
                        Pc::Shoulders(v) => v.iter().map(|v| (v, 0., 0.)).collect(),
                        Pc::Shoulders2(v) => v
                            .iter()
                            .map(|v| (&v.shoulder, v.motor_frequency_hz, v.motor_damping_ratio))
                            .collect(),
                        _ => Vec::new(),
                    })
                    .collect();
                let (value, hz, damping) = values.get(key).copied().ok_or_else(|| {
                    Error::new("physics.joint.shoulder", "missing constraint payload")
                })?;
                row.kind = wr::JointKind::CONE_TWIST;
                row.frame_a = frame(&value.frame_a);
                row.frame_b = frame(&value.frame_b);
                row.lower = value.lower_twist_angle.to_radians();
                row.upper = value.upper_twist_angle.to_radians();
                row.cone = value.cone_angle.to_radians();
                row.limit_enabled =
                    wem::Bool::from(value.upper_twist_angle > value.lower_twist_angle);
                row.motor = motor(value.motor_mode)?;
                row.max_motor_force = value.max_motor_torque;
                row.angular_spring = spring(hz, damping);
            }
            2 => {
                let values: Vec<_> = source
                    .chunks
                    .iter()
                    .flat_map(|c| match c {
                        Pc::Welds(v) => v.iter().map(|v| (v, 0., 0.)).collect(),
                        Pc::Welds2(v) => v
                            .iter()
                            .map(|v| (&v.weld, v.linear_frequency_hz, v.linear_damping_ratio))
                            .collect(),
                        Pc::Welds3(v) => v
                            .iter()
                            .map(|v| {
                                (
                                    &v.weld.weld,
                                    v.weld.linear_frequency_hz,
                                    v.weld.linear_damping_ratio,
                                )
                            })
                            .collect(),
                        _ => Vec::new(),
                    })
                    .collect();
                let (value, hz, damping) = values.get(key).copied().ok_or_else(|| {
                    Error::new("physics.joint.weld", "missing constraint payload")
                })?;
                row.kind = wr::JointKind::WELD;
                row.frame_a = frame(&value.frame_a);
                row.frame_b = frame(&value.frame_b);
                row.angular_spring =
                    spring(value.angular_frequency_hz, value.angular_damping_ratio);
                row.linear_spring = spring(hz, damping);
            }
            3 => {
                let values: Vec<_> = source
                    .chunks
                    .iter()
                    .flat_map(|c| match c {
                        Pc::Revolutes(v) => v.iter().map(|v| (v, 0., 0.)).collect(),
                        Pc::Revolutes2(v) => v
                            .iter()
                            .map(|v| (&v.revolute, v.motor_frequency_hz, v.motor_damping_ratio))
                            .collect(),
                        _ => Vec::new(),
                    })
                    .collect();
                let (value, hz, damping) = values.get(key).copied().ok_or_else(|| {
                    Error::new("physics.joint.revolute", "missing constraint payload")
                })?;
                row.kind = wr::JointKind::REVOLUTE;
                row.frame_a = frame(&value.frame_a);
                row.frame_b = frame(&value.frame_b);
                row.lower = value.lower_angle.to_radians();
                row.upper = value.upper_angle.to_radians();
                row.limit_enabled = wem::Bool::from(value.upper_angle > value.lower_angle);
                row.motor = motor(value.motor_mode)?;
                row.max_motor_force = value.max_motor_torque;
                row.angular_spring = spring(hz, damping);
            }
            4 => {
                let values: Vec<_> = source
                    .chunks
                    .iter()
                    .flat_map(|c| match c {
                        Pc::Prismatics(v) => v.iter().map(|v| (v, 0., 0.)).collect(),
                        Pc::Prismatics2(v) => v
                            .iter()
                            .map(|v| (&v.prismatic, v.motor_frequency_hz, v.motor_damping_ratio))
                            .collect(),
                        _ => Vec::new(),
                    })
                    .collect();
                let (value, hz, damping) = values.get(key).copied().ok_or_else(|| {
                    Error::new("physics.joint.prismatic", "missing constraint payload")
                })?;
                row.kind = wr::JointKind::PRISMATIC;
                row.frame_a = frame(&value.frame_a);
                row.frame_b = frame(&value.frame_b);
                row.lower = value.lower_limit;
                row.upper = value.upper_limit;
                row.limit_enabled = wem::Bool::from(value.upper_limit > value.lower_limit);
                row.reference_translation = value.reference_translation;
                row.motor = motor(value.motor_mode)?;
                row.max_motor_force = value.max_motor_force;
                row.motor_speed = value.motor_speed;
                row.linear_spring = spring(hz, damping);
            }
            5 => {
                let value = source
                    .chunks
                    .iter()
                    .filter_map(|c| {
                        if let Pc::Distances(v) = c {
                            Some(v)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .nth(key)
                    .ok_or_else(|| {
                        Error::new("physics.joint.distance", "missing constraint payload")
                    })?;
                row.kind = wr::JointKind::DISTANCE;
                row.frame_a = translate(value.anchor_a);
                row.frame_b = translate(value.anchor_b);
                row.rest_length = value.distance;
            }
            _ => {
                return Err(Error::new(
                    "physics.joint.type",
                    "unknown native joint category",
                ));
            }
        }
        rows.push(row);
    }
    Ok(rows)
}
