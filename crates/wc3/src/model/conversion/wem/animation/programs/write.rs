//! Output record capabilities follow authored presence rather than fabricating missing controls.
use super::super::{
    super::{metadata, text},
    routing,
};
use super::{
    codec::{append, error},
    controllers, declarations,
    inventory::NativePhysics,
    write_physics,
};
use crate::model::conversion::{Context, Error};
use crate::model::semantic::{
    profiles::{wem::animation::ReadRule, Wem},
    ClipId, Model, TextureId,
};
use crate::model::wem::{records, Bool, InlineOption, RawDocument, Ref};
use std::collections::BTreeMap;

pub(in crate::model::conversion::wem) struct ClipDraft {
    pub id: ClipId,
    pub name: Ref<records::targets::Character>,
    pub duration: f32,
    pub looping: Bool,
    pub flags: records::ClipFlags,
    pub containers: Ref<records::targets::SubTrackContainer>,
    pub events: Ref<records::targets::ClipEvent>,
    pub native: records::NativeBag,
    pub bounds: Option<records::Extent>,
    pub selections: Ref<records::targets::ClipTrackSet>,
    pub read_rule: Option<records::ReadRule>,
    physics: Option<NativePhysics>,
}
pub(in crate::model::conversion::wem) struct WrittenModel {
    pub channels: Ref<records::targets::AnimChannel>,
    pub selections: Ref<records::targets::TrackSet>,
    pub clips: Vec<ClipDraft>,
}
fn seconds(value: f64) -> Result<f32, Error> {
    let narrowed = value as f32;
    if !narrowed.is_finite() || f64::from(narrowed) != value {
        return Err(error(
            "authored seconds are not exactly representable in a WEM float",
        ));
    }
    Ok(narrowed)
}
pub(in crate::model::conversion::wem) fn write_model(
    document: &mut RawDocument<'_>,
    model: &Model<'_, Wem>,
    routing: &routing::WriteContext<'_, '_>,
    textures: &BTreeMap<TextureId, u32>,
    context: &mut Context,
) -> Result<WrittenModel, Error> {
    let declarations = declarations::write(
        document,
        &model.controls().channels,
        &model.controls().channel_selections,
        routing,
        textures,
    )?;
    let programs = controllers::write(document, model, &declarations, textures, context)?;
    let mut clips = Vec::new();
    for (id, clip) in model.clips() {
        let mut events = Vec::new();
        for value in &clip.controls.events {
            let node = value
                .node
                .map(|id| {
                    routing
                        .simulation
                        .nodes
                        .get(&id)
                        .copied()
                        .ok_or_else(|| error("event node belongs to another model"))
                })
                .transpose()?
                .unwrap_or(u32::MAX);
            events.push(records::ClipEvent {
                time: seconds(value.time.as_seconds())?,
                node,
                name: text::write_bytes(document, value.name.as_bytes())?,
                value: value.value,
            });
        }
        let mut selected = Vec::new();
        for value in &clip.controls.selections {
            selected.push(records::ClipTrackSet {
                set: *declarations
                    .selection_ids
                    .get(&value.selection)
                    .ok_or_else(|| error("clip selection belongs to another model"))?,
                priority: value.priority,
            });
        }
        let read_rule = clip.controls.read_rule.map(|value| match value {
            ReadRule::Wc3 => records::ReadRule::Wc3,
            ReadRule::Sc2 => records::ReadRule::Sc2,
            ReadRule::Wow => records::ReadRule::Wow,
            ReadRule::Unknown(value) => records::ReadRule::Unknown(value),
        });
        clips.push(ClipDraft {
            id,
            name: text::write_bytes(document, clip.name.as_bytes())?,
            duration: seconds(clip.duration.as_seconds())?,
            looping: Bool(u8::from(clip.looping)),
            flags: records::ClipFlags::from_bits_retain(clip.controls.flags.bits()),
            containers: programs.reference(&clip.controls.programs)?,
            events: append(document, &events)?.cast(),
            native: metadata::write(document, &clip.controls.metadata)?,
            bounds: clip.controls.bounds.map(|bounds| records::Extent {
                minimum: bounds.minimum,
                maximum: bounds.maximum,
                sphere_radius: bounds.sphere_radius,
            }),
            selections: append(document, &selected)?.cast(),
            read_rule,
            physics: clip
                .controls
                .physics
                .as_ref()
                .map(|source| {
                    write_physics::write(
                        document,
                        source,
                        &programs,
                        &declarations,
                        routing,
                        context,
                    )
                })
                .transpose()?,
        });
    }
    Ok(WrittenModel {
        channels: declarations.channels,
        selections: declarations.selections,
        clips,
    })
}

