//! Native programme inventories are resolved before publication into independent declarations.
use super::super::super::{metadata, text};
use super::super::{curves, routing};
use super::{codec::error, declarations, inventory::Prepared, physics};
use crate::model::conversion::{Context, EncodingHint, Error, IssueKind};
use crate::model::semantic::{
    profiles::{
        wem::animation::{
            AnyChannelId, AnyTrack, Channels, ClipControls, ClipFlags, ControllerControls, Event,
            PlaybackSelection, ReadRule, Restoration, Selections,
        },
        Wem,
    },
    Bounds, ClipId, Controller, ControllerId, ModelBuilder, TextureId, Time, Values,
};
use crate::model::wem::{records, RawDocument, Ref};
use std::collections::{BTreeMap, HashMap};

fn tracks<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::SubTrack>,
    channels: &Channels,
    ids: &HashMap<u32, AnyChannelId>,
    textures: &[TextureId],
) -> Result<Vec<AnyTrack<'a>>, Error> {
    let Some(section) = document.referenced_section(reference).map_err(error)? else {
        return Ok(Vec::new());
    };
    let mut output = Vec::new();
    macro_rules! read {
        ($d:ident) => {
            for row in document
                .resolve::<records::SubTrack<records::$d>>(reference.cast())
                .map_err(error)?
                .iter()
            {
                let row = row.map_err(error)?;
                let id = ids
                    .get(&row.channel)
                    .ok_or_else(|| error("STRK names an undeclared channel UID"))?;
                let declaration = channels
                    .declaration(*id)
                    .ok_or_else(|| error("channel belongs to another model"))?;
                output.push(curves::read(document, &row, declaration, textures)?);
            }
        };
    }
    match section.revision() {
        1 => read!(V1),
        2 => read!(V2),
        _ => return Err(error("unsupported STRK revision")),
    }
    Ok(output)
}

pub(super) struct NativeController<'a> {
    pub key: (u32, u32),
    pub value: Controller<'a, Wem>,
}
pub(super) struct Playback<'a> {
    pub id: ClipId,
    pub programs: Vec<(u32, u32)>,
    pub controls: ClipControls<'a>,
    pub physics: Option<physics::Pending<'a>>,
}
pub(in crate::model::conversion::wem) struct DecodedPrograms<'a> {
    pub(super) channels: Channels,
    pub(super) selections: Selections<'a>,
    pub(super) controllers: Vec<NativeController<'a>>,
    pub(super) clips: Vec<Playback<'a>>,
    pub encoding_hints: Vec<EncodingHint<'a>>,
}

fn controllers<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::SubTrackContainer>,
    channels: &Channels,
    ids: &HashMap<u32, AnyChannelId>,
    textures: &[TextureId],
    known: &mut BTreeMap<(u32, u32), NativeController<'a>>,
    context: &mut Context,
) -> Result<Vec<(u32, u32)>, Error> {
    let mut result = Vec::new();
    for (ordinal, row) in document
        .resolve::<records::SubTrackContainer>(reference.cast())
        .map_err(error)?
        .iter()
        .enumerate()
    {
        let row = row.map_err(error)?;
        let key = (reference.index, u32::try_from(ordinal).map_err(error)?);
        if let std::collections::btree_map::Entry::Vacant(entry) = known.entry(key) {
            if row.concurrent.0 > 1 {
                context.issue(
                    "wem.controller.concurrent",
                    IssueKind::Normalized,
                    "nonzero concurrent byte is represented as true",
                );
            }
            let value = Controller {
                name: text::read(document, row.name)?,
                priority: row.priority,
                concurrent: row.concurrent.0 != 0,
                controls: ControllerControls {
                    tracks: tracks(document, row.sub_tracks, channels, ids, textures)?,
                    metadata: metadata::read(document, &row.native)?,
                },
                ..Controller::default()
            };
            entry.insert(NativeController { key, value });
        }
        result.push(key);
    }
    Ok(result)
}

