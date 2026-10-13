//! Selection, replay, blending, and bounds metadata for native sequences.
use super::super::effects::metadata as native_metadata;
use super::*;
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
pub(super) struct SourceSequence {
    pub value: nr::Sequence<nr::V2>,
    pub deprecated: u32,
    pub revision: u32,
}
pub(super) fn read<E: Envelope>(
    doc: &native::Document<'_, E>,
    reference: Ref<nr::targets::Sequence>,
) -> Result<Vec<SourceSequence>, Error> {
    let mut rows = Vec::new();
    macro_rules! read {
        ($v:ty) => {
            for row in doc
                .resolve::<nr::Sequence<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let row = row.map_err(read_error)?;
                rows.push(SourceSequence {
                    value: nr::Sequence::<nr::V2> {
                        id: row.id,
                        index: row.index,
                        name: row.name,
                        start_frame: row.start_frame,
                        end_frame: row.end_frame,
                        move_speed: row.move_speed,
                        flags: row.flags,
                        frequency: row.frequency,
                        replay_start: row.replay_start,
                        replay_end: row.replay_end,
                        blend_time: row.blend_time,
                        bounds: row.bounds,
                        animation_sets: row.animation_sets,
                        ..Default::default()
                    },
                    deprecated: row.deprecated_unknown.word(),
                    revision: <nr::Sequence<$v> as native::Record>::VERSION,
                });
            }
        };
    }
    if reference.entries != 0 {
        match revision(doc, reference).map_err(read_error)? {
            0 => read!(nr::V0),
            1 => read!(nr::V1),
            2 => read!(nr::V2),
            _ => return Err(Error::new("m3.sequences", "unsupported sequence revision")),
        }
    }
    Ok(rows)
}
pub(super) fn metadata<E: Envelope>(
    source: &SourceSequence,
    doc: &native::Document<'_, E>,
    raw: &mut wem::RawDocument<'static>,
) -> Result<wr::NativeBag, Error> {
    let row = &source.value;
    let mut fields = vec![
        ("conversion.clip_id".to_owned(), i64::from(row.id)),
        ("m3.sequence.index".to_owned(), i64::from(row.index)),
        ("m3.sequence.flags".to_owned(), i64::from(row.flags.bits())),
        (
            "m3.sequence.start_frame".to_owned(),
            i64::from(row.start_frame),
        ),
        (
            "m3.sequence.move_speed".to_owned(),
            i64::from(row.move_speed.to_bits()),
        ),
        ("m3.sequence.frequency".to_owned(), i64::from(row.frequency)),
        (
            "m3.sequence.replay_start".to_owned(),
            i64::from(row.replay_start),
        ),
        (
            "m3.sequence.replay_end".to_owned(),
            i64::from(row.replay_end),
        ),
        (
            "m3.sequence.blend_time".to_owned(),
            i64::from(row.blend_time),
        ),
        (
            "m3.sequence.deprecated_unknown".to_owned(),
            i64::from(source.deprecated),
        ),
        (
            "m3.sequence.source_version".to_owned(),
            i64::from(source.revision),
        ),
        (
            "m3.sequence.animation_sets.count".to_owned(),
            i64::from(row.animation_sets.entries),
        ),
    ];
    for (index, value) in doc
        .resolve::<nr::Byte>(row.animation_sets.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        fields.push((
            format!("m3.sequence.animation_sets[{index}]"),
            i64::from(value.map_err(read_error)?.value),
        ));
    }
    let mut entries = Vec::new();
    for (name, value) in fields {
        entries.push(wr::NativeEntry::<wr::V2> {
            name: text(raw, &name)?,
            value,
            ..Default::default()
        });
    }
    Ok(wr::NativeBag {
        entries: put(raw, &entries)?.cast(),
    })
}
pub(super) fn decorate<E: Envelope>(
    model: &sem::Model<'_>,
    doc: &mut native::Document<'static, E>,
    rows: &mut [nr::Sequence<nr::V2>],
) -> Result<Vec<(u32, u32)>, Error> {
    let Some(native) = &model.native else {
        return Ok(vec![(2, 0); rows.len()]);
    };
    let raw = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let clips: Vec<_> = raw
        .resolve::<wr::Clip<wr::V7>>(root.clips.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let mut versions = Vec::new();
    for (row, clip) in rows
        .iter_mut()
        .zip(clips.iter().filter(|clip| clip.model == native.model))
    {
        let data = native_metadata::read(raw, &clip.native)?;
        let get = |key: &str| {
            data.get(&format!("m3.sequence.{key}"))
                .copied()
                .unwrap_or(0)
        };
        if data.contains_key("m3.sequence.source_version") {
            let mut flags = nr::SequenceFlags(get("flags") as u32);
            flags.set_not_looping(!clip.looping.get());
            row.flags = flags;
            row.index = get("index") as i32;
            row.start_frame = get("start_frame") as u32;
            row.end_frame = row
                .start_frame
                .checked_add(row.end_frame)
                .ok_or_else(|| Error::new("sequence.duration", "clock interval overflows u32"))?;
            row.move_speed = f32::from_bits(get("move_speed") as u32);
            row.frequency = get("frequency") as u32;
            row.replay_start = get("replay_start") as u32;
            row.replay_end = get("replay_end") as u32;
            row.blend_time = get("blend_time") as u32;
            row.bounds = nr::Extent {
                minimum: clip.bounds.minimum,
                maximum: clip.bounds.maximum,
                radius: clip.bounds.sphere_radius,
            };
            let sets: Vec<_> = (0..get("animation_sets.count"))
                .map(|index| nr::Byte {
                    value: get(&format!("animation_sets[{index}]")) as u8,
                })
                .collect();
            row.animation_sets = append(doc, &sets)?.cast();
        }
        versions.push((
            data.get("m3.sequence.source_version").copied().unwrap_or(2) as u32,
            get("deprecated_unknown") as u32,
        ));
    }
    versions.resize(rows.len(), (2, 0));
    Ok(versions)
}
pub(super) fn write<E: Envelope>(
    doc: &mut native::Document<'static, E>,
    rows: &[nr::Sequence<nr::V2>],
    versions: &[(u32, u32)],
) -> Result<Ref<nr::targets::Sequence>, Error> {
    let revision = versions
        .iter()
        .map(|(version, _)| *version)
        .min()
        .unwrap_or(2);
    if revision == 2 {
        return Ok(append(doc, rows)?.cast());
    }
    macro_rules! write {
        ($v:ty) => {{
            let rows: Vec<_> = rows
                .iter()
                .zip(versions)
                .map(|(row, (_, deprecated))| nr::Sequence::<$v> {
                    id: row.id,
                    index: row.index,
                    name: row.name,
                    start_frame: row.start_frame,
                    end_frame: row.end_frame,
                    move_speed: row.move_speed,
                    flags: row.flags,
                    frequency: row.frequency,
                    replay_start: row.replay_start,
                    replay_end: row.replay_end,
                    blend_time: row.blend_time,
                    deprecated_unknown: *deprecated,
                    bounds: row.bounds,
                    animation_sets: row.animation_sets,
                })
                .collect();
            Ok(append(doc, &rows)?.cast())
        }};
    }
    match revision {
        0 => write!(nr::V0),
        1 => write!(nr::V1),
        _ => Err(Error::new("m3.sequences", "unsupported source revision")),
    }
}
