//! Rigid bodies represented by WEM's native physics set.
mod bodies;
mod joints;
mod shape_controls;
mod shapes;
use super::metadata::{animated, put, restore};
use super::{metadata, native, nr, read_error, revision, sem, wem, wr, Envelope, Error, Ref};
use crate::model::conversion::m3::append;
use crate::model::conversion::Context;
use std::collections::BTreeMap;

pub(in crate::model::conversion::m3) fn attach<E: Envelope>(
    source: &native::Model<'_, E>,
    document: wem::Document<'static>,
    context: &mut Context,
) -> Result<wem::Document<'static>, Error> {
    let references = native::visit_m3_root!(source.root_record().map_err(read_error)?, |r| (
        r.rigid_bodies,
        r.physics_joints,
        r.physics_constraints
    ));
    if references.2.entries != 0 {
        context.unsupported(
            "m3.physics.constraints",
            "legacy constraint records have no established WEM interpretation",
        )?;
    }
    if references.0.entries == 0 {
        return Ok(document);
    }
    let mut doc = document.into_raw();
    let root_ref = wem::Ref::from_section(doc.root_reference().entries, doc.root_reference().index);
    let root = doc
        .resolve::<wr::DocumentRecord<wr::V1>>(root_ref)?
        .get(0)
        .ok_or_else(|| Error::new("wem.root", "missing root"))??;
    let mut model = doc
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.models", "missing model"))??;
    let mut nodes = doc
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let bodies = bodies::import(
        source.document(),
        references.0,
        &mut doc,
        &mut nodes,
        context,
    )?;
    let joints = joints::import(
        source.document(),
        references.1,
        &bodies,
        &mut doc,
        &mut nodes,
        context,
    )?;
    model.physics.joints = put(&mut doc, &joints)?.cast();
    model.physics.bodies = put(&mut doc, &bodies)?.cast();
    model.physics.next_id = (bodies.len() + joints.len()) as u32;
    let nodes_id = doc
        .section_id(model.nodes.nodes.index)
        .ok_or_else(|| Error::new("wem.nodes", "missing nodes"))?;
    doc.replace(nodes_id, &nodes)?;
    let model_id = doc
        .section_id(root.models.index)
        .ok_or_else(|| Error::new("wem.models", "missing model section"))?;
    doc.replace(model_id, &[model])?;
    Ok(wem::Document::from_raw(doc)?)
}

pub(super) fn export<E: Envelope>(
    model: &sem::Model<'_>,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
    context: &mut Context,
) -> Result<Ref<nr::targets::RigidBody>, Error> {
    let Some(native) = &model.native else {
        return Ok(Ref::default());
    };
    let source = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let record = source
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    if record.physics.soft.entries != 0 || record.physics.rigs.entries != 0 {
        context.unsupported(
            "wem.physics.soft",
            "M3 cloth/soft export requires matching native simulation data",
        )?;
    }
    bodies::export(source, record, doc, bone_map, context)
}

pub(in crate::model::conversion::m3) fn export_joints<E: Envelope>(
    model: &sem::Model<'_>,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
    context: &mut Context,
) -> Result<Ref<nr::targets::PhysicsJoint>, Error> {
    let Some(native) = &model.native else {
        return Ok(Ref::default());
    };
    let source = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let record = source
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    joints::export(source, &record, doc, bone_map, context)
}