pub(in crate::model::conversion::wem) struct SourceTables {
    pub model: usize,
    pub channels: Ref<records::targets::AnimChannel>,
    pub selections: Ref<records::targets::TrackSet>,
}
pub(in crate::model::conversion::wem) fn decode_model<'a>(
    document: &'a RawDocument<'_>,
    prepared: &Prepared,
    tables: SourceTables,
    routing: &routing::ReadContext<'_, '_>,
    textures: &[TextureId],
    context: &mut Context,
) -> Result<DecodedPrograms<'a>, Error> {
    let SourceTables {
        model: model_index,
        channels: source_channels,
        selections: source_selections,
    } = tables;
    let nodes = routing.simulation.nodes;
    let (channels, ids) = declarations::channels(document, source_channels, routing, textures)?;
    let (selections, selection_ids) =
        declarations::selections(document, source_selections, &channels, &ids)?;
    let mut known = BTreeMap::new();
    let mut clips = Vec::new();
    let mut encoding_hints = Vec::new();
    for source in prepared
        .clips
        .iter()
        .filter(|clip| clip.model == model_index)
    {
        let programs = controllers(
            document,
            source.containers,
            &channels,
            &ids,
            textures,
            &mut known,
            context,
        )?;
        let mut selected = Vec::new();
        for row in document
            .resolve::<records::ClipTrackSet>(source.selections.cast())
            .map_err(error)?
            .iter()
        {
            let row = row.map_err(error)?;
            let selection = *selection_ids
                .get(row.set as usize)
                .ok_or_else(|| error("CTSU names a missing selection"))?;
            selected.push(PlaybackSelection {
                selection,
                priority: row.priority,
            });
        }
        let mut events = Vec::new();
        for row in document
            .resolve::<records::ClipEvent>(source.events.cast())
            .map_err(error)?
            .iter()
        {
            let row = row.map_err(error)?;
            let node = if row.node == u32::MAX {
                None
            } else {
                Some(
                    *nodes
                        .get(row.node as usize)
                        .ok_or_else(|| error("event names a missing node"))?,
                )
            };
            events.push(Event {
                time: Time::seconds(f64::from(row.time))?,
                node,
                name: text::read(document, row.name)?,
                value: row.value,
            });
        }
        let bounds = source.bounds.as_ref().map(|extent| Bounds {
            minimum: extent.minimum,
            maximum: extent.maximum,
            sphere_radius: extent.sphere_radius,
        });
        let read_rule = source.read_rule.map(|value| match value {
            records::ReadRule::Wc3 => ReadRule::Wc3,
            records::ReadRule::Sc2 => ReadRule::Sc2,
            records::ReadRule::Wow => ReadRule::Wow,
            records::ReadRule::Unknown(word) => ReadRule::Unknown(word),
        });
        let controls = ClipControls {
            programs: Vec::new(),
            selections: selected,
            events,
            flags: ClipFlags::from_bits_retain(source.flags.bits()),
            read_rule,
            bounds,
            physics: None,
            metadata: metadata::read(document, &source.metadata)?,
        };
        let pending = if let Some(recipe) = &source.physics {
            let (mut value, bake) = physics::recipe(document, recipe, nodes, context)?;
            let mut originals = Vec::new();
            let mut soft_originals = None;
            if let Some(bake) = bake {
                let restored = controllers(
                    document,
                    bake.source,
                    &channels,
                    &ids,
                    textures,
                    &mut known,
                    context,
                )?;
                let positions = super::codec::read_words(document, bake.positions)?;
                let mut offset = 0;
                for key in restored {
                    let count = known
                        .get(&key)
                        .expect("decoded controller")
                        .value
                        .controls
                        .tracks
                        .len();
                    if offset + count > positions.len() {
                        return Err(error("saved programme curve positions are incomplete"));
                    }
                    let subset = positions.clone();
                    let start = offset;
                    originals.push((
                        key,
                        Values::view(count, move |i| {
                            subset.get(start + i).expect("checked position")
                        }),
                    ));
                    offset += count;
                }
                if offset != positions.len() {
                    return Err(error("saved programme has extra curve positions"));
                }
                soft_originals = bake
                    .soft_source
                    .map(|reference| {
                        controllers(
                            document, reference, &channels, &ids, textures, &mut known, context,
                        )
                    })
                    .transpose()?;
                value.baked = Some(physics::report(document, &bake, &ids, routing)?);
                encoding_hints.push(EncodingHint::WemBakeFingerprints {
                    clip: source.id,
                    inputs: bake.fingerprints[0],
                    written: bake.fingerprints[1],
                });
            }
            Some(physics::Pending {
                value,
                source: originals,
                soft_source: soft_originals,
            })
        } else {
            None
        };
        clips.push(Playback {
            id: source.id,
            programs,
            controls,
            physics: pending,
        });
    }
    Ok(DecodedPrograms {
        channels,
        selections,
        controllers: known.into_values().collect(),
        clips,
        encoding_hints,
    })
}

pub(in crate::model::conversion::wem) fn install<'a>(
    decoded: DecodedPrograms<'a>,
    builder: &mut ModelBuilder<'a, Wem>,
) -> Result<Vec<EncodingHint<'a>>, Error> {
    let mut controls = builder.model().controls().clone();
    controls.channels = decoded.channels;
    controls.channel_selections = decoded.selections;
    builder.set_controls(controls);
    let mut ids = BTreeMap::new();
    for controller in decoded.controllers {
        ids.insert(controller.key, builder.add_controller(controller.value)?);
    }
    for playback in decoded.clips {
        let mut clip = builder
            .model()
            .clip(playback.id)
            .ok_or_else(|| error("prepared clip belongs to another model"))?
            .clone();
        clip.controls = playback.controls;
        clip.controls.programs = playback
            .programs
            .iter()
            .map(|key| {
                ids.get(key)
                    .copied()
                    .ok_or_else(|| error("missing decoded programme"))
            })
            .collect::<Result<Vec<ControllerId>, Error>>()?;
        if let Some(mut pending) = playback.physics {
            if let Some(report) = &mut pending.value.baked {
                report.source = pending
                    .source
                    .into_iter()
                    .map(|(key, positions)| {
                        Ok(Restoration {
                            program: *ids
                                .get(&key)
                                .ok_or_else(|| error("missing restoration programme"))?,
                            positions,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                report.soft_source = pending
                    .soft_source
                    .map(|keys| {
                        keys.into_iter()
                            .map(|key| {
                                ids.get(&key)
                                    .copied()
                                    .ok_or_else(|| error("missing soft restoration programme"))
                            })
                            .collect::<Result<Vec<_>, Error>>()
                    })
                    .transpose()?;
            }
            clip.controls.physics = Some(pending.value);
        }
        builder.set_clip(playback.id, clip)?;
    }
    Ok(decoded.encoding_hints)
}
