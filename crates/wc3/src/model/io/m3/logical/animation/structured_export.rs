//! Event commands and coupled extent curves become typed native blocks.
use super::super::Graph;
use super::{
    blocks::Blocks,
    export::{append, text},
    AnimationIds,
};
use crate::model::conversion::{Context, Error};
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::profiles::M3;
use crate::model::ValueSlot;
use crate::model::{profiles::m3 as surface, Controller, DynamicTrack, NodeId, Time};
use native::Envelope;
use std::collections::BTreeMap;

pub(super) fn signed_time(
    value: Time,
    origin: Time,
    context: &mut Context,
) -> Result<nr::I32, Error> {
    let exact = (value.as_seconds() + origin.as_seconds()) * 1000.;
    if !exact.is_finite()
        || exact.round() < f64::from(i32::MIN)
        || exact.round() > f64::from(i32::MAX)
    {
        return Err(Error::new(
            "m3.animation.time",
            "structured key time exceeds native signed milliseconds",
        ));
    }
    let raw = exact.round() as i32;
    context.approximate(
        "m3.animation.time",
        ((f64::from(raw) - exact) / 1000.).abs() as f32,
        "structured key time quantized to milliseconds",
    )?;
    Ok(nr::I32 { value: raw })
}
fn end(
    control: Option<&surface::BlockControls>,
    origin: Time,
    _clip: &Controller<'_, M3>,
    context: &mut Context,
) -> Result<u32, Error> {
    let seconds = control.map_or(origin.as_seconds() + 0.0, |c| c.end.as_seconds());
    let exact = seconds * 1000.;
    if !exact.is_finite() || exact.round() < 0. || exact.round() > f64::from(u32::MAX) {
        return Err(Error::new(
            "m3.animation.end",
            "block end exceeds native unsigned milliseconds",
        ));
    }
    let raw = exact.round() as u32;
    context.approximate(
        "m3.animation.end",
        ((f64::from(raw) - exact) / 1000.).abs() as f32,
        "block end quantized to milliseconds",
    )?;
    Ok(raw)
}