pub(in crate::model::conversion::wem) fn write_clips(
    document: &mut RawDocument<'_>,
    drafts: Vec<(u32, ClipDraft)>,
) -> Result<(Ref<records::targets::Clip>, BTreeMap<ClipId, u32>), Error> {
    if drafts.is_empty() {
        return Ok((Ref::default(), BTreeMap::new()));
    }
    let bounds = drafts[0].1.bounds.is_some();
    let rule = drafts[0].1.read_rule.is_some();
    if drafts
        .iter()
        .any(|(_, row)| row.bounds.is_some() != bounds || row.read_rule.is_some() != rule)
    {
        return Err(error(
            "one CLIP table cannot mix authored bounds or read-rule presence",
        ));
    }
    let mut revision = if !bounds {
        1
    } else if rule {
        4
    } else if drafts.iter().any(|(_, row)| row.selections.entries != 0) {
        3
    } else {
        2
    };
    for (_, row) in &drafts {
        if let Some(physics) = &row.physics {
            revision = revision.max(match physics {
                NativePhysics::Legacy(_) => 5,
                NativePhysics::Cloth(_) => 6,
                NativePhysics::Soft(_) => 7,
            });
        }
    }
    if (!bounds && revision != 1) || (!rule && revision >= 4) {
        return Err(error(
            "recorded physics requires explicitly authored bounds and playback read rule",
        ));
    }
    if revision == 1 && drafts.iter().any(|(_, row)| row.selections.entries != 0) {
        return Err(error(
            "channel selections require explicitly authored bounds in this native table",
        ));
    }
    let ids = drafts
        .iter()
        .enumerate()
        .map(|(ordinal, (_, row))| Ok((row.id, u32::try_from(ordinal).map_err(error)?)))
        .collect::<Result<BTreeMap<_, _>, Error>>()?;
    macro_rules! rows {
        ($d:ident,$bounds:expr,$sets:expr,$rule:expr,$physics:expr) => {{
            let mut rows = Vec::new();
            for (model, row) in drafts {
                let bounds = ($bounds)(&row);
                let track_sets = ($sets)(&row);
                let read_rule = ($rule)(&row);
                rows.push(records::Clip::<records::$d> {
                    name: row.name,
                    model,
                    duration: row.duration,
                    looping: row.looping,
                    flags: row.flags,
                    containers: row.containers,
                    events: row.events,
                    native: row.native,
                    bounds,
                    track_sets,
                    read_rule,
                    physics: ($physics)(row.physics)?,
                });
            }
            append(document, &rows)?.cast()
        }};
    }
    let reference = match revision {
        1 => rows!(
            V1,
            |_: &ClipDraft| (),
            |_: &ClipDraft| (),
            |_: &ClipDraft| (),
            |_: Option<NativePhysics>| Ok::<_, Error>(())
        ),
        2 => rows!(
            V2,
            |row: &ClipDraft| row.bounds.clone().expect("checked presence"),
            |_: &ClipDraft| (),
            |_: &ClipDraft| (),
            |_: Option<NativePhysics>| Ok::<_, Error>(())
        ),
        3 => rows!(
            V3,
            |row: &ClipDraft| row.bounds.clone().expect("checked presence"),
            |row: &ClipDraft| row.selections,
            |_: &ClipDraft| (),
            |_: Option<NativePhysics>| Ok::<_, Error>(())
        ),
        4 => rows!(
            V4,
            |row: &ClipDraft| row.bounds.clone().expect("checked presence"),
            |row: &ClipDraft| row.selections,
            |row: &ClipDraft| row.read_rule.expect("checked presence"),
            |_: Option<NativePhysics>| Ok::<_, Error>(())
        ),
        5 => rows!(
            V5,
            |row: &ClipDraft| row.bounds.clone().expect("checked presence"),
            |row: &ClipDraft| row.selections,
            |row: &ClipDraft| row.read_rule.expect("checked presence"),
            |value| match value {
                None => Ok::<_, Error>(InlineOption::from(None)),
                Some(NativePhysics::Legacy(value)) => Ok(InlineOption::from(Some(value))),
                _ => Err(error("incompatible recorded physics declaration")),
            }
        ),
        6 => rows!(
            V6,
            |row: &ClipDraft| row.bounds.clone().expect("checked presence"),
            |row: &ClipDraft| row.selections,
            |row: &ClipDraft| row.read_rule.expect("checked presence"),
            |value| match value {
                None => Ok::<_, Error>(InlineOption::from(None)),
                Some(value) => Ok(InlineOption::from(Some(upgrade6(value)?))),
            }
        ),
        _ => rows!(
            V7,
            |row: &ClipDraft| row.bounds.clone().expect("checked presence"),
            |row: &ClipDraft| row.selections,
            |row: &ClipDraft| row.read_rule.expect("checked presence"),
            |value| match value {
                None => Ok::<_, Error>(InlineOption::from(None)),
                Some(value) => Ok(InlineOption::from(Some(upgrade7(value)?))),
            }
        ),
    };
    Ok((reference, ids))
}

