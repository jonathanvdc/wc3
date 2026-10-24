//! Emits contemporary native joint tables from actual logical constraints.
use super::*;
use crate::model::error::Location;
use native::companions::*;
use native::records::{PhysicsJointKind, PhysicsMotorMode};
use profile::{Constraint, Joint};
#[derive(Default)]
pub(super) struct Constraints {
    pub joints: Vec<PhysicsJoint>,
    pub spherical: Vec<PhysicsSpherical>,
    pub shoulder: Vec<PhysicsShoulder2>,
    pub weld: Vec<PhysicsWeld3>,
    pub revolute: Vec<PhysicsRevolute2>,
    pub prismatic: Vec<PhysicsPrismatic2>,
    pub distance: Vec<PhysicsDistance>,
}
impl Constraints {
    pub(super) fn add(
        &mut self,
        joint: &Joint,
        bodies: &[NodeId],
        context: &mut Context,
    ) -> Result<(), Error> {
        let (kind, index) = match joint.constraint {
            Constraint::Spherical { friction_torque } => {
                let index = self.spherical.len();
                self.spherical.push(PhysicsSpherical {
                    anchor_a: (joint.frame_a.origin).to_array(),
                    anchor_b: (joint.frame_b.origin).to_array(),
                    friction_torque,
                });
                (PhysicsJointKind::Spherical, index)
            }
            Constraint::Shoulder { twist, cone, motor } => {
                let index = self.shoulder.len();
                self.shoulder.push(PhysicsShoulder2 {
                    shoulder: PhysicsShoulder {
                        frame_a: native_frame(joint.frame_a),
                        frame_b: native_frame(joint.frame_b),
                        lower_twist_angle: twist[0].to_degrees() as f32,
                        upper_twist_angle: twist[1].to_degrees() as f32,
                        cone_angle: cone.to_degrees() as f32,
                        max_motor_torque: motor.maximum_force,
                        motor_mode: PhysicsMotorMode::from_raw(motor.mode.raw()),
                    },
                    motor_frequency_hz: motor.spring.frequency_hz,
                    motor_damping_ratio: motor.spring.damping_ratio,
                });
                (PhysicsJointKind::Shoulder, index)
            }
            Constraint::Weld { linear, angular } => {
                let index = self.weld.len();
                self.weld.push(PhysicsWeld3 {
                    weld: PhysicsWeld2 {
                        weld: PhysicsWeld {
                            frame_a: native_frame(joint.frame_a),
                            frame_b: native_frame(joint.frame_b),
                            angular_frequency_hz: angular.frequency_hz,
                            angular_damping_ratio: angular.damping_ratio,
                        },
                        linear_frequency_hz: linear.frequency_hz,
                        linear_damping_ratio: linear.damping_ratio,
                    },
                    unused: 0.,
                });
                (PhysicsJointKind::Weld, index)
            }
            Constraint::Revolute {
                angles,
                friction_torque,
                motor,
            } => {
                if friction_torque != 0. {
                    context.unsupported(
                        Location::new()
                            .field("physics")
                            .field("joint")
                            .field("friction"),
                        "M2 hinge joints cannot express independent friction torque",
                    )?;
                }
                let index = self.revolute.len();
                self.revolute.push(PhysicsRevolute2 {
                    revolute: PhysicsRevolute {
                        frame_a: native_frame(joint.frame_a),
                        frame_b: native_frame(joint.frame_b),
                        lower_angle: angles[0].to_degrees() as f32,
                        upper_angle: angles[1].to_degrees() as f32,
                        max_motor_torque: motor.maximum_force,
                        motor_mode: PhysicsMotorMode::from_raw(motor.mode.raw()),
                    },
                    motor_frequency_hz: motor.spring.frequency_hz,
                    motor_damping_ratio: motor.spring.damping_ratio,
                });
                (PhysicsJointKind::Revolute, index)
            }
            Constraint::Prismatic {
                distances,
                reference_translation,
                motor_speed,
                friction_force,
                motor,
            } => {
                if friction_force != 0. {
                    context.unsupported(
                        Location::new()
                            .field("physics")
                            .field("joint")
                            .field("friction"),
                        "M2 sliding joints cannot express independent friction force",
                    )?;
                }
                let index = self.prismatic.len();
                self.prismatic.push(PhysicsPrismatic2 {
                    prismatic: PhysicsPrismatic {
                        frame_a: native_frame(joint.frame_a),
                        frame_b: native_frame(joint.frame_b),
                        lower_limit: distances[0],
                        upper_limit: distances[1],
                        reference_translation,
                        motor_speed,
                        max_motor_force: motor.maximum_force,
                        motor_mode: PhysicsMotorMode::from_raw(motor.mode.raw()),
                    },
                    motor_frequency_hz: motor.spring.frequency_hz,
                    motor_damping_ratio: motor.spring.damping_ratio,
                });
                (PhysicsJointKind::Prismatic, index)
            }
            Constraint::Distance { distance, spring } => {
                if spring.frequency_hz != 0. || spring.damping_ratio != 0. {
                    context.unsupported(
                        Location::new()
                            .field("physics")
                            .field("joint")
                            .field("spring"),
                        "M2 distance joints cannot express a compliant spring",
                    )?;
                }
                let index = self.distance.len();
                self.distance.push(PhysicsDistance {
                    anchor_a: (joint.frame_a.origin).to_array(),
                    anchor_b: (joint.frame_b.origin).to_array(),
                    distance,
                });
                (PhysicsJointKind::Distance, index)
            }
        };
        let body_index = |node| {
            bodies.iter().position(|id| *id == node).ok_or_else(|| {
                Error::requirement(
                    Location::new()
                        .field("physics")
                        .field("joint")
                        .field("body"),
                    "body is absent",
                )
            })
        };
        self.joints.push(PhysicsJoint {
            body_a: body_index(joint.body_a)? as u32,
            body_b: body_index(joint.body_b)? as u32,
            joint_type: kind,
            joint_id: i16::try_from(index).map_err(|_| {
                Error::requirement(
                    Location::new().field("physics").field("joint"),
                    "constraint index exceeds native range",
                )
            })?,
            reserved: 0,
        });
        Ok(())
    }
}