pub(super) struct Program<'a, 'data> {
    pub controller: &'a Controller<'data, M3>,
    pub controls: Option<&'a surface::ControllerControls<'data>>,
    pub origin: Time,
}
pub(super) struct Output<'a, 'data, E: Envelope> {
    pub blocks: &'a mut Blocks,
    pub document: &'a mut native::Container<'data, E>,
    pub identities: &'a mut Vec<nr::U32>,
    pub selectors: &'a mut Vec<nr::U32>,
}
pub(super) fn add<E: Envelope>(
    model: &Graph<'_>,
    program: Program<'_, '_>,
    nodes: &[NodeId],
    ids: &AnimationIds,
    output: Output<'_, '_, E>,
    context: &mut Context,
) -> Result<(), Error> {
    let Program {
        controller: clip,
        controls,
        origin,
    } = program;
    let Output {
        blocks,
        document,
        identities,
        selectors,
    } = output;
    let mut groups: Vec<(surface::BlockControls, bool)> = controls
        .into_iter()
        .flat_map(|c| c.blocks.iter())
        .filter(|b| b.encoding == surface::KeyEncoding::Event)
        .cloned()
        .map(|b| (b, true))
        .collect();
    if let Some(controls) = controls {
        for block in &controls.unbound {
            if let surface::UnboundKeys::Events(events) = &block.keys {
                groups.push((
                    surface::BlockControls {
                        source_id: block.identity.unwrap_or_default(),
                        targets: Vec::new(),
                        events: events.clone(),
                        extents: Vec::new(),
                        encoding: surface::KeyEncoding::Event,
                        flags: block.flags,
                        end: block.end,
                    },
                    block.identity.is_some(),
                ));
            }
        }
    }
    let occurrences: BTreeMap<_, _> = clip
        .animations
        .iter()
        .flat_map(|id| &model.animation(*id).unwrap().events)
        .map(|(id, event)| (*id, event))
        .collect();
    let ungrouped = occurrences
        .keys()
        .filter(|id| !groups.iter().any(|(b, _)| b.events.contains(id)))
        .copied()
        .collect::<Vec<_>>();
    if !ungrouped.is_empty() {
        let end = occurrences
            .values()
            .map(|e| e.time)
            .max_by(|a, b| a.as_seconds().total_cmp(&b.as_seconds()))
            .unwrap_or_default();
        groups.push((
            surface::BlockControls {
                source_id: 0,
                targets: Vec::new(),
                events: ungrouped,
                extents: Vec::new(),
                encoding: surface::KeyEncoding::Event,
                flags: Default::default(),
                end,
            },
            true,
        ));
    }
    for (group, indexed) in groups {
        let control = Some(&group);
        let mut times = Vec::new();
        let mut rows = Vec::new();
        for (_, event) in occurrences
            .iter()
            .filter(|(id, _)| control.is_none_or(|c| c.events.contains(id)))
        {
            let Some(feature) = event.feature.and_then(|id| model.feature(id)) else {
                context.unsupported("m3.animation.event", "event has no typed native command")?;
                continue;
            };
            let surface::Feature::Event(command) = &feature.payload else {
                context.unsupported(
                    "m3.animation.event",
                    "event feature is not a native command",
                )?;
                continue;
            };
            let bone_index =
                if let Some(node) = event.node {
                    u16::try_from(nodes.iter().position(|id| *id == node).ok_or_else(|| {
                        Error::new("m3.animation.event", "anchor has no native bone")
                    })?)
                    .map_err(|_| Error::new("m3.animation.event", "bone index exceeds u16"))?
                } else {
                    u16::MAX
                };
            times.push(signed_time(event.time, origin, context)?);
            rows.push(nr::Event::<nr::V2> {
                name: text(document, &event.name)?,
                unknown: command.control,
                bone_index,
                padding: 0,
                transform: command.transform,
                event_type: command.event_type,
                option_string: text(document, &command.options)?,
                rtt_channel_index: command.render_target_channel.unwrap_or_default(),
                extra_parameter: command.parameter.unwrap_or_default(),
            });
        }
        let index = blocks.events.len();
        blocks.events.push(nr::AnimationBlockEvent {
            timestamps: append(document, &times)?.cast(),
            keys: append(document, &rows)?.cast(),
            flags: nr::AnimationBlockFlags::from(control.map_or(0, |c| c.flags.bits())),
            end_frame: end(control, origin, clip, context)?,
        });
        if indexed {
            identities.push(nr::U32 {
                value: group.source_id,
            });
            selectors.push(nr::U32 {
                value: u32::from(u16::try_from(index).map_err(|_| {
                    Error::new("m3.animation.event", "event block count exceeds selector")
                })?),
            });
        }
    }
    if let Some(controls) = controls {
        for control in controls
            .blocks
            .iter()
            .filter(|b| b.encoding == surface::KeyEncoding::Extent)
        {
            let mut original: Option<(Vec<Time>, Vec<nr::Extent>)> = None;
            let mut extent_ids = Vec::new();
            for feature in &control.extents {
                let surface::Feature::Extent(binding) = &model
                    .feature(*feature)
                    .ok_or_else(|| Error::new("m3.animation.extent", "extent feature is missing"))?
                    .payload
                else {
                    return Err(Error::new(
                        "m3.animation.extent",
                        "structured block references another feature kind",
                    ));
                };
                let matching = |target: ValueSlot| {
                    model
                        .animation_tracks(&clip.animations)
                        .find(|c| c.target() == target)
                };
                let (
                    Some(DynamicTrack::Vector3(minimum)),
                    Some(DynamicTrack::Vector3(maximum)),
                    Some(DynamicTrack::Scalar(radius)),
                ) = (
                    matching(binding.minimum.slot()),
                    matching(binding.maximum.slot()),
                    matching(binding.radius.slot()),
                )
                else {
                    context.unsupported(
                        "m3.animation.extent",
                        "extent requires all three coupled channels",
                    )?;
                    continue;
                };
                if minimum.curve.times() != maximum.curve.times()
                    || minimum.curve.times() != radius.curve.times()
                {
                    context.unsupported(
                        "m3.animation.extent",
                        "coupled extent channels require matching timestamps",
                    )?;
                    continue;
                }
                let times: Vec<_> = minimum.curve.times().iter().collect();
                let rows: Vec<_> = minimum
                    .curve
                    .values()
                    .iter()
                    .zip(maximum.curve.values().iter())
                    .zip(radius.curve.values().iter())
                    .map(|((minimum, maximum), radius)| nr::Extent {
                        minimum,
                        maximum,
                        radius,
                    })
                    .collect();
                if original
                    .as_ref()
                    .is_some_and(|(t, v)| *t != times || *v != rows)
                {
                    context.unsupported(
                        "m3.animation.extent",
                        "shared native extent identity has divergent edited curves",
                    )?;
                    continue;
                }
                if let Some(identity) = ids.get(&binding.minimum.slot()) {
                    if !extent_ids.contains(&identity.id) {
                        extent_ids.push(identity.id);
                    }
                } else {
                    context.unsupported(
                        "m3.animation.extent",
                        "extent has no native property identity",
                    )?;
                    continue;
                }
                original = Some((times, rows));
            }
            if let Some((times, rows)) = original {
                let times = times
                    .into_iter()
                    .map(|t| signed_time(t, origin, context))
                    .collect::<Result<Vec<_>, _>>()?;
                let keys: Vec<_> = rows
                    .into_iter()
                    .map(|value| nr::ExtentRecord { value })
                    .collect();
                let index = u16::try_from(blocks.extents.len()).map_err(|_| {
                    Error::new("m3.animation.extent", "extent block count exceeds selector")
                })?;
                blocks.extents.push(nr::AnimationBlockExtent {
                    timestamps: append(document, &times)?.cast(),
                    keys: append(document, &keys)?.cast(),
                    flags: nr::AnimationBlockFlags::from(control.flags.bits()),
                    end_frame: end(Some(control), origin, clip, context)?,
                });
                for identity in extent_ids {
                    identities.push(nr::U32 { value: identity });
                    selectors.push(nr::U32 {
                        value: (12 << 16) | u32::from(index),
                    });
                }
            }
        }
    }
    Ok(())
}
