//! StarCraft editable-model payloads map directly to independently authored controls.
mod codec;
mod properties;
mod read;
mod write;
use crate::model::conversion::Error;
use crate::model::semantic::{
    profiles::wem::{
        actors::starcraft::{Control, Feature, ParticleCopy, Variation, VariationKind},
        SectionId,
    },
    FeatureId, MaterialSlotId, NodeId,
};
use crate::model::wem::{records, RawDocument};
pub(in crate::model::conversion::wem) use properties::{bind_property, property_code, Bound};
use std::collections::BTreeMap;
struct ReadContext<'view, 'data, 'ids> {
    document: &'view RawDocument<'data>,
    nodes: &'ids [NodeId],
    features: &'ids BTreeMap<u32, FeatureId>,
    slots: &'ids [MaterialSlotId],
    sections: &'ids [SectionId],
}
struct WriteContext<'view, 'data> {
    document: &'view mut RawDocument<'data>,
    nodes: &'view BTreeMap<NodeId, u32>,
    features: &'view BTreeMap<FeatureId, u32>,
    slots: &'view BTreeMap<MaterialSlotId, u32>,
    sections: &'view BTreeMap<SectionId, u32>,
}
fn control<T: Clone>(value: &records::Sc2Property<T>) -> Control<T> {
    Control {
        initial: value.init_value.clone(),
        reset: value.null_value.clone(),
        binding: None,
    }
}
fn write_control<T: Clone>(value: &Control<T>) -> records::Sc2Property<T> {
    records::Sc2Property {
        init_value: value.initial.clone(),
        null_value: value.reset.clone(),
    }
}
fn variation(value: &records::Sc2Variation) -> Variation {
    Variation {
        kind: VariationKind::from_raw(value.type_.raw()),
        amplitude: control(&value.amplitude),
        frequency: control(&value.frequency),
    }
}
fn write_variation(value: &Variation) -> records::Sc2Variation {
    records::Sc2Variation {
        type_: records::Sc2VariationKind::from_raw(value.kind.raw()),
        amplitude: write_control(&value.amplitude),
        frequency: write_control(&value.frequency),
    }
}
fn feature(index: u32, features: &BTreeMap<u32, FeatureId>) -> Result<Option<FeatureId>, Error> {
    if index == u32::MAX {
        Ok(None)
    } else {
        features.get(&index).copied().map(Some).ok_or_else(|| {
            Error::new(
                "wem.starcraft.link",
                "node has no attached emitter behavior",
            )
        })
    }
}
fn slot(index: u32, slots: &[MaterialSlotId]) -> Result<Option<MaterialSlotId>, Error> {
    if index == u32::MAX {
        Ok(None)
    } else {
        slots.get(index as usize).copied().map(Some).ok_or_else(|| {
            Error::new(
                "wem.starcraft.material",
                "appearance slot belongs to another model",
            )
        })
    }
}
fn node(index: u32, nodes: &[NodeId]) -> Result<Option<NodeId>, Error> {
    if index == u32::MAX {
        Ok(None)
    } else {
        nodes.get(index as usize).copied().map(Some).ok_or_else(|| {
            Error::new(
                "wem.starcraft.spline",
                "spline node belongs to another model",
            )
        })
    }
}
fn feature_index(
    value: Option<FeatureId>,
    values: &BTreeMap<FeatureId, u32>,
) -> Result<u32, Error> {
    value
        .map(|id| {
            values
                .get(&id)
                .copied()
                .ok_or_else(|| Error::new("wem.starcraft.link", "emitter has no output node"))
        })
        .transpose()
        .map(|v| v.unwrap_or(u32::MAX))
}
fn slot_index(
    value: Option<MaterialSlotId>,
    values: &BTreeMap<MaterialSlotId, u32>,
) -> Result<u32, Error> {
    value
        .map(|id| {
            values.get(&id).copied().ok_or_else(|| {
                Error::new(
                    "wem.starcraft.material",
                    "appearance slot has no output ordinal",
                )
            })
        })
        .transpose()
        .map(|v| v.unwrap_or(u32::MAX))
}
fn node_index(value: Option<NodeId>, values: &BTreeMap<NodeId, u32>) -> Result<u32, Error> {
    value
        .map(|id| {
            values.get(&id).copied().ok_or_else(|| {
                Error::new("wem.starcraft.spline", "spline node has no output ordinal")
            })
        })
        .transpose()
        .map(|v| v.unwrap_or(u32::MAX))
}
pub(in crate::model::conversion::wem) fn read<'a, D: records::NodePayloadDialect>(
    document: &'a RawDocument<'_>,
    payload: &records::NodePayload<D>,
    nodes: &[NodeId],
    features: &BTreeMap<u32, FeatureId>,
    slots: &[MaterialSlotId],
    sections: &[SectionId],
) -> Result<Option<Feature<'a>>, Error> {
    let context = ReadContext {
        document,
        nodes,
        features,
        slots,
        sections,
    };
    Ok(Some(match payload {
        records::NodePayload::Sc2ParticleEmitter(value) => {
            if let Some(source) = feature(value.copy_of, features)? {
                Feature::ParticleCopy(
                    (ParticleCopy {
                        source,
                        emission_rate: control(&value.emission_rate),
                        squirt_amount: control(&value.squirt_amount),
                    })
                    .into(),
                )
            } else {
                Feature::Particle((read::read_particle(&context, value)?).into())
            }
        }
        records::NodePayload::Sc2RibbonEmitter(value) => {
            Feature::Ribbon((read::read_ribbon(&context, value)?).into())
        }
        _ => return Ok(None),
    }))
}
pub(in crate::model::conversion::wem) fn write<D: records::NodePayloadDialect>(
    document: &mut RawDocument<'_>,
    feature: &Feature<'_>,
    nodes: &BTreeMap<NodeId, u32>,
    features: &BTreeMap<FeatureId, u32>,
    slots: &BTreeMap<MaterialSlotId, u32>,
    sections: &BTreeMap<SectionId, u32>,
) -> Result<records::NodePayload<D>, Error> {
    let mut context = WriteContext {
        document,
        nodes,
        features,
        slots,
        sections,
    };
    Ok(match feature {
        Feature::Particle(value) => records::NodePayload::Sc2ParticleEmitter(Box::new(
            write::write_particle(&mut context, value)?,
        )),
        Feature::ParticleCopy(value) => {
            records::NodePayload::Sc2ParticleEmitter(Box::new(records::Sc2ParticleEmitterPayload {
                copy_of: feature_index(Some(value.source), features)?,
                emission_rate: write_control(&value.emission_rate),
                squirt_amount: write_control(&value.squirt_amount),
                material_slot: u32::MAX,
                collision_spawn: u32::MAX,
                trail_link: u32::MAX,
                ribbon_link: u32::MAX,
                splat_projection_index: -1,
                ..Default::default()
            }))
        }
        Feature::Ribbon(value) => records::NodePayload::Sc2RibbonEmitter(Box::new(
            write::write_ribbon(&mut context, value)?,
        )),
    })
}
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
