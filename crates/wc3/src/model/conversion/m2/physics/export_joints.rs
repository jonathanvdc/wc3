//! Native constraint payloads generated from WEM joint fields.
use super::*;
#[derive(Default)]
pub(super) struct Constraints {
    pub shoulders: Vec<native::companions::PhysicsShoulder2>,
    pub welds: Vec<native::companions::PhysicsWeld3>,
    pub revolutes: Vec<native::companions::PhysicsRevolute2>,
    pub sphericals: Vec<native::companions::PhysicsSpherical>,
    pub distances: Vec<native::companions::PhysicsDistance>,
    pub prismatics: Vec<native::companions::PhysicsPrismatic2>,
    pub links: Vec<native::companions::PhysicsJoint>,
}
impl Constraints {
    pub(super) fn add(
        &mut self,
        joint: &wr::PhysicsJoint<wr::V3>,
        bodies: &BTreeMap<u32, u32>,
        context: &mut Context,
    ) -> Result<(), Error> {
        if joint.break_force != 0. || joint.break_torque != 0. || joint.collide_connected.get() {
            context.unsupported(
                "physics.joint",
                "native PHYS has no per-constraint breaking or collide-connected fields",
            )?;
        }
        let a = *bodies
            .get(&joint.body_a)
            .ok_or_else(|| Error::new("physics.joint.body_a", "joint references a missing body"))?;
        let b = *bodies
            .get(&joint.body_b)
            .ok_or_else(|| Error::new("physics.joint.body_b", "joint references a missing body"))?;
        let (kind, index) = match joint.kind {
            wr::JointKind::Spherical => {
                if joint.friction != wr::JointFriction::None
                    && joint.friction != wr::JointFriction::Torque
                {
                    context.unsupported(
                        "physics.joint.friction",
                        "native spherical friction is a torque coefficient",
                    )?;
                }
                let index = self.sphericals.len();
                self.sphericals.push(native::companions::PhysicsSpherical {
                    anchor_a: joint.frame_a[3][..3].try_into().unwrap(),
                    anchor_b: joint.frame_b[3][..3].try_into().unwrap(),
                    friction_torque: joint.friction_amount,
                });
                (PhysicsJointKind::Spherical, index)
            }
            wr::JointKind::ConeTwist => {
                let index = self.shoulders.len();
                self.shoulders.push(native::companions::PhysicsShoulder2 {
                    shoulder: native::companions::PhysicsShoulder {
                        frame_a: native_frame(joint.frame_a),
                        frame_b: native_frame(joint.frame_b),
                        lower_twist_angle: joint.lower.to_degrees(),
                        upper_twist_angle: if joint.limit_enabled.get() {
                            joint.upper.to_degrees()
                        } else {
                            joint.lower.to_degrees()
                        },
                        cone_angle: joint.cone.to_degrees(),
                        max_motor_torque: joint.max_motor_force,
                        motor_mode: PhysicsMotorMode::from_raw(u32::from(joint.motor.raw())),
                    },
                    motor_frequency_hz: joint.angular_spring.hz,
                    motor_damping_ratio: joint.angular_spring.damping,
                });
                (PhysicsJointKind::Shoulder, index)
            }
            wr::JointKind::Weld => {
                let index = self.welds.len();
                self.welds.push(native::companions::PhysicsWeld3 {
                    weld: native::companions::PhysicsWeld2 {
                        weld: native::companions::PhysicsWeld {
                            frame_a: native_frame(joint.frame_a),
                            frame_b: native_frame(joint.frame_b),
                            angular_frequency_hz: joint.angular_spring.hz,
                            angular_damping_ratio: joint.angular_spring.damping,
                        },
                        linear_frequency_hz: joint.linear_spring.hz,
                        linear_damping_ratio: joint.linear_spring.damping,
                    },
                    unused: 0.,
                });
                (PhysicsJointKind::Weld, index)
            }
            wr::JointKind::Revolute => {
                let index = self.revolutes.len();
                self.revolutes.push(native::companions::PhysicsRevolute2 {
                    revolute: native::companions::PhysicsRevolute {
                        frame_a: native_frame(joint.frame_a),
                        frame_b: native_frame(joint.frame_b),
                        lower_angle: joint.lower.to_degrees(),
                        upper_angle: if joint.limit_enabled.get() {
                            joint.upper.to_degrees()
                        } else {
                            joint.lower.to_degrees()
                        },
                        max_motor_torque: joint.max_motor_force,
                        motor_mode: PhysicsMotorMode::from_raw(u32::from(joint.motor.raw())),
                    },
                    motor_frequency_hz: joint.angular_spring.hz,
                    motor_damping_ratio: joint.angular_spring.damping,
                });
                (PhysicsJointKind::Revolute, index)
            }
            wr::JointKind::Prismatic => {
                let index = self.prismatics.len();
                self.prismatics.push(native::companions::PhysicsPrismatic2 {
                    prismatic: native::companions::PhysicsPrismatic {
                        frame_a: native_frame(joint.frame_a),
                        frame_b: native_frame(joint.frame_b),
                        lower_limit: joint.lower,
                        upper_limit: if joint.limit_enabled.get() {
                            joint.upper
                        } else {
                            joint.lower
                        },
                        reference_translation: joint.reference_translation,
                        max_motor_force: joint.max_motor_force,
                        motor_speed: joint.motor_speed,
                        motor_mode: PhysicsMotorMode::from_raw(u32::from(joint.motor.raw())),
                    },
                    motor_frequency_hz: joint.linear_spring.hz,
                    motor_damping_ratio: joint.linear_spring.damping,
                });
                (PhysicsJointKind::Prismatic, index)
            }
            wr::JointKind::Distance => {
                let index = self.distances.len();
                self.distances.push(native::companions::PhysicsDistance {
                    anchor_a: joint.frame_a[3][..3].try_into().unwrap(),
                    anchor_b: joint.frame_b[3][..3].try_into().unwrap(),
                    distance: joint.rest_length,
                });
                (PhysicsJointKind::Distance, index)
            }
            _ => {
                return Err(Error::new(
                    "physics.joint.kind",
                    "unknown constraint category",
                ));
            }
        };
        self.links.push(native::companions::PhysicsJoint {
            body_a: a,
            body_b: b,
            joint_type: kind,
            joint_id: i16::try_from(index)
                .map_err(|_| Error::new("physics.joint.index", "constraint index exceeds i16"))?,
            ..Default::default()
        });
        Ok(())
    }
    pub(super) fn chunks(self) -> Vec<Pc<'static>> {
        let mut chunks = Vec::new();
        if !self.shoulders.is_empty() {
            chunks.push(Pc::Shoulders2(self.shoulders));
        }
        if !self.welds.is_empty() {
            chunks.push(Pc::Welds3(self.welds));
        }
        if !self.revolutes.is_empty() {
            chunks.push(Pc::Revolutes2(self.revolutes));
        }
        if !self.sphericals.is_empty() {
            chunks.push(Pc::Sphericals(self.sphericals));
        }
        if !self.distances.is_empty() {
            chunks.push(Pc::Distances(self.distances));
        }
        if !self.prismatics.is_empty() {
            chunks.push(Pc::Prismatics2(self.prismatics));
        }
        if !self.links.is_empty() {
            chunks.push(Pc::Joints(self.links));
        }
        chunks
    }
}
