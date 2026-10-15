//! Joint frames and unresolved control words remain authored values.
use super::super::{materials::read_error, write::append, Graph};
use crate::model::conversion::Error;
use crate::model::formats::m3::records::{self as records, PhysicsJoint};
use crate::model::formats::m3::{Container, Envelope, Ref};
use crate::model::{
    profiles::m3::{values::JointKind, Joint, System},
    NodeId,
};
use std::collections::BTreeMap;

pub(super) fn read<E: Envelope>(
    document: &Container<'_, E>,
    source: Ref<records::targets::PhysicsJoint>,
    nodes: &[NodeId],
) -> Result<Vec<Joint>, Error> {
    let node = |index: u32| {
        nodes.get(index as usize).copied().ok_or_else(|| {
            Error::new(
                "m3.physics.joint.node",
                "joint endpoint does not identify a declared bone",
            )
        })
    };
    document
        .resolve::<PhysicsJoint>(source.cast())
        .map_err(read_error)?
        .iter()
        .map(|row| {
            let row = row.map_err(read_error)?;
            Ok(Joint {
                joint_type: JointKind::from_raw(row.joint_type.raw()),
                node_a: node(row.bone_index1)?,
                node_b: node(row.bone_index2)?,
                frame_a: row.matrix_body1,
                frame_b: row.matrix_body2,
                limit_control: row.enable_limits,
                limit_min: row.limit_min,
                limit_max: row.limit_max,
                cone_angle: row.cone_angle,
                friction_control: row.enable_friction,
                friction: row.friction,
                damping_ratio: row.damping_ratio,
                angular_frequency: row.angular_frequency,
                break_threshold: row.break_threshold,
                collision_control: row.collision_control,
            })
        })
        .collect()
}

pub(super) fn write<E: Envelope>(
    graph: &Graph<'_>,
    document: &mut Container<'_, E>,
    nodes: &BTreeMap<NodeId, u32>,
) -> Result<Ref<records::targets::PhysicsJoint>, Error> {
    let node = |id: NodeId| {
        nodes.get(&id).copied().ok_or_else(|| {
            Error::new(
                "m3.physics.joint.node",
                "joint endpoint has no native bone definition",
            )
        })
    };
    let mut output = Vec::new();
    for (_, system) in graph.systems() {
        let System::Physics(system) = system else {
            continue;
        };
        for joint in &system.joints {
            output.push(PhysicsJoint {
                joint_type: records::JointKind::from_raw(joint.joint_type.raw()),
                bone_index1: node(joint.node_a)?,
                bone_index2: node(joint.node_b)?,
                matrix_body1: joint.frame_a,
                matrix_body2: joint.frame_b,
                enable_limits: joint.limit_control,
                limit_min: joint.limit_min,
                limit_max: joint.limit_max,
                cone_angle: joint.cone_angle,
                enable_friction: joint.friction_control,
                friction: joint.friction,
                damping_ratio: joint.damping_ratio,
                angular_frequency: joint.angular_frequency,
                break_threshold: joint.break_threshold,
                collision_control: joint.collision_control,
            });
        }
    }
    append(document, &output).map(Ref::cast)
}
