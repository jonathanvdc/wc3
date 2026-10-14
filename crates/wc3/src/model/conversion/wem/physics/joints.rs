//! Two-body constraints preserve motors, springs, and independent frame transforms.
use super::codec::{allocate, append, mapped, read_error, revision, Scalar};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::physics::{
    BodyId, Friction, Joint, JointId, JointKind, Motor, Physics,
};
use crate::model::wem::{records, Bool, RawDocument, Ref};
use std::collections::BTreeMap;
pub(super) fn read(
    document: &RawDocument<'_>,
    reference: Ref<records::targets::PhysicsJoint>,
    target: &mut Physics<'_>,
    bodies: &BTreeMap<u32, BodyId>,
) -> Result<BTreeMap<u32, JointId>, Error> {
    let mut ids = BTreeMap::new();
    if reference.entries == 0 {
        return Ok(ids);
    }
    macro_rules! rows {
        ($revision:ty) => {{
            for value in document
                .resolve::<records::PhysicsJoint<$revision>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let value = value.map_err(read_error)?;
                let id = target.add_joint(joint(&value, bodies)?);
                if ids.insert(value.id, id).is_some() {
                    return Err(Error::new(
                        "wem.physics.joint",
                        "duplicate constraint identity",
                    ));
                }
            }
        }};
    }
    match revision(document, reference)? {
        Some(1) => rows!(records::V1),
        Some(2) => rows!(records::V2),
        Some(3) => rows!(records::V3),
        _ => {
            return Err(Error::new(
                "wem.physics.joint",
                "unsupported constraint revision",
            ));
        }
    }
    Ok(ids)
}
fn joint<D: records::PhysicsJointDialect>(
    value: &records::PhysicsJoint<D>,
    bodies: &BTreeMap<u32, BodyId>,
) -> Result<Joint, Error>
where
    D::Motor: Scalar<records::JointMotorMode>,
    D::MaxMotorForce: Scalar<f32>,
    D::MotorSpeed: Scalar<f32>,
    D::ReferenceTranslation: Scalar<f32>,
    D::Locked: Scalar<Bool>,
{
    let body = |id| {
        bodies.get(&id).copied().ok_or_else(|| {
            Error::new(
                "wem.physics.joint",
                "constraint body belongs to another simulation domain",
            )
        })
    };
    Ok(Joint {
        bodies: [body(value.body_a)?, body(value.body_b)?],
        kind: match value.kind {
            records::JointKind::Spherical => JointKind::Spherical,
            records::JointKind::Revolute => JointKind::Revolute,
            records::JointKind::ConeTwist => JointKind::ConeTwist,
            records::JointKind::Weld => JointKind::Weld,
            records::JointKind::Prismatic => JointKind::Prismatic,
            records::JointKind::Distance => JointKind::Distance,
            records::JointKind::Count => JointKind::Count,
            records::JointKind::Unknown(code) => JointKind::Unknown(code),
        },
        frames: [value.frame_a, value.frame_b],
        collide_connected: value.collide_connected.get(),
        limit_enabled: value.limit_enabled.get(),
        limits: [value.lower, value.upper],
        cone: value.cone,
        friction: match value.friction {
            records::JointFriction::None => Friction::None,
            records::JointFriction::Torque => Friction::Torque,
            records::JointFriction::GravityHold => Friction::GravityHold,
            records::JointFriction::Count => Friction::Count,
            records::JointFriction::Unknown(code) => Friction::Unknown(code),
        },
        friction_amount: value.friction_amount,
        angular_spring: [value.angular_spring.hz, value.angular_spring.damping],
        linear_spring: [value.linear_spring.hz, value.linear_spring.damping],
        rest_length: value.rest_length,
        breaking: [value.break_force, value.break_torque],
        motor: Scalar::<records::JointMotorMode>::scalar(&value.motor).map(|motor| match motor {
            records::JointMotorMode::Off => Motor::Off,
            records::JointMotorMode::Position => Motor::Position,
            records::JointMotorMode::Velocity => Motor::Velocity,
            records::JointMotorMode::Count => Motor::Count,
            records::JointMotorMode::Unknown(code) => Motor::Unknown(code),
        }),
        max_motor_force: Scalar::<f32>::scalar(&value.max_motor_force),
        motor_speed: Scalar::<f32>::scalar(&value.motor_speed),
        reference_translation: Scalar::<f32>::scalar(&value.reference_translation),
        locked: Scalar::<Bool>::scalar(&value.locked).is_some_and(Bool::get),
    })
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    source: &Physics<'_>,
    bodies: &BTreeMap<BodyId, u32>,
    next: &mut u32,
) -> Result<(Ref<records::targets::PhysicsJoint>, BTreeMap<JointId, u32>), Error> {
    let mut ids = BTreeMap::new();
    for (id, _) in source.joints() {
        ids.insert(id, allocate(next)?);
    }
    if source.joints().all(|(_, joint)| {
        joint.motor.is_none()
            && joint.max_motor_force.is_none()
            && joint.motor_speed.is_none()
            && joint.reference_translation.is_none()
            && !joint.locked
    }) {
        let rows = source
            .joints()
            .map(|(id, value)| {
                let id = ids[&id];
                Ok(records::PhysicsJoint::<records::V1> {
                    id,
                    body_a: mapped(bodies, value.bodies[0], "constraint body")?,
                    body_b: mapped(bodies, value.bodies[1], "constraint body")?,
                    kind: kind(value.kind),
                    frame_a: value.frames[0],
                    frame_b: value.frames[1],
                    collide_connected: value.collide_connected.into(),
                    limit_enabled: value.limit_enabled.into(),
                    lower: value.limits[0],
                    upper: value.limits[1],
                    cone: value.cone,
                    friction: friction(value.friction),
                    friction_amount: value.friction_amount,
                    angular_spring: records::JointSpring {
                        hz: value.angular_spring[0],
                        damping: value.angular_spring[1],
                    },
                    linear_spring: records::JointSpring {
                        hz: value.linear_spring[0],
                        damping: value.linear_spring[1],
                    },
                    rest_length: value.rest_length,
                    break_force: value.breaking[0],
                    break_torque: value.breaking[1],
                    motor: (),
                    max_motor_force: (),
                    motor_speed: (),
                    reference_translation: (),
                    locked: (),
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        return Ok((append(document, &rows)?.cast(), ids));
    }
    let rows = source
        .joints()
        .map(|(id, value)| {
            let id = ids[&id];
            Ok(records::PhysicsJoint::<records::V3> {
                id,
                body_a: mapped(bodies, value.bodies[0], "constraint body")?,
                body_b: mapped(bodies, value.bodies[1], "constraint body")?,
                kind: kind(value.kind),
                frame_a: value.frames[0],
                frame_b: value.frames[1],
                collide_connected: value.collide_connected.into(),
                limit_enabled: value.limit_enabled.into(),
                lower: value.limits[0],
                upper: value.limits[1],
                cone: value.cone,
                friction: friction(value.friction),
                friction_amount: value.friction_amount,
                angular_spring: records::JointSpring {
                    hz: value.angular_spring[0],
                    damping: value.angular_spring[1],
                },
                linear_spring: records::JointSpring {
                    hz: value.linear_spring[0],
                    damping: value.linear_spring[1],
                },
                rest_length: value.rest_length,
                break_force: value.breaking[0],
                break_torque: value.breaking[1],
                motor: motor(value.motor.ok_or_else(|| {
                    Error::new(
                        "wem.physics.joint.motor",
                        "mixed constraints require complete motor declarations",
                    )
                })?),
                max_motor_force: value.max_motor_force.ok_or_else(|| {
                    Error::new("wem.physics.joint.motor", "motor force is undeclared")
                })?,
                motor_speed: value.motor_speed.ok_or_else(|| {
                    Error::new("wem.physics.joint.motor", "motor speed is undeclared")
                })?,
                reference_translation: value.reference_translation.ok_or_else(|| {
                    Error::new(
                        "wem.physics.joint.motor",
                        "motor reference translation is undeclared",
                    )
                })?,
                locked: value.locked.into(),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok((append(document, &rows)?.cast(), ids))
}
fn kind(value: JointKind) -> records::JointKind {
    match value {
        JointKind::Spherical => records::JointKind::Spherical,
        JointKind::Revolute => records::JointKind::Revolute,
        JointKind::ConeTwist => records::JointKind::ConeTwist,
        JointKind::Weld => records::JointKind::Weld,
        JointKind::Prismatic => records::JointKind::Prismatic,
        JointKind::Distance => records::JointKind::Distance,
        JointKind::Count => records::JointKind::Count,
        JointKind::Unknown(code) => records::JointKind::Unknown(code),
    }
}
fn friction(value: Friction) -> records::JointFriction {
    match value {
        Friction::None => records::JointFriction::None,
        Friction::Torque => records::JointFriction::Torque,
        Friction::GravityHold => records::JointFriction::GravityHold,
        Friction::Count => records::JointFriction::Count,
        Friction::Unknown(code) => records::JointFriction::Unknown(code),
    }
}
fn motor(value: Motor) -> records::JointMotorMode {
    match value {
        Motor::Off => records::JointMotorMode::Off,
        Motor::Position => records::JointMotorMode::Position,
        Motor::Velocity => records::JointMotorMode::Velocity,
        Motor::Count => records::JointMotorMode::Count,
        Motor::Unknown(code) => records::JointMotorMode::Unknown(code),
    }
}

#[cfg(test)]
mod tests {
    use super::super::codec::append;
    use super::*;
    use crate::model::semantic::profiles::wem::physics::{Body, Motion};
    use crate::model::semantic::profiles::Wem;
    use crate::model::semantic::{Animated, Model, Node};
    #[test]
    fn legacy_motor_absence_does_not_invent_a_modern_declaration() {
        let mut builder = Model::<Wem>::builder("joint");
        let node = builder.add_node(Node::named("anchor")).unwrap();
        let mut physics = Physics::default();
        let body = || Body {
            node,
            motion: Motion::Dynamic,
            simulates: Animated::constant(true),
            shapes: Vec::new(),
            damping: [0.; 2],
            inertia_scale: 1.,
            gravity_scale: 1.,
            inherit_dynamic: false,
            exempt_from_ragdoll: false,
            force_channels: Default::default(),
            sc2: None,
            wow: None,
            locked: false,
        };
        let a = physics.add_body(body());
        let b = physics.add_body(body());
        let mut document = RawDocument::new();
        let source = append(
            &mut document,
            &[records::PhysicsJoint::<records::V1> {
                id: 13,
                body_a: 81,
                body_b: 99,
                angular_spring: records::JointSpring {
                    hz: 7.,
                    damping: 0.5,
                },
                ..Default::default()
            }],
        )
        .unwrap();
        let joint_ids = read(
            &document,
            source.cast(),
            &mut physics,
            &BTreeMap::from([(81, a), (99, b)]),
        )
        .unwrap();
        let joint_id = joint_ids[&13];
        assert_eq!(physics.joint(joint_id).unwrap().motor, None);
        let mut joint = physics.joint(joint_id).unwrap().clone();
        joint.limits = [-3., 11.];
        physics.set_joint(joint_id, joint).unwrap();
        let mut output = RawDocument::new();
        let mut next = 3;
        let (reference, _ids) = write(
            &mut output,
            &physics,
            &BTreeMap::from([(a, 1), (b, 2)]),
            &mut next,
        )
        .unwrap();
        assert_eq!(
            output
                .referenced_section(reference)
                .unwrap()
                .unwrap()
                .revision(),
            1
        );
        let rows = output
            .resolve::<records::PhysicsJoint<records::V1>>(reference.cast())
            .unwrap();
        let value = rows.get(0).unwrap().unwrap();
        assert_eq!(value.body_a, 1);
        assert_eq!(value.body_b, 2);
        assert_eq!(value.upper, 11.);
        assert_eq!(value.angular_spring.hz, 7.);
    }
}
