//! Bodies retain authored parameters without inferring mass from collider volume.
use super::super::{
    animation::AnimationIds,
    write::{animated, append},
    Graph,
};
use crate::model::conversion::{Context, Error};
use crate::model::formats::m3::{records, Container, Envelope, Ref};
use crate::model::{
    profiles::m3::{Body, BodyParameters, System},
    NodeId,
};
use std::collections::BTreeMap;

fn node(body: &Body<'_>, nodes: &BTreeMap<NodeId, u32>) -> Result<u16, Error> {
    nodes
        .get(&body.node)
        .copied()
        .and_then(|index| u16::try_from(index).ok())
        .ok_or_else(|| {
            Error::new(
                "m3.physics.body.node",
                "body requires a bone attachment fitting the native 16-bit index",
            )
        })
}
fn parameters<'view>(body: &'view Body<'_>) -> Result<&'view BodyParameters, Error> {
    body.parameters.as_ref().ok_or_else(|| {
        Error::new(
            "m3.physics.body.parameters",
            "explicit and unspecified body parameters cannot share one native body program",
        )
    })
}
pub(super) fn write<E: Envelope>(
    graph: &Graph<'_>,
    document: &mut Container<'_, E>,
    nodes: &BTreeMap<NodeId, u32>,
    ids: &AnimationIds,
    context: &mut Context,
) -> Result<Ref<records::targets::RigidBody>, Error> {
    let candidates: Vec<_> = graph
        .systems()
        .iter()
        .filter_map(|(_, system)| {
            if let System::Physics(system) = system {
                Some(&system.bodies)
            } else {
                None
            }
        })
        .flatten()
        .collect();
    let bodies = candidates;
    let shape_refs = bodies
        .iter()
        .map(|body| super::shape_write::write(graph, document, &body.shapes, context))
        .collect::<Result<Vec<_>, Error>>()?;
    if bodies.is_empty() {
        return Ok(Ref::default());
    }
    if bodies
        .iter()
        .all(|body| body.parameters.is_none() && body.dynamic.is_none())
    {
        let rows = bodies
            .iter()
            .enumerate()
            .map(|(index, body)| {
                Ok(records::RigidBody::<records::V2> {
                    parent_bone_index: node(body, nodes)?,
                    rigid_body_shape: shape_refs[index],
                    flags: records::RigidBodyFlags(body.flags.bits()),
                    local_forces: records::LocalForces(body.local_forces.bits()),
                    world_forces: records::WorldForces(body.world_forces.bits()),
                    priority: body.priority,
                    ..Default::default()
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        return append(document, &rows).map(Ref::cast);
    }
    macro_rules! common {
        ($body:expr, $params:expr) => {
            records::RigidBody {
                simulation_type: $body.simulation_type,
                parent_bone_index: node($body, nodes)?,
                physics_type: records::PhysicalMaterial::from_raw($params.material.raw()),
                mass: $params.mass,
                friction: $params.friction,
                restitution: $params.restitution,
                linear_damping: $params.linear_damping,
                angular_damping: $params.angular_damping,
                gravity_factor: $params.gravity_factor,
                flags: records::RigidBodyFlags($body.flags.bits()),
                local_forces: records::LocalForces($body.local_forces.bits()),
                world_forces: records::WorldForces($body.world_forces.bits()),
                priority: $body.priority,
                ..Default::default()
            }
        };
    }
    if bodies.iter().all(|body| body.dynamic.is_none()) {
        let rows = bodies
            .iter()
            .enumerate()
            .map(|(index, body)| {
                let p = parameters(body)?;
                let mut row: records::RigidBody<records::V3> = common!(body, p);
                row.rigid_body_shape = shape_refs[index];
                Ok(row)
            })
            .collect::<Result<Vec<_>, Error>>()?;
        append(document, &rows).map(Ref::cast)
    } else {
        let rows=bodies.iter().enumerate().map(|(index,body)|{
            let p=parameters(body)?;
            let state=body.dynamic.as_ref().ok_or_else(||Error::new("m3.physics.body.state","animated and unspecified dynamic states cannot share one native body program"))?;
            let mut row:records::RigidBody<records::V4>=common!(body,p);
            row.rigid_body_shape=shape_refs[index];
            row.dynamic_state=animated(&state.state,ids);
            row.dynamic_blend_out=state.blend_out;
            Ok(row)
        }).collect::<Result<Vec<_>,Error>>()?;
        append(document, &rows).map(Ref::cast)
    }
}
