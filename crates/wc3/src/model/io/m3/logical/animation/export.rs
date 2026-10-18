//! Native sequence/controller assembly from actual authored playback programs.
use super::super::{
    bindings::{self, Target},
    Graph,
};
use super::{blocks::Blocks, AnimationId, AnimationIds};
use crate::model::conversion::{Context, Error};
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::{
    profiles::m3 as surface, Channel, Interpolation, NodeId, Time, TimeSource, ValueSlot,
};
use native::{Envelope, Ref};
use std::collections::{BTreeSet, HashMap};

pub(in crate::model::io::m3::logical) struct Program {
    pub sequences: Ref<nr::targets::Sequence>,
    pub controllers: Ref<nr::targets::SubTrackContainer>,
    pub groups: Ref<nr::targets::AnimationGroup>,
    pub states: Ref<nr::targets::AnimationState>,
}
pub(super) fn append<
    E: Envelope,
    T: native::Record + native::Write<E> + native::RecordLayout<E>,
>(
    document: &mut native::Container<'_, E>,
    rows: &[T],
) -> Result<Ref<T>, Error> {
    if rows.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(rows.len())
        .map_err(|_| Error::new("m3.animation.count", "array exceeds native count"))?;
    let section = document
        .append(rows)
        .map_err(|e| Error::new("m3.animation.write", e.to_string()))?;
    Ok(Ref::from_section(count, section.index()))
}
pub(super) fn text<E: Envelope>(
    document: &mut native::Container<'_, E>,
    text: &str,
) -> Result<Ref<nr::targets::Character>, Error> {
    let rows: Vec<_> = text
        .bytes()
        .chain([0])
        .map(|value| nr::Character { value })
        .collect();
    Ok(append(document, &rows)?.cast())
}
fn target(value: &Target) -> ValueSlot {
    match value {
        Target::Scalar(p) => p.slot(),
        Target::Vector2(p) => p.slot(),
        Target::Vector3(p) => p.slot(),
        Target::Vector4(p) => p.slot(),
        Target::Unsigned(p) => p.slot(),
        Target::Unsigned16(p) => p.slot(),
        Target::Rotation(p) => p.slot(),
        Target::Signed(p) => p.slot(),
        Target::Byte(p) => p.slot(),
        Target::Bytes4(p) => p.slot(),
        Target::Unsigned4(p) => p.slot(),
    }
}
fn interpolation(channel: &Channel<'_>) -> Interpolation {
    match channel {
        Channel::Scalar(t) => t.curve.interpolation(),
        Channel::Vector2(t) => t.curve.interpolation(),
        Channel::Vector3(t) => t.curve.interpolation(),
        Channel::Vector4(t) => t.curve.interpolation(),
        Channel::Rotation(t) => t.curve.interpolation(),
        Channel::Unsigned(t) => t.curve.interpolation(),
        Channel::Signed(t) => t.curve.interpolation(),
        Channel::Unsigned16(t) => t.curve.interpolation(),
        Channel::Byte(t) => t.curve.interpolation(),
        Channel::Boolean(t) => t.curve.interpolation(),
        Channel::Bytes4(t) => t.curve.interpolation(),
        Channel::Unsigned4(t) => t.curve.interpolation(),
        Channel::Texture(t) => t.curve.interpolation(),
        Channel::OptionalTexture(t) => t.curve.interpolation(),
    }
}

pub(in crate::model::io::m3::logical) fn prepare(
    model: &Graph<'_>,
    context: &mut Context,
) -> Result<AnimationIds, Error> {
    let existing = bindings::collect(model);
    let mut reserved: BTreeSet<_> = existing.entries.keys().copied().collect();
    let mut original = HashMap::new();
    for (id, values) in existing.entries {
        for value in values {
            original.insert(target(&value), id);
        }
    }
    for (_, controller) in model.controllers() {
        for block in &controller.controls.blocks {
            reserved.insert(block.source_id);
            for target in &block.targets {
                original.insert(*target, block.source_id);
            }
        }
    }
    for (_, controller) in model.controllers() {
        reserved.extend(
            controller
                .controls
                .unbound
                .iter()
                .filter_map(|block| block.identity),
        );
    }
    let mut next = reserved.last().copied().unwrap_or(0).checked_add(1);
    let mut ids = AnimationIds::new(model.values());
    let mut claimed = HashMap::new();
    for (_, program) in model.controllers() {
        for channel in model.animation_channels(&program.animations) {
            let mode = match interpolation(channel) {
                Interpolation::Step => surface::values::Interpolation::Constant,
                Interpolation::Linear => surface::values::Interpolation::Linear,
                _ => {
                    context.unsupported(
                        "m3.animation.interpolation",
                        "M3 has no encoding for the requested interpolation directive",
                    )?;
                    continue;
                }
            };
            if let Some(previous) = ids.get(&channel.target()) {
                if previous.interpolation != mode {
                    context.unsupported(
                        "m3.animation.interpolation",
                        "one property uses conflicting native interpolation modes",
                    )?;
                }
                continue;
            }
            let id = if let Some(id) = original
                .get(&channel.target())
                .copied()
                .filter(|id| *id != 0)
                .filter(|id| !claimed.contains_key(id))
            {
                id
            } else {
                let id =
                    next.ok_or_else(|| Error::new("m3.animation.id", "identity domain exhausted"))?;
                next = id.checked_add(1);
                id
            };
            ids.insert(
                channel.target(),
                AnimationId {
                    id,
                    interpolation: mode,
                    keyed: true,
                },
            );
            claimed.insert(id, channel.target());
        }
    }
    if model
        .global_channels()
        .find(|v| matches!(v.clock(), TimeSource::Shared(_)))
        .is_some()
        || model
            .global_events()
            .find(|v| matches!(v.clock, TimeSource::Shared(_)))
            .is_some()
    {
        context.unsupported(
            "m3.animation.clock",
            "native M3 animation belongs to controller programs",
        )?;
    }
    Ok(ids)
}

