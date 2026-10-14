//! Ordinary destinations resolve hierarchy, appearance, and polygon-group identities.
use super::super::{actors, document};
use super::{operations, targets};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::animation::{
    MaterialMount, NodeOperation, Operation, Target,
};
use crate::model::semantic::profiles::wem::{Feature, SectionId};
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{
    FeatureId, MaterialSlotId, MeshId, Model, NodeId, OwnerId, Property, PropertyTarget,
};
use crate::model::wem::records;
use std::collections::BTreeMap;

pub(in crate::model::conversion::wem) struct ReadContext<'a, 'data> {
    pub simulation: targets::ReadContext<'a, 'data>,
    pub model: &'a Model<'data, Wem>,
    pub meshes: &'a [MeshId],
    pub sections: &'a [Vec<SectionId>],
    pub slots: &'a [MaterialSlotId],
}
fn error(message: &str) -> Error {
    Error::new("wem.animation.target", message)
}
fn mount(
    source: &records::MaterialChannelRef,
    model: &Model<'_, Wem>,
    slots: &[MaterialSlotId],
) -> Result<MaterialMount, Error> {
    let family = document::read_family(source.profile);
    let matches: Vec<_> = model
        .appearance()
        .sets()
        .filter(|(_, set)| set.family == family)
        .collect();
    if matches.len() != 1 {
        return Err(error(
            "shader convention must identify exactly one material set",
        ));
    }
    let (set, value) = matches[0];
    let alternative = value
        .looks()
        .nth(source.look as usize)
        .map(|(id, _)| id)
        .ok_or_else(|| error("shader alternative is outside the selected set"))?;
    let slot = *slots
        .get(source.slot as usize)
        .ok_or_else(|| error("shader slot is outside the appearance"))?;
    Ok(MaterialMount {
        set,
        alternative,
        slot,
    })
}
fn material<'a, 'data>(
    model: &'a Model<'data, Wem>,
    mount: MaterialMount,
) -> Result<&'a crate::model::semantic::profiles::wem::Material<'data>, Error> {
    let material = model
        .appearance()
        .set(mount.set)
        .and_then(|set| set.look(mount.alternative))
        .and_then(|look| look.bindings().find(|(slot, _)| *slot == mount.slot))
        .and_then(|(_, material)| material)
        .ok_or_else(|| error("animated shader slot is unbound"))?;
    model
        .material(material)
        .map(|value| &value.surface)
        .ok_or_else(|| error("animated shader belongs to another model"))
}
pub(in crate::model::conversion::wem) fn read<T>(
    source: &records::TrackTarget,
    context: &ReadContext<'_, '_>,
) -> Result<Target<T>, Error> {
    if let Some(value) = targets::read(source, &context.simulation)? {
        return Ok(value);
    }
    let operation = operations::read(source.channel)?;
    Ok(match source.kind {
        records::TrackTargetKind::Node => {
            let node = *context
                .simulation
                .nodes
                .get(source.node as usize)
                .ok_or_else(|| error("animated node is outside the hierarchy"))?;
            if operation == Operation::EmitterProperty {
                for (id, attached) in context
                    .model
                    .features()
                    .filter(|(_, attached)| attached.node == node)
                {
                    let mut actor = attached.payload.clone();
                    if let Some(target) = actors::bind_property(&mut actor, id, source.sub)? {
                        return Ok(Target::Property(Property::new(target)));
                    }
                }
            }
            let control = match operation {
                Operation::Translation if source.sub == 0 => NodeOperation::Translation,
                Operation::Rotation if source.sub == 0 => NodeOperation::Rotation,
                Operation::Scale if source.sub == 0 => NodeOperation::Scale,
                Operation::Visibility if source.sub == 0 => NodeOperation::Visibility,
                Operation::Color if source.sub == 0 => NodeOperation::Color,
                Operation::Alpha if source.sub == 0 => NodeOperation::Alpha,
                Operation::Intensity if source.sub == 0 => NodeOperation::Intensity,
                Operation::AttenuationStart if source.sub == 0 => {
                    NodeOperation::Attenuation { end: false }
                }
                Operation::AttenuationEnd if source.sub == 0 => {
                    NodeOperation::Attenuation { end: true }
                }
                Operation::EmitterProperty => NodeOperation::UnknownEmitter(source.sub),
                _ => NodeOperation::Other {
                    operation,
                    selection: source.sub,
                },
            };
            Target::Node {
                node,
                operation: control,
            }
        }
        records::TrackTargetKind::MaterialLayer => {
            let mount = mount(&source.material, context.model, context.slots)?;
            if source.sub == u32::MAX {
                Target::Material { mount, operation }
            } else {
                let layer = material(context.model, mount)?
                    .common
                    .body
                    .layer_ids()
                    .nth(source.sub as usize)
                    .ok_or_else(|| error("shader contribution is outside the selected material"))?;
                Target::MaterialLayer {
                    mount,
                    layer,
                    operation,
                }
            }
        }
        records::TrackTargetKind::MaterialFeature => {
            let mount = mount(&source.material, context.model, context.slots)?;
            if !material(context.model, mount)?
                .common
                .features
                .iter()
                .any(|feature| feature.id == source.sub)
            {
                return Err(error("shader feature is absent from the selected material"));
            }
            Target::MaterialFeature {
                mount,
                feature: source.sub,
                operation,
            }
        }
        records::TrackTargetKind::Section => {
            let mesh = *context
                .meshes
                .get(source.mesh as usize)
                .ok_or_else(|| error("animated mesh is outside the model"))?;
            let section = *context
                .sections
                .get(source.mesh as usize)
                .and_then(|sections| sections.get(source.sub as usize))
                .ok_or_else(|| error("animated polygon group is outside the mesh"))?;
            Target::Section {
                mesh,
                section,
                operation,
            }
        }
        _ => return Err(error("unrecognized animated destination domain")),
    })
}
pub(in crate::model::conversion::wem) struct WriteContext<'a, 'data> {
    pub simulation: targets::WriteContext<'a, 'data>,
    pub model: &'a Model<'data, Wem>,
    pub meshes: &'a BTreeMap<MeshId, u32>,
    pub sections: &'a BTreeMap<(MeshId, SectionId), u32>,
    pub slots: &'a BTreeMap<MaterialSlotId, u32>,
    pub features: &'a BTreeMap<FeatureId, NodeId>,
}
fn write_mount(
    source: MaterialMount,
    context: &WriteContext<'_, '_>,
) -> Result<records::MaterialChannelRef, Error> {
    let set = context
        .model
        .appearance()
        .set(source.set)
        .ok_or_else(|| error("shader convention belongs to another appearance"))?;
    let look = set
        .looks()
        .position(|(id, _)| id == source.alternative)
        .ok_or_else(|| error("shader alternative belongs to another convention"))?;
    let slot = *context
        .slots
        .get(&source.slot)
        .ok_or_else(|| error("shader slot has no destination index"))?;
    Ok(records::MaterialChannelRef {
        profile: document::write_family(set.family),
        slot,
        look: u32::try_from(look).map_err(|_| error("shader alternative count exceeds u32"))?,
    })
}
pub(in crate::model::conversion::wem) fn write<T>(
    source: &Target<T>,
    context: &WriteContext<'_, '_>,
) -> Result<records::TrackTarget, Error> {
    if let Some(value) = targets::write(source, &context.simulation)? {
        return Ok(value);
    }
    let mut result = records::TrackTarget {
        node: u32::MAX,
        mesh: u32::MAX,
        ..Default::default()
    };
    match source {
        Target::Node { node, operation } => {
            result.kind = records::TrackTargetKind::Node;
            result.node = *context
                .simulation
                .nodes
                .get(node)
                .ok_or_else(|| error("animated node has no destination index"))?;
            let (operation, selection) = match operation {
                NodeOperation::Translation => (Operation::Translation, 0),
                NodeOperation::Rotation => (Operation::Rotation, 0),
                NodeOperation::Scale => (Operation::Scale, 0),
                NodeOperation::Visibility => (Operation::Visibility, 0),
                NodeOperation::Color => (Operation::Color, 0),
                NodeOperation::Alpha => (Operation::Alpha, 0),
                NodeOperation::Intensity => (Operation::Intensity, 0),
                NodeOperation::Attenuation { end } => (
                    if *end {
                        Operation::AttenuationEnd
                    } else {
                        Operation::AttenuationStart
                    },
                    0,
                ),
                NodeOperation::UnknownEmitter(value) => (Operation::EmitterProperty, *value),
                NodeOperation::Other {
                    operation,
                    selection,
                } => (*operation, *selection),
            };
            result.channel = operations::write(operation);
            result.sub = selection;
        }
        Target::Material { mount, operation } => {
            result.kind = records::TrackTargetKind::MaterialLayer;
            result.material = write_mount(*mount, context)?;
            result.sub = u32::MAX;
            result.channel = operations::write(*operation);
        }
        Target::MaterialLayer {
            mount,
            layer,
            operation,
        } => {
            result.kind = records::TrackTargetKind::MaterialLayer;
            result.material = write_mount(*mount, context)?;
            result.sub = material(context.model, *mount)?
                .common
                .body
                .layer_ids()
                .position(|id| id == *layer)
                .map(|index| {
                    u32::try_from(index).map_err(|_| error("shader contribution count exceeds u32"))
                })
                .transpose()?
                .ok_or_else(|| error("shader contribution belongs to another material"))?;
            result.channel = operations::write(*operation);
        }
        Target::MaterialFeature {
            mount,
            feature,
            operation,
        } => {
            if !material(context.model, *mount)?
                .common
                .features
                .iter()
                .any(|value| value.id == *feature)
            {
                return Err(error("shader feature belongs to another material"));
            }
            result.kind = records::TrackTargetKind::MaterialFeature;
            result.material = write_mount(*mount, context)?;
            result.sub = *feature;
            result.channel = operations::write(*operation);
        }
        Target::Section {
            mesh,
            section,
            operation,
        } => {
            result.kind = records::TrackTargetKind::Section;
            result.mesh = *context
                .meshes
                .get(mesh)
                .ok_or_else(|| error("animated mesh has no destination index"))?;
            result.sub = *context
                .sections
                .get(&(*mesh, *section))
                .ok_or_else(|| error("animated polygon group belongs to another mesh"))?;
            result.channel = operations::write(*operation);
        }
        Target::Property(property) => {
            let target = property.target();
            let PropertyTarget::Named {
                owner: OwnerId::Feature(id),
                ..
            } = target
            else {
                return Err(error("authored field has no native channel operation"));
            };
            let id = *id;
            let actor = context
                .model
                .feature(id)
                .ok_or_else(|| error("animated field owner is absent"))?;
            let selection = match &actor.payload {
                Feature::Warcraft(value) => {
                    actors::warcraft_property_code(value, target).map(|code| (code, None))
                }
                Feature::Starcraft(value) => actors::starcraft::property_code(value, target),
                Feature::Wow(value) => {
                    actors::wow::property_code(value, target).map(|code| (code, None))
                }
                _ => None,
            }
            .ok_or_else(|| error("authored field has no native emitter selection"))?;
            let element = selection
                .1
                .map(|index| {
                    u32::try_from(index + 1).map_err(|_| error("emitter element exceeds u32"))
                })
                .transpose()?
                .unwrap_or(0);
            if selection.0 > u16::MAX as u32 || element > u16::MAX as u32 {
                return Err(error("emitter selection exceeds packed capacity"));
            }
            result.kind = records::TrackTargetKind::Node;
            result.channel = records::Channel::EmitterProperty;
            result.node = *context
                .features
                .get(&id)
                .and_then(|node| context.simulation.nodes.get(node))
                .ok_or_else(|| error("animated actor has no destination node"))?;
            result.sub = selection.0 | (element << 16);
        }
        _ => {
            return Err(error(
                "animated destination was not encoded by its owning domain",
            ));
        }
    }
    Ok(result)
}
