//! Inverse-kinematics chains use real node relationships and independent solver features.
use super::{hierarchy::Skeleton, materials::read_error, write::append, Graph, GraphBuilder};
use crate::model::conversion::Error;
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::{profiles::m3 as surface, NodeId};
use native::{Envelope, Ref};
use std::collections::BTreeMap;
fn node(index: u16, nodes: &[NodeId]) -> Result<NodeId, Error> {
    nodes
        .get(index as usize)
        .copied()
        .ok_or_else(|| Error::new("m3.ik.node", "solver node exceeds skeleton"))
}
fn dependents<E: Envelope>(
    document: &native::Container<'_, E>,
    reference: Ref<nr::targets::U16>,
    nodes: &[NodeId],
) -> Result<Vec<NodeId>, Error> {
    document
        .resolve::<nr::U16>(reference.cast())
        .map_err(read_error)?
        .iter()
        .map(|row| node(row.map_err(read_error)?.value, nodes))
        .collect()
}
pub(super) fn read<E: Envelope>(
    source: &native::Document<'_, E>,
    builder: &mut GraphBuilder<'_>,
    skeleton: &Skeleton<'_>,
) -> Result<(), Error> {
    let root = source.root_record().map_err(read_error)?;
    let (joints, two_joints, ccd, solvers) = native::visit_m3_root!(root, |r| (
        r.ik_joints,
        r.ik_two_joints,
        super::materials::Slot::<Ref<nr::targets::IKCCD>>::value(&r.ik_c_c_d),
        r.one_bone_solvers
    ));
    let document = source.container();
    for row in document
        .resolve::<nr::IKJoint>(joints.cast())
        .map_err(read_error)?
        .iter()
    {
        let r = row.map_err(read_error)?;
        let anchor = node(r.bone_index1, &skeleton.nodes)?;
        builder.add_feature(
            anchor,
            surface::Feature::IkJoint(
                (surface::IkJoint {
                    dependents: dependents(document, r.dependents, &skeleton.nodes)?,
                    second: node(r.bone_index2, &skeleton.nodes)?,
                    raycast_up: r.raycast_up,
                    raycast_down: r.raycast_down,
                    max_speed: r.max_speed,
                    goal_threshold: r.goal_threshold,
                })
                .into(),
            ),
        )?;
    }
    for row in document
        .resolve::<nr::IKTwoJoint>(two_joints.cast())
        .map_err(read_error)?
        .iter()
    {
        let r = row.map_err(read_error)?;
        let anchor = node(r.bone_base, &skeleton.nodes)?;
        builder.add_feature(
            anchor,
            surface::Feature::IkTwoJoint(
                (surface::IkTwoJoint {
                    dependents: dependents(document, r.dependents, &skeleton.nodes)?,
                    target: node(r.bone_target, &skeleton.nodes)?,
                    end: node(r.bone_end, &skeleton.nodes)?,
                    reserved: r.padding,
                    hinge_axis: r.hinge_axis,
                    max_angle_inner: r.max_angle_inner,
                    max_angle_outer: r.max_angle_outer,
                    search_up: r.search_up,
                    search_down: r.search_down,
                })
                .into(),
            ),
        )?;
    }
    for row in document
        .resolve::<nr::IKCCD>(ccd.cast())
        .map_err(read_error)?
        .iter()
    {
        let r = row.map_err(read_error)?;
        let anchor = node(r.bone_base, &skeleton.nodes)?;
        builder.add_feature(
            anchor,
            surface::Feature::IkCcd(
                (surface::IkCcd {
                    dependents: dependents(document, r.dependents, &skeleton.nodes)?,
                    target: node(r.bone_target, &skeleton.nodes)?,
                    search_up: r.search_up,
                    search_down: r.search_down,
                })
                .into(),
            ),
        )?;
    }
    for row in document
        .resolve::<nr::OneBoneSolver>(solvers.cast())
        .map_err(read_error)?
        .iter()
    {
        let r = row.map_err(read_error)?;
        let anchor = node(r.bone, &skeleton.nodes)?;
        builder.add_feature(
            anchor,
            surface::Feature::OneBoneSolver(
                (surface::OneBoneSolver {
                    dependents: dependents(document, r.dependents, &skeleton.nodes)?,
                    bone_fallback: if r.bone_fallback == u16::MAX {
                        None
                    } else {
                        Some(node(r.bone_fallback, &skeleton.nodes)?)
                    },
                    flags: surface::values::OneBoneFlags(r.flags.bits()),
                    max_angle: r.max_angle,
                })
                .into(),
            ),
        )?;
    }
    Ok(())
}
#[derive(Default)]
pub(super) struct Solvers {
    pub joint: Ref<nr::targets::IKJoint>,
    pub two_joint: Ref<nr::targets::IKTwoJoint>,
    pub ccd: Ref<nr::targets::IKCCD>,
    pub one_bone: Ref<nr::targets::OneBoneSolver>,
}
fn index(node: NodeId, bones: &BTreeMap<NodeId, u32>) -> Result<u16, Error> {
    u16::try_from(
        *bones
            .get(&node)
            .ok_or_else(|| Error::new("m3.ik.node", "solver node has no native bone"))?,
    )
    .map_err(|_| Error::new("m3.ik.node", "native solver bone capacity exceeded"))
}
fn write_dependents<E: Envelope>(
    nodes: &[NodeId],
    bones: &BTreeMap<NodeId, u32>,
    document: &mut native::Container<'_, E>,
) -> Result<Ref<nr::targets::U16>, Error> {
    let rows = nodes
        .iter()
        .map(|node| {
            Ok(nr::U16 {
                value: index(*node, bones)?,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(append(document, &rows)?.cast())
}
pub(super) fn write<E: Envelope>(
    model: &Graph<'_>,
    document: &mut native::Container<'_, E>,
    bones: &BTreeMap<NodeId, u32>,
) -> Result<Solvers, Error> {
    let mut joints = Vec::new();
    let mut two = Vec::new();
    let mut ccd = Vec::new();
    let mut one = Vec::new();
    for (_, attached) in model.features() {
        match &attached.payload {
            surface::Feature::IkJoint(r) => joints.push(nr::IKJoint {
                dependents: write_dependents(&r.dependents, bones, document)?,
                bone_index1: index(attached.node, bones)?,
                bone_index2: index(r.second, bones)?,
                raycast_up: r.raycast_up,
                raycast_down: r.raycast_down,
                max_speed: r.max_speed,
                goal_threshold: r.goal_threshold,
            }),
            surface::Feature::IkTwoJoint(r) => two.push(nr::IKTwoJoint {
                dependents: write_dependents(&r.dependents, bones, document)?,
                bone_base: index(attached.node, bones)?,
                bone_target: index(r.target, bones)?,
                bone_end: index(r.end, bones)?,
                padding: r.reserved,
                hinge_axis: r.hinge_axis,
                max_angle_inner: r.max_angle_inner,
                max_angle_outer: r.max_angle_outer,
                search_up: r.search_up,
                search_down: r.search_down,
            }),
            surface::Feature::IkCcd(r) => ccd.push(nr::IKCCD {
                dependents: write_dependents(&r.dependents, bones, document)?,
                bone_base: index(attached.node, bones)?,
                bone_target: index(r.target, bones)?,
                search_up: r.search_up,
                search_down: r.search_down,
            }),
            surface::Feature::OneBoneSolver(r) => one.push(nr::OneBoneSolver {
                dependents: write_dependents(&r.dependents, bones, document)?,
                bone: index(attached.node, bones)?,
                bone_fallback: r
                    .bone_fallback
                    .map(|node| index(node, bones))
                    .transpose()?
                    .unwrap_or(u16::MAX),
                flags: nr::OneBoneFlags(r.flags.bits()),
                max_angle: r.max_angle,
            }),
            _ => {}
        }
    }
    Ok(Solvers {
        joint: append(document, &joints)?.cast(),
        two_joint: append(document, &two)?.cast(),
        ccd: append(document, &ccd)?.cast(),
        one_bone: append(document, &one)?.cast(),
    })
}