fn upgrade6(value: NativePhysics) -> Result<records::ClipPhysics<records::V6>, Error> {
    match value {
        NativePhysics::Cloth(value) => Ok(value),
        NativePhysics::Legacy(value) if value.baked.value.is_none() => Ok(records::ClipPhysics {
            plays: value.plays,
            comes_from: value.comes_from,
            comes_from_clip: value.comes_from_clip,
            preheat: value.preheat,
            goes_to: value.goes_to,
            goes_to_clip: value.goes_to_clip,
            match_: value.match_,
            matching_where: value.matching_where,
            window: value.window,
            ease: value.ease,
            curve: value.curve,
            tolerance: value.tolerance,
            mesh_measure: value.mesh_measure,
            key_rate: value.key_rate,
            step_rate: value.step_rate,
            floor: value.floor,
            travel: value.travel,
            world: value.world,
            baked: InlineOption::from(None),
        }),
        _ => Err(error("mixed bake report declaration capabilities")),
    }
}

fn upgrade7(value: NativePhysics) -> Result<records::ClipPhysics<records::V7>, Error> {
    macro_rules! upgrade {
        ($value:expr) => {{
            let value = $value;
            if value.baked.value.is_some() {
                return Err(error("mixed bake report declaration capabilities"));
            }
            Ok(records::ClipPhysics {
                plays: value.plays,
                comes_from: value.comes_from,
                comes_from_clip: value.comes_from_clip,
                preheat: value.preheat,
                goes_to: value.goes_to,
                goes_to_clip: value.goes_to_clip,
                match_: value.match_,
                matching_where: value.matching_where,
                window: value.window,
                ease: value.ease,
                curve: value.curve,
                tolerance: value.tolerance,
                mesh_measure: value.mesh_measure,
                key_rate: value.key_rate,
                step_rate: value.step_rate,
                floor: value.floor,
                travel: value.travel,
                world: value.world,
                baked: InlineOption::from(None),
            })
        }};
    }
    match value {
        NativePhysics::Soft(value) => Ok(value),
        NativePhysics::Legacy(value) => upgrade!(value),
        NativePhysics::Cloth(value) => upgrade!(value),
    }
}
