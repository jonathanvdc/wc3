//! Named turrets reference independently editable bone-attached parts.
use super::{
    materials::{read_error, revision, string},
    slots::Slot,
    write::{append, string as write_string},
    Graph, GraphBuilder,
};
use crate::model::conversion::{Context, Error, IssueKind};
use crate::model::formats::m3::{self as m3, records, Container, Envelope, Ref};
use crate::model::profiles::m3 as surface;
use crate::model::{
    profiles::m3::{Feature, System, TurretBehavior, TurretGroup},
    FeatureId, NodeId,
};
use std::collections::BTreeMap;
fn node(index: u32, nodes: &[NodeId]) -> Result<NodeId, Error> {
    nodes.get(index as usize).copied().ok_or_else(|| {
        Error::new(
            "m3.turret.bone",
            "turret part references an undeclared bone",
        )
    })
}
pub(super) fn read<'a, E: Envelope>(
    source: &'a m3::Document<'_, E>,
    builder: &mut GraphBuilder<'a>,
    nodes: &[NodeId],
    context: &mut Context,
) -> Result<(), Error> {
    let root = source.root_record().map_err(read_error)?;
    let (parts, groups) = m3::visit_m3_root!(root, |r| (r.turret_behaviors, r.turret_groups));
    let document = source.container();
    let mut features = Vec::new();
    macro_rules! parts {
        ($v:ty) => {
            for row in document
                .resolve::<records::TurretBehavior<$v>>(parts.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = row.map_err(read_error)?;
                features.push(
                    builder.add_feature(
                        node(u32::from(r.bone_index), nodes)?,
                        Feature::Turret(
                            (TurretBehavior {
                                transform: r.transform,
                                unknown1: Slot::<[f32; 4]>::slot(r.unknown1).unwrap_or_default(),
                                unknown2: Slot::<[f32; 4]>::slot(r.unknown2).unwrap_or_default(),
                                use_as_main_turret: surface::values::TurretPartFlags(
                                    r.use_as_main_turret.bits(),
                                ),
                                turret_group_id: r.turret_group_id,
                                yaw_limited: surface::values::TurretAxisFlags(r.yaw_limited.bits()),
                                yaw_min: r.yaw_min,
                                yaw_max: r.yaw_max,
                                yaw_weight: Slot::<f32>::slot(r.yaw_weight).unwrap_or_default(),
                                pitch_limited: surface::values::TurretAxisFlags(
                                    r.pitch_limited.bits(),
                                ),
                                pitch_min: r.pitch_min,
                                pitch_max: r.pitch_max,
                                pitch_weight: Slot::<f32>::slot(r.pitch_weight).unwrap_or_default(),
                                unknown3: r.unknown3,
                                unknown4: r.unknown4,
                                main_bone_offset: Slot::<[f32; 3]>::slot(r.main_bone_offset)
                                    .unwrap_or_default(),
                            })
                            .into(),
                        ),
                    )?,
                );
            }
        };
    }
    if !parts.is_empty() {
        let version = revision(document, parts).map_err(read_error)?;
        if version != 4 {
            context.issue(
                "m3.turret.parts",
                IssueKind::Normalized,
                "older turret parts expand into the complete part controls; export uses PATU4",
            );
        }
        match version {
            0 => parts!(records::V0),
            1 => parts!(records::V1),
            2 => parts!(records::V2),
            3 => parts!(records::V3),
            4 => parts!(records::V4),
            _ => {
                return Err(Error::new(
                    "m3.turret.parts",
                    "unknown turret part revision",
                ));
            }
        }
    }
    let mut definitions = Vec::new();
    for row in document
        .resolve::<records::TurretGroup>(groups.cast())
        .map_err(read_error)?
        .iter()
    {
        let row = row.map_err(read_error)?;
        let mut parts = Vec::new();
        for index in document
            .resolve::<records::U32>(row.parts.cast())
            .map_err(read_error)?
            .iter()
        {
            let index = index.map_err(read_error)?.value as usize;
            parts.push(*features.get(index).ok_or_else(|| {
                Error::new(
                    "m3.turret.parts",
                    "group references an undeclared turret part",
                )
            })?);
        }
        definitions.push(TurretGroup {
            name: string(document, row.name)?,
            parts,
        });
    }
    if !definitions.is_empty() {
        builder.add_system(System::TurretGroups(definitions))?;
    }
    Ok(())
}
#[derive(Default)]
pub(super) struct Output {
    pub parts: Ref<records::targets::TurretBehavior>,
    pub groups: Ref<records::targets::TurretGroup>,
}
pub(super) fn write<E: Envelope>(
    graph: &Graph<'_>,
    document: &mut Container<'_, E>,
    bones: &BTreeMap<NodeId, u32>,
) -> Result<Output, Error> {
    let mut parts = Vec::new();
    let mut indices = BTreeMap::<FeatureId, u32>::new();
    for (id, attached) in graph.features() {
        let Feature::Turret(r) = &attached.payload else {
            continue;
        };
        let bone = *bones
            .get(&attached.node)
            .ok_or_else(|| Error::new("m3.turret.bone", "turret part has no native bone"))?;
        indices.insert(
            id,
            u32::try_from(parts.len()).map_err(|_| {
                Error::new(
                    "m3.turret.parts",
                    "native part count exceeds 32-bit capacity",
                )
            })?,
        );
        parts.push(records::TurretBehavior::<records::V4> {
            transform: r.transform,
            unknown1: r.unknown1,
            unknown2: r.unknown2,
            bone_index: u16::try_from(bone)
                .map_err(|_| Error::new("m3.turret.bone", "native bone exceeds 16-bit capacity"))?,
            use_as_main_turret: records::TurretPartFlags(r.use_as_main_turret.bits()),
            turret_group_id: r.turret_group_id,
            yaw_limited: records::TurretAxisFlags(r.yaw_limited.bits()),
            yaw_min: r.yaw_min,
            yaw_max: r.yaw_max,
            yaw_weight: r.yaw_weight,
            pitch_limited: records::TurretAxisFlags(r.pitch_limited.bits()),
            pitch_min: r.pitch_min,
            pitch_max: r.pitch_max,
            pitch_weight: r.pitch_weight,
            unknown3: r.unknown3,
            unknown4: r.unknown4,
            main_bone_offset: r.main_bone_offset,
        });
    }
    let mut groups = Vec::new();
    for (_, system) in graph.systems() {
        let System::TurretGroups(definitions) = system else {
            continue;
        };
        for group in definitions {
            let members = group
                .parts
                .iter()
                .map(|id| {
                    indices
                        .get(id)
                        .copied()
                        .map(|value| records::U32 { value })
                        .ok_or_else(|| {
                            Error::new("m3.turret.parts", "group member is not a turret part")
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            groups.push(records::TurretGroup {
                name: write_string(document, &group.name)?,
                parts: append(document, &members)?.cast(),
            });
        }
    }
    Ok(Output {
        parts: append(document, &parts)?.cast(),
        groups: append(document, &groups)?.cast(),
    })
}
