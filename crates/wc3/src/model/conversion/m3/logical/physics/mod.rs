//! Authored physics relationships are separate from cooked acceleration storage.
mod bodies;
mod body_fields;
mod constraints;
mod joints;
mod shape_write;
mod shapes;
use super::{
    animation::AnimationIds,
    bindings::{self, Bindings},
    Graph, GraphBuilder,
};
use crate::model::conversion::{Context, Error};
use crate::model::m3::{self, records, Document, Envelope, Model, Ref};
use crate::model::semantic::{
    profiles::m3::{PhysicsSystem, System},
    NodeId, OwnerId,
};
use std::collections::BTreeMap;

pub(super) fn read<'view, E: Envelope + Send + Sync>(
    source: &'view Model<'_, E>,
    builder: &mut GraphBuilder<'view>,
    nodes: &[NodeId],
    bindings: &mut Bindings,
    context: &mut Context,
) -> Result<(), Error> {
    let root = source.root_record().map_err(super::materials::read_error)?;
    let (joint_ref, body_ref, constraint_ref) = m3::visit_m3_root!(root, |r| (
        r.physics_joints,
        r.rigid_bodies,
        r.physics_constraints
    ));
    let joints = joints::read(source.document(), joint_ref, nodes)?;
    let inputs = body_fields::read(source.document(), body_ref, nodes, context)?;
    let constraints = constraints::read(
        source.document(),
        constraint_ref,
        &inputs
            .iter()
            .map(|input| input.body.id())
            .collect::<Vec<_>>(),
    )?;
    if inputs.is_empty() && joints.is_empty() && constraints.is_empty() {
        return Ok(());
    }
    let id = builder.add_system(System::Physics(PhysicsSystem::default()))?;
    let mut bodies = Vec::with_capacity(inputs.len());
    for mut input in inputs {
        input.body.shapes = shapes::read(source.document(), input.shapes, builder, context)?;
        bodies.push(input.body);
    }
    let mut system = System::Physics(PhysicsSystem {
        bodies,
        joints,
        constraints,
    });
    bindings::system(&mut system, OwnerId::System(id), bindings);
    builder.set_system(id, system)?;
    Ok(())
}
#[derive(Default)]
pub(super) struct Output {
    pub constraints: Ref<records::targets::PhysicsConstraint>,
    pub joints: Ref<records::targets::PhysicsJoint>,
    pub bodies: Ref<records::targets::RigidBody>,
}
pub(super) fn write<E: Envelope>(
    graph: &Graph<'_>,
    document: &mut Document<'_, E>,
    nodes: &BTreeMap<NodeId, u32>,
    ids: &AnimationIds,
    context: &mut Context,
) -> Result<Output, Error> {
    Ok(Output {
        constraints: constraints::write(graph, document)?,
        joints: joints::write(graph, document, nodes)?,
        bodies: bodies::write(graph, document, nodes, ids, context)?,
    })
}