pub(super) fn unsigned_time(value: Time, context: &mut Context) -> Result<u32, Error> {
    let exact = value.as_seconds() * 1000.;
    if !exact.is_finite() || exact.round() < 0. || exact.round() > f64::from(u32::MAX) {
        return Err(Error::new(
            "m3.animation.time",
            "time exceeds native unsigned milliseconds",
        ));
    }
    let raw = exact.round() as u32;
    context.approximate(
        "m3.animation.time",
        ((f64::from(raw) - exact) / 1000.).abs() as f32,
        "time quantized to native milliseconds",
    )?;
    Ok(raw)
}

pub(in crate::model::io::m3::logical) fn write<E: Envelope>(
    model: &Graph<'_>,
    document: &mut native::Container<'_, E>,
    ids: &AnimationIds,
    nodes: &[NodeId],
    context: &mut Context,
) -> Result<Program, Error> {
    let mut sequences = Vec::new();
    let mut controllers = Vec::new();
    let mut groups = Vec::new();
    let mut states = Vec::new();
    let mut controller_indices = HashMap::new();
    for (controller_id, logical) in model.controllers() {
        let native_controls = Some(&logical.controls);
        let mut blocks = Blocks::default();
        let mut identities = Vec::new();
        let mut selectors = Vec::new();
        let mut used = BTreeSet::new();
        for channel in model.animation_channels(&logical.animations) {
            if channel.clock() != TimeSource::Animation {
                context.unsupported("m3.animation.clock", "controller uses a foreign clock")?;
                continue;
            }
            if native_controls.is_some_and(|c|c.blocks.iter().any(|b|b.encoding==surface::KeyEncoding::Extent&&b.extents.iter().any(|id|model.feature(*id).is_some_and(|f|matches!(&f.payload,surface::Feature::Extent(binding)if channel.target()==binding.minimum.slot()||channel.target()==binding.maximum.slot()||channel.target()==binding.radius.slot()))))){continue}
            let Some(identity) = ids.get(&channel.target()) else {
                context.unsupported(
                    "m3.animation.destination",
                    "channel lacks a native property identity",
                )?;
                continue;
            };
            if !used.insert(identity.id) {
                context.unsupported(
                    "m3.animation.identity",
                    "multiple property curves require a shared native controller identity",
                )?;
                continue;
            }
            let block = native_controls.and_then(|c| {
                c.blocks
                    .iter()
                    .find(|b| b.targets.iter().any(|target| *target == channel.target()))
            });
            if let Some(selector) = blocks.add(
                document,
                channel,
                Time::default(),
                Time::default(),
                block,
                context,
            )? {
                identities.push(nr::U32 { value: identity.id });
                selectors.push(nr::U32 { value: selector });
            }
        }
        super::structured_export::add(
            model,
            super::structured_export::Program {
                controller: logical,
                controls: native_controls,
                origin: Time::default(),
            },
            nodes,
            ids,
            super::structured_export::Output {
                blocks: &mut blocks,
                document,
                identities: &mut identities,
                selectors: &mut selectors,
            },
            context,
        )?;
        for source in &logical.controls.unbound {
            if let Some(selector) = super::raw_export::add(document, &mut blocks, source, context)?
            {
                identities.push(nr::U32 {
                    value: source.identity.unwrap(),
                });
                selectors.push(nr::U32 { value: selector });
            }
        }
        let state_properties = native_controls
            .filter(|c| !c.state_properties.is_empty())
            .map(|controls| {
                let mut properties = BTreeSet::new();
                for source_id in &controls.state_properties {
                    let mapped: Vec<_> = controls
                        .blocks
                        .iter()
                        .filter(|block| block.source_id == *source_id)
                        .flat_map(|block| &block.targets)
                        .filter_map(|target| ids.get(target).map(|value| value.id))
                        .collect();
                    if mapped.is_empty() {
                        properties.insert(*source_id);
                    } else {
                        properties.extend(mapped);
                    }
                }
                properties.into_iter().collect::<Vec<_>>()
            })
            .unwrap_or_else(|| identities.iter().map(|r| r.value).collect());
        let state_index = u16::try_from(states.len())
            .map_err(|_| Error::new("m3.animation.states", "state count exceeds u16"))?;
        states.push(nr::AnimationState {
            anim_ids: append(
                document,
                &state_properties
                    .iter()
                    .map(|&value| nr::U32 { value })
                    .collect::<Vec<_>>(),
            )?
            .cast(),
            ..Default::default()
        });
        let mut output = nr::SubTrackContainer::<nr::V4> {
            name: text(document, logical.name.as_ref())?,
            runs_concurrent: u16::from(logical.concurrent),
            anim_priority: u16::try_from(logical.priority)
                .map_err(|_| Error::new("m3.animation.priority", "priority exceeds native u16"))?,
            animation_state_index: state_index,
            animation_state_index_copy: state_index,
            anim_ids: append(document, &identities)?.cast(),
            anim_refs: append(document, &selectors)?.cast(),
            ..Default::default()
        };
        blocks.install(document, &mut output)?;
        controller_indices.insert(controller_id, controllers.len() as u32);
        controllers.push(output);
    }
    for (_, clip) in model.clips() {
        let controls = &clip.controls;
        let start = unsigned_time(clip.source_start, context)?;
        let end = start
            .checked_add(unsigned_time(clip.duration(), context)?)
            .ok_or_else(|| {
                Error::new(
                    "m3.sequence.duration",
                    "timeline overflows native milliseconds",
                )
            })?;
        if !clip.animations.is_empty() {
            context.unsupported(
                "m3.sequence.program",
                "M3 clips select controllers rather than owning channels or events",
            )?;
        }
        let name = text(document, clip.name())?;
        if let Some(selection) = &controls.selection {
            let selected = selection
                .controllers
                .iter()
                .map(|id| {
                    controller_indices
                        .get(id)
                        .copied()
                        .map(|value| nr::U32 { value })
                        .ok_or_else(|| {
                            Error::new("m3.sequence.controller", "selected controller is missing")
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            groups.push(nr::AnimationGroup {
                name: text(document, &selection.name)?,
                subtrack_indices: append(document, &selected)?.cast(),
            });
        }
        let mut flags = nr::SequenceFlags::from(controls.flags.bits());
        flags.set_not_looping(!clip.is_looping());
        let bounds = controls.bounds;
        sequences.push(nr::Sequence::<nr::V1> {
            id: controls.source_id,
            index: controls.sequence_number,
            deprecated_unknown: controls.legacy_control.unwrap_or_default(),
            name,
            start_frame: start,
            end_frame: end,
            move_speed: controls.movement_speed,
            flags,
            frequency: controls.frequency,
            replay_start: controls.replay_range[0],
            replay_end: controls.replay_range[1],
            blend_time: unsigned_time(controls.blend_duration, context)?,
            bounds: nr::Extent {
                minimum: bounds.minimum,
                maximum: bounds.maximum,
                radius: bounds.sphere_radius,
            },
            animation_sets: append(
                document,
                &controls
                    .animation_sets
                    .iter()
                    .map(|&value| nr::Byte { value })
                    .collect::<Vec<_>>(),
            )?
            .cast(),
        });
    }
    for (_, system) in model.systems() {
        if let surface::System::ControllerSelections(selections) = system {
            for selection in selections {
                let selected = selection
                    .controllers
                    .iter()
                    .map(|id| {
                        controller_indices
                            .get(id)
                            .copied()
                            .map(|value| nr::U32 { value })
                            .ok_or_else(|| {
                                Error::new(
                                    "m3.controller_selection",
                                    "selected controller is missing",
                                )
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                groups.push(nr::AnimationGroup {
                    name: text(document, &selection.name)?,
                    subtrack_indices: append(document, &selected)?.cast(),
                });
            }
        }
    }
    let sequences = if model
        .clips()
        .iter()
        .any(|(_, clip)| clip.controls.legacy_control.is_some())
    {
        append(document, &sequences)?.cast()
    } else {
        let sequences = sequences
            .into_iter()
            .map(|value| nr::Sequence::<nr::V2> {
                id: value.id,
                index: value.index,
                name: value.name,
                start_frame: value.start_frame,
                end_frame: value.end_frame,
                move_speed: value.move_speed,
                flags: value.flags,
                frequency: value.frequency,
                replay_start: value.replay_start,
                replay_end: value.replay_end,
                blend_time: value.blend_time,
                deprecated_unknown: (),
                bounds: value.bounds,
                animation_sets: value.animation_sets,
            })
            .collect::<Vec<_>>();
        append(document, &sequences)?.cast()
    };
    Ok(Program {
        sequences,
        controllers: append(document, &controllers)?.cast(),
        groups: append(document, &groups)?.cast(),
        states: append(document, &states)?.cast(),
    })
}
