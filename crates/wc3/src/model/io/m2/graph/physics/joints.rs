//! Versioned joint payloads become direct typed constraints.
use super::*;
use crate::model::error::Location;
use glam::Vec2;
use native::records::PhysicsJointKind;
use profile::{values::PhysicsMotorMode, Constraint, Joint, Motor, Spring};
fn motor(mode: native::records::PhysicsMotorMode, force: f32, hz: f32, damping: f32) -> Motor {
    Motor {
        mode: PhysicsMotorMode::from_raw(mode.raw()),
        maximum_force: force,
        spring: Spring {
            frequency_hz: hz,
            damping_ratio: damping,
        },
    }
}
macro_rules! rows {
    ($file:expr,$variant:ident) => {
        $file
            .chunks
            .iter()
            .filter_map(|chunk| {
                if let Pc::$variant(rows) = chunk {
                    Some(rows)
                } else {
                    None
                }
            })
            .flatten()
    };
}
pub(super) fn joints(file: &PhysicsFile<'_>, bodies: &[NodeId]) -> Result<Vec<Joint>, Error> {
    let mut out = Vec::new();
    for definition in rows!(file, Joints) {
        let index = usize::try_from(definition.joint_id).map_err(|_| {
            Error::requirement(
                Location::new().field("physics").field("joint"),
                "negative payload index",
            )
        })?;
        let missing = || {
            Error::requirement(
                Location::new().field("physics").field("joint"),
                "constraint payload is absent",
            )
        };
        let (frame_a, frame_b, constraint) = match definition.joint_type {
            PhysicsJointKind::Spherical => {
                let value = rows!(file, Sphericals).nth(index).ok_or_else(missing)?;
                (
                    anchor(value.anchor_a),
                    anchor(value.anchor_b),
                    Constraint::Spherical {
                        friction_torque: value.friction_torque,
                    },
                )
            }
            PhysicsJointKind::Distance => {
                let value = rows!(file, Distances).nth(index).ok_or_else(missing)?;
                (
                    anchor(value.anchor_a),
                    anchor(value.anchor_b),
                    Constraint::Distance {
                        distance: value.distance,
                        spring: Spring::default(),
                    },
                )
            }
            PhysicsJointKind::Shoulder => {
                let values: Vec<_> = file
                    .chunks
                    .iter()
                    .flat_map(|chunk| match chunk {
                        Pc::Shoulders(rows) => {
                            rows.iter().map(|row| (row, 0., 0.)).collect::<Vec<_>>()
                        }
                        Pc::Shoulders2(rows) => rows
                            .iter()
                            .map(|row| {
                                (
                                    &row.shoulder,
                                    row.motor_frequency_hz,
                                    row.motor_damping_ratio,
                                )
                            })
                            .collect(),
                        _ => Vec::new(),
                    })
                    .collect();
                let (value, hz, damping) = values.get(index).copied().ok_or_else(missing)?;
                (
                    frame(&value.frame_a),
                    frame(&value.frame_b),
                    Constraint::Shoulder {
                        twist: [
                            f64::from(value.lower_twist_angle).to_radians(),
                            f64::from(value.upper_twist_angle).to_radians(),
                        ],
                        cone: f64::from(value.cone_angle).to_radians(),
                        motor: motor(value.motor_mode, value.max_motor_torque, hz, damping),
                    },
                )
            }
            PhysicsJointKind::Weld => {
                let values: Vec<_> = file
                    .chunks
                    .iter()
                    .flat_map(|chunk| match chunk {
                        Pc::Welds(rows) => rows.iter().map(|row| (row, 0., 0.)).collect::<Vec<_>>(),
                        Pc::Welds2(rows) => rows
                            .iter()
                            .map(|row| {
                                (&row.weld, row.linear_frequency_hz, row.linear_damping_ratio)
                            })
                            .collect(),
                        Pc::Welds3(rows) => rows
                            .iter()
                            .map(|row| {
                                (
                                    &row.weld.weld,
                                    row.weld.linear_frequency_hz,
                                    row.weld.linear_damping_ratio,
                                )
                            })
                            .collect(),
                        _ => Vec::new(),
                    })
                    .collect();
                let (value, hz, damping) = values.get(index).copied().ok_or_else(missing)?;
                (
                    frame(&value.frame_a),
                    frame(&value.frame_b),
                    Constraint::Weld {
                        linear: Spring {
                            frequency_hz: hz,
                            damping_ratio: damping,
                        },
                        angular: Spring {
                            frequency_hz: value.angular_frequency_hz,
                            damping_ratio: value.angular_damping_ratio,
                        },
                    },
                )
            }
            PhysicsJointKind::Revolute => {
                let values: Vec<_> = file
                    .chunks
                    .iter()
                    .flat_map(|chunk| match chunk {
                        Pc::Revolutes(rows) => {
                            rows.iter().map(|row| (row, 0., 0.)).collect::<Vec<_>>()
                        }
                        Pc::Revolutes2(rows) => rows
                            .iter()
                            .map(|row| {
                                (
                                    &row.revolute,
                                    row.motor_frequency_hz,
                                    row.motor_damping_ratio,
                                )
                            })
                            .collect(),
                        _ => Vec::new(),
                    })
                    .collect();
                let (value, hz, damping) = values.get(index).copied().ok_or_else(missing)?;
                (
                    frame(&value.frame_a),
                    frame(&value.frame_b),
                    Constraint::Revolute {
                        angles: [
                            f64::from(value.lower_angle).to_radians(),
                            f64::from(value.upper_angle).to_radians(),
                        ],
                        friction_torque: 0.,
                        motor: motor(value.motor_mode, value.max_motor_torque, hz, damping),
                    },
                )
            }
            PhysicsJointKind::Prismatic => {
                let values: Vec<_> = file
                    .chunks
                    .iter()
                    .flat_map(|chunk| match chunk {
                        Pc::Prismatics(rows) => {
                            rows.iter().map(|row| (row, 0., 0.)).collect::<Vec<_>>()
                        }
                        Pc::Prismatics2(rows) => rows
                            .iter()
                            .map(|row| {
                                (
                                    &row.prismatic,
                                    row.motor_frequency_hz,
                                    row.motor_damping_ratio,
                                )
                            })
                            .collect(),
                        _ => Vec::new(),
                    })
                    .collect();
                let (value, hz, damping) = values.get(index).copied().ok_or_else(missing)?;
                (
                    frame(&value.frame_a),
                    frame(&value.frame_b),
                    Constraint::Prismatic {
                        distances: Vec2::from_array([value.lower_limit, value.upper_limit]),
                        reference_translation: value.reference_translation,
                        motor_speed: value.motor_speed,
                        friction_force: 0.,
                        motor: motor(value.motor_mode, value.max_motor_force, hz, damping),
                    },
                )
            }
            PhysicsJointKind::Unknown(value) => {
                return Err(Error::requirement(
                    Location::new().field("physics").field("joint"),
                    format!("unknown constraint selector {value}"),
                ));
            }
        };
        let body_a = *bodies.get(definition.body_a as usize).ok_or_else(|| {
            Error::requirement(
                Location::new()
                    .field("physics")
                    .field("joint")
                    .field("body_a"),
                "body is absent",
            )
        })?;
        let body_b = *bodies.get(definition.body_b as usize).ok_or_else(|| {
            Error::requirement(
                Location::new()
                    .field("physics")
                    .field("joint")
                    .field("body_b"),
                "body is absent",
            )
        })?;
        out.push(Joint {
            body_a,
            body_b,
            frame_a,
            frame_b,
            constraint,
        });
    }
    Ok(out)
}
