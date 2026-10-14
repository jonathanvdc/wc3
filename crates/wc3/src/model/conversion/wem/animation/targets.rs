//! Packed animation selectors resolve through actual operation and simulation owners.
use crate::model::conversion::wem::{physics, stages};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::animation::channels::{
    FrameComponent, Operation, PhysicsControl, StageSourceControl, Target,
};
use crate::model::semantic::profiles::wem::PosePipeline;
use crate::model::semantic::{NodeId, SystemId};
use crate::model::wem::records::{Channel, TrackTarget, TrackTargetKind};
use std::collections::BTreeMap;

pub(in crate::model::conversion::wem) struct ReadContext<'a, 'data> {
    pub nodes: &'a [NodeId],
    pub physics: Option<(SystemId, &'a physics::Decoded<'data>)>,
    pub stages: &'a stages::Decoded<'data>,
}
pub(in crate::model::conversion::wem) struct WriteContext<'a, 'data> {
    pub nodes: &'a BTreeMap<NodeId, u32>,
    pub physics: Option<(SystemId, &'a physics::Encoded)>,
    pub stages: &'a stages::Encoded,
    pub pipeline: &'a PosePipeline<'data>,
}
fn error(message: impl ToString) -> Error {
    Error::new("wem.animation.target", message.to_string())
}
fn lookup<K: Ord, V: Copy>(values: &BTreeMap<K, V>, id: K) -> Result<V, Error> {
    values
        .get(&id)
        .copied()
        .ok_or_else(|| error("animated destination has no matching object identity"))
}
fn pack(low: u32, high: u32) -> Result<u32, Error> {
    if low > u16::MAX as u32 || high > u16::MAX as u32 {
        return Err(error(
            "animated destination exceeds the packed selector capacity",
        ));
    }
    Ok(low | (high << 16))
}
pub(in crate::model::conversion::wem) fn read<T>(
    raw: &TrackTarget,
    context: &ReadContext<'_, '_>,
) -> Result<Option<Target<T>>, Error> {
    let stage_control = matches!(
        raw.channel,
        Channel::StageWeight | Channel::StageSourceWeight | Channel::StageSourceEnabled
    );
    if stage_control {
        if raw.kind != TrackTargetKind::Node {
            return Err(error("pose operation controls require a node destination"));
        }
        let stage_uid = raw.sub & 0xffff;
        if raw.channel == Channel::StageWeight && raw.sub >> 16 != 0 {
            return Err(error(
                "pose weight selector has an unexpected input identity",
            ));
        }
        let stage = lookup(&context.stages.stages, stage_uid)?;
        let operation = context
            .stages
            .value
            .stage(stage)
            .ok_or_else(|| error("pose operation is absent"))?;
        if operation.driven.first().copied() != context.nodes.get(raw.node as usize).copied() {
            return Err(error("pose control does not name the first driven node"));
        }
        return Ok(Some(if raw.channel == Channel::StageWeight {
            Target::Stage { stage }
        } else {
            Target::StageSource {
                stage,
                source: lookup(&context.stages.sources, (stage_uid, raw.sub >> 16))?,
                control: if raw.channel == Channel::StageSourceWeight {
                    StageSourceControl::Weight
                } else {
                    StageSourceControl::Enabled
                },
            }
        }));
    }
    if raw.kind != TrackTargetKind::Physics {
        return Ok(None);
    }
    let (system, physics) = context
        .physics
        .ok_or_else(|| error("simulation control has no simulation system"))?;
    let low = raw.sub & 0xffff;
    let high = raw.sub >> 16;
    let control = match raw.channel {
        Channel::PhysicsDynamic => PhysicsControl::BodyDynamic(lookup(&physics.bodies, raw.sub)?),
        Channel::PhysicsRagdoll => PhysicsControl::RigRagdoll(lookup(&physics.rigs, raw.sub)?),
        Channel::PhysicsBlend => {
            if let Some(id) = physics.bodies.get(&raw.sub) {
                PhysicsControl::BodyBlend(*id)
            } else {
                PhysicsControl::RigBlend(lookup(&physics.rigs, raw.sub)?)
            }
        }
        Channel::ClothActive => PhysicsControl::ClothActive(lookup(&physics.cloths, raw.sub)?),
        Channel::ClothDriverTranslation | Channel::ClothDriverRotation => {
            PhysicsControl::ClothDriver {
                cloth: lookup(&physics.cloths, low)?,
                driver: high as u16,
                component: if raw.channel == Channel::ClothDriverTranslation {
                    FrameComponent::Translation
                } else {
                    FrameComponent::Rotation
                },
            }
        }
        Channel::SoftAmount => PhysicsControl::SoftEffect {
            body: lookup(&physics.soft.bodies, low)?,
            effect: lookup(&physics.soft.effects, (low, high))?,
        },
        Channel::SoftLumpTranslation | Channel::SoftLumpRotation | Channel::SoftLumpScale => {
            PhysicsControl::SoftLump {
                body: lookup(&physics.soft.bodies, low)?,
                lump: lookup(&physics.soft.lumps, (low, high))?,
                component: match raw.channel {
                    Channel::SoftLumpTranslation => FrameComponent::Translation,
                    Channel::SoftLumpRotation => FrameComponent::Rotation,
                    _ => FrameComponent::Scale,
                },
            }
        }
        Channel::Unknown(value) => PhysicsControl::Unknown {
            operation: Operation::Unknown(value),
            selection: raw.sub,
        },
        _ => return Err(error("channel is not a simulation control")),
    };
    Ok(Some(Target::Physics { system, control }))
}
pub(in crate::model::conversion::wem) fn write<T>(
    target: &Target<T>,
    context: &WriteContext<'_, '_>,
) -> Result<Option<TrackTarget>, Error> {
    let mut raw = TrackTarget::default();
    match target {
        Target::Stage { stage } | Target::StageSource { stage, .. } => {
            let operation = context
                .pipeline
                .stage(*stage)
                .ok_or_else(|| error("pose operation is absent"))?;
            let node = operation
                .driven
                .first()
                .copied()
                .ok_or_else(|| error("animated pose operation has no driven node"))?;
            raw.node = lookup(context.nodes, node)?;
            raw.kind = TrackTargetKind::Node;
            let stage = lookup(&context.stages.stages, *stage)?;
            match target {
                Target::Stage { .. } => {
                    raw.channel = Channel::StageWeight;
                    raw.sub = pack(stage, 0)?;
                }
                Target::StageSource {
                    source, control, ..
                } => {
                    raw.channel = match control {
                        StageSourceControl::Weight => Channel::StageSourceWeight,
                        StageSourceControl::Enabled => Channel::StageSourceEnabled,
                    };
                    raw.sub = pack(stage, lookup(&context.stages.sources, *source)?)?;
                }
                _ => unreachable!(),
            }
        }
        Target::Physics { system, control } => {
            let (owner, physics) = context
                .physics
                .ok_or_else(|| error("simulation destination has no simulation system"))?;
            if owner != *system {
                return Err(error(
                    "animated simulation object belongs to another system",
                ));
            }
            raw.kind = TrackTargetKind::Physics;
            (raw.channel, raw.sub) = match *control {
                PhysicsControl::BodyDynamic(id) => {
                    (Channel::PhysicsDynamic, lookup(&physics.bodies, id)?)
                }
                PhysicsControl::BodyBlend(id) => {
                    (Channel::PhysicsBlend, lookup(&physics.bodies, id)?)
                }
                PhysicsControl::RigRagdoll(id) => {
                    (Channel::PhysicsRagdoll, lookup(&physics.rigs, id)?)
                }
                PhysicsControl::RigBlend(id) => (Channel::PhysicsBlend, lookup(&physics.rigs, id)?),
                PhysicsControl::ClothActive(id) => {
                    (Channel::ClothActive, lookup(&physics.cloths, id)?)
                }
                PhysicsControl::ClothDriver {
                    cloth,
                    driver,
                    component,
                } => {
                    let channel = match component {
                        FrameComponent::Translation => Channel::ClothDriverTranslation,
                        FrameComponent::Rotation => Channel::ClothDriverRotation,
                        FrameComponent::Scale => {
                            return Err(error("cloth drivers have no scale channel"));
                        }
                    };
                    (
                        channel,
                        pack(lookup(&physics.cloths, cloth)?, u32::from(driver))?,
                    )
                }
                PhysicsControl::SoftEffect { body, effect } => (
                    Channel::SoftAmount,
                    pack(
                        lookup(&physics.soft.bodies, body)?,
                        lookup(&physics.soft.effects, (body, effect))?,
                    )?,
                ),
                PhysicsControl::SoftLump {
                    body,
                    lump,
                    component,
                } => {
                    let channel = match component {
                        FrameComponent::Translation => Channel::SoftLumpTranslation,
                        FrameComponent::Rotation => Channel::SoftLumpRotation,
                        FrameComponent::Scale => Channel::SoftLumpScale,
                    };
                    (
                        channel,
                        pack(
                            lookup(&physics.soft.bodies, body)?,
                            lookup(&physics.soft.lumps, (body, lump))?,
                        )?,
                    )
                }
                PhysicsControl::Unknown {
                    operation: Operation::Unknown(value),
                    selection,
                } => (Channel::Unknown(value), selection),
                PhysicsControl::Unknown { .. } => {
                    return Err(error(
                        "recognized simulation channels require typed object destinations",
                    ));
                }
            };
        }
        _ => return Ok(None),
    }
    Ok(Some(raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::semantic::profiles::wem::{PosePipeline, PoseStage, StageKind, StageSource};
    use crate::model::semantic::Scope;
    use crate::model::wem::{records, RawDocument};

    #[test]
    fn reordered_operations_relocate_channels_through_stable_input_handles() {
        let scope = Scope::default();
        let node = scope.allocate();
        let mut pipeline = PosePipeline::default();
        let mut operation = PoseStage::new(StageKind::Link);
        operation.driven.push(node);
        let input = operation.add_source(StageSource::new(None, 0.5)).unwrap();
        let selected = pipeline.add_stage(operation);
        pipeline.add_stage(PoseStage::new(StageKind::Position));
        let mut document = RawDocument::new();
        let nodes = BTreeMap::from([(node, 0)]);
        let encoded = stages::write(
            &mut document,
            &pipeline,
            &nodes,
            None,
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .unwrap();
        let target = Target::<f32>::StageSource {
            stage: selected,
            source: input,
            control: StageSourceControl::Weight,
        };
        let first = write(
            &target,
            &WriteContext {
                nodes: &nodes,
                physics: None,
                stages: &encoded,
                pipeline: &pipeline,
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(first.sub, 1 << 16);
        let mut decoded = stages::read(
            &document,
            encoded.value,
            &[node],
            None,
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .unwrap();
        let imported = read::<f32>(
            &first,
            &ReadContext {
                nodes: &[node],
                physics: None,
                stages: &decoded,
            },
        )
        .unwrap()
        .unwrap();
        let Target::StageSource { stage, source, .. } = imported else {
            panic!("operation input expected")
        };
        decoded.value.move_stage(stage, 1).unwrap();
        let mut destination = RawDocument::new();
        let relocated = stages::write(
            &mut destination,
            &decoded.value,
            &nodes,
            None,
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .unwrap();
        let last = write(
            &Target::<f32>::StageSource {
                stage,
                source,
                control: StageSourceControl::Weight,
            },
            &WriteContext {
                nodes: &nodes,
                physics: None,
                stages: &relocated,
                pipeline: &decoded.value,
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(last.sub, (1 << 16) | 1);
        let mut missing = first.clone();
        missing.sub = 7 << 16;
        assert!(read::<f32>(
            &missing,
            &ReadContext {
                nodes: &[node],
                physics: None,
                stages: &decoded
            }
        )
        .is_err());
    }

    #[test]
    fn packed_selectors_reject_overflow_without_truncating_identities() {
        assert_eq!(pack(65535, 65535).unwrap(), u32::MAX);
        assert!(pack(65536, 1).is_err());
        assert!(pack(1, 65536).is_err());
        let scope = Scope::default();
        let body = scope.allocate();
        let system = scope.allocate();
        let encoded = physics::Encoded {
            value: records::PhysicsSet::default(),
            bodies: BTreeMap::from([(body, 91)]),
            rigs: Default::default(),
            cloths: Default::default(),
            soft: Default::default(),
        };
        let stages = stages::Encoded {
            value: Default::default(),
            stages: Default::default(),
            sources: Default::default(),
        };
        let pipeline = PosePipeline::default();
        let nodes = BTreeMap::new();
        let context = WriteContext {
            nodes: &nodes,
            physics: Some((system, &encoded)),
            stages: &stages,
            pipeline: &pipeline,
        };
        let raw = write(
            &Target::<f32>::Physics {
                system,
                control: PhysicsControl::BodyBlend(body),
            },
            &context,
        )
        .unwrap()
        .unwrap();
        assert_eq!(raw.channel, Channel::PhysicsBlend);
        assert_eq!(raw.sub, 91);
        assert!(write(
            &Target::<f32>::Physics {
                system: Scope::default().allocate(),
                control: PhysicsControl::BodyBlend(body)
            },
            &context
        )
        .is_err());
    }
}
