//! Sequence groups select controllers whose channels bind to logical properties.
use super::super::bindings::{Bindings, Target};
use super::super::GraphBuilder;
use super::{events, extents, keys, raw_keys};
use crate::model::conversion::{Context, Error};
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::ValueSlot;
use crate::model::{
    profiles::m3 as surface, Bounds, Clip, ClipId, Controller, Interpolation, NodeId, Time,
};
use native::{Envelope, Ref};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

trait LegacyControl {
    fn authored(&self) -> Option<u32>;
}
impl LegacyControl for () {
    fn authored(&self) -> Option<u32> {
        None
    }
}
impl LegacyControl for u32 {
    fn authored(&self) -> Option<u32> {
        Some(*self)
    }
}
fn error(error: native::ReadError) -> Error {
    Error::new("m3.animation", error.to_string())
}
fn revision<E: Envelope, T: native::Target>(
    document: &native::Container<'_, E>,
    reference: Ref<T>,
) -> Result<u32, Error> {
    document
        .referenced_section(reference)
        .map_err(error)?
        .map(|section| section.revision())
        .ok_or_else(|| Error::new("m3.animation.reference", "missing section"))
}
fn target_property(target: &Target) -> ValueSlot {
    match target {
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

pub(in crate::model::io::m3::logical) fn read<'a, E: Envelope + Send + Sync>(
    source: &'a native::Document<'_, E>,
    nodes: &[NodeId],
    bindings: &Bindings,
    builder: &mut GraphBuilder<'a>,
    context: &mut Context,
) -> Result<Vec<ClipId>, Error> {
    let document = source.container();
    let root = source.root_record().map_err(error)?;
    let (container_ref, group_ref) =
        native::visit_m3_root!(root, |r| (r.sub_track_collections, r.animation_groups));
    let mut containers = Vec::new();
    macro_rules! read_controllers {
        ($v:ty) => {{
            for row in document
                .resolve::<nr::SubTrackContainer<$v>>(container_ref.cast())
                .map_err(error)?
                .iter()
            {
                let c = row.map_err(error)?;
                containers.push(nr::SubTrackContainer::<nr::V4> {
                    name: c.name,
                    runs_concurrent: c.runs_concurrent,
                    anim_priority: c.anim_priority,
                    animation_state_index: c.animation_state_index,
                    animation_state_index_copy: c.animation_state_index_copy,
                    anim_ids: c.anim_ids,
                    anim_refs: c.anim_refs,
                    unknown: c.unknown,
                    sdev: c.sdev,
                    sd2v: c.sd2v,
                    sd3v: c.sd3v,
                    sd4q: c.sd4q,
                    sdcc: c.sdcc,
                    sdr3: c.sdr3,
                    sdu8: c.sdu8,
                    sds6: c.sds6,
                    sdu6: c.sdu6,
                    sds3: c.sds3,
                    sdu3: c.sdu3,
                    sdfg: c.sdfg,
                    sdmb: c.sdmb,
                });
            }
        }};
    }
    if container_ref.entries != 0 {
        match revision(document, container_ref)? {
            0 => read_controllers!(nr::V0),
            1 => read_controllers!(nr::V1),
            2 => read_controllers!(nr::V2),
            3 => read_controllers!(nr::V3),
            4 => read_controllers!(nr::V4),
            _ => return Err(Error::new("m3.controllers", "unsupported revision")),
        }
    }
    let state_ref = native::visit_m3_root!(root, |r| r.animation_states);
    let states = document
        .resolve::<nr::AnimationState>(state_ref.cast())
        .map_err(error)?;
    let mut event_cache = events::Cache::new();
    let mut layers = Vec::new();
    for controller in &containers {
        let layer = builder.add_controller(Controller {
            name: document
                .string(controller.name)
                .map_err(error)?
                .unwrap_or_default()
                .into(),
            priority: i32::from(controller.anim_priority),
            concurrent: controller.runs_concurrent != 0,
            ..Default::default()
        })?;
        layers.push(layer);
        let mut controls = surface::ControllerControls::default();
        if let Some(state) = states.get(usize::from(controller.animation_state_index)) {
            let state = state.map_err(error)?;
            controls.state_properties = document
                .resolve::<nr::U32>(state.anim_ids.cast())
                .map_err(error)?
                .iter()
                .map(|row| row.map(|row| row.value))
                .collect::<Result<_, _>>()
                .map_err(error)?;
        } else if state_ref.entries != 0 {
            return Err(Error::new(
                "m3.animation.state",
                "controller references a missing state",
            ));
        }
        let identities = document
            .resolve::<nr::U32>(controller.anim_ids.cast())
            .map_err(error)?;
        let selectors = document
            .resolve::<nr::U32>(controller.anim_refs.cast())
            .map_err(error)?;
        if identities.len() != selectors.len() {
            return Err(Error::new("m3.controllers", "ID/selector count differs"));
        }
        let mut used = BTreeSet::new();
        let mut event_groups: BTreeMap<u32, Vec<crate::model::EventId>> = BTreeMap::new();
        for (identity, selector) in identities.iter().zip(selectors.iter()) {
            let identity = identity.map_err(error)?.value;
            let selector = selector.map_err(error)?.value;
            used.insert(selector);
            match selector >> 16 {
                0 => {
                    let occurrences = if let Some(events) = event_groups.get(&selector) {
                        events.clone()
                    } else {
                        let events = events::read(
                            document,
                            controller,
                            selector,
                            layer,
                            nodes,
                            builder,
                            &mut event_cache,
                        )?;
                        event_groups.insert(selector, events.clone());
                        events
                    };
                    controls.blocks.push(keys::controls(
                        document,
                        controller,
                        selector,
                        identity,
                        Vec::new(),
                        occurrences,
                        Vec::new(),
                    )?);
                    continue;
                }
                12 => {
                    let extent = extents::read(
                        document, controller, selector, identity, layer, builder, context,
                    )?;
                    controls.blocks.push(keys::controls(
                        document,
                        controller,
                        selector,
                        identity,
                        Vec::new(),
                        Vec::new(),
                        extent,
                    )?);
                    continue;
                }
                _ => (),
            }
            let Some(destinations) = bindings.entries.get(&identity) else {
                controls.unbound.push(raw_keys::read(
                    document,
                    controller,
                    selector,
                    Some(identity),
                )?);
                continue;
            };
            controls.blocks.push(keys::controls(
                document,
                controller,
                selector,
                identity,
                destinations.iter().map(target_property).collect(),
                Vec::new(),
                Vec::new(),
            )?);
            for destination in destinations {
                let mode = match bindings
                    .modes
                    .get(&target_property(destination))
                    .copied()
                    .unwrap_or_default()
                {
                    surface::values::Interpolation::Constant => Interpolation::Step,
                    surface::values::Interpolation::Linear => Interpolation::Linear,
                    _ => {
                        context
                            .unsupported("m3.animation.interpolation", "unknown interpolation")?;
                        continue;
                    }
                };
                if let Some(channel) =
                    keys::read(document, controller, selector, destination, mode, context)?
                {
                    builder.add_controller_channel(layer, channel)?;
                }
            }
        }
        let counts = [
            controller.sdev.entries,
            controller.sd2v.entries,
            controller.sd3v.entries,
            controller.sd4q.entries,
            controller.sdcc.entries,
            controller.sdr3.entries,
            controller.sdu8.entries,
            controller.sds6.entries,
            controller.sdu6.entries,
            controller.sds3.entries,
            controller.sdu3.entries,
            controller.sdfg.entries,
            controller.sdmb.entries,
        ];
        for (family, count) in counts.into_iter().enumerate() {
            for index in 0..count {
                let selector = ((family as u32) << 16) | index;
                if index <= u32::from(u16::MAX) && used.contains(&selector) {
                    continue;
                }
                if family == 0 {
                    let occurrences = events::read(
                        document,
                        controller,
                        index,
                        layer,
                        nodes,
                        builder,
                        &mut event_cache,
                    )?;
                    let block = document
                        .resolve::<nr::AnimationBlockEvent>(controller.sdev.cast())
                        .map_err(error)?
                        .get(index as usize)
                        .ok_or_else(|| Error::new("m3.animation.event", "missing orphan block"))?
                        .map_err(error)?;
                    controls.unbound.push(surface::UnboundBlock {
                        identity: None,
                        flags: surface::BlockFlags::from_bits_retain(block.flags.bits()),
                        end: Time::seconds(f64::from(block.end_frame) / 1000.)?,
                        keys: surface::UnboundKeys::Events(occurrences),
                    });
                } else {
                    controls.unbound.push(raw_keys::read_index(
                        document,
                        controller,
                        family as u32,
                        index as usize,
                        None,
                    )?);
                }
            }
        }
        builder.set_controller_controls(layer, controls)?;
    }
    let groups = document
        .resolve::<nr::AnimationGroup>(group_ref.cast())
        .map_err(error)?;
    let mut clips = Vec::new();
    macro_rules! sequences {
        ($v:ty) => {{
            for (index, row) in document
                .resolve::<nr::Sequence<$v>>(root.sequences().cast())
                .map_err(error)?
                .iter()
                .enumerate()
            {
                let row = row.map_err(error)?;
                let duration = row
                    .end_frame
                    .checked_sub(row.start_frame)
                    .ok_or_else(|| Error::new("m3.sequence", "ends before start"))?;
                let name = document
                    .string(row.name)
                    .map_err(error)?
                    .unwrap_or_default();
                let mut clip =
                    Clip::named(name).with_duration(Time::seconds(f64::from(duration) / 1000.)?);
                if row.flags.not_looping() {
                    clip = clip.once();
                }
                let animation_sets = document
                    .resolve::<nr::Byte>(row.animation_sets.cast())
                    .map_err(error)?
                    .iter()
                    .map(|row| row.map(|row| row.value))
                    .collect::<Result<_, _>>()
                    .map_err(error)?;
                let mut flags = surface::values::SequenceFlags::from(row.flags.bits());
                flags.set_not_looping(false);
                let mut selected = Vec::new();
                let mut group_name = None;
                if let Some(group) = groups.get(index) {
                    let group = group.map_err(error)?;
                    group_name = Some(Cow::Borrowed(
                        document
                            .string(group.name)
                            .map_err(error)?
                            .unwrap_or_default(),
                    ));
                    for row in document
                        .resolve::<nr::U32>(group.subtrack_indices.cast())
                        .map_err(error)?
                        .iter()
                    {
                        selected.push(*layers.get(row.map_err(error)?.value as usize).ok_or_else(
                            || Error::new("m3.controllers", "missing selected controller"),
                        )?);
                    }
                }
                clip.source_start = Time::seconds(f64::from(row.start_frame) / 1000.)?;
                clip.controls = surface::SequenceControls {
                    source_id: row.id,
                    sequence_number: row.index,
                    legacy_control: row.deprecated_unknown.authored(),
                    flags,
                    movement_speed: row.move_speed,
                    frequency: row.frequency,
                    replay_range: [row.replay_start, row.replay_end],
                    blend_duration: Time::seconds(f64::from(row.blend_time) / 1000.)?,
                    bounds: Bounds {
                        minimum: row.bounds.minimum,
                        maximum: row.bounds.maximum,
                        sphere_radius: row.bounds.radius,
                    },
                    animation_sets,
                    selection: group_name.map(|name| surface::ControllerSelection {
                        name,
                        controllers: selected,
                    }),
                };
                clips.push(builder.add_clip(clip)?);
            }
        }};
    }
    if root.sequences().entries != 0 {
        match revision(document, root.sequences())? {
            0 => sequences!(nr::V0),
            1 => sequences!(nr::V1),
            2 => sequences!(nr::V2),
            _ => return Err(Error::new("m3.sequences", "unsupported revision")),
        }
    }
    let mut unselected = Vec::new();
    for group in groups.iter().skip(clips.len()) {
        let group = group.map_err(error)?;
        let mut controllers = Vec::new();
        for row in document
            .resolve::<nr::U32>(group.subtrack_indices.cast())
            .map_err(error)?
            .iter()
        {
            controllers.push(
                *layers
                    .get(row.map_err(error)?.value as usize)
                    .ok_or_else(|| {
                        Error::new("m3.controller_selection", "selected controller is missing")
                    })?,
            );
        }
        let name = Cow::Borrowed(
            document
                .string(group.name)
                .map_err(error)?
                .unwrap_or_default(),
        );
        unselected.push(surface::ControllerSelection { name, controllers });
    }
    if !unselected.is_empty() {
        builder.add_system(surface::System::ControllerSelections(unselected))?;
    }
    Ok(clips)
}
