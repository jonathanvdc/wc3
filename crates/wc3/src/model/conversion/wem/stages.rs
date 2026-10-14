//! Pipeline operations rebuild from stable hierarchy and simulation identities.
use super::text;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::physics::{ClothId, RigId};
use crate::model::semantic::profiles::wem::poses::Components;
use crate::model::semantic::profiles::wem::{
    ClothBinding, PosePipeline, PoseStage, RagdollBinding, StageId, StageKind, StageSource,
    StageSourceId,
};
use crate::model::semantic::{NodeId, Quaternion, SystemId};
use crate::model::wem::{self, records, RawDocument, Record, RecordLayout, Ref, Target};
use std::collections::BTreeMap;
pub(super) struct Decoded<'a> {
    pub value: PosePipeline<'a>,
    pub stages: BTreeMap<u32, StageId>,
    pub sources: BTreeMap<(u32, u32), StageSourceId>,
}
pub(super) struct Encoded {
    pub value: Ref<records::targets::PoseStage>,
    pub stages: BTreeMap<StageId, u32>,
    pub sources: BTreeMap<StageSourceId, u32>,
}
fn error(value: impl std::fmt::Display) -> Error {
    Error::new("wem.pose.stage", value.to_string())
}
fn rows<T: Record + RecordLayout + for<'r> wem::Read<'r>, U: Target>(
    doc: &RawDocument<'_>,
    reference: Ref<U>,
) -> Result<Vec<T>, Error> {
    doc.resolve::<T>(reference.cast())
        .map_err(error)?
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(error)
}
fn append<T: Record + wem::Write>(
    doc: &mut RawDocument<'_>,
    values: &[T],
) -> Result<Ref<T>, Error> {
    if values.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(values.len()).map_err(error)?;
    let section = doc.append(values).map_err(error)?;
    Ok(Ref::from_section(count, section.index()))
}
fn nodes(
    doc: &RawDocument<'_>,
    reference: Ref<records::targets::U32>,
    ids: &[NodeId],
) -> Result<Vec<NodeId>, Error> {
    rows::<records::U32, _>(doc, reference)?
        .into_iter()
        .map(|row| {
            ids.get(row.value as usize)
                .copied()
                .ok_or_else(|| error("pipeline node is absent"))
        })
        .collect()
}
fn optional_node(ids: &[NodeId], index: u32) -> Result<Option<NodeId>, Error> {
    if index == u32::MAX {
        Ok(None)
    } else {
        ids.get(index as usize)
            .copied()
            .map(Some)
            .ok_or_else(|| error("pipeline node is absent"))
    }
}
fn write_nodes(
    doc: &mut RawDocument<'_>,
    values: &[NodeId],
    ids: &BTreeMap<NodeId, u32>,
) -> Result<Ref<records::targets::U32>, Error> {
    Ok(append(
        doc,
        &values
            .iter()
            .map(|id| {
                ids.get(id)
                    .copied()
                    .map(|value| records::U32 { value })
                    .ok_or_else(|| error("pipeline node has no destination identity"))
            })
            .collect::<Result<Vec<_>, _>>()?,
    )?
    .cast())
}
trait SimulationSlot {
    fn identity(&self) -> Option<u32>;
}
impl SimulationSlot for () {
    fn identity(&self) -> Option<u32> {
        None
    }
}
impl SimulationSlot for u32 {
    fn identity(&self) -> Option<u32> {
        Some(*self)
    }
}
fn binding<T: Copy>(index: Option<u32>, ids: &BTreeMap<u32, T>) -> Result<Option<T>, Error> {
    match index {
        None | Some(u32::MAX) => Ok(None),
        Some(index) => match ids.get(&index) {
            Some(id) => Ok(Some(*id)),
            None if index == 0 => Ok(None),
            None => Err(error("pipeline simulation object is absent")),
        },
    }
}
fn read_kind(value: records::StageKind) -> StageKind {
    match value {
        records::StageKind::Position => StageKind::Position,
        records::StageKind::Orientation => StageKind::Orientation,
        records::StageKind::LookAt => StageKind::LookAt,
        records::StageKind::Link => StageKind::Link,
        records::StageKind::LimbIk => StageKind::LimbIk,
        records::StageKind::ChainIk => StageKind::ChainIk,
        records::StageKind::Spring => StageKind::Spring,
        records::StageKind::Ragdoll => StageKind::Ragdoll,
        records::StageKind::Cloth => StageKind::Cloth,
        records::StageKind::Count => StageKind::Count,
        records::StageKind::Unknown(value) => StageKind::Unknown(value),
    }
}
fn write_kind(value: StageKind) -> records::StageKind {
    match value {
        StageKind::Position => records::StageKind::Position,
        StageKind::Orientation => records::StageKind::Orientation,
        StageKind::LookAt => records::StageKind::LookAt,
        StageKind::Link => records::StageKind::Link,
        StageKind::LimbIk => records::StageKind::LimbIk,
        StageKind::ChainIk => records::StageKind::ChainIk,
        StageKind::Spring => records::StageKind::Spring,
        StageKind::Ragdoll => records::StageKind::Ragdoll,
        StageKind::Cloth => records::StageKind::Cloth,
        StageKind::Count => records::StageKind::Count,
        StageKind::Unknown(value) => records::StageKind::Unknown(value),
    }
}
fn read_rows<'a, D: records::PoseStageDialect>(
    doc: &'a RawDocument<'_>,
    reference: Ref<records::targets::PoseStage>,
    node_ids: &[NodeId],
    system: Option<SystemId>,
    rigs: &BTreeMap<u32, RigId>,
    cloths: &BTreeMap<u32, ClothId>,
) -> Result<Decoded<'a>, Error>
where
    D::Rig: SimulationSlot + RecordLayout + for<'r> wem::Read<'r>,
    D::Cloth: SimulationSlot + RecordLayout + for<'r> wem::Read<'r>,
{
    let mut result = Decoded {
        value: PosePipeline::default(),
        stages: BTreeMap::new(),
        sources: BTreeMap::new(),
    };
    for row in rows::<records::PoseStage<D>, _>(doc, reference)? {
        let mut stage = PoseStage::new(read_kind(row.kind));
        stage.name = text::read(doc, row.name)?;
        stage.enabled = row.enabled.get();
        stage.driven = nodes(doc, row.driven, node_ids)?;
        stage.targets = nodes(doc, row.targets, node_ids)?;
        stage.keep_offset = row.keep_offset.get();
        stage.axis = row.axis;
        stage.up = row.up;
        stage.up_node = optional_node(node_ids, row.up_node)?;
        stage.offset = Components {
            translation: row.offset.translation,
            rotation: Quaternion(row.offset.rotation),
            scale: row.offset.scale,
        };
        stage.stiffness = row.stiffness;
        stage.damping = row.damping;
        stage.gravity = row.gravity;
        stage.ragdoll = binding(row.rig.identity(), rigs)?
            .map(|rig| {
                system
                    .map(|system| RagdollBinding { system, rig })
                    .ok_or_else(|| error("ragdoll stage has no simulation domain"))
            })
            .transpose()?;
        stage.cloth = binding(row.cloth.identity(), cloths)?
            .map(|cloth| {
                system
                    .map(|system| ClothBinding { system, cloth })
                    .ok_or_else(|| error("cloth stage has no simulation domain"))
            })
            .transpose()?;
        for source in rows::<records::StageSource, _>(doc, row.sources)? {
            let id = stage.add_source(StageSource::new(
                optional_node(node_ids, source.node)?,
                source.weight,
            ))?;
            if result.sources.insert((row.id, source.id), id).is_some() {
                return Err(error("duplicate weighted input producer identity"));
            }
        }
        let id = result.value.add_stage(stage);
        if result.stages.insert(row.id, id).is_some() {
            return Err(error("duplicate pose stage producer identity"));
        }
    }
    Ok(result)
}
pub(super) fn read<'a>(
    doc: &'a RawDocument<'_>,
    reference: Ref<records::targets::PoseStage>,
    nodes: &[NodeId],
    system: Option<SystemId>,
    rigs: &BTreeMap<u32, RigId>,
    cloths: &BTreeMap<u32, ClothId>,
) -> Result<Decoded<'a>, Error> {
    let Some(section) = doc.referenced_section(reference).map_err(error)? else {
        return Ok(Decoded {
            value: Default::default(),
            stages: BTreeMap::new(),
            sources: BTreeMap::new(),
        });
    };
    match section.revision() {
        1 => read_rows::<records::V1>(doc, reference, nodes, system, rigs, cloths),
        2 => read_rows::<records::V2>(doc, reference, nodes, system, rigs, cloths),
        _ => Err(error("unsupported pose stage revision")),
    }
}
pub(super) fn write(
    doc: &mut RawDocument<'_>,
    pipeline: &PosePipeline<'_>,
    nodes: &BTreeMap<NodeId, u32>,
    system: Option<SystemId>,
    rigs: &BTreeMap<RigId, u32>,
    cloths: &BTreeMap<ClothId, u32>,
) -> Result<Encoded, Error> {
    pipeline.validate()?;
    let mut rows = Vec::new();
    let mut stages = BTreeMap::new();
    let mut sources = BTreeMap::new();
    for (index, (id, stage)) in pipeline.stages().enumerate() {
        let index = u32::try_from(index).map_err(error)?;
        stages.insert(id, index);
        let mut inputs = Vec::new();
        for (index, source) in stage.sources().enumerate() {
            let index = u32::try_from(index + 1).map_err(error)?;
            sources.insert(source.id(), index);
            inputs.push(records::StageSource {
                id: index,
                node: match source.node {
                    None => u32::MAX,
                    Some(id) => *nodes
                        .get(&id)
                        .ok_or_else(|| error("pipeline input has no destination node"))?,
                },
                weight: source.weight,
            });
        }
        rows.push(records::PoseStage::<records::V2> {
            id: index,
            name: text::write_bytes(doc, stage.name.as_bytes())?,
            kind: write_kind(stage.kind),
            enabled: stage.enabled.into(),
            driven: write_nodes(doc, &stage.driven, nodes)?,
            targets: write_nodes(doc, &stage.targets, nodes)?,
            sources: append(doc, &inputs)?.cast(),
            keep_offset: stage.keep_offset.into(),
            axis: stage.axis,
            up: stage.up,
            up_node: match stage.up_node {
                None => u32::MAX,
                Some(id) => *nodes
                    .get(&id)
                    .ok_or_else(|| error("up frame has no destination node"))?,
            },
            offset: records::Transform {
                translation: stage.offset.translation,
                rotation: stage.offset.rotation.components(),
                scale: stage.offset.scale,
            },
            stiffness: stage.stiffness,
            damping: stage.damping,
            gravity: stage.gravity,
            rig: match stage.ragdoll {
                None => 0,
                Some(value) => {
                    if Some(value.system) != system {
                        return Err(error("ragdoll belongs to another simulation domain"));
                    }
                    *rigs
                        .get(&value.rig)
                        .ok_or_else(|| error("ragdoll has no destination identity"))?
                }
            },
            cloth: match stage.cloth {
                None => 0,
                Some(value) => {
                    if Some(value.system) != system {
                        return Err(error("cloth belongs to another simulation domain"));
                    }
                    *cloths
                        .get(&value.cloth)
                        .ok_or_else(|| error("cloth has no destination identity"))?
                }
            },
        });
    }
    Ok(Encoded {
        value: append(doc, &rows)?.cast(),
        stages,
        sources,
    })
}

#[cfg(test)]
#[path = "stages_tests.rs"]
mod tests;
