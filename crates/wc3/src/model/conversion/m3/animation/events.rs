//! Timed event commands with editable names, parameters, and transform poses.
use super::super::effects::metadata as native_metadata;
use super::super::hierarchy;
use super::*;
use std::array::from_fn;
trait Word {
    fn word(self) -> u32;
}
impl Word for () {
    fn word(self) -> u32 {
        0
    }
}
impl Word for u32 {
    fn word(self) -> u32 {
        self
    }
}
pub(super) fn import_events<E: Envelope>(
    input: ImportBlock<'_, '_, E>,
    raw: &mut wem::RawDocument<'static>,
    model: &mut wr::ModelRecord<wr::V9>,
    events: &mut Vec<wr::ClipEvent>,
) -> Result<(), Error> {
    let ImportBlock {
        source,
        controller,
        selector,
        id: animation_id,
        start,
        ..
    } = input;
    let block = source
        .resolve::<nr::AnimationBlockEvent>(controller.sdev.cast())
        .map_err(read_error)?
        .get((selector & 0xffff) as usize)
        .ok_or_else(|| Error::new("m3.events", "missing event block"))?
        .map_err(read_error)?;
    let times: Vec<_> = source
        .resolve::<nr::I32>(block.timestamps.cast())
        .map_err(read_error)?
        .iter()
        .collect::<Result<_, _>>()
        .map_err(read_error)?;
    let mut nodes: Vec<_> = raw
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let mut schemas: Vec<_> = raw
        .resolve::<wr::PoseSchema<wr::V2>>(model.nodes.pose_schema.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let pose_slot = schemas.len();
    schemas.push(wr::PoseSchema::<wr::V2> {
        name: text(raw, "M3 event transform")?,
        storage: wr::PoseStorage::MATRIX,
        ..Default::default()
    });
    macro_rules! events {
        ($v:ty) => {{
            let rows = source
                .resolve::<nr::Event<$v>>(block.keys.cast())
                .map_err(read_error)?;
            if rows.len() != times.len() {
                return Err(Error::new("m3.events", "timestamp/event count differs"));
            }
            for (time, row) in times.iter().zip(rows.iter()) {
                let row = row.map_err(read_error)?;
                let name = string(source, row.name).map_err(read_error)?;
                let options = string(source, row.option_string).map_err(read_error)?;
                let fields = [
                    ("m3.effect.bone", i64::from(row.bone_index)),
                    ("m3.auxiliary", 1),
                    ("m3.event.animation_id", i64::from(animation_id)),
                    ("m3.event.unknown", i64::from(row.unknown)),
                    ("m3.event.padding", i64::from(row.padding)),
                    (
                        "m3.event.rtt_channel_index",
                        i64::from(row.rtt_channel_index.word()),
                    ),
                    (
                        "m3.event.extra_parameter",
                        i64::from(row.extra_parameter.word()),
                    ),
                    ("m3.event.flags", i64::from(block.flags)),
                    ("m3.event.end", i64::from(block.end_frame)),
                    ("m3.event.pose_slot", pose_slot as i64),
                ];
                let mut entries = Vec::new();
                for (name, value) in fields {
                    entries.push(wr::NativeEntry::<wr::V2> {
                        name: text(raw, name)?,
                        value,
                        ..Default::default()
                    });
                }
                entries.push(wr::NativeEntry::<wr::V2> {
                    name: text(raw, "m3.event.options")?,
                    text: text(raw, &options)?,
                    ..Default::default()
                });
                let identity = from_fn(|row| from_fn(|column| if row == column { 1. } else { 0. }));
                let mut poses = vec![wr::Matrix44 { value: identity }; pose_slot + 1];
                poses[pose_slot] = wr::Matrix44 {
                    value: row.transform,
                };
                let node = nodes.len() as u32;
                let name_ref = text(raw, &name)?;
                nodes.push(wr::Node::<wr::V19> {
                    name: name_ref,
                    parent: if row.bone_index == u16::MAX {
                        u32::MAX
                    } else {
                        u32::from(row.bone_index)
                    },
                    kind: wr::NodeKind::EVENT,
                    pose_matrices: put(raw, &poses)?.cast(),
                    payload: wr::NodePayload::Event(Box::new(wr::EventPayload::<wr::V19> {
                        id: row.event_type,
                        code: name_ref,
                        ..Default::default()
                    })),
                    native: wr::NativeBag {
                        entries: put(raw, &entries)?.cast(),
                    },
                    ..Default::default()
                });
                events.push(wr::ClipEvent {
                    time: (time.value as f64 - f64::from(start)) as f32 / 1000.,
                    node,
                    name: name_ref,
                    value: row.event_type,
                });
            }
        }};
    }
    if block.keys.entries != 0 {
        match revision(source, block.keys).map_err(read_error)? {
            0 => events!(nr::V0),
            1 => events!(nr::V1),
            2 => events!(nr::V2),
            _ => return Err(Error::new("m3.events", "unsupported event revision")),
        }
    }
    model.nodes.nodes = put(raw, &nodes)?.cast();
    model.nodes.pose_schema = put(raw, &schemas)?.cast();
    Ok(())
}

pub(super) fn export_events<E: Envelope>(
    model: &sem::Model<'_>,
    clip: &wr::Clip<wr::V7>,
    doc: &mut native::Document<'static, E>,
    context: &mut Context,
) -> Result<Vec<(u32, nr::AnimationBlockEvent)>, Error> {
    let Some(native) = &model.native else {
        return Ok(Vec::new());
    };
    let raw = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let record = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let nodes = raw.resolve::<wr::Node<wr::V19>>(record.nodes.nodes.cast())?;
    let mapping = hierarchy::mapping(model)?;
    struct EventGroup {
        times: Vec<nr::I32>,
        events: Vec<nr::Event<nr::V2>>,
        flags: u32,
        end_frame: u32,
    }
    let mut groups: BTreeMap<u32, EventGroup> = BTreeMap::new();
    for event in raw.resolve::<wr::ClipEvent>(clip.events.cast())?.iter() {
        let event = event?;
        let node = nodes
            .get(event.node as usize)
            .ok_or_else(|| Error::new("wem.events.node", "missing event node"))??;
        let fields = native_metadata::read(raw, &node.native)?;
        let get = |key: &str| fields.get(&format!("m3.event.{key}")).copied().unwrap_or(0);
        let id = get("animation_id") as u32;
        if id == 0 {
            return Err(Error::new(
                "wem.events.id",
                "event lacks native command channel identity",
            ));
        }
        let options = raw
            .resolve::<wr::NativeEntry<wr::V2>>(node.native.entries.cast())?
            .iter()
            .find_map(|entry| match entry {
                Ok(entry) if raw.string(entry.name).ok().flatten() == Some("m3.event.options") => {
                    Some(
                        raw.string(entry.text)
                            .map(|s| s.unwrap_or_default().to_owned()),
                    )
                }
                Err(error) => Some(Err(error)),
                _ => None,
            })
            .transpose()?
            .unwrap_or_default();
        let matrix = raw
            .resolve::<wr::Matrix44>(node.pose_matrices.cast())?
            .get(get("pose_slot") as usize)
            .ok_or_else(|| Error::new("wem.events.transform", "missing event transform pose"))??;
        let parent = model
            .nodes
            .get(event.node as usize)
            .and_then(|node| node.parent);
        let bone = match parent {
            Some(parent) => *mapping
                .get(parent as usize)
                .ok_or_else(|| Error::new("wem.events.parent", "missing bone mapping"))?,
            None => u32::from(u16::MAX),
        };
        let group = groups.entry(id).or_insert_with(|| EventGroup {
            times: Vec::new(),
            events: Vec::new(),
            flags: get("flags") as u32,
            end_frame: get("end") as u32,
        });
        group.times.push(clock::frame(
            event.time,
            clock::start(raw, clip)?,
            context,
            "events.time",
        )?);
        group.events.push(nr::Event::<nr::V2> {
            name: append_string(doc, raw.string(event.name)?.unwrap_or_default())?,
            unknown: get("unknown") as u32,
            bone_index: u16::try_from(bone)
                .map_err(|_| Error::new("events.bone", "bone exceeds u16"))?,
            padding: get("padding") as u16,
            transform: matrix.value,
            event_type: event.value,
            option_string: append_string(doc, &options)?,
            rtt_channel_index: get("rtt_channel_index") as u32,
            extra_parameter: get("extra_parameter") as u32,
        });
    }
    let mut output = Vec::new();
    for (
        id,
        EventGroup {
            times,
            events,
            flags,
            end_frame,
        },
    ) in groups
    {
        output.push((
            id,
            nr::AnimationBlockEvent {
                timestamps: append(doc, &times)?.cast(),
                keys: append(doc, &events)?.cast(),
                flags,
                end_frame,
            },
        ));
    }
    Ok(output)
}
